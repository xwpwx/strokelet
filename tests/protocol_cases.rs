use std::os::unix::net::{UnixListener, UnixStream};
use std::time::{SystemTime, UNIX_EPOCH};
use strokelet::{
    Chord, ClientUpdate, CopyOutput, Decision, Direction, EmitError, Injection, InjectionLedger,
    KeySink, LineCodec, Link, MAX_LINE_BYTES, Observation, Outcome, OutputEvent, PathError,
    PathFacts, PollAction, ProtocolError, RUNTIME_DIR_MODE, SOCKET_MODE, SessionFacts,
    SessionState, is_active_unlocked, peer_is_target, validate_runtime_path,
};

struct LogSink {
    events: Vec<OutputEvent>,
    attempts: usize,
    fail_at: Option<usize>,
}

impl KeySink for LogSink {
    fn emit(&mut self, event: OutputEvent) -> Result<(), EmitError> {
        let attempt = self.attempts;
        self.attempts += 1;
        if self.fail_at == Some(attempt) {
            return Err(EmitError);
        }
        self.events.push(event);
        Ok(())
    }
}

fn ready_link(now_ms: u64) -> Link {
    let mut link = Link::new();
    link.connected(now_ms);
    link.ingest(r#"{"type":"ready","version":1}"#, now_ms)
        .unwrap();
    link.ingest(r#"{"type":"pause","paused":false}"#, now_ms)
        .unwrap();
    link.ingest(
        r#"{"type":"state","active":true,"locked":false,"modifiers":[]}"#,
        now_ms,
    )
    .unwrap();
    link
}

fn fresh_logind() -> Observation<SessionState> {
    Observation::Known(SessionState {
        target_owned: true,
        active: true,
        locked: false,
    })
}

#[test]
fn reload_is_accepted_before_ready() {
    let mut link = Link::new();
    link.connected(0);
    assert_eq!(
        link.ingest(r#"{"type":"reload"}"#, 0).unwrap(),
        ClientUpdate::Reload
    );
}

#[test]
fn handshake_reaches_ready_once() {
    let mut link = Link::new();
    assert_eq!(
        link.ingest(r#"{"type":"ready","version":1}"#, 0).unwrap(),
        ClientUpdate::Ready
    );
    assert!(link.is_ready());
    assert_eq!(
        link.ingest(r#"{"type":"ready","version":1}"#, 10),
        Err(ProtocolError::DuplicateReady)
    );
    assert!(!link.is_ready());
}

#[test]
fn rejects_unknown_version_and_command_like_messages() {
    let mut link = Link::new();
    assert_eq!(
        link.ingest(r#"{"type":"ready","version":2}"#, 0),
        Err(ProtocolError::BadVersion)
    );
    let mut codec = LineCodec::new();
    for line in [
        r#"{"type":"copy"}"#,
        r#"{"type":"command","cmd":"id"}"#,
        r#"{"type":"key","code":29}"#,
        r#"{"type":"ready"}"#,
    ] {
        let mut fresh = Link::new();
        assert_eq!(
            fresh.ingest(line, 0),
            Err(ProtocolError::InvalidJson),
            "{line}"
        );
    }
    let chunk = b"not-json\n";
    assert_eq!(codec.push(chunk).unwrap(), vec!["not-json".to_string()]);
}

#[test]
fn line_over_4096_without_newline_is_rejected() {
    let mut codec = LineCodec::new();
    let bytes = vec![b'a'; MAX_LINE_BYTES + 1];
    assert_eq!(codec.push(&bytes), Err(ProtocolError::LineTooLong));
}

#[test]
fn truncated_json_is_rejected() {
    let mut link = Link::new();
    assert_eq!(
        link.ingest(r#"{"type":"ready""#, 0),
        Err(ProtocolError::InvalidJson)
    );
}

#[test]
fn old_begin_id_dies_on_disconnect() {
    let mut link = ready_link(0);
    let begin = link.begin(7).unwrap();
    assert!(begin.contains("\"id\":7"));
    link.disconnect();
    assert_eq!(link.end(7, Outcome::Unmatched), Err(ProtocolError::Closed));
    assert_eq!(link.begin(8), Err(ProtocolError::Closed));
}

#[test]
fn health_timeout_and_stale_state_block_copy() {
    let mut link = ready_link(0);
    assert_eq!(link.poll(250), PollAction::SendPing);
    assert_eq!(link.poll(400), PollAction::None);
    link.ingest(r#"{"type":"pong"}"#, 900).unwrap();
    assert_eq!(link.poll(1000), PollAction::SendPing);
    assert_eq!(link.poll(1901), PollAction::Disconnect);
    assert!(!link.is_ready());

    let live = ready_link(0);
    let fresh = live.desktop(fresh_logind(), Observation::Known(true), 500);
    assert_eq!(
        InjectionLedger::new().decide(1, true, Decision::Stroke(Direction::Up), fresh),
        Injection::CopyOnce
    );
    let stale = live.desktop(fresh_logind(), Observation::Known(true), 1001);
    assert_eq!(
        InjectionLedger::new().decide(2, true, Decision::Stroke(Direction::Up), stale),
        Injection::None
    );
}

#[test]
fn inject_switch_starts_off_and_ipc_cannot_copy() {
    let link = ready_link(0);
    let desktop = link.desktop(fresh_logind(), Observation::Known(true), 10);
    let mut ledger = InjectionLedger::new();
    assert_eq!(
        ledger.decide(1, false, Decision::Stroke(Direction::Up), desktop),
        Injection::None
    );
    assert_eq!(
        ledger.decide(1, true, Decision::Stroke(Direction::Up), desktop),
        Injection::None
    );
    assert_eq!(InjectionLedger::reject_ipc(), Injection::None);
}

#[test]
fn copy_chord_is_ctrl_down_c_down_c_up_ctrl_up() {
    let mut output = CopyOutput::new(LogSink {
        events: Vec::new(),
        attempts: 0,
        fail_at: None,
    });
    let chord = Chord::parse(&["ctrl".into()], "c").unwrap();
    output.send_chord(&chord).unwrap();
    let ctrl = chord.modifiers()[0].code();
    let key = chord.key_code();
    assert_eq!(
        output.sink().events,
        vec![
            OutputEvent::Key {
                code: ctrl,
                down: true
            },
            OutputEvent::SynReport,
            OutputEvent::Key {
                code: key,
                down: true
            },
            OutputEvent::SynReport,
            OutputEvent::Key {
                code: key,
                down: false
            },
            OutputEvent::SynReport,
            OutputEvent::Key {
                code: ctrl,
                down: false
            },
            OutputEvent::SynReport,
        ]
    );
}

#[test]
fn partial_copy_releases_only_owned_keys() {
    let mut output = CopyOutput::new(LogSink {
        events: Vec::new(),
        attempts: 0,
        fail_at: Some(2),
    });
    let chord = Chord::parse(&["ctrl".into()], "c").unwrap();
    assert_eq!(output.send_chord(&chord), Err(EmitError));
    let ctrl = chord.modifiers()[0].code();
    assert_eq!(
        output.sink().events,
        vec![
            OutputEvent::Key {
                code: ctrl,
                down: true
            },
            OutputEvent::SynReport,
            OutputEvent::Key {
                code: ctrl,
                down: false
            },
            OutputEvent::SynReport,
        ]
    );
}

#[test]
fn session_query_failure_inactive_and_locked_are_unavailable() {
    assert!(!is_active_unlocked(SessionFacts {
        query_ok: false,
        uid_matches: true,
        active: true,
        locked: false,
    }));
    assert!(!is_active_unlocked(SessionFacts {
        query_ok: true,
        uid_matches: false,
        active: true,
        locked: false,
    }));
    assert!(!is_active_unlocked(SessionFacts {
        query_ok: true,
        uid_matches: true,
        active: false,
        locked: false,
    }));
    assert!(!is_active_unlocked(SessionFacts {
        query_ok: true,
        uid_matches: true,
        active: true,
        locked: true,
    }));
    assert!(is_active_unlocked(SessionFacts {
        query_ok: true,
        uid_matches: true,
        active: true,
        locked: false,
    }));
}

#[test]
fn runtime_path_rejects_symlink_and_wrong_owner() {
    let good = PathFacts {
        is_symlink: false,
        dir_uid: 0,
        dir_gid: 1000,
        dir_mode: RUNTIME_DIR_MODE,
        sock_uid: 1000,
        sock_mode: SOCKET_MODE,
    };
    assert_eq!(validate_runtime_path(good, 1000, 1000), Ok(()));
    assert_eq!(
        validate_runtime_path(
            PathFacts {
                is_symlink: true,
                ..good
            },
            1000,
            1000
        ),
        Err(PathError::Symlink)
    );
    assert_eq!(
        validate_runtime_path(
            PathFacts {
                dir_mode: 0o750,
                ..good
            },
            1000,
            1000
        ),
        Err(PathError::Directory)
    );
}

#[test]
fn peer_credential_accepts_only_target_uid() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("strokelet-peer-{stamp}.sock"));
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).unwrap();
    let client = UnixStream::connect(&path).unwrap();
    let (server, _) = listener.accept().unwrap();
    let uid = strokelet::current_uid();
    assert!(peer_is_target(&server, uid));
    assert!(!peer_is_target(&server, uid.wrapping_add(1)));
    drop(client);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn symlink_socket_path_is_rejected_from_metadata() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("strokelet-link-{stamp}"));
    std::fs::create_dir(&dir).unwrap();
    let real = dir.join("real.sock");
    let link = dir.join("link.sock");
    let _listener = UnixListener::bind(&real).unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let facts = strokelet::facts_from_paths(&dir, &link).unwrap();
    assert_eq!(
        validate_runtime_path(facts, facts.sock_uid, facts.dir_gid),
        Err(PathError::Symlink)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
