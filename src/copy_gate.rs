use crate::Decision;

/// 一次释放是否可以注入 Ctrl+C。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Injection {
    None,
    CopyOnce,
}

/// 桌面侧观察。未知和超时都不能当成安全。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Observation<T> {
    Unknown,
    TimedOut,
    Known(T),
}

/// 目标 logind 会话是否归属正确、活动且未锁定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionState {
    pub target_owned: bool,
    pub active: bool,
    pub locked: bool,
}

/// 物理修饰键是否都已松开。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub none_held: bool,
}

/// 注入前必须重新核对的桌面状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopState {
    pub session: Observation<SessionState>,
    pub extension_ready: Observation<bool>,
    pub paused: Observation<bool>,
    pub modifiers: Observation<Modifiers>,
    pub communication: Observation<()>,
    pub permission_ok: Observation<bool>,
}

/// 同一次右键释放只判定一次。失败后不能改用更新的状态补注入。
#[derive(Debug)]
pub struct CopyGate {
    settled: bool,
}

impl Default for CopyGate {
    fn default() -> Self {
        Self::new()
    }
}

impl CopyGate {
    pub fn new() -> Self {
        Self { settled: false }
    }

    pub fn authorize(&mut self, decision: Decision, desktop: DesktopState) -> Injection {
        if self.settled {
            return Injection::None;
        }
        self.settled = true;
        if decision == Decision::Copy && desktop_allows_copy(desktop) {
            Injection::CopyOnce
        } else {
            Injection::None
        }
    }

    /// IPC 客户端不能请求复制。这个入口不消耗手势释放的一次判定。
    pub fn reject_ipc() -> Injection {
        Injection::None
    }
}

fn desktop_allows_copy(desktop: DesktopState) -> bool {
    let Some(session) = known(desktop.session) else {
        return false;
    };
    if !session.target_owned || !session.active || session.locked {
        return false;
    }
    if known(desktop.extension_ready) != Some(true) {
        return false;
    }
    if known(desktop.paused) != Some(false) {
        return false;
    }
    if known(desktop.modifiers).is_none_or(|mods| !mods.none_held) {
        return false;
    }
    if known(desktop.communication).is_none() {
        return false;
    }
    known(desktop.permission_ok) == Some(true)
}

fn known<T>(observation: Observation<T>) -> Option<T> {
    match observation {
        Observation::Known(value) => Some(value),
        Observation::Unknown | Observation::TimedOut => None,
    }
}
