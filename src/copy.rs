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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Super,
}

impl Modifier {
    pub fn code(self) -> u16 {
        match self {
            Modifier::Ctrl => KeyCode::KEY_LEFTCTRL.0,
            Modifier::Shift => KeyCode::KEY_LEFTSHIFT.0,
            Modifier::Alt => KeyCode::KEY_LEFTALT.0,
            Modifier::Super => KeyCode::KEY_LEFTMETA.0,
        }
    }

    fn parse(name: &str) -> Option<Self> {
        match name {
            "ctrl" => Some(Modifier::Ctrl),
            "shift" => Some(Modifier::Shift),
            "alt" => Some(Modifier::Alt),
            "super" => Some(Modifier::Super),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    modifiers: Vec<Modifier>,
    key_name: String,
    key_code: u16,
}

impl Chord {
    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub fn key_name(&self) -> &str {
        &self.key_name
    }

    pub fn key_code(&self) -> u16 {
        self.key_code
    }

    pub fn parse(modifiers: &[String], key: &str) -> Result<Self, String> {
        if modifiers.len() > 4 {
            return Err("a shortcut can have at most 4 modifiers".into());
        }
        let mut parsed = Vec::new();
        for name in modifiers {
            let modifier =
                Modifier::parse(name).ok_or_else(|| format!("unknown modifier {name}"))?;
            if parsed.contains(&modifier) {
                return Err(format!("duplicate modifier {name}"));
            }
            parsed.push(modifier);
        }
        parsed.sort_by_key(|modifier| modifier.code());
        let key_code = key_code(key).ok_or_else(|| format!("unknown key {key}"))?;
        Ok(Self {
            modifiers: parsed,
            key_name: key.to_string(),
            key_code,
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
    let mut codes = vec![
        Modifier::Ctrl.code(),
        Modifier::Shift.code(),
        Modifier::Alt.code(),
        Modifier::Super.code(),
    ];
    codes.extend(KEYS.iter().map(|(_, code)| *code));
    codes
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
        for modifier in &chord.modifiers {
            self.press(modifier.code())?;
            self.syn()?;
        }
        self.press(chord.key_code)?;
        self.syn()?;
        self.release_one(chord.key_code)?;
        self.syn()?;
        for modifier in chord.modifiers.iter().rev() {
            self.release_one(modifier.code())?;
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
