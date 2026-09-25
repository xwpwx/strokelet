use evdev::{EventType, KeyCode, RelativeAxisCode};
use strokelet::{
    Decision, DeviceProfile, DeviceReject, Direction, FrameProcessor, Limits, classify_device,
    event_tuple, grab_allowed, key_event, rel_event, syn_dropped, syn_report, virtual_mouse_codes,
};

fn processor() -> FrameProcessor {
    FrameProcessor::new(Limits::default())
}

fn feed(
    proc: &mut FrameProcessor,
    events: &[evdev::InputEvent],
    now_ms: u64,
) -> Vec<(u16, u16, i32)> {
    events
        .iter()
        .flat_map(|event| proc.handle(*event, now_ms))
        .map(|event| event_tuple(&event))
        .collect()
}

fn view_key(code: KeyCode, value: i32) -> (u16, u16, i32) {
    (EventType::KEY.0, code.0, value)
}

fn view_rel(code: RelativeAxisCode, value: i32) -> (u16, u16, i32) {
    (EventType::RELATIVE.0, code.0, value)
}

fn view_syn() -> (u16, u16, i32) {
    event_tuple(&syn_report())
}

#[test]
fn passthrough_preserves_order_and_frame_boundary() {
    let mut proc = processor();
    let out = feed(
        &mut proc,
        &[
            rel_event(RelativeAxisCode::REL_X, 4),
            key_event(KeyCode::BTN_LEFT, 1),
            rel_event(RelativeAxisCode::REL_WHEEL, -1),
            syn_report(),
        ],
        10,
    );
    assert_eq!(
        out,
        vec![
            view_rel(RelativeAxisCode::REL_X, 4),
            view_key(KeyCode::BTN_LEFT, 1),
            view_rel(RelativeAxisCode::REL_WHEEL, -1),
            view_syn(),
        ]
    );
    assert_eq!(proc.last_decision(), None);
}

#[test]
fn stationary_right_click_is_replayed_once() {
    let mut proc = processor();
    let press = feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 1), syn_report()],
        10,
    );
    assert!(press.is_empty());
    let release = feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 0), syn_report()],
        40,
    );
    assert_eq!(
        release,
        vec![
            view_key(KeyCode::BTN_RIGHT, 1),
            view_syn(),
            view_key(KeyCode::BTN_RIGHT, 0),
            view_syn(),
        ]
    );
    assert_eq!(proc.last_decision(), Some(Decision::RightClick));
}

#[test]
fn upward_stroke_forwards_motion_without_right_button() {
    let mut proc = processor();
    feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 1), syn_report()],
        0,
    );
    let motion = feed(
        &mut proc,
        &[rel_event(RelativeAxisCode::REL_Y, -80), syn_report()],
        100,
    );
    assert_eq!(
        motion,
        vec![view_rel(RelativeAxisCode::REL_Y, -80), view_syn()]
    );
    let release = feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 0), syn_report()],
        120,
    );
    assert!(release.is_empty());
    assert_eq!(proc.last_decision(), Some(Decision::Stroke(Direction::Up)));
}

#[test]
fn long_running_demo_still_times_gesture_from_press() {
    let mut proc = processor();
    feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 1), syn_report()],
        10_000,
    );
    feed(
        &mut proc,
        &[rel_event(RelativeAxisCode::REL_Y, -80), syn_report()],
        10_100,
    );
    feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 0), syn_report()],
        10_120,
    );
    assert_eq!(proc.last_decision(), Some(Decision::Stroke(Direction::Up)));
}

#[test]
fn timeout_does_not_copy_or_replay_menu() {
    let mut proc = processor();
    feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 1), syn_report()],
        0,
    );
    let motion = feed(
        &mut proc,
        &[rel_event(RelativeAxisCode::REL_Y, -80), syn_report()],
        2501,
    );
    assert_eq!(
        motion,
        vec![view_rel(RelativeAxisCode::REL_Y, -80), view_syn()]
    );
    let release = feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 0), syn_report()],
        2501,
    );
    assert!(release.is_empty());
    assert_eq!(proc.last_decision(), Some(Decision::Cancel));
}

#[test]
fn wheel_or_other_button_cancels_and_is_forwarded() {
    let mut proc = processor();
    feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 1), syn_report()],
        0,
    );
    let wheel = feed(
        &mut proc,
        &[
            rel_event(RelativeAxisCode::REL_WHEEL_HI_RES, 120),
            syn_report(),
        ],
        30,
    );
    assert_eq!(
        wheel,
        vec![
            view_rel(RelativeAxisCode::REL_WHEEL_HI_RES, 120),
            view_syn()
        ]
    );
    let release = feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 0), syn_report()],
        40,
    );
    assert!(release.is_empty());
    assert_eq!(proc.last_decision(), Some(Decision::Cancel));

    let mut other = processor();
    feed(
        &mut other,
        &[key_event(KeyCode::BTN_RIGHT, 1), syn_report()],
        0,
    );
    let side = feed(
        &mut other,
        &[key_event(KeyCode::BTN_SIDE, 1), syn_report()],
        20,
    );
    assert_eq!(side, vec![view_key(KeyCode::BTN_SIDE, 1), view_syn()]);
    feed(
        &mut other,
        &[key_event(KeyCode::BTN_RIGHT, 0), syn_report()],
        30,
    );
    assert_eq!(other.last_decision(), Some(Decision::Cancel));
}

#[test]
fn syn_dropped_cancels_without_copy() {
    let mut proc = processor();
    feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 1), syn_report()],
        0,
    );
    let dropped = feed(
        &mut proc,
        &[rel_event(RelativeAxisCode::REL_Y, -80), syn_dropped()],
        50,
    );
    assert_eq!(dropped, vec![event_tuple(&syn_dropped())]);
    let release = feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 0), syn_report()],
        60,
    );
    assert!(release.is_empty());
    assert_eq!(proc.last_decision(), Some(Decision::Cancel));
}

#[test]
fn merged_frame_counts_as_one_vector() {
    let mut proc = processor();
    feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 1), syn_report()],
        0,
    );
    feed(
        &mut proc,
        &[
            rel_event(RelativeAxisCode::REL_X, 20),
            rel_event(RelativeAxisCode::REL_Y, -100),
            syn_report(),
        ],
        40,
    );
    feed(
        &mut proc,
        &[key_event(KeyCode::BTN_RIGHT, 0), syn_report()],
        50,
    );
    assert_eq!(proc.last_decision(), Some(Decision::Stroke(Direction::Up)));
}

#[test]
fn device_filter_and_grab_policy() {
    let mouse = DeviceProfile {
        name: "Logitech".into(),
        rel: vec![
            RelativeAxisCode::REL_X.0,
            RelativeAxisCode::REL_Y.0,
            RelativeAxisCode::REL_WHEEL.0,
            RelativeAxisCode::REL_WHEEL_HI_RES.0,
        ],
        keys: vec![
            KeyCode::BTN_LEFT.0,
            KeyCode::BTN_RIGHT.0,
            KeyCode::BTN_SIDE.0,
        ],
        abs: vec![],
    };
    assert_eq!(classify_device(&mouse), Ok(()));
    let (rel, keys) = virtual_mouse_codes(&mouse);
    assert!(rel.contains(&RelativeAxisCode::REL_WHEEL_HI_RES.0));
    assert!(!rel.contains(&RelativeAxisCode::REL_HWHEEL.0));
    assert!(keys.contains(&KeyCode::BTN_SIDE.0));
    assert!(!keys.contains(&KeyCode::BTN_MIDDLE.0));
    assert_eq!(
        classify_device(&DeviceProfile {
            name: "strokelet virtual mouse".into(),
            ..mouse.clone()
        }),
        Err(DeviceReject::Virtual)
    );
    assert_eq!(
        classify_device(&DeviceProfile {
            name: "touch".into(),
            rel: vec![],
            keys: vec![],
            abs: vec![0, 1],
        }),
        Err(DeviceReject::Absolute)
    );
    assert!(!grab_allowed(false, true, false));
    assert!(!grab_allowed(true, false, false));
    assert!(!grab_allowed(true, true, true));
    assert!(grab_allowed(true, true, false));
}
