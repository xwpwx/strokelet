use std::env;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use evdev::uinput::VirtualDevice;
use evdev::{AbsoluteAxisCode, AttributeSet, Device, InputEvent, KeyCode, RelativeAxisCode};
use strokelet::{
    CancelReason, CopyKey, CopyOutput, Decision, DeviceProfile, EmitError, FrameProcessor,
    Injection, InjectionLedger, KeySink, Limits, LineCodec, Link, Observation, Outcome,
    OutputEvent, PollAction, SessionFacts, classify_device, current_uid, facts_from_paths,
    grab_allowed, is_active_unlocked, key_event, peer_is_target, status_message, syn_report,
    validate_runtime_path, virtual_mouse_codes,
};

fn main() -> ExitCode {
    match dispatch(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("strokelet: {message}");
            ExitCode::from(1)
        }
    }
}

fn dispatch(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("status") => {
            println!("{}", status_message());
            Ok(())
        }
        Some("list-devices") => list_devices(),
        Some("run") => run(&args[1..]),
        Some("--help") | Some("help") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("unknown command {other}; try strokelet help")),
    }
}

fn print_help() {
    println!(
        "\
strokelet: demo

Commands:
  status
  list-devices
  run --device PATH --uid UID --session ID [--timeout-seconds 120]
      [--passthrough-only] [--inject-copy]

Grabbing waits until the extension is ready, the session is active and
unlocked, and no keys are held. The default timeout is 120 seconds.
This is a foreground demo, not a system service."
    );
}

fn list_devices() -> Result<(), String> {
    let mut found = false;
    for (path, device) in evdev::enumerate() {
        found = true;
        let name = device.name().unwrap_or("").to_string();
        let rel = axis_codes(device.supported_relative_axes());
        let abs = abs_codes(device.supported_absolute_axes());
        let keys = device
            .supported_keys()
            .map(|set| set.iter().map(|code| code.0).collect())
            .unwrap_or_default();
        let profile = DeviceProfile {
            name: name.clone(),
            rel: rel.clone(),
            keys,
            abs: abs.clone(),
        };
        let verdict = match classify_device(&profile) {
            Ok(()) => "accepted".to_string(),
            Err(reason) => format!("rejected {reason:?}"),
        };
        println!(
            "{verdict}\t{}\tname={name}\trel={rel:?}\tabs={abs:?}",
            path.display()
        );
    }
    if !found {
        eprintln!("strokelet: no input devices could be opened; check /dev/input permissions");
    }
    Ok(())
}

fn axis_codes(axes: Option<&evdev::AttributeSetRef<RelativeAxisCode>>) -> Vec<u16> {
    axes.map(|set| set.iter().map(|code| code.0).collect())
        .unwrap_or_default()
}

fn abs_codes(axes: Option<&evdev::AttributeSetRef<AbsoluteAxisCode>>) -> Vec<u16> {
    axes.map(|set| set.iter().map(|code| code.0).collect())
        .unwrap_or_default()
}

struct RunOpts {
    device: PathBuf,
    uid: u32,
    session: String,
    timeout: Duration,
    passthrough_only: bool,
    inject_copy: bool,
}

struct UinputKeys {
    device: VirtualDevice,
}

impl UinputKeys {
    fn new(device: VirtualDevice) -> Self {
        Self { device }
    }
}

impl KeySink for UinputKeys {
    fn emit(&mut self, event: OutputEvent) -> Result<(), EmitError> {
        let input = match event {
            OutputEvent::Key {
                code: CopyKey::LeftCtrl,
                down,
            } => key_event(KeyCode::KEY_LEFTCTRL, if down { 1 } else { 0 }),
            OutputEvent::Key {
                code: CopyKey::C,
                down,
            } => key_event(KeyCode::KEY_C, if down { 1 } else { 0 }),
            OutputEvent::SynReport => syn_report(),
        };
        self.device.emit(&[input]).map_err(|_| EmitError)
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let opts = parse_run(args)?;
    if opts.uid != current_uid() {
        return Err(format!(
            "refusing to run as uid {} for target uid {}",
            current_uid(),
            opts.uid
        ));
    }
    let facts = read_session(&opts.session, opts.uid);
    if !is_active_unlocked(facts) {
        return Err("target session is unknown, inactive, locked, or owned by another user".into());
    }
    let listener = bind_runtime_socket(opts.uid)?;
    let result = run_bound(&opts, listener);
    let _ = fs::remove_file(runtime_socket(opts.uid));
    result
}

fn run_bound(opts: &RunOpts, listener: UnixListener) -> Result<(), String> {
    let mut device = Device::open(&opts.device).map_err(|err| {
        format!(
            "cannot open {}: {err}. Choose a relative mouse with list-devices",
            opts.device.display()
        )
    })?;
    set_nonblocking(device.as_raw_fd())?;
    let profile = profile_of(&device);
    classify_device(&profile).map_err(|reason| format!("device rejected: {reason:?}"))?;
    let (rel, keys) = virtual_mouse_codes(&profile);
    let mut mouse = virtual_mouse(&rel, &keys)?;
    let mut keyboard = CopyOutput::new(UinputKeys::new(virtual_keyboard()?));
    if opts.passthrough_only {
        eprintln!("strokelet: passthrough only; Ctrl+C stays disarmed");
    } else if !opts.inject_copy {
        eprintln!("strokelet: copy injection is off; pass --inject-copy to arm Ctrl+C");
    }
    eprintln!(
        "strokelet: listening for the extension; timeout {}s",
        opts.timeout.as_secs()
    );
    let drive_result = drive(opts, listener, &mut device, &mut mouse, &mut keyboard);
    let _ = device.ungrab();
    let _ = keyboard.release_owned_keys();
    drive_result?;
    eprintln!("strokelet: stopped and released owned keys");
    Ok(())
}

fn drive(
    opts: &RunOpts,
    listener: UnixListener,
    device: &mut Device,
    mouse: &mut VirtualDevice,
    keyboard: &mut CopyOutput<UinputKeys>,
) -> Result<(), String> {
    listener
        .set_nonblocking(true)
        .map_err(|err| err.to_string())?;
    let mut client: Option<UnixStream> = None;
    let mut codec = LineCodec::new();
    let mut link = Link::new();
    let mut processor = FrameProcessor::new(Limits::default());
    let mut ledger = InjectionLedger::new();
    let mut gesture_id = 1u64;
    let mut grabbed = false;
    let started = Instant::now();
    while started.elapsed() < opts.timeout {
        let now_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if client.is_none() {
            match listener.accept() {
                Ok((stream, _)) if peer_is_target(&stream, opts.uid) => {
                    let _ = stream.set_nonblocking(true);
                    let _ = (&stream).write_all(Link::hello().as_bytes());
                    link = Link::new();
                    link.connected(now_ms);
                    codec = LineCodec::new();
                    client = Some(stream);
                }
                Ok((stream, _)) => drop(stream),
                Err(err) if err.kind() == ErrorKind::WouldBlock => {}
                Err(err) => return Err(err.to_string()),
            }
        }
        if let Some(stream) = client.as_mut() {
            let mut buf = [0u8; 1024];
            match stream.read(&mut buf) {
                Ok(0) => {
                    link.disconnect();
                    client = None;
                    if grabbed {
                        let _ = device.ungrab();
                        grabbed = false;
                    }
                }
                Ok(size) => {
                    for line in codec.push(&buf[..size]).map_err(|err| format!("{err:?}"))? {
                        if let Err(err) = link.ingest(&line, now_ms) {
                            eprintln!("strokelet: protocol {err:?}");
                            link.disconnect();
                            client = None;
                        }
                    }
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {}
                Err(err) => return Err(err.to_string()),
            }
        }
        if let Some(stream) = client.as_mut() {
            match link.poll(now_ms) {
                PollAction::SendPing => {
                    let _ = stream.write_all(Link::ping().as_bytes());
                }
                PollAction::Disconnect => {
                    client = None;
                    if grabbed {
                        let _ = device.ungrab();
                        grabbed = false;
                    }
                    eprintln!("strokelet: extension timed out");
                }
                PollAction::None => {}
            }
        }
        let facts = read_session(&opts.session, opts.uid);
        let session_ok = is_active_unlocked(facts);
        let hold = !session_ok || link.is_paused();
        if hold && grabbed {
            let _ = device.ungrab();
            grabbed = false;
            eprintln!("strokelet: session or pause released the mouse");
        }
        if !grabbed && grab_allowed(link.is_ready() && !link.is_paused(), session_ok, false) {
            device.grab().map_err(|err| format!("grab failed: {err}"))?;
            grabbed = true;
            eprintln!("strokelet: grabbed {}", opts.device.display());
        }
        if grabbed {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        publish_gesture(
                            &mut Live {
                                processor: &mut processor,
                                link: &mut link,
                                ledger: &mut ledger,
                                mouse,
                                keyboard,
                                client: &mut client,
                                opts,
                                gesture_id: &mut gesture_id,
                            },
                            event,
                            now_ms,
                            facts,
                        )?;
                    }
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {}
                Err(err) => return Err(format!("device read failed: {err}")),
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}

struct Live<'a> {
    processor: &'a mut FrameProcessor,
    link: &'a mut Link,
    ledger: &'a mut InjectionLedger,
    mouse: &'a mut VirtualDevice,
    keyboard: &'a mut CopyOutput<UinputKeys>,
    client: &'a mut Option<UnixStream>,
    opts: &'a RunOpts,
    gesture_id: &'a mut u64,
}

fn publish_gesture(
    live: &mut Live<'_>,
    event: InputEvent,
    now_ms: u64,
    facts: SessionFacts,
) -> Result<(), String> {
    let was_tracking = live.processor.is_tracking();
    let previous = live.processor.last_decision();
    let forwarded = live.processor.handle(event, now_ms);
    if !forwarded.is_empty() {
        live.mouse
            .emit(&forwarded)
            .map_err(|err| format!("virtual mouse write failed: {err}"))?;
    }
    if !was_tracking
        && live.processor.is_tracking()
        && let Ok(line) = live.link.begin(*live.gesture_id)
    {
        eprintln!("strokelet: {line}");
        send_line(live.client, &line);
    }
    if live.processor.last_decision() != previous
        && let Some(decision) = live.processor.last_decision()
    {
        let id = *live.gesture_id;
        *live.gesture_id += 1;
        let session = session_observation(facts);
        let desktop = live.link.desktop(session, Observation::Known(true), now_ms);
        let injection = live
            .ledger
            .decide(id, live.opts.inject_copy, decision, desktop);
        let outcome = match (decision, injection) {
            (Decision::RightClick, _) => Outcome::Click,
            (Decision::Copy, Injection::CopyOnce) => {
                if live.keyboard.send_copy().is_ok() {
                    Outcome::CopyInjected
                } else {
                    let _ = live.keyboard.release_owned_keys();
                    Outcome::Unmatched
                }
            }
            _ => Outcome::Unmatched,
        };
        eprintln!("strokelet: decision={decision:?} outcome={outcome:?}");
        let line = if outcome == Outcome::Unmatched && decision != Decision::RightClick {
            live.link.cancel(id, CancelReason::Unmatched)
        } else {
            live.link.end(id, outcome)
        };
        if let Ok(line) = line {
            eprintln!("strokelet: {line}");
            send_line(live.client, &line);
        }
    }
    Ok(())
}

fn send_line(client: &mut Option<UnixStream>, line: &str) {
    if let Some(stream) = client.as_mut() {
        let _ = stream.write_all(line.as_bytes());
    }
}

fn session_observation(facts: SessionFacts) -> Observation<strokelet::SessionState> {
    match strokelet::session_state(facts) {
        Some(state) => Observation::Known(state),
        None => Observation::Unknown,
    }
}

fn profile_of(device: &Device) -> DeviceProfile {
    DeviceProfile {
        name: device.name().unwrap_or("").to_string(),
        rel: axis_codes(device.supported_relative_axes()),
        keys: device
            .supported_keys()
            .map(|set| set.iter().map(|code| code.0).collect())
            .unwrap_or_default(),
        abs: abs_codes(device.supported_absolute_axes()),
    }
}

fn runtime_dir(uid: u32) -> PathBuf {
    PathBuf::from(format!("/run/strokelet/{uid}"))
}

fn runtime_socket(uid: u32) -> PathBuf {
    runtime_dir(uid).join("strokelet.sock")
}

fn bind_runtime_socket(uid: u32) -> Result<UnixListener, String> {
    let dir = runtime_dir(uid);
    let socket = runtime_socket(uid);
    if !dir.is_dir() {
        return Err(format!(
            "missing {}; root must create it as owner root, group $(id -gn), mode 0770",
            dir.display()
        ));
    }
    if socket.exists() {
        return Err(format!(
            "refusing existing socket {}; remove it only if it belongs to this demo",
            socket.display()
        ));
    }
    let listener = UnixListener::bind(&socket).map_err(|err| err.to_string())?;
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))
        .map_err(|err| err.to_string())?;
    let facts = facts_from_paths(&dir, &socket).map_err(|err| err.to_string())?;
    let gid = nix::unistd::Gid::current().as_raw();
    validate_runtime_path(facts, uid, gid).map_err(|err| {
        format!("runtime path rejected: {err:?}. directory must be root:{gid} mode 0770, socket mode 0600")
    })?;
    Ok(listener)
}

fn virtual_mouse(rel: &[u16], keys: &[u16]) -> Result<VirtualDevice, String> {
    let mut rel_set = AttributeSet::<RelativeAxisCode>::new();
    for code in rel {
        rel_set.insert(RelativeAxisCode(*code));
    }
    let mut key_set = AttributeSet::<KeyCode>::new();
    for code in keys {
        key_set.insert(KeyCode(*code));
    }
    let mut builder = VirtualDevice::builder().map_err(|err| format!("uinput: {err}"))?;
    builder = builder.name("strokelet virtual mouse");
    builder = builder
        .with_relative_axes(&rel_set)
        .map_err(|err| format!("uinput relative: {err}"))?;
    builder
        .with_keys(&key_set)
        .map_err(|err| format!("uinput keys: {err}"))?
        .build()
        .map_err(|err| format!("uinput build: {err}"))
}

fn virtual_keyboard() -> Result<VirtualDevice, String> {
    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::KEY_LEFTCTRL);
    keys.insert(KeyCode::KEY_C);
    VirtualDevice::builder()
        .map_err(|err| format!("uinput: {err}"))?
        .name("strokelet virtual keyboard")
        .with_keys(&keys)
        .map_err(|err| format!("uinput keys: {err}"))?
        .build()
        .map_err(|err| format!("uinput build: {err}"))
}

fn set_nonblocking(fd: i32) -> Result<(), String> {
    let flags = unsafe { nix::libc::fcntl(fd, nix::libc::F_GETFL) };
    if flags < 0 {
        return Err("cannot read device fd flags".into());
    }
    let result = unsafe { nix::libc::fcntl(fd, nix::libc::F_SETFL, flags | nix::libc::O_NONBLOCK) };
    if result < 0 {
        return Err("cannot set the device nonblocking".into());
    }
    Ok(())
}

fn read_session(session: &str, target_uid: u32) -> SessionFacts {
    let output = std::process::Command::new("loginctl")
        .args([
            "show-session",
            session,
            "-p",
            "Active",
            "-p",
            "LockedHint",
            "-p",
            "User",
        ])
        .output();
    let Ok(output) = output else {
        return SessionFacts {
            query_ok: false,
            uid_matches: false,
            active: false,
            locked: true,
        };
    };
    if !output.status.success() {
        return SessionFacts {
            query_ok: false,
            uid_matches: false,
            active: false,
            locked: true,
        };
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut active = false;
    let mut locked = true;
    let mut uid = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("Active=") {
            active = value == "yes";
        } else if let Some(value) = line.strip_prefix("LockedHint=") {
            locked = value == "yes";
        } else if let Some(value) = line.strip_prefix("User=") {
            uid = value.parse().ok();
        }
    }
    SessionFacts {
        query_ok: uid.is_some(),
        uid_matches: uid == Some(target_uid),
        active,
        locked,
    }
}

fn parse_run(args: &[String]) -> Result<RunOpts, String> {
    let mut device = None;
    let mut uid = None;
    let mut session = None;
    let mut timeout = 120u64;
    let mut passthrough_only = false;
    let mut inject_copy = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--device" => {
                device = Some(PathBuf::from(next_value(&mut iter, "--device")?));
            }
            "--uid" => {
                uid = Some(
                    next_value(&mut iter, "--uid")?
                        .parse()
                        .map_err(|_| "uid must be a number".to_string())?,
                );
            }
            "--session" => session = Some(next_value(&mut iter, "--session")?),
            "--timeout-seconds" => {
                timeout = next_value(&mut iter, "--timeout-seconds")?
                    .parse()
                    .map_err(|_| "timeout must be a number".to_string())?;
            }
            "--passthrough-only" => passthrough_only = true,
            "--inject-copy" => inject_copy = true,
            other => return Err(format!("unknown run option {other}")),
        }
    }
    if passthrough_only {
        inject_copy = false;
    }
    Ok(RunOpts {
        device: device.ok_or("run requires --device")?,
        uid: uid.ok_or("run requires --uid")?,
        session: session.ok_or("run requires --session")?,
        timeout: Duration::from_secs(timeout),
        passthrough_only,
        inject_copy,
    })
}

fn next_value(iter: &mut std::slice::Iter<String>, flag: &str) -> Result<String, String> {
    iter.next()
        .cloned()
        .ok_or_else(|| format!("{flag} needs a value"))
}
