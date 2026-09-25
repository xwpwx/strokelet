use std::collections::BTreeSet;

use crate::{Decision, DesktopState, Injection};

/// 同一个手势 id 最多尝试一次注入。关闭注入开关时也不会改记为成功。
#[derive(Debug, Default)]
pub struct InjectionLedger {
    settled: BTreeSet<u64>,
}

impl InjectionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decide(
        &mut self,
        gesture_id: u64,
        inject_enabled: bool,
        decision: Decision,
        desktop: DesktopState,
    ) -> Injection {
        if !self.settled.insert(gesture_id) {
            return Injection::None;
        }
        if !inject_enabled {
            return Injection::None;
        }
        let mut gate = crate::CopyGate::new();
        gate.authorize(decision, desktop)
    }

    pub fn reject_ipc() -> Injection {
        Injection::None
    }
}
