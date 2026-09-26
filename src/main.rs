use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use evdev::uinput::VirtualDevice;
use evdev::{
    AbsoluteAxisCode, AttributeSet, Device, EventType, InputEvent, KeyCode, RelativeAxisCode,
    SynchronizationCode,
};
use strokelet::{
    CancelReason, CaptureUpdate, ChordCapture, CopyOutput, Decision, DeviceProfile, EmitError,
    FrameProcessor, GestureConfig, Injection, InjectionLedger, KeySink, Limits, LineCodec, Link,
    ListedMouse, MouseButton, Observation, Outcome, OutputEvent, PickError, PollAction,
    SessionFacts, TriggerButton, chord_device_codes, classify_device, config_path, current_uid,
    device_path, facts_from_paths, grab_allowed, is_active_unlocked, key_event, parse_config,
    peer_is_target, pick_mouse, status_message, syn_report, validate_runtime_path,
    virtual_mouse_codes,
};

static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn request_stop(_signal: i32) {
    STOP.store(true, Ordering::Relaxed);
}

fn arm_stop_signals() {
    unsafe {
        let mut action: nix::libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = request_stop as *const () as usize;
        nix::libc::sigemptyset(&mut action.sa_mask);
        action.sa_flags = 0;
        nix::libc::sigaction(nix::libc::SIGINT, &action, std::ptr::null_mut());
        nix::libc::sigaction(nix::libc::SIGTERM, &action, std::ptr::null_mut());
    }
}

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
        Some("settings") => open_settings(),
        Some("capture-chord") => capture_chord(),
        Some("capture-stroke") => {
            capture_stroke(args.get(1).map(String::as_str).unwrap_or("right"))
        }
        Some("capture-button") => {
            capture_button(args.get(1).map(String::as_str).unwrap_or("right"))
        }
        Some("check-gestures") => check_gestures(),
        Some("--help") | Some("help") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("unknown command {other}; try strokelet help")),
    }
}

fn open_settings() -> Result<(), String> {
    let script = settings_script()?;
    let executable = env::current_exe().map_err(|err| format!("cannot find strokelet: {err}"))?;
    let status = std::process::Command::new("gjs")
        .arg("-m")
        .arg(&script)
        .env("STROKELET_BIN", executable)
        .status()
        .map_err(|err| format!("cannot start gjs: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("settings window exited with {status}"))
    }
}

fn capture_chord() -> Result<(), String> {
    let mut grabbed = match grab_keyboards() {
        Ok(devices) => devices,
        Err(message) => {
            emit_capture_line(&serde_json::json!({"type": "error", "message": message}));
            return Err(message);
        }
    };
    let value = read_chord(&mut grabbed);
    drop(grabbed);
    emit_capture_line(&value);
    if value.get("type").and_then(|kind| kind.as_str()) == Some("error") {
        return Err(value
            .get("message")
            .and_then(|message| message.as_str())
            .unwrap_or("capture failed")
            .to_string());
    }
    Ok(())
}

struct GrabbedDevice(Device);

impl Drop for GrabbedDevice {
    fn drop(&mut self) {
        let _ = self.0.ungrab();
    }
}

fn grab_keyboards() -> Result<Vec<GrabbedDevice>, String> {
    let mut grabbed = Vec::new();
    for (_path, mut device) in evdev::enumerate() {
        if !is_physical_keyboard(&device) {
            continue;
        }
        if set_nonblocking(device.as_raw_fd()).is_err() {
            continue;
        }
        if device.grab().is_err() {
            continue;
        }
        grabbed.push(GrabbedDevice(device));
    }
    if grabbed.is_empty() {
        return Err("没有找到键盘".into());
    }
    Ok(grabbed)
}

fn is_physical_keyboard(device: &Device) -> bool {
    let name = device.name().unwrap_or("").to_ascii_lowercase();
    if name.contains("strokelet") || name.contains("ydotool") {
        return false;
    }
    device
        .supported_keys()
        .is_some_and(|keys| keys.contains(KeyCode::KEY_A) && keys.contains(KeyCode::KEY_LEFTCTRL))
}

fn read_chord(devices: &mut [GrabbedDevice]) -> serde_json::Value {
    let mut capture = ChordCapture::new();
    let started = Instant::now();
    loop {
        let limit = if capture.has_main() {
            Duration::from_secs(10)
        } else {
            Duration::from_secs(5)
        };
        let remaining = limit.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            let message = if capture.has_main() {
                "时间到了，这次没有保存"
            } else {
                "5 秒内没有按下主键"
            };
            return serde_json::json!({"type": "cancel", "message": message});
        }
        let mut fds: Vec<nix::libc::pollfd> = devices
            .iter()
            .map(|device| nix::libc::pollfd {
                fd: device.0.as_raw_fd(),
                events: nix::libc::POLLIN,
                revents: 0,
            })
            .collect();
        let timeout = i32::try_from(remaining.as_millis()).unwrap_or(i32::MAX);
        let ready =
            unsafe { nix::libc::poll(fds.as_mut_ptr(), fds.len() as nix::libc::nfds_t, timeout) };
        if ready < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return serde_json::json!({"type": "error", "message": err.to_string()});
        }
        if ready == 0 {
            continue;
        }
        for (index, slot) in fds.iter().enumerate() {
            if slot.revents & nix::libc::POLLIN == 0 {
                continue;
            }
            let events = match devices[index].0.fetch_events() {
                Ok(events) => events,
                Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
                Err(err) => {
                    return serde_json::json!({"type": "error", "message": err.to_string()});
                }
            };
            for event in events {
                if event.event_type() != EventType::KEY {
                    continue;
                }
                let update = match event.value() {
                    1 => capture.down(event.code()),
                    0 => capture.up(event.code()),
                    _ => None,
                };
                if let Some(done) = publish_capture(update) {
                    return done;
                }
            }
        }
    }
}

fn publish_capture(update: Option<CaptureUpdate>) -> Option<serde_json::Value> {
    let update = update?;
    match update {
        CaptureUpdate::Live(label) => {
            emit_capture_line(&serde_json::json!({"type": "live", "label": label}));
            None
        }
        CaptureUpdate::Finished(chord) => Some(serde_json::json!({
            "type": "chord",
            "modifierCodes": chord.modifier_codes(),
            "keyCode": chord.key_code(),
            "label": chord.key_name(),
        })),
        CaptureUpdate::Cancelled(message) => {
            Some(serde_json::json!({"type": "cancel", "message": message}))
        }
    }
}

fn check_gestures() -> Result<(), String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|err| err.to_string())?;
    parse_config(&text).map(|_| ())
}

fn capture_stroke(trigger_name: &str) -> Result<(), String> {
    let requested = TriggerButton::parse(trigger_name)
        .ok_or_else(|| format!("unknown trigger {trigger_name}"))?;
    let trigger = saved_trigger(requested);
    finish_sample(
        take_sample_from_demo(
            "capture",
            &format!(
                "按住{}，在屏幕上画一笔，然后松开。",
                trigger_phrase(trigger)
            ),
            "8 秒内没有画完一笔",
        ),
        || capture_stroke_locally(requested),
    )
}

fn capture_button(trigger_name: &str) -> Result<(), String> {
    if TriggerButton::parse(trigger_name).is_none() {
        return Err(format!("unknown trigger {trigger_name}"));
    }
    finish_sample(
        take_sample_from_demo(
            "capture-button",
            "先按住起始键，再按另一个。左键、右键、中键或侧键都可以。",
            "8 秒内没有按到另一个鼠标键",
        ),
        capture_button_locally,
    )
}

fn saved_trigger(fallback: TriggerButton) -> TriggerButton {
    GestureConfig::load(&config_path())
        .map(|config| config.trigger)
        .unwrap_or(fallback)
}

fn finish_sample(
    result: Result<SampleAsk, String>,
    offline: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    match result {
        Ok(SampleAsk::Line(line)) => {
            println!("{line}");
            let _ = std::io::stdout().flush();
            Ok(())
        }
        Ok(SampleAsk::Offline) => offline(),
        Err(message) => {
            emit_capture_line(&serde_json::json!({"type": "cancel", "message": message}));
            Err(message)
        }
    }
}

enum SampleAsk {
    Offline,
    Line(String),
}

/// 演示已经抓住鼠标时，向它要下一笔或下一次组合，而不是再抢一次设备。
fn take_sample_from_demo(
    request: &str,
    status: &str,
    timeout_message: &str,
) -> Result<SampleAsk, String> {
    let mut stream = match UnixStream::connect(runtime_socket(current_uid())) {
        Ok(stream) => stream,
        Err(_) => return Ok(SampleAsk::Offline),
    };
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .map_err(|err| err.to_string())?;
    stream
        .write_all(format!("{{\"type\":\"{request}\"}}\n").as_bytes())
        .map_err(|err| err.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|err| err.to_string())?;
    let mut reader = BufReader::new(stream);
    let first = read_sample_line(&mut reader, "正在运行的演示没有回传。请先重新启动演示。")?;
    if first.contains("\"offline\"") {
        return Ok(SampleAsk::Offline);
    }
    if !first.contains("\"ready\"") {
        return Err("正在运行的演示没有回传。请先重新启动演示。".into());
    }
    emit_capture_line(&serde_json::json!({
        "type": "status",
        "message": status,
    }));
    reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(9)))
        .map_err(|err| err.to_string())?;
    Ok(SampleAsk::Line(read_sample_line(
        &mut reader,
        timeout_message,
    )?))
}

fn read_sample_line(reader: &mut BufReader<UnixStream>, timed_out: &str) -> Result<String, String> {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => Err("正在运行的演示没有回传轨迹。请先重新启动演示。".into()),
        Ok(_) => Ok(line.trim().to_string()),
        Err(err) if err.kind() == ErrorKind::TimedOut || err.kind() == ErrorKind::WouldBlock => {
            Err(timed_out.to_string())
        }
        Err(err) => Err(err.to_string()),
    }
}

fn capture_stroke_locally(trigger: TriggerButton) -> Result<(), String> {
    let mut grabbed = match grab_mice() {
        Ok(devices) => devices,
        Err(message) => {
            emit_capture_line(&serde_json::json!({"type": "error", "message": message}));
            return Err(message);
        }
    };
    emit_capture_line(&serde_json::json!({
        "type": "status",
        "message": format!("鼠标已暂时独占。按住{}，画一笔后松开。", trigger_phrase(trigger)),
    }));
    let value = read_stroke(&mut grabbed, trigger.evdev_code());
    drop(grabbed);
    emit_capture_line(&value);
    if value.get("type").and_then(|kind| kind.as_str()) == Some("error") {
        return Err(value
            .get("message")
            .and_then(|message| message.as_str())
            .unwrap_or("capture failed")
            .to_string());
    }
    Ok(())
}

fn trigger_phrase(trigger: TriggerButton) -> &'static str {
    match trigger {
        TriggerButton::Left => "左键",
        TriggerButton::Right => "右键",
        TriggerButton::Middle => "中键",
        TriggerButton::Forward => "侧键前进",
        TriggerButton::Back => "侧键后退",
    }
}

fn capture_button_locally() -> Result<(), String> {
    let mut grabbed = match grab_mice() {
        Ok(devices) => devices,
        Err(message) => {
            emit_capture_line(&serde_json::json!({"type": "error", "message": message}));
            return Err(message);
        }
    };
    emit_capture_line(&serde_json::json!({
        "type": "status",
        "message": "鼠标已暂时独占。先按住起始键，再按另一个。",
    }));
    let value = read_button(&mut grabbed);
    drop(grabbed);
    emit_capture_line(&value);
    if value.get("type").and_then(|kind| kind.as_str()) == Some("error") {
        return Err(value
            .get("message")
            .and_then(|message| message.as_str())
            .unwrap_or("capture failed")
            .to_string());
    }
    Ok(())
}

fn read_button(devices: &mut [GrabbedDevice]) -> serde_json::Value {
    let started = Instant::now();
    let mut pressed_at: Option<Instant> = None;
    let mut hold: Option<u16> = None;
    loop {
        let origin = pressed_at.unwrap_or(started);
        let remaining = Duration::from_secs(5).saturating_sub(origin.elapsed());
        if remaining.is_zero() {
            let message = if hold.is_some() {
                "没有按到另一个鼠标键"
            } else {
                "5 秒内没有按下鼠标键"
            };
            return serde_json::json!({"type": "cancel", "message": message});
        }
        let mut fds: Vec<nix::libc::pollfd> = devices
            .iter()
            .map(|device| nix::libc::pollfd {
                fd: device.0.as_raw_fd(),
                events: nix::libc::POLLIN,
                revents: 0,
            })
            .collect();
        let timeout = i32::try_from(remaining.as_millis()).unwrap_or(i32::MAX);
        let ready =
            unsafe { nix::libc::poll(fds.as_mut_ptr(), fds.len() as nix::libc::nfds_t, timeout) };
        if ready < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return serde_json::json!({"type": "error", "message": err.to_string()});
        }
        if ready == 0 {
            continue;
        }
        for (index, slot) in fds.iter().enumerate() {
            if slot.revents & nix::libc::POLLIN == 0 {
                continue;
            }
            let events = match devices[index].0.fetch_events() {
                Ok(events) => events,
                Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
                Err(err) => {
                    return serde_json::json!({"type": "error", "message": err.to_string()});
                }
            };
            for event in events {
                if event.event_type() != EventType::KEY {
                    continue;
                }
                if event.value() == 1 {
                    if MouseButton::from_code(event.code()).is_none() {
                        return serde_json::json!({"type": "cancel", "message": "这个键不能当作组合"});
                    }
                    if hold.is_none() {
                        hold = Some(event.code());
                        pressed_at = Some(Instant::now());
                    } else if hold != Some(event.code()) {
                        let hold_name = MouseButton::from_code(hold.unwrap_or(event.code()))
                            .map(MouseButton::name)
                            .unwrap_or("left");
                        let press_name = MouseButton::from_code(event.code())
                            .map(MouseButton::name)
                            .unwrap_or("right");
                        return serde_json::json!({"type": "button", "hold": hold_name, "button": press_name});
                    }
                } else if event.value() == 0 && hold == Some(event.code()) {
                    return serde_json::json!({"type": "cancel", "message": "没有按到另一个鼠标键"});
                }
            }
        }
    }
}

fn first_side_request(bytes: &[u8]) -> Option<&'static str> {
    let text = String::from_utf8_lossy(bytes);
    if text.contains("\"capture-button\"") {
        Some("capture-button")
    } else if text.contains("\"capture\"") {
        Some("capture")
    } else if text.contains("\"reload\"") {
        Some("reload")
    } else {
        None
    }
}

fn grab_mice() -> Result<Vec<GrabbedDevice>, String> {
    let mut grabbed = Vec::new();
    let mut missed = false;
    for (_path, mut device) in evdev::enumerate() {
        if !is_physical_mouse(&device) {
            continue;
        }
        if set_nonblocking(device.as_raw_fd()).is_err() || device.grab().is_err() {
            missed = true;
            continue;
        }
        grabbed.push(GrabbedDevice(device));
    }
    if grabbed.is_empty() || missed {
        return Err("鼠标被占用。请先在面板上暂停 Strokelet。".into());
    }
    Ok(grabbed)
}

fn is_physical_mouse(device: &Device) -> bool {
    let name = device.name().unwrap_or("").to_ascii_lowercase();
    if name.contains("strokelet") || name.contains("ydotool") {
        return false;
    }
    device.supported_relative_axes().is_some_and(|axes| {
        axes.contains(RelativeAxisCode::REL_X) && axes.contains(RelativeAxisCode::REL_Y)
    })
}

fn read_stroke(devices: &mut [GrabbedDevice], trigger: u16) -> serde_json::Value {
    let started = Instant::now();
    let mut pressed_at: Option<Instant> = None;
    let mut down = false;
    let mut x = 0.0;
    let mut y = 0.0;
    let mut pending_x = 0.0;
    let mut pending_y = 0.0;
    let mut points = vec![(0.0, 0.0)];
    loop {
        let limit = if pressed_at.is_some() {
            Duration::from_millis(2500)
        } else {
            Duration::from_secs(5)
        };
        let origin = pressed_at.unwrap_or(started);
        let remaining = limit.saturating_sub(origin.elapsed());
        if remaining.is_zero() {
            let message = if down {
                "时间太长，这次没有保存"
            } else {
                "5 秒内没有按下触发键"
            };
            return serde_json::json!({"type": "cancel", "message": message});
        }
        let mut fds: Vec<nix::libc::pollfd> = devices
            .iter()
            .map(|device| nix::libc::pollfd {
                fd: device.0.as_raw_fd(),
                events: nix::libc::POLLIN,
                revents: 0,
            })
            .collect();
        let timeout = i32::try_from(remaining.as_millis()).unwrap_or(i32::MAX);
        let ready =
            unsafe { nix::libc::poll(fds.as_mut_ptr(), fds.len() as nix::libc::nfds_t, timeout) };
        if ready < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return serde_json::json!({"type": "error", "message": err.to_string()});
        }
        if ready == 0 {
            continue;
        }
        for (index, slot) in fds.iter().enumerate() {
            if slot.revents & nix::libc::POLLIN == 0 {
                continue;
            }
            let events = match devices[index].0.fetch_events() {
                Ok(events) => events,
                Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
                Err(err) => {
                    return serde_json::json!({"type": "error", "message": err.to_string()});
                }
            };
            for event in events {
                if event.event_type() == EventType::RELATIVE && down {
                    if event.code() == RelativeAxisCode::REL_X.0 {
                        pending_x += f64::from(event.value());
                    } else if event.code() == RelativeAxisCode::REL_Y.0 {
                        pending_y += f64::from(event.value());
                    }
                } else if event.event_type() == EventType::SYNCHRONIZATION
                    && event.code() == SynchronizationCode::SYN_REPORT.0
                    && down
                {
                    if pending_x != 0.0 || pending_y != 0.0 {
                        x += pending_x;
                        y += pending_y;
                        if points.len() < 2048 {
                            points.push((x, y));
                        }
                        pending_x = 0.0;
                        pending_y = 0.0;
                    }
                } else if event.event_type() == EventType::KEY && event.code() == trigger {
                    if event.value() == 1 && !down {
                        down = true;
                        pressed_at = Some(Instant::now());
                        x = 0.0;
                        y = 0.0;
                        points = vec![(0.0, 0.0)];
                    } else if event.value() == 0 && down {
                        let length = points.windows(2).fold(0.0, |total, pair| {
                            total + (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1)
                        });
                        if length < 80.0 || points.len() < 2 {
                            return serde_json::json!({"type": "cancel", "message": "轨迹太短"});
                        }
                        return serde_json::json!({"type": "stroke", "points": points});
                    }
                }
            }
        }
    }
}

fn emit_capture_line(value: &serde_json::Value) {
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{value}");
    let _ = stdout.flush();
}

fn print_help() {
    println!(
        "\
strokelet: demo

Commands:
  status
  list-devices
  settings
  capture-chord
  capture-stroke [right|middle|forward|back]
  capture-button
  check-gestures
  run --auto
  run --device PATH --uid UID --session ID [--timeout-seconds 120]
      [--passthrough-only] [--inject-copy]

--auto finds the Wayland session and the mouse, injects shortcuts, and
runs until stopped. --timeout-seconds 0 also runs until stopped.
Grabbing waits until the extension is ready, the session is active and
unlocked, and no keys are held. The default timeout without --auto is
120 seconds.
capture-chord exclusively grabs keyboards until one shortcut is recorded.
The settings window starts it; Ctrl+C there cannot reach the shell until it exits."
    );
}

fn list_devices() -> Result<(), String> {
    let mut mice = accepted_mice();
    mice.sort_by(|left, right| left.path.cmp(&right.path));
    if mice.is_empty() {
        eprintln!("strokelet: no relative mouse could be opened; check /dev/input permissions");
    }
    for mouse in &mice {
        println!("accepted\t{}\tname={}", mouse.path, mouse.name);
    }
    for (path, device) in evdev::enumerate() {
        let profile = profile_of(&device);
        if let Err(reason) = classify_device(&profile) {
            println!(
                "rejected {reason:?}\t{}\tname={}",
                path.display(),
                profile.name
            );
        }
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
    timeout: Option<Duration>,
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
            OutputEvent::Key { code, down } => key_event(KeyCode(code), if down { 1 } else { 0 }),
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
    arm_stop_signals();
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
    let mut rules = GestureConfig::load(&config_path()).map_err(|err| err.to_string())?;
    if opts.passthrough_only {
        eprintln!("strokelet: passthrough only; Ctrl+C stays disarmed");
    } else if !opts.inject_copy {
        eprintln!("strokelet: copy injection is off; pass --inject-copy to arm Ctrl+C");
    }
    match opts.timeout {
        Some(limit) => eprintln!(
            "strokelet: listening for the extension; timeout {}s",
            limit.as_secs()
        ),
        None => eprintln!("strokelet: listening for the extension until stopped"),
    }
    let drive_result = drive(
        opts,
        listener,
        &mut device,
        &mut mouse,
        &mut keyboard,
        &mut rules,
    );
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
    rules: &mut GestureConfig,
) -> Result<(), String> {
    listener
        .set_nonblocking(true)
        .map_err(|err| err.to_string())?;
    let mut client: Option<UnixStream> = None;
    let mut codec = LineCodec::new();
    let mut link = Link::new();
    let mut processor = FrameProcessor::new(Limits::default());
    apply_rules(&mut processor, rules);
    let mut ledger = InjectionLedger::new();
    let mut gesture_id = 1u64;
    let mut grabbed = false;
    let mut sample: Option<SampleWait> = None;
    let started = Instant::now();
    while opts.timeout.is_none_or(|limit| started.elapsed() < limit) {
        if STOP.load(Ordering::Relaxed) {
            eprintln!("strokelet: stopping");
            break;
        }
        let now_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        match listener.accept() {
            Ok((mut stream, _)) if peer_is_target(&stream, opts.uid) => {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(80)));
                let mut buf = [0u8; 1024];
                let early = stream.read(&mut buf);
                let _ = stream.set_nonblocking(true);
                match early
                    .ok()
                    .filter(|size| *size > 0)
                    .and_then(|size| first_side_request(&buf[..size]))
                {
                    Some(request @ ("capture" | "capture-button")) if grabbed => {
                        if stream.write_all(b"{\"type\":\"ready\"}\n").is_ok() {
                            reply_sample(
                                &mut sample,
                                &serde_json::json!({"type": "cancel", "message": "已开始另一次录制"}),
                            );
                            sample = Some(SampleWait {
                                stream,
                                started: Instant::now(),
                                kind: if request == "capture-button" {
                                    SampleKind::Button
                                } else {
                                    SampleKind::Stroke
                                },
                            });
                            eprintln!("strokelet: waiting for one sample");
                        }
                    }
                    Some("capture" | "capture-button") => {
                        let _ = stream.write_all(b"{\"type\":\"offline\"}\n");
                    }
                    Some("reload") => match GestureConfig::load(&config_path()) {
                        Ok(next) => {
                            *rules = next;
                            apply_rules(&mut processor, rules);
                            eprintln!("strokelet: reloaded gesture rules");
                        }
                        Err(err) => eprintln!("strokelet: reload rejected: {err}"),
                    },
                    _ if client.is_none() => {
                        let _ = (&stream).write_all(Link::hello().as_bytes());
                        link = Link::new();
                        link.connected(now_ms);
                        codec = LineCodec::new();
                        client = Some(stream);
                    }
                    _ => {}
                }
            }
            Ok((stream, _)) => drop(stream),
            Err(err) if err.kind() == ErrorKind::WouldBlock => {}
            Err(err) => return Err(err.to_string()),
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
                        match link.ingest(&line, now_ms) {
                            Ok(strokelet::ClientUpdate::Reload) => {
                                match GestureConfig::load(&config_path()) {
                                    Ok(next) => {
                                        *rules = next;
                                        apply_rules(&mut processor, rules);
                                        eprintln!("strokelet: reloaded gesture rules");
                                    }
                                    Err(err) => eprintln!("strokelet: reload rejected: {err}"),
                                }
                            }
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("strokelet: protocol {err:?}");
                                link.disconnect();
                                client = None;
                            }
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
        poll_sample(&mut sample);
        let facts = read_session(&opts.session, opts.uid);
        let session_ok = is_active_unlocked(facts);
        let hold = !session_ok || link.is_paused();
        if hold && grabbed {
            let _ = device.ungrab();
            grabbed = false;
            reply_sample(
                &mut sample,
                &serde_json::json!({"type": "cancel", "message": "演示放开了鼠标，这次没有保存"}),
            );
            eprintln!("strokelet: session or pause released the mouse");
        }
        if !grabbed && grab_allowed(link.is_ready() && !link.is_paused(), session_ok, false) {
            device.grab().map_err(|err| format!("grab failed: {err}"))?;
            grabbed = true;
            eprintln!("strokelet: grabbed {}", opts.device.display());
        }
        processor.set_button_capture(
            sample
                .as_ref()
                .is_some_and(|wait| wait.kind == SampleKind::Button),
        );
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
                                rules,
                                gesture_id: &mut gesture_id,
                                sample: &mut sample,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum SampleKind {
    Stroke,
    Button,
}

struct SampleWait {
    stream: UnixStream,
    started: Instant,
    kind: SampleKind,
}

struct Live<'a> {
    processor: &'a mut FrameProcessor,
    link: &'a mut Link,
    ledger: &'a mut InjectionLedger,
    mouse: &'a mut VirtualDevice,
    keyboard: &'a mut CopyOutput<UinputKeys>,
    client: &'a mut Option<UnixStream>,
    opts: &'a RunOpts,
    rules: &'a GestureConfig,
    gesture_id: &'a mut u64,
    sample: &'a mut Option<SampleWait>,
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
        let sampling = live.sample.is_some() && decision != Decision::None;
        if sampling {
            let kind = live
                .sample
                .as_ref()
                .map(|wait| wait.kind)
                .unwrap_or(SampleKind::Stroke);
            reply_sample(
                live.sample,
                &sample_payload(kind, decision, live.processor.stroke_points()),
            );
            live.processor.set_button_capture(false);
        }
        let mut screen_name = None;
        let outcome = if sampling {
            match decision {
                Decision::RightClick => Outcome::Click,
                _ => Outcome::Unmatched,
            }
        } else {
            match (decision, injection) {
                (Decision::RightClick, _) => Outcome::Click,
                (Decision::Stroke, Injection::CopyOnce) => {
                    let matched = live
                        .rules
                        .rule_for_points(live.processor.stroke_points())
                        .map(|rule| (rule.chord.clone(), rule.screen_name.clone()));
                    inject_matched(live.keyboard, matched, &mut screen_name)
                }
                (Decision::Button { hold, press }, Injection::CopyOnce) => {
                    let matched = live
                        .rules
                        .rule_for_button(hold, press)
                        .map(|rule| (rule.chord.clone(), rule.screen_name.clone()));
                    inject_matched(live.keyboard, matched, &mut screen_name)
                }
                _ => Outcome::Unmatched,
            }
        };
        if sampling {
            eprintln!("strokelet: decision={decision:?} outcome=Sampled");
        } else {
            eprintln!("strokelet: decision={decision:?} outcome={outcome:?}");
        }
        let line = if outcome == Outcome::Unmatched && decision != Decision::RightClick {
            live.link.cancel(id, CancelReason::Unmatched)
        } else {
            live.link.end_with_name(id, outcome, screen_name.as_deref())
        };
        if let Ok(line) = line {
            eprintln!("strokelet: {line}");
            send_line(live.client, &line);
        }
    }
    Ok(())
}

fn poll_sample(sample: &mut Option<SampleWait>) {
    let disconnected = sample.as_mut().is_some_and(|wait| {
        let mut buf = [0u8; 64];
        match wait.stream.read(&mut buf) {
            Ok(0) => true,
            Err(err)
                if err.kind() != ErrorKind::WouldBlock && err.kind() != ErrorKind::TimedOut =>
            {
                true
            }
            _ => false,
        }
    });
    if disconnected {
        sample.take();
        eprintln!("strokelet: sample stroke cancelled");
        return;
    }
    if sample
        .as_ref()
        .is_some_and(|wait| wait.started.elapsed() >= Duration::from_secs(8))
    {
        let message = match sample.as_ref().map(|wait| wait.kind) {
            Some(SampleKind::Button) => "8 秒内没有按到另一个鼠标键",
            _ => "8 秒内没有画完一笔",
        };
        reply_sample(
            sample,
            &serde_json::json!({"type": "cancel", "message": message}),
        );
        eprintln!("strokelet: sample stroke timed out");
    }
}

fn reply_sample(sample: &mut Option<SampleWait>, value: &serde_json::Value) {
    let Some(mut wait) = sample.take() else {
        return;
    };
    let _ = wait.stream.set_nonblocking(false);
    let _ = wait.stream.set_write_timeout(Some(Duration::from_secs(1)));
    let line = format!("{value}\n");
    let _ = wait.stream.write_all(line.as_bytes());
}

fn apply_rules(processor: &mut FrameProcessor, rules: &GestureConfig) {
    processor.set_trigger(rules.trigger.evdev_code());
    processor.set_button_chords(&rules.button_chords());
}

fn inject_matched(
    keyboard: &mut CopyOutput<UinputKeys>,
    matched: Option<(strokelet::Chord, String)>,
    screen_name: &mut Option<String>,
) -> Outcome {
    if let Some((chord, name)) = matched
        && keyboard.send_chord(&chord).is_ok()
    {
        if !name.is_empty() {
            *screen_name = Some(name);
        }
        Outcome::CopyInjected
    } else {
        let _ = keyboard.release_owned_keys();
        Outcome::Unmatched
    }
}

fn sample_payload(
    kind: SampleKind,
    decision: Decision,
    points: &[(f64, f64)],
) -> serde_json::Value {
    match (kind, decision) {
        (SampleKind::Button, Decision::Button { hold, press }) => {
            let hold_name = MouseButton::from_code(hold)
                .map(MouseButton::name)
                .unwrap_or("left");
            let press_name = MouseButton::from_code(press)
                .map(MouseButton::name)
                .unwrap_or("right");
            serde_json::json!({"type": "button", "hold": hold_name, "button": press_name})
        }
        (SampleKind::Button, _) => {
            serde_json::json!({"type": "cancel", "message": "没有按到另一个鼠标键"})
        }
        (_, Decision::Stroke) => serde_json::json!({"type": "stroke", "points": points}),
        (_, Decision::RightClick) => serde_json::json!({"type": "cancel", "message": "轨迹太短"}),
        _ => serde_json::json!({"type": "cancel", "message": "这次没有保存"}),
    }
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
    for code in chord_device_codes() {
        keys.insert(KeyCode(code));
    }
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
    let mut timeout = Some(120u64);
    let mut timeout_set = false;
    let mut passthrough_only = false;
    let mut inject_copy = false;
    let mut auto = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--auto" => auto = true,
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
                timeout_set = true;
                timeout = Some(
                    next_value(&mut iter, "--timeout-seconds")?
                        .parse()
                        .map_err(|_| "timeout must be a number".to_string())?,
                );
            }
            "--passthrough-only" => passthrough_only = true,
            "--inject-copy" => inject_copy = true,
            other => return Err(format!("unknown run option {other}")),
        }
    }
    if passthrough_only {
        inject_copy = false;
    } else if auto {
        inject_copy = true;
    }
    let uid = if let Some(uid) = uid {
        uid
    } else if auto {
        current_uid()
    } else {
        return Err("run requires --uid".into());
    };
    if auto && session.is_none() {
        session = Some(display_session(uid)?);
    }
    if auto {
        let kind = session_type(session.as_deref().unwrap_or_default())?;
        if kind != "wayland" {
            return Err(format!("图形会话不是 Wayland（{kind}）"));
        }
    }
    if auto && device.is_none() {
        device = Some(resolve_mouse()?);
    }
    let timeout = if auto && !timeout_set {
        None
    } else {
        match timeout {
            Some(0) | None => None,
            Some(seconds) => Some(Duration::from_secs(seconds)),
        }
    };
    Ok(RunOpts {
        device: device.ok_or("run requires --device")?,
        uid,
        session: session.ok_or("run requires --session")?,
        timeout,
        passthrough_only,
        inject_copy,
    })
}

fn settings_script() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("STROKELET_SETTINGS") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("设置窗口不在 {}", path.display()));
    }
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("settings/app.js");
    let installed = PathBuf::from("/usr/share/strokelet/settings/app.js");
    let from_source_tree = env::current_exe()
        .ok()
        .is_some_and(|path| path.components().any(|part| part.as_os_str() == "target"));
    if from_source_tree && source.is_file() {
        return Ok(source);
    }
    if installed.is_file() {
        return Ok(installed);
    }
    if source.is_file() {
        return Ok(source);
    }
    Err("找不到设置窗口".into())
}

fn display_session(uid: u32) -> Result<String, String> {
    let text = loginctl_value(&["show-user", &uid.to_string(), "-p", "Display", "--value"])?;
    if text.is_empty() {
        return Err("当前用户没有图形会话".into());
    }
    Ok(text)
}

fn session_type(session: &str) -> Result<String, String> {
    let text = loginctl_value(&["show-session", session, "-p", "Type", "--value"])?;
    if text.is_empty() {
        return Err(format!("读不到会话 {session} 的类型"));
    }
    Ok(text)
}

fn loginctl_value(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("loginctl")
        .args(args)
        .output()
        .map_err(|err| format!("loginctl: {err}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn resolve_mouse() -> Result<PathBuf, String> {
    let mice = accepted_mice();
    let saved = saved_device();
    if let Some(saved) = saved.as_deref() {
        let known = mice.iter().any(|mouse| mouse.matches(saved));
        if !known {
            eprintln!("strokelet: 保存的鼠标现在不可用，改为自动选择：{saved}");
        }
    }
    match pick_mouse(saved.as_deref(), &mice) {
        Ok(path) => {
            eprintln!("strokelet: using mouse {path}");
            Ok(PathBuf::from(path))
        }
        Err(PickError::Missing) => Err(
            "没有找到可用的鼠标。接上鼠标后再试，或把设备路径写入 ~/.config/strokelet/device"
                .into(),
        ),
        Err(PickError::Ambiguous(paths)) => {
            let lines = paths
                .iter()
                .map(|path| {
                    let name = mice
                        .iter()
                        .find(|mouse| mouse.path == *path)
                        .map(|mouse| mouse.name.as_str())
                        .unwrap_or("");
                    format!("{path}\t{name}")
                })
                .collect::<Vec<_>>()
                .join("\n");
            Err(format!(
                "找到多只鼠标，把要使用的路径写入 {}：\n{lines}",
                device_path().display()
            ))
        }
    }
}

fn saved_device() -> Option<String> {
    let text = fs::read_to_string(device_path()).ok()?;
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        None
    } else {
        Some(line.to_string())
    }
}

fn accepted_mice() -> Vec<ListedMouse> {
    let mut by_device = HashMap::<u64, ListedMouse>::new();
    for (path, device) in evdev::enumerate() {
        let profile = profile_of(&device);
        if classify_device(&profile).is_err() {
            continue;
        }
        let Some(rdev) = fs::metadata(&path).ok().map(|meta| meta.rdev()) else {
            continue;
        };
        let preferred = preferred_path(&path);
        let mouse = ListedMouse {
            path: preferred.display().to_string(),
            name: profile.name,
            aliases: vec![path.display().to_string(), preferred.display().to_string()],
        };
        match by_device.get(&rdev) {
            Some(existing) if existing.path.contains("-event-mouse") => {}
            _ => {
                by_device.insert(rdev, mouse);
            }
        }
    }
    by_device.into_values().collect()
}

fn preferred_path(event: &Path) -> PathBuf {
    let Ok(canon) = fs::canonicalize(event) else {
        return event.to_path_buf();
    };
    let mut found = Vec::new();
    if let Ok(entries) = fs::read_dir("/dev/input/by-id") {
        for entry in entries.flatten() {
            let link = entry.path();
            if fs::canonicalize(&link).ok().as_ref() == Some(&canon) {
                found.push(link);
            }
        }
    }
    if let Some(path) = found
        .iter()
        .find(|path| path.to_string_lossy().contains("-event-mouse"))
    {
        return path.clone();
    }
    found
        .into_iter()
        .next()
        .unwrap_or_else(|| event.to_path_buf())
}

fn next_value(iter: &mut std::slice::Iter<String>, flag: &str) -> Result<String, String> {
    iter.next()
        .cloned()
        .ok_or_else(|| format!("{flag} needs a value"))
}

#[cfg(test)]
mod tests {
    use super::first_side_request;

    #[test]
    fn capture_and_reload_stay_off_the_extension_socket() {
        assert_eq!(
            first_side_request(br#"{"type":"capture-button"}"#),
            Some("capture-button")
        );
        assert_eq!(
            first_side_request(br#"{"type":"capture"}"#),
            Some("capture")
        );
        assert_eq!(first_side_request(br#"{"type":"reload"}"#), Some("reload"));
        assert_eq!(first_side_request(br#"{"type":"ready","version":1}"#), None);
    }
}
