use crate::Chord;
use crate::copy::{is_modifier, is_shortcut_key, modifier_label};

/// 设置窗口录制一条快捷键。只看按下和松开，不读设备。
#[derive(Debug)]
pub struct ChordCapture {
    held: Vec<u16>,
    modifiers: Vec<u16>,
    main: Option<u16>,
    latched: Vec<u16>,
    finished: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureUpdate {
    Live(String),
    Finished(Chord),
    Cancelled(String),
}

impl Default for ChordCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl ChordCapture {
    pub fn new() -> Self {
        Self {
            held: Vec::new(),
            modifiers: Vec::new(),
            main: None,
            latched: Vec::new(),
            finished: false,
        }
    }

    pub fn has_main(&self) -> bool {
        self.main.is_some()
    }

    pub fn down(&mut self, code: u16) -> Option<CaptureUpdate> {
        if self.finished || self.held.contains(&code) {
            return None;
        }
        self.held.push(code);
        if is_modifier(code) {
            if self.main.is_some() {
                return None;
            }
            if self.modifiers.len() >= 4 {
                return Some(self.cancel("修饰键最多 4 个"));
            }
            self.modifiers.push(code);
            self.modifiers.sort_unstable();
            return Some(CaptureUpdate::Live(self.preview()));
        }
        if !is_shortcut_key(code) {
            return Some(self.cancel("这个键不能绑定"));
        }
        if code == evdev::KeyCode::KEY_ESC.0 && self.modifiers.is_empty() {
            return Some(self.cancel("已取消"));
        }
        if self.main.is_some() {
            return Some(self.cancel("按下了两个主键，请再录一次"));
        }
        self.main = Some(code);
        self.latched = self.modifiers.clone();
        Some(CaptureUpdate::Live(self.preview()))
    }

    pub fn up(&mut self, code: u16) -> Option<CaptureUpdate> {
        if self.finished {
            return None;
        }
        self.held.retain(|held| *held != code);
        if self.main == Some(code) {
            let key = code;
            let label = self.preview();
            self.finished = true;
            return match Chord::from_codes(&self.latched, key, &label) {
                Ok(chord) => Some(CaptureUpdate::Finished(chord)),
                Err(message) => Some(CaptureUpdate::Cancelled(message)),
            };
        }
        if self.main.is_none() && self.modifiers.contains(&code) {
            self.modifiers.retain(|modifier| *modifier != code);
            return Some(CaptureUpdate::Live(self.preview()));
        }
        None
    }

    fn cancel(&mut self, message: &str) -> CaptureUpdate {
        self.finished = true;
        CaptureUpdate::Cancelled(message.to_string())
    }

    fn preview(&self) -> String {
        let modifiers = if self.main.is_some() {
            &self.latched
        } else {
            &self.modifiers
        };
        let mut parts: Vec<String> = modifiers
            .iter()
            .filter_map(|code| modifier_label(*code).map(str::to_string))
            .collect();
        if let Some(code) = self.main {
            parts.push(main_label(code));
        }
        parts.join("+")
    }
}

fn main_label(code: u16) -> String {
    let raw = format!("{:?}", evdev::KeyCode(code));
    match raw.strip_prefix("KEY_") {
        Some(name) if name.len() == 1 => name.to_string(),
        Some("SPACE") => "Space".to_string(),
        Some("ENTER") | Some("KPENTER") => "Enter".to_string(),
        Some("ESC") => "Escape".to_string(),
        Some("TAB") => "Tab".to_string(),
        Some("BACKSPACE") => "Backspace".to_string(),
        Some("DELETE") => "Delete".to_string(),
        Some(name) => name.to_string(),
        None => raw,
    }
}
