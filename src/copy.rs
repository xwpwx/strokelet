#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    LeftCtrl,
    C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEvent {
    Key { code: KeyCode, down: bool },
    SynReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitError;

pub trait KeySink {
    fn emit(&mut self, event: OutputEvent) -> Result<(), EmitError>;
}

/// 固定左 Ctrl+C。失败时只松开本对象已经按下的键。
#[derive(Debug)]
pub struct CopyOutput<S> {
    sink: S,
    owned: Vec<KeyCode>,
}

impl<S: KeySink> CopyOutput<S> {
    pub fn new(sink: S) -> Self {
        Self {
            sink,
            owned: Vec::new(),
        }
    }

    pub fn send_copy(&mut self) -> Result<(), EmitError> {
        if let Err(error) = self.write_chord() {
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

    fn write_chord(&mut self) -> Result<(), EmitError> {
        self.press(KeyCode::LeftCtrl)?;
        self.syn()?;
        self.press(KeyCode::C)?;
        self.syn()?;
        self.release_one(KeyCode::C)?;
        self.syn()?;
        self.release_one(KeyCode::LeftCtrl)?;
        self.syn()?;
        Ok(())
    }

    fn press(&mut self, code: KeyCode) -> Result<(), EmitError> {
        self.sink.emit(OutputEvent::Key { code, down: true })?;
        self.owned.push(code);
        Ok(())
    }

    fn release_one(&mut self, code: KeyCode) -> Result<(), EmitError> {
        self.sink.emit(OutputEvent::Key { code, down: false })?;
        self.owned.retain(|held| *held != code);
        Ok(())
    }

    fn syn(&mut self) -> Result<(), EmitError> {
        self.sink.emit(OutputEvent::SynReport)
    }
}
