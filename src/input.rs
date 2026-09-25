use evdev::{
    AbsoluteAxisCode, EventType, InputEvent, KeyCode, RelativeAxisCode, SynchronizationCode,
};

use crate::{Decision, Gesture, Limits};

const WHEEL_CODES: [u16; 4] = [
    RelativeAxisCode::REL_WHEEL.0,
    RelativeAxisCode::REL_HWHEEL.0,
    RelativeAxisCode::REL_WHEEL_HI_RES.0,
    RelativeAxisCode::REL_HWHEEL_HI_RES.0,
];

const MIRROR_REL: [u16; 6] = [
    RelativeAxisCode::REL_X.0,
    RelativeAxisCode::REL_Y.0,
    RelativeAxisCode::REL_WHEEL.0,
    RelativeAxisCode::REL_HWHEEL.0,
    RelativeAxisCode::REL_WHEEL_HI_RES.0,
    RelativeAxisCode::REL_HWHEEL_HI_RES.0,
];

const MIRROR_KEYS: [u16; 8] = [
    KeyCode::BTN_LEFT.0,
    KeyCode::BTN_RIGHT.0,
    KeyCode::BTN_MIDDLE.0,
    KeyCode::BTN_SIDE.0,
    KeyCode::BTN_EXTRA.0,
    KeyCode::BTN_FORWARD.0,
    KeyCode::BTN_BACK.0,
    KeyCode::BTN_TASK.0,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceProfile {
    pub name: String,
    pub rel: Vec<u16>,
    pub keys: Vec<u16>,
    pub abs: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceReject {
    Virtual,
    Absolute,
    NotRelative,
}

pub fn classify_device(profile: &DeviceProfile) -> Result<(), DeviceReject> {
    if profile.name.to_ascii_lowercase().contains("strokelet") {
        return Err(DeviceReject::Virtual);
    }
    let relative = profile.rel.contains(&RelativeAxisCode::REL_X.0)
        && profile.rel.contains(&RelativeAxisCode::REL_Y.0);
    let absolute = profile.abs.contains(&AbsoluteAxisCode::ABS_X.0)
        || profile.abs.contains(&AbsoluteAxisCode::ABS_Y.0);
    if absolute || !relative {
        return Err(if absolute {
            DeviceReject::Absolute
        } else {
            DeviceReject::NotRelative
        });
    }
    Ok(())
}

pub fn virtual_mouse_codes(profile: &DeviceProfile) -> (Vec<u16>, Vec<u16>) {
    let rel = MIRROR_REL
        .into_iter()
        .filter(|code| profile.rel.contains(code))
        .collect();
    let keys = MIRROR_KEYS
        .into_iter()
        .filter(|code| profile.keys.contains(code))
        .collect();
    (rel, keys)
}

pub fn grab_allowed(extension_ready: bool, session_ok: bool, any_key_down: bool) -> bool {
    extension_ready && session_ok && !any_key_down
}

pub fn key_event(code: KeyCode, value: i32) -> InputEvent {
    InputEvent::new(EventType::KEY.0, code.0, value)
}

pub fn rel_event(code: RelativeAxisCode, value: i32) -> InputEvent {
    InputEvent::new(EventType::RELATIVE.0, code.0, value)
}

pub fn syn_report() -> InputEvent {
    InputEvent::new(
        EventType::SYNCHRONIZATION.0,
        SynchronizationCode::SYN_REPORT.0,
        0,
    )
}

pub fn syn_dropped() -> InputEvent {
    InputEvent::new(
        EventType::SYNCHRONIZATION.0,
        SynchronizationCode::SYN_DROPPED.0,
        0,
    )
}

/// 以 SYN_REPORT 为帧边界。右键先暂存，普通事件原样转发。
#[derive(Debug)]
pub struct FrameProcessor {
    gesture: Gesture,
    tracking: bool,
    frame: Vec<InputEvent>,
    now_ms: u64,
    last_decision: Option<Decision>,
}

impl FrameProcessor {
    pub fn new(limits: Limits) -> Self {
        Self {
            gesture: Gesture::new(limits),
            tracking: false,
            frame: Vec::new(),
            now_ms: 0,
            last_decision: None,
        }
    }

    pub fn last_decision(&self) -> Option<Decision> {
        self.last_decision
    }

    pub fn is_tracking(&self) -> bool {
        self.tracking
    }

    pub fn handle(&mut self, event: InputEvent, now_ms: u64) -> Vec<InputEvent> {
        self.now_ms = now_ms;
        if is_dropped(&event) {
            self.frame.clear();
            if self.tracking {
                self.gesture.cancel();
            }
            return vec![event];
        }
        if is_report(&event) {
            return self.finish_frame();
        }
        self.frame.push(event);
        Vec::new()
    }

    fn finish_frame(&mut self) -> Vec<InputEvent> {
        let frame = std::mem::take(&mut self.frame);
        let mut dx = 0;
        let mut dy = 0;
        let mut right_down = false;
        let mut right_up = false;
        let mut cancel_now = false;
        for event in &frame {
            match classify_part(event) {
                Part::RelX(value) => dx += value,
                Part::RelY(value) => dy += value,
                Part::Wheel | Part::OtherKey => cancel_now = true,
                Part::RightDown => right_down = true,
                Part::RightUp => right_up = true,
                Part::Other => {}
            }
        }
        if right_down && !self.tracking {
            self.gesture.press();
            self.tracking = true;
            self.last_decision = None;
        }
        if self.tracking && (dx != 0 || dy != 0) {
            self.gesture.motion(f64::from(dx), f64::from(dy));
        }
        if self.tracking && cancel_now {
            self.gesture.cancel();
        }
        if self.tracking {
            self.gesture.advance(self.now_ms);
        }
        let mut out = Vec::new();
        let forwarded: Vec<_> = frame
            .iter()
            .copied()
            .filter(|event| !is_right_button(event))
            .collect();
        if !forwarded.is_empty() {
            out.extend(forwarded);
            out.push(syn_report());
        }
        if right_up && self.tracking {
            let decision = self.gesture.release(self.now_ms);
            self.tracking = false;
            self.last_decision = Some(decision);
            if decision == Decision::RightClick {
                out.extend(replay_right_click());
            }
        }
        out
    }
}

fn replay_right_click() -> [InputEvent; 4] {
    [
        key_event(KeyCode::BTN_RIGHT, 1),
        syn_report(),
        key_event(KeyCode::BTN_RIGHT, 0),
        syn_report(),
    ]
}

#[derive(Clone, Copy)]
enum Part {
    RelX(i32),
    RelY(i32),
    Wheel,
    RightDown,
    RightUp,
    OtherKey,
    Other,
}

fn classify_part(event: &InputEvent) -> Part {
    if event.event_type() == EventType::RELATIVE {
        return match event.code() {
            code if code == RelativeAxisCode::REL_X.0 => Part::RelX(event.value()),
            code if code == RelativeAxisCode::REL_Y.0 => Part::RelY(event.value()),
            code if WHEEL_CODES.contains(&code) => Part::Wheel,
            _ => Part::Other,
        };
    }
    if event.event_type() == EventType::KEY && event.code() == KeyCode::BTN_RIGHT.0 {
        return match event.value() {
            1 => Part::RightDown,
            0 => Part::RightUp,
            _ => Part::Other,
        };
    }
    if event.event_type() == EventType::KEY {
        return Part::OtherKey;
    }
    Part::Other
}

fn is_right_button(event: &InputEvent) -> bool {
    event.event_type() == EventType::KEY && event.code() == KeyCode::BTN_RIGHT.0
}

fn is_report(event: &InputEvent) -> bool {
    event.event_type() == EventType::SYNCHRONIZATION
        && event.code() == SynchronizationCode::SYN_REPORT.0
}

fn is_dropped(event: &InputEvent) -> bool {
    event.event_type() == EventType::SYNCHRONIZATION
        && event.code() == SynchronizationCode::SYN_DROPPED.0
}

pub fn event_tuple(event: &InputEvent) -> (u16, u16, i32) {
    (event.event_type().0, event.code(), event.value())
}
