use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{DesktopState, Modifiers, Observation, SessionState};

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_LINE_BYTES: usize = 4096;
pub const PING_INTERVAL_MS: u64 = 250;
pub const HEALTH_TIMEOUT_MS: u64 = 1000;
pub const STATE_TTL_MS: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Click,
    CopyInjected,
    Unmatched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CancelReason {
    Timeout,
    Disconnect,
    Unmatched,
    Paused,
    Session,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ServerLine {
    Hello {
        version: u32,
    },
    Begin {
        id: u64,
    },
    End {
        id: u64,
        outcome: Outcome,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    Cancel {
        id: u64,
        reason: CancelReason,
    },
    Ping,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ClientLine {
    Ready {
        version: u32,
    },
    Pause {
        paused: bool,
    },
    State {
        active: bool,
        locked: bool,
        modifiers: Vec<String>,
    },
    Pong,
    Reload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    LineTooLong,
    InvalidJson,
    BadVersion,
    DuplicateReady,
    NotReady,
    Closed,
    UnknownGesture,
    LateGesture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Fresh,
    Ready,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExtState {
    active: bool,
    locked: bool,
    modifiers: Vec<String>,
    at_ms: u64,
}

/// 一条连接上的协议状态。断连后旧手势 id 不能再结束或恢复。
#[derive(Debug)]
pub struct Link {
    phase: Phase,
    active: Option<u64>,
    retired: BTreeSet<u64>,
    paused: Option<bool>,
    state: Option<ExtState>,
    last_rx_ms: Option<u64>,
    last_ping_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientUpdate {
    Ready,
    Pause(bool),
    State,
    Pong,
    Reload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollAction {
    None,
    SendPing,
    Disconnect,
}

#[derive(Debug)]
pub struct LineCodec {
    buf: Vec<u8>,
}

impl Default for Link {
    fn default() -> Self {
        Self::new()
    }
}

impl Link {
    pub fn new() -> Self {
        Self {
            phase: Phase::Fresh,
            active: None,
            retired: BTreeSet::new(),
            paused: None,
            state: None,
            last_rx_ms: None,
            last_ping_ms: None,
        }
    }

    pub fn connected(&mut self, now_ms: u64) {
        self.last_rx_ms = Some(now_ms);
    }

    pub fn is_ready(&self) -> bool {
        self.phase == Phase::Ready
    }

    pub fn is_paused(&self) -> bool {
        self.paused == Some(true)
    }

    pub fn hello() -> String {
        encode(&ServerLine::Hello {
            version: PROTOCOL_VERSION,
        })
    }

    pub fn ingest(&mut self, line: &str, now_ms: u64) -> Result<ClientUpdate, ProtocolError> {
        if self.phase == Phase::Closed {
            return Err(ProtocolError::Closed);
        }
        let message: ClientLine =
            serde_json::from_str(line).map_err(|_| ProtocolError::InvalidJson)?;
        let update = match message {
            ClientLine::Ready { version } => {
                if version != PROTOCOL_VERSION {
                    self.phase = Phase::Closed;
                    return Err(ProtocolError::BadVersion);
                }
                if self.phase == Phase::Ready {
                    self.phase = Phase::Closed;
                    return Err(ProtocolError::DuplicateReady);
                }
                self.phase = Phase::Ready;
                ClientUpdate::Ready
            }
            ClientLine::Pause { paused } => {
                self.require_ready()?;
                self.paused = Some(paused);
                ClientUpdate::Pause(paused)
            }
            ClientLine::State {
                active,
                locked,
                modifiers,
            } => {
                self.require_ready()?;
                self.state = Some(ExtState {
                    active,
                    locked,
                    modifiers,
                    at_ms: now_ms,
                });
                ClientUpdate::State
            }
            ClientLine::Pong => {
                self.require_ready()?;
                ClientUpdate::Pong
            }
            ClientLine::Reload => ClientUpdate::Reload,
        };
        self.last_rx_ms = Some(now_ms);
        Ok(update)
    }

    pub fn begin(&mut self, id: u64) -> Result<String, ProtocolError> {
        self.require_ready()?;
        if self.active.is_some() || self.retired.contains(&id) {
            return Err(ProtocolError::LateGesture);
        }
        self.active = Some(id);
        Ok(encode(&ServerLine::Begin { id }))
    }

    pub fn end(&mut self, id: u64, outcome: Outcome) -> Result<String, ProtocolError> {
        self.end_with_name(id, outcome, None)
    }

    pub fn end_with_name(
        &mut self,
        id: u64,
        outcome: Outcome,
        name: Option<&str>,
    ) -> Result<String, ProtocolError> {
        self.finish(id)?;
        let name = name
            .map(str::trim)
            .filter(|text| !text.is_empty() && !text.chars().any(char::is_control))
            .map(str::to_string);
        Ok(encode(&ServerLine::End { id, outcome, name }))
    }

    pub fn cancel(&mut self, id: u64, reason: CancelReason) -> Result<String, ProtocolError> {
        self.finish(id)?;
        Ok(encode(&ServerLine::Cancel { id, reason }))
    }

    pub fn disconnect(&mut self) {
        if let Some(id) = self.active.take() {
            self.retired.insert(id);
        }
        self.phase = Phase::Closed;
    }

    pub fn poll(&mut self, now_ms: u64) -> PollAction {
        if self.phase == Phase::Closed {
            return PollAction::None;
        }
        if let Some(last) = self.last_rx_ms
            && now_ms.saturating_sub(last) > HEALTH_TIMEOUT_MS
        {
            self.disconnect();
            return PollAction::Disconnect;
        }
        if self.phase != Phase::Ready {
            return PollAction::None;
        }
        let due = self
            .last_ping_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= PING_INTERVAL_MS);
        if due {
            self.last_ping_ms = Some(now_ms);
            PollAction::SendPing
        } else {
            PollAction::None
        }
    }

    pub fn ping() -> String {
        encode(&ServerLine::Ping)
    }

    pub fn desktop(
        &self,
        logind: Observation<SessionState>,
        permission_ok: Observation<bool>,
        now_ms: u64,
    ) -> DesktopState {
        let fresh = self
            .state
            .as_ref()
            .is_some_and(|state| now_ms.saturating_sub(state.at_ms) <= STATE_TTL_MS);
        DesktopState {
            session: combine_session(logind, self.state.as_ref(), fresh),
            extension_ready: Observation::Known(self.phase == Phase::Ready),
            paused: match self.paused {
                Some(paused) => Observation::Known(paused),
                None => Observation::Unknown,
            },
            modifiers: match (self.state.as_ref(), fresh) {
                (Some(state), true) => Observation::Known(Modifiers {
                    none_held: state.modifiers.is_empty(),
                }),
                (Some(_), false) => Observation::TimedOut,
                (None, _) => Observation::Unknown,
            },
            communication: match self.last_rx_ms {
                Some(last) if now_ms.saturating_sub(last) <= HEALTH_TIMEOUT_MS => {
                    Observation::Known(())
                }
                Some(_) => Observation::TimedOut,
                None => Observation::Unknown,
            },
            permission_ok,
        }
    }

    fn require_ready(&self) -> Result<(), ProtocolError> {
        if self.phase == Phase::Ready {
            Ok(())
        } else if self.phase == Phase::Closed {
            Err(ProtocolError::Closed)
        } else {
            Err(ProtocolError::NotReady)
        }
    }

    fn finish(&mut self, id: u64) -> Result<(), ProtocolError> {
        if self.phase == Phase::Closed {
            return Err(ProtocolError::Closed);
        }
        if self.retired.contains(&id) {
            return Err(ProtocolError::LateGesture);
        }
        if self.active != Some(id) {
            return Err(ProtocolError::UnknownGesture);
        }
        self.active = None;
        self.retired.insert(id);
        Ok(())
    }
}

impl Default for LineCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl LineCodec {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, ProtocolError> {
        let mut lines = Vec::new();
        for byte in bytes {
            if *byte == b'\n' {
                let line = String::from_utf8_lossy(&self.buf).trim().to_string();
                self.buf.clear();
                if !line.is_empty() {
                    lines.push(line);
                }
            } else if self.buf.len() == MAX_LINE_BYTES {
                self.buf.clear();
                return Err(ProtocolError::LineTooLong);
            } else {
                self.buf.push(*byte);
            }
        }
        Ok(lines)
    }
}

pub fn encode<T: Serialize>(message: &T) -> String {
    let mut line = serde_json::to_string(message).expect("protocol message is serializable");
    line.push('\n');
    line
}

fn combine_session(
    logind: Observation<SessionState>,
    extension: Option<&ExtState>,
    fresh: bool,
) -> Observation<SessionState> {
    let Observation::Known(mut session) = logind else {
        return logind;
    };
    let Some(extension) = extension else {
        return Observation::Unknown;
    };
    if !fresh {
        return Observation::TimedOut;
    }
    session.active = session.active && extension.active;
    session.locked = session.locked || extension.locked;
    Observation::Known(session)
}
