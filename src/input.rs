use evdev::{
    AbsoluteAxisCode, EventType, InputEvent, KeyCode, RelativeAxisCode, SynchronizationCode,
};

use crate::config::MouseButton;
use crate::{Decision, Gesture, Limits, WheelDirection};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedMouse {
    pub path: String,
    pub name: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickError {
    Missing,
    Ambiguous(Vec<String>),
}

/// 保存的路径优先。多只鼠标时，只有一条稳定的 `event-mouse` 路径才自动选中。
pub fn pick_mouse(saved: Option<&str>, mice: &[ListedMouse]) -> Result<String, PickError> {
    let saved = saved.map(str::trim).filter(|value| !value.is_empty());
    if let Some(saved) = saved
        && let Some(mouse) = mice.iter().find(|mouse| mouse.matches(saved))
    {
        return Ok(mouse.path.clone());
    }
    let preferred: Vec<&ListedMouse> = mice
        .iter()
        .filter(|mouse| mouse.path.contains("-event-mouse"))
        .collect();
    let pool: Vec<&ListedMouse> = if preferred.is_empty() {
        mice.iter().collect()
    } else {
        preferred
    };
    match pool.as_slice() {
        [] => Err(PickError::Missing),
        [only] => Ok(only.path.clone()),
        many => {
            let mut paths: Vec<String> = many.iter().map(|mouse| mouse.path.clone()).collect();
            paths.sort();
            Err(PickError::Ambiguous(paths))
        }
    }
}

impl ListedMouse {
    pub fn matches(&self, saved: &str) -> bool {
        self.path == saved || self.aliases.iter().any(|alias| alias == saved)
    }
}

pub fn classify_device(profile: &DeviceProfile) -> Result<(), DeviceReject> {
    let name = profile.name.to_ascii_lowercase();
    if name.contains("strokelet") || name.contains("ydotool") || name.contains("virtual") {
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
    pressed_at_ms: u64,
    trigger_code: u16,
    last_decision: Option<Decision>,
    click_slop: f64,
    chord_pairs: Vec<(u16, u16)>,
    wheel_rules: Vec<(u16, WheelDirection)>,
    button_capture: bool,
    wheel_capture: bool,
    fired_chord: Option<(u16, u16)>,
    consumed: bool,
    notch: u32,
    suppress_echo: Option<WheelDirection>,
    swallow: Option<u16>,
    pending_hold: Option<u16>,
    hold_x: f64,
    hold_y: f64,
    hold_forwarded: bool,
}

impl FrameProcessor {
    pub fn new(limits: Limits) -> Self {
        Self {
            gesture: Gesture::new(limits),
            tracking: false,
            frame: Vec::new(),
            now_ms: 0,
            pressed_at_ms: 0,
            trigger_code: KeyCode::BTN_RIGHT.0,
            last_decision: None,
            click_slop: limits.click_slop_counts,
            chord_pairs: Vec::new(),
            wheel_rules: Vec::new(),
            button_capture: false,
            wheel_capture: false,
            fired_chord: None,
            consumed: false,
            notch: 0,
            suppress_echo: None,
            swallow: None,
            pending_hold: None,
            hold_x: 0.0,
            hold_y: 0.0,
            hold_forwarded: false,
        }
    }

    pub fn set_button_chords(&mut self, pairs: &[(u16, u16)]) {
        self.chord_pairs = pairs.to_vec();
    }

    pub fn set_wheel_rules(&mut self, rules: &[(u16, WheelDirection)]) {
        self.wheel_rules = rules.to_vec();
    }

    /// 录制组合键时，先按下的鼠标键是起始键，不限于当前轨迹触发键。
    pub fn set_button_capture(&mut self, enabled: bool) {
        self.button_capture = enabled;
    }

    /// 录制滚轮时，先按下的鼠标键是起始键，下一格滚轮完成这次录制。
    pub fn set_wheel_capture(&mut self, enabled: bool) {
        self.wheel_capture = enabled;
    }

    pub fn last_decision(&self) -> Option<Decision> {
        self.last_decision
    }

    pub fn stroke_points(&self) -> &[(f64, f64)] {
        self.gesture.points()
    }

    pub fn is_tracking(&self) -> bool {
        self.tracking
    }

    pub fn set_limits(&mut self, limits: crate::Limits) {
        self.click_slop = limits.click_slop_counts;
        self.gesture.set_limits(limits);
    }

    pub fn set_trigger(&mut self, code: u16) {
        if code != self.trigger_code && self.tracking {
            self.gesture.abandon();
            self.tracking = false;
            self.fired_chord = None;
            self.consumed = false;
            self.suppress_echo = None;
            self.swallow = None;
            self.pending_hold = None;
            self.hold_forwarded = false;
        }
        self.trigger_code = code;
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
        let mut wheel_y = 0;
        let mut wheel_x = 0;
        let mut wheel_y_hi = 0;
        let mut wheel_x_hi = 0;
        let mut right_down = false;
        let mut right_up = false;
        let mut cancel_now = false;
        let mut extra_down = None;
        let mut extra_up = None;
        for event in &frame {
            match self.classify_part(event) {
                Part::RelX(value) => dx += value,
                Part::RelY(value) => dy += value,
                Part::WheelY(value) => wheel_y += value,
                Part::WheelX(value) => wheel_x += value,
                Part::WheelYHi(value) => wheel_y_hi += value,
                Part::WheelXHi(value) => wheel_x_hi += value,
                Part::OtherKey => cancel_now = true,
                Part::RightDown => right_down = true,
                Part::RightUp => right_up = true,
                Part::ExtraDown(code) => extra_down = Some(code),
                Part::ExtraUp(code) => extra_up = Some(code),
                Part::Other => {}
            }
        }
        if !self.tracking
            && self.pending_hold.is_none()
            && self.fired_chord.is_none()
            && let Some(code) = extra_down.filter(|code| self.is_separate_hold(*code))
        {
            self.pending_hold = Some(code);
            self.hold_x = 0.0;
            self.hold_y = 0.0;
            self.hold_forwarded = false;
            self.last_decision = None;
        }
        let trigger_code = self.trigger_code;
        let trigger_completes_hold = right_down
            && self.fired_chord.is_none()
            && self
                .pending_hold
                .is_some_and(|hold| self.claims_pair(hold, trigger_code));
        let mut decided_this_frame = false;
        let capture_hold = self.button_capture || self.wheel_capture;
        if capture_hold
            && right_down
            && !self.tracking
            && self.pending_hold.is_none()
            && self.fired_chord.is_none()
        {
            self.pending_hold = Some(trigger_code);
            self.hold_x = 0.0;
            self.hold_y = 0.0;
            self.hold_forwarded = false;
            self.last_decision = None;
        } else if trigger_completes_hold {
            if let Some(hold) = self.pending_hold {
                self.fire_pair(hold, trigger_code);
                decided_this_frame = true;
            }
        } else if right_down && !self.tracking {
            self.gesture.press();
            self.tracking = true;
            self.pressed_at_ms = self.now_ms;
            self.last_decision = None;
            self.fired_chord = None;
            self.consumed = false;
        }
        let held_ms = self.now_ms.saturating_sub(self.pressed_at_ms);
        if self.tracking && (dx != 0 || dy != 0) {
            self.gesture.motion(f64::from(dx), f64::from(dy));
        }
        if self.pending_hold.is_some() && (dx != 0 || dy != 0) {
            self.hold_x += f64::from(dx);
            self.hold_y += f64::from(dy);
        }
        if self.tracking {
            self.gesture.advance(held_ms);
        }
        if self.fired_chord.is_none()
            && let Some(hold) = self.pending_hold
            && let Some(press) =
                extra_down.filter(|code| *code != hold && self.claims_pair(hold, *code))
        {
            self.fire_pair(hold, press);
            decided_this_frame = true;
        }
        let claimed = self.tracking
            && self.fired_chord.is_none()
            && extra_down.is_some_and(|code| self.claims_pair(trigger_code, code));
        let mut swallow_wheel = false;
        if let Some((direction, hi_only)) =
            wheel_notch(wheel_y, wheel_x, wheel_y_hi, wheel_x_hi)
        {
            let echo = self.suppress_echo == Some(direction)
                && !hi_only
                && wheel_y_hi == 0
                && wheel_x_hi == 0;
            if echo {
                swallow_wheel = self.consumed;
                self.suppress_echo = None;
            } else if let Some(hold) = self.active_hold()
                && self.claims_wheel(hold, direction)
            {
                self.fire_wheel(hold, direction);
                decided_this_frame = true;
                swallow_wheel = true;
                self.suppress_echo = hi_only.then_some(direction);
            } else {
                self.suppress_echo = None;
                cancel_now = true;
            }
        } else {
            self.suppress_echo = None;
        }
        if self.tracking && cancel_now {
            self.gesture.cancel();
        } else if let Some(code) = extra_down.filter(|_| claimed) {
            self.fire_pair(trigger_code, code);
            decided_this_frame = true;
        } else if self.tracking && extra_down.is_some() && self.pending_hold != extra_down {
            self.gesture.cancel();
        }
        let started_drag = !capture_hold
            && self.fired_chord.is_none()
            && !self.hold_forwarded
            && self.pending_hold.is_some()
            && self.hold_x.hypot(self.hold_y) > self.click_slop;
        let already_forwarded = self.hold_forwarded;
        if started_drag {
            self.hold_forwarded = true;
        }
        let hide_press = if self.fired_chord.is_some() {
            self.fired_chord.map(|pair| pair.1).or(self.swallow)
        } else {
            self.swallow
        };
        let hide_hold = self.pending_hold.filter(|_| !already_forwarded);
        let mut out = Vec::new();
        if started_drag && let Some(hold) = self.pending_hold {
            out.extend([key_event(KeyCode(hold), 1), syn_report()]);
        }
        let forwarded: Vec<_> = frame
            .iter()
            .copied()
            .filter(|event| {
                !is_trigger_button(event, trigger_code)
                    && !is_hidden(event, hide_press)
                    && !is_hidden(event, hide_hold)
                    && !(swallow_wheel && is_wheel_event(event))
            })
            .collect();
        if !forwarded.is_empty() {
            out.extend(forwarded);
            out.push(syn_report());
        }
        if decided_this_frame {
            self.swallow = self.fired_chord.map(|pair| pair.1);
        }
        if extra_up.is_some() && extra_up == self.swallow {
            self.swallow = None;
        }
        if decided_this_frame && self.hold_forwarded {
            if let Some(hold) = self.pending_hold {
                out.extend([key_event(KeyCode(hold), 0), syn_report()]);
            }
            self.hold_forwarded = false;
        }
        let released_hold = self
            .pending_hold
            .filter(|hold| (*hold == trigger_code && right_up) || extra_up == Some(*hold));
        if let Some(hold) = released_hold {
            self.pending_hold = None;
            if self.fired_chord.is_some() || self.consumed {
                self.fired_chord = None;
                self.consumed = false;
                self.suppress_echo = None;
                if !decided_this_frame {
                    self.last_decision = None;
                }
            } else if self.hold_forwarded {
                self.hold_forwarded = false;
            } else if capture_hold {
                self.last_decision = Some(Decision::Cancel);
            } else {
                out.extend(replay_click(hold));
            }
        }
        if right_up && self.tracking {
            if self.fired_chord.take().is_some() || self.consumed {
                let _ = self.gesture.release(held_ms);
                self.tracking = false;
                self.consumed = false;
                self.suppress_echo = None;
                if !decided_this_frame {
                    self.last_decision = None;
                }
            } else {
                let decision = self.gesture.release(held_ms);
                self.tracking = false;
                self.last_decision = Some(decision);
                if decision == Decision::RightClick {
                    out.extend(replay_click(trigger_code));
                }
            }
        }
        out
    }

    fn fire_pair(&mut self, hold: u16, press: u16) {
        self.fired_chord = Some((hold, press));
        self.consumed = true;
        self.swallow = Some(press);
        if self.tracking {
            self.gesture.cancel();
        }
        self.last_decision = Some(Decision::Button { hold, press });
    }

    fn fire_wheel(&mut self, hold: u16, direction: WheelDirection) {
        self.consumed = true;
        self.notch = self.notch.wrapping_add(1);
        if self.tracking {
            self.gesture.cancel();
        }
        self.last_decision = Some(Decision::Wheel {
            hold,
            direction,
            notch: self.notch,
        });
    }

    fn active_hold(&self) -> Option<u16> {
        self.pending_hold
            .or_else(|| self.tracking.then_some(self.trigger_code))
    }

    fn is_separate_hold(&self, code: u16) -> bool {
        if code == self.trigger_code || MouseButton::from_code(code).is_none() {
            return false;
        }
        self.button_capture
            || self.wheel_capture
            || self.chord_pairs.iter().any(|(hold, _)| *hold == code)
            || self.wheel_rules.iter().any(|(hold, _)| *hold == code)
    }

    fn claims_pair(&self, hold: u16, press: u16) -> bool {
        if hold == press {
            return false;
        }
        if self.button_capture
            && MouseButton::from_code(hold).is_some()
            && MouseButton::from_code(press).is_some()
        {
            return true;
        }
        self.chord_pairs.contains(&(hold, press))
    }

    fn claims_wheel(&self, hold: u16, direction: WheelDirection) -> bool {
        if self.wheel_capture && MouseButton::from_code(hold).is_some() {
            return true;
        }
        self.wheel_rules
            .iter()
            .any(|(rule_hold, rule_direction)| *rule_hold == hold && *rule_direction == direction)
    }
}

fn wheel_notch(y: i32, x: i32, y_hi: i32, x_hi: i32) -> Option<(WheelDirection, bool)> {
    let vertical = if y != 0 { y } else { y_hi };
    let horizontal = if x != 0 { x } else { x_hi };
    if vertical == 0 && horizontal == 0 {
        return None;
    }
    let use_vertical = vertical != 0 && (horizontal == 0 || vertical.abs() >= horizontal.abs());
    if use_vertical {
        let direction = if vertical > 0 {
            WheelDirection::Up
        } else {
            WheelDirection::Down
        };
        Some((direction, y == 0))
    } else {
        let direction = if horizontal > 0 {
            WheelDirection::Right
        } else {
            WheelDirection::Left
        };
        Some((direction, x == 0))
    }
}

fn replay_click(code: u16) -> [InputEvent; 4] {
    [
        key_event(KeyCode(code), 1),
        syn_report(),
        key_event(KeyCode(code), 0),
        syn_report(),
    ]
}

#[derive(Clone, Copy)]
enum Part {
    RelX(i32),
    RelY(i32),
    WheelY(i32),
    WheelX(i32),
    WheelYHi(i32),
    WheelXHi(i32),
    RightDown,
    RightUp,
    ExtraDown(u16),
    ExtraUp(u16),
    OtherKey,
    Other,
}

impl FrameProcessor {
    fn classify_part(&self, event: &InputEvent) -> Part {
        if event.event_type() == EventType::RELATIVE {
            return match event.code() {
                code if code == RelativeAxisCode::REL_X.0 => Part::RelX(event.value()),
                code if code == RelativeAxisCode::REL_Y.0 => Part::RelY(event.value()),
                code if code == RelativeAxisCode::REL_WHEEL.0 => Part::WheelY(event.value()),
                code if code == RelativeAxisCode::REL_HWHEEL.0 => Part::WheelX(event.value()),
                code if code == RelativeAxisCode::REL_WHEEL_HI_RES.0 => {
                    Part::WheelYHi(event.value())
                }
                code if code == RelativeAxisCode::REL_HWHEEL_HI_RES.0 => {
                    Part::WheelXHi(event.value())
                }
                _ => Part::Other,
            };
        }
        if event.event_type() == EventType::KEY && event.code() == self.trigger_code {
            return match event.value() {
                1 => Part::RightDown,
                0 => Part::RightUp,
                _ => Part::Other,
            };
        }
        if event.event_type() == EventType::KEY && MouseButton::from_code(event.code()).is_some() {
            return match event.value() {
                1 => Part::ExtraDown(event.code()),
                0 => Part::ExtraUp(event.code()),
                _ => Part::Other,
            };
        }
        if event.event_type() == EventType::KEY {
            return Part::OtherKey;
        }
        Part::Other
    }
}

fn is_hidden(event: &InputEvent, code: Option<u16>) -> bool {
    code.is_some_and(|code| event.event_type() == EventType::KEY && event.code() == code)
}

fn is_wheel_event(event: &InputEvent) -> bool {
    event.event_type() == EventType::RELATIVE && WHEEL_CODES.contains(&event.code())
}

fn is_trigger_button(event: &InputEvent, trigger_code: u16) -> bool {
    event.event_type() == EventType::KEY && event.code() == trigger_code
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

#[cfg(test)]
mod tests {
    use super::{ListedMouse, PickError, pick_mouse};

    fn mouse(path: &str, aliases: &[&str]) -> ListedMouse {
        ListedMouse {
            path: path.to_string(),
            name: path.to_string(),
            aliases: aliases.iter().map(|alias| (*alias).to_string()).collect(),
        }
    }

    #[test]
    fn saved_alias_wins_over_other_mice() {
        let mice = vec![
            mouse("/dev/input/by-id/usb-a-event-mouse", &["/dev/input/event3"]),
            mouse("/dev/input/by-id/usb-b-event-mouse", &["/dev/input/event5"]),
        ];
        let chosen = pick_mouse(Some("/dev/input/event5\n"), &mice).unwrap();
        assert_eq!(chosen, "/dev/input/by-id/usb-b-event-mouse");
    }

    #[test]
    fn one_stable_mouse_is_chosen_when_several_devices_exist() {
        let mice = vec![
            mouse("/dev/input/event3", &[]),
            mouse("/dev/input/by-id/usb-a-event-mouse", &["/dev/input/event8"]),
        ];
        let chosen = pick_mouse(None, &mice).unwrap();
        assert_eq!(chosen, "/dev/input/by-id/usb-a-event-mouse");
    }

    #[test]
    fn two_mice_need_an_explicit_choice() {
        let mice = vec![
            mouse("/dev/input/by-id/usb-b-event-mouse", &[]),
            mouse("/dev/input/by-id/usb-a-event-mouse", &[]),
        ];
        assert_eq!(
            pick_mouse(None, &mice).unwrap_err(),
            PickError::Ambiguous(vec![
                "/dev/input/by-id/usb-a-event-mouse".to_string(),
                "/dev/input/by-id/usb-b-event-mouse".to_string(),
            ])
        );
    }

    #[test]
    fn missing_saved_path_falls_back_to_the_only_mouse() {
        let mice = vec![mouse("/dev/input/event3", &[])];
        let chosen = pick_mouse(Some("/dev/input/event9"), &mice).unwrap();
        assert_eq!(chosen, "/dev/input/event3");
    }

    #[test]
    fn no_mouse_is_missing() {
        assert_eq!(pick_mouse(None, &[]).unwrap_err(), PickError::Missing);
    }
}
