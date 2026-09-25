use evdev::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEvent {
    Key { code: u16, down: bool },
    SynReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitError;

pub trait KeySink {
    fn emit(&mut self, event: OutputEvent) -> Result<(), EmitError>;
}

/// GTK 的键码比 evdev 大 8。C 是 54→46，左 Ctrl 是 37→29，左 Shift 是 50→42。
pub fn evdev_from_gtk_keycode(keycode: u16) -> Option<u16> {
    keycode.checked_sub(8)
}

const MODIFIER_CODES: [u16; 8] = [
    KeyCode::KEY_LEFTCTRL.0,
    KeyCode::KEY_RIGHTCTRL.0,
    KeyCode::KEY_LEFTSHIFT.0,
    KeyCode::KEY_RIGHTSHIFT.0,
    KeyCode::KEY_LEFTALT.0,
    KeyCode::KEY_RIGHTALT.0,
    KeyCode::KEY_LEFTMETA.0,
    KeyCode::KEY_RIGHTMETA.0,
];

const MACHINE_KEYS: [u16; 6] = [
    KeyCode::KEY_SYSRQ.0,
    KeyCode::KEY_POWER.0,
    KeyCode::KEY_SLEEP.0,
    KeyCode::KEY_WAKEUP.0,
    KeyCode::KEY_SUSPEND.0,
    KeyCode::KEY_POWER2.0,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    modifiers: Vec<u16>,
    key_code: u16,
    key_label: String,
}

impl Chord {
    pub fn modifier_codes(&self) -> &[u16] {
        &self.modifiers
    }

    pub fn key_name(&self) -> &str {
        &self.key_label
    }

    pub fn key_code(&self) -> u16 {
        self.key_code
    }

    /// 旧配置里的名字。`ctrl` 固定是左 Ctrl，不会和右 Ctrl 合成同一个键。
    pub fn parse(modifiers: &[String], key: &str) -> Result<Self, String> {
        if modifiers.len() > 4 {
            return Err("a shortcut can have at most 4 modifiers".into());
        }
        let mut codes = Vec::new();
        for name in modifiers {
            let code = named_modifier(name).ok_or_else(|| format!("unknown modifier {name}"))?;
            if codes.contains(&code) {
                return Err(format!("duplicate modifier {name}"));
            }
            codes.push(code);
        }
        let key_code = key_code(key).ok_or_else(|| format!("unknown key {key}"))?;
        Self::from_codes(&codes, key_code, key)
    }

    pub fn from_codes(modifiers: &[u16], key: u16, label: &str) -> Result<Self, String> {
        if modifiers.len() > 4 {
            return Err("a shortcut can have at most 4 modifiers".into());
        }
        if label.is_empty() {
            return Err("shortcut label is empty".into());
        }
        let mut codes = modifiers.to_vec();
        codes.sort_unstable();
        if codes.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err("duplicate modifier".into());
        }
        for code in &codes {
            if !is_modifier(*code) {
                return Err(format!("not a modifier key {code}"));
            }
        }
        if !is_main_key(key) {
            return Err(format!("key {key} cannot be a shortcut"));
        }
        Ok(Self {
            modifiers: codes,
            key_code: key,
            key_label: label.to_string(),
        })
    }
}

pub fn key_code(name: &str) -> Option<u16> {
    KEYS.iter()
        .find(|(label, _)| *label == name)
        .map(|(_, code)| *code)
}

pub fn key_names() -> impl Iterator<Item = &'static str> {
    KEYS.iter().map(|(label, _)| *label)
}

pub fn chord_device_codes() -> Vec<u16> {
    let mut codes = Vec::new();
    for code in 1..=248 {
        if is_device_key(code) {
            codes.push(code);
        }
    }
    for code in 0x160..=0x27a {
        if is_device_key(code) {
            codes.push(code);
        }
    }
    codes
}

fn named_modifier(name: &str) -> Option<u16> {
    match name {
        "ctrl" => Some(KeyCode::KEY_LEFTCTRL.0),
        "shift" => Some(KeyCode::KEY_LEFTSHIFT.0),
        "alt" => Some(KeyCode::KEY_LEFTALT.0),
        "super" => Some(KeyCode::KEY_LEFTMETA.0),
        _ => None,
    }
}

pub(crate) fn is_modifier(code: u16) -> bool {
    MODIFIER_CODES.contains(&code)
}

pub(crate) fn modifier_label(code: u16) -> Option<&'static str> {
    match KeyCode(code) {
        KeyCode::KEY_LEFTCTRL => Some("Ctrl"),
        KeyCode::KEY_RIGHTCTRL => Some("Right Ctrl"),
        KeyCode::KEY_LEFTSHIFT => Some("Shift"),
        KeyCode::KEY_RIGHTSHIFT => Some("Right Shift"),
        KeyCode::KEY_LEFTALT => Some("Alt"),
        KeyCode::KEY_RIGHTALT => Some("Right Alt"),
        KeyCode::KEY_LEFTMETA => Some("Super"),
        KeyCode::KEY_RIGHTMETA => Some("Right Super"),
        _ => None,
    }
}

pub(crate) fn is_shortcut_key(code: u16) -> bool {
    is_main_key(code)
}

fn is_machine_key(code: u16) -> bool {
    MACHINE_KEYS.contains(&code)
}

fn is_button(code: u16) -> bool {
    (0x100..=0x109).contains(&code)
        || (0x110..=0x117).contains(&code)
        || (0x120..=0x12f).contains(&code)
        || (0x130..=0x13e).contains(&code)
        || (0x140..=0x14f).contains(&code)
        || (0x150..=0x151).contains(&code)
        || (0x2c0..=0x2e7).contains(&code)
}

fn is_key_range(code: u16) -> bool {
    (1..=248).contains(&code) || (0x160..=0x27a).contains(&code)
}

fn is_main_key(code: u16) -> bool {
    is_key_range(code) && !is_modifier(code) && !is_machine_key(code) && !is_button(code)
}

fn is_device_key(code: u16) -> bool {
    is_key_range(code) && !is_machine_key(code) && !is_button(code)
}

/// 按修饰键然后主键的顺序按下，再按相反顺序松开。失败时只松开已经按下的键。
#[derive(Debug)]
pub struct CopyOutput<S> {
    sink: S,
    owned: Vec<u16>,
}

impl<S: KeySink> CopyOutput<S> {
    pub fn new(sink: S) -> Self {
        Self {
            sink,
            owned: Vec::new(),
        }
    }

    pub fn send_chord(&mut self, chord: &Chord) -> Result<(), EmitError> {
        if let Err(error) = self.write_chord(chord) {
            let _ = self.release_owned_keys();
            return Err(error);
        }
        Ok(())
    }

    pub fn release_owned_keys(&mut self) -> Result<(), EmitError> {
        while let Some(code) = self.owned.pop() {
            self.sink.emit(OutputEvent::Key { code, down: false })?;
            self.sink.emit(OutputEvent::SynReport)?;
        }
        Ok(())
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    fn write_chord(&mut self, chord: &Chord) -> Result<(), EmitError> {
        for code in &chord.modifiers {
            self.press(*code)?;
            self.syn()?;
        }
        self.press(chord.key_code)?;
        self.syn()?;
        self.release_one(chord.key_code)?;
        self.syn()?;
        for code in chord.modifiers.iter().rev() {
            self.release_one(*code)?;
            self.syn()?;
        }
        Ok(())
    }

    fn press(&mut self, code: u16) -> Result<(), EmitError> {
        self.sink.emit(OutputEvent::Key { code, down: true })?;
        self.owned.push(code);
        Ok(())
    }

    fn release_one(&mut self, code: u16) -> Result<(), EmitError> {
        self.sink.emit(OutputEvent::Key { code, down: false })?;
        self.owned.retain(|held| *held != code);
        Ok(())
    }

    fn syn(&mut self) -> Result<(), EmitError> {
        self.sink.emit(OutputEvent::SynReport)
    }
}

const KEYS: &[(&str, u16)] = &[
    ("a", KeyCode::KEY_A.0),
    ("b", KeyCode::KEY_B.0),
    ("c", KeyCode::KEY_C.0),
    ("d", KeyCode::KEY_D.0),
    ("e", KeyCode::KEY_E.0),
    ("f", KeyCode::KEY_F.0),
    ("g", KeyCode::KEY_G.0),
    ("h", KeyCode::KEY_H.0),
    ("i", KeyCode::KEY_I.0),
    ("j", KeyCode::KEY_J.0),
    ("k", KeyCode::KEY_K.0),
    ("l", KeyCode::KEY_L.0),
    ("m", KeyCode::KEY_M.0),
    ("n", KeyCode::KEY_N.0),
    ("o", KeyCode::KEY_O.0),
    ("p", KeyCode::KEY_P.0),
    ("q", KeyCode::KEY_Q.0),
    ("r", KeyCode::KEY_R.0),
    ("s", KeyCode::KEY_S.0),
    ("t", KeyCode::KEY_T.0),
    ("u", KeyCode::KEY_U.0),
    ("v", KeyCode::KEY_V.0),
    ("w", KeyCode::KEY_W.0),
    ("x", KeyCode::KEY_X.0),
    ("y", KeyCode::KEY_Y.0),
    ("z", KeyCode::KEY_Z.0),
    ("0", KeyCode::KEY_0.0),
    ("1", KeyCode::KEY_1.0),
    ("2", KeyCode::KEY_2.0),
    ("3", KeyCode::KEY_3.0),
    ("4", KeyCode::KEY_4.0),
    ("5", KeyCode::KEY_5.0),
    ("6", KeyCode::KEY_6.0),
    ("7", KeyCode::KEY_7.0),
    ("8", KeyCode::KEY_8.0),
    ("9", KeyCode::KEY_9.0),
    ("f1", KeyCode::KEY_F1.0),
    ("f2", KeyCode::KEY_F2.0),
    ("f3", KeyCode::KEY_F3.0),
    ("f4", KeyCode::KEY_F4.0),
    ("f5", KeyCode::KEY_F5.0),
    ("f6", KeyCode::KEY_F6.0),
    ("f7", KeyCode::KEY_F7.0),
    ("f8", KeyCode::KEY_F8.0),
    ("f9", KeyCode::KEY_F9.0),
    ("f10", KeyCode::KEY_F10.0),
    ("f11", KeyCode::KEY_F11.0),
    ("f12", KeyCode::KEY_F12.0),
    ("arrow-up", KeyCode::KEY_UP.0),
    ("arrow-down", KeyCode::KEY_DOWN.0),
    ("arrow-left", KeyCode::KEY_LEFT.0),
    ("arrow-right", KeyCode::KEY_RIGHT.0),
    ("enter", KeyCode::KEY_ENTER.0),
    ("escape", KeyCode::KEY_ESC.0),
    ("tab", KeyCode::KEY_TAB.0),
    ("backspace", KeyCode::KEY_BACKSPACE.0),
    ("delete", KeyCode::KEY_DELETE.0),
];
