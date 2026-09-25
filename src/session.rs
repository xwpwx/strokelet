use crate::SessionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionFacts {
    pub query_ok: bool,
    pub uid_matches: bool,
    pub active: bool,
    pub locked: bool,
}

/// 查询失败、会话不属于目标用户、失活或锁定都不可用。
pub fn is_active_unlocked(facts: SessionFacts) -> bool {
    facts.query_ok && facts.uid_matches && facts.active && !facts.locked
}

pub fn session_state(facts: SessionFacts) -> Option<SessionState> {
    if !facts.query_ok {
        return None;
    }
    Some(SessionState {
        target_owned: facts.uid_matches,
        active: facts.active,
        locked: facts.locked,
    })
}
