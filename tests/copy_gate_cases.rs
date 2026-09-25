use strokelet::{
    CopyGate, Decision, DesktopState, Injection, Modifiers, Observation, SessionState,
};

fn ready() -> DesktopState {
    DesktopState {
        session: Observation::Known(SessionState {
            target_owned: true,
            active: true,
            locked: false,
        }),
        extension_ready: Observation::Known(true),
        paused: Observation::Known(false),
        modifiers: Observation::Known(Modifiers { none_held: true }),
        communication: Observation::Known(()),
        permission_ok: Observation::Known(true),
    }
}

fn assert_blocks(name: &str, desktop: DesktopState) {
    let mut gate = CopyGate::new();
    assert_eq!(
        gate.authorize(Decision::Copy, desktop),
        Injection::None,
        "{name}"
    );
}

#[test]
fn fresh_copy_injects_once() {
    let mut gate = CopyGate::new();
    assert_eq!(gate.authorize(Decision::Copy, ready()), Injection::CopyOnce);
    assert_eq!(gate.authorize(Decision::Copy, ready()), Injection::None);
}

#[test]
fn rejected_copy_is_not_retried() {
    let mut desktop = ready();
    desktop.paused = Observation::Known(true);
    let mut gate = CopyGate::new();
    assert_eq!(gate.authorize(Decision::Copy, desktop), Injection::None);
    assert_eq!(gate.authorize(Decision::Copy, ready()), Injection::None);
}

#[test]
fn non_copy_decisions_do_not_inject() {
    for decision in [Decision::RightClick, Decision::Cancel, Decision::None] {
        let mut gate = CopyGate::new();
        assert_eq!(gate.authorize(decision, ready()), Injection::None);
        assert_eq!(gate.authorize(Decision::Copy, ready()), Injection::None);
    }
}

#[test]
fn ipc_request_never_injects() {
    let mut gate = CopyGate::new();
    assert_eq!(CopyGate::reject_ipc(), Injection::None);
    assert_eq!(gate.authorize(Decision::Copy, ready()), Injection::CopyOnce);
}

#[test]
fn incomplete_observations_block_copy() {
    let mut session_unknown = ready();
    session_unknown.session = Observation::Unknown;
    let mut session_timeout = ready();
    session_timeout.session = Observation::TimedOut;
    let mut extension_unknown = ready();
    extension_unknown.extension_ready = Observation::Unknown;
    let mut extension_timeout = ready();
    extension_timeout.extension_ready = Observation::TimedOut;
    let mut pause_unknown = ready();
    pause_unknown.paused = Observation::Unknown;
    let mut pause_timeout = ready();
    pause_timeout.paused = Observation::TimedOut;
    let mut modifiers_unknown = ready();
    modifiers_unknown.modifiers = Observation::Unknown;
    let mut modifiers_timeout = ready();
    modifiers_timeout.modifiers = Observation::TimedOut;
    let mut communication_unknown = ready();
    communication_unknown.communication = Observation::Unknown;
    let mut communication_timeout = ready();
    communication_timeout.communication = Observation::TimedOut;
    let mut permission_unknown = ready();
    permission_unknown.permission_ok = Observation::Unknown;
    let mut permission_timeout = ready();
    permission_timeout.permission_ok = Observation::TimedOut;

    for (name, desktop) in [
        ("session unknown", session_unknown),
        ("session timed out", session_timeout),
        ("extension unknown", extension_unknown),
        ("extension timed out", extension_timeout),
        ("pause unknown", pause_unknown),
        ("pause timed out", pause_timeout),
        ("modifiers unknown", modifiers_unknown),
        ("modifiers timed out", modifiers_timeout),
        ("communication unknown", communication_unknown),
        ("communication timed out", communication_timeout),
        ("permission unknown", permission_unknown),
        ("permission timed out", permission_timeout),
    ] {
        assert_blocks(name, desktop);
    }
}

#[test]
fn unsafe_desktop_blocks_copy() {
    let mut wrong_session = ready();
    wrong_session.session = Observation::Known(SessionState {
        target_owned: false,
        active: true,
        locked: false,
    });
    let mut inactive = ready();
    inactive.session = Observation::Known(SessionState {
        target_owned: true,
        active: false,
        locked: false,
    });
    let mut locked = ready();
    locked.session = Observation::Known(SessionState {
        target_owned: true,
        active: true,
        locked: true,
    });
    let mut extension_down = ready();
    extension_down.extension_ready = Observation::Known(false);
    let mut paused = ready();
    paused.paused = Observation::Known(true);
    let mut permission_denied = ready();
    permission_denied.permission_ok = Observation::Known(false);
    let mut modifiers_held = ready();
    modifiers_held.modifiers = Observation::Known(Modifiers { none_held: false });

    for (name, desktop) in [
        ("wrong session", wrong_session),
        ("inactive session", inactive),
        ("locked session", locked),
        ("extension not ready", extension_down),
        ("paused", paused),
        ("permission denied", permission_denied),
        ("modifier held", modifiers_held),
    ] {
        assert_blocks(name, desktop);
    }
}
