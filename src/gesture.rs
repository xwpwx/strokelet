/// 一次右键手势的决策。同一次按下的重复释放返回 [`Decision::None`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    None,
    RightClick,
    Copy,
    Cancel,
}

/// 规格中的可调初值。横向比例、直线度和横向下限是固定判据，不在这里。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub start_counts: f64,
    pub click_slop_counts: f64,
    pub min_up_counts: f64,
    pub max_duration_ms: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            start_counts: 12.0,
            click_slop_counts: 12.0,
            min_up_counts: 80.0,
            max_duration_ms: 2500,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Pending,
    Drawing,
    CancelledUntilRelease,
}

/// 纯手势状态机。只消费已合并的帧位移和按下后的单调时间，不接触设备。
#[derive(Debug)]
pub struct Gesture {
    limits: Limits,
    phase: Phase,
    x: f64,
    y: f64,
    path_len: f64,
    max_distance: f64,
    max_lateral: f64,
    elapsed_ms: u64,
}

const MIN_STRAIGHTNESS: f64 = 0.85;
const LATERAL_RATIO: f64 = 0.30;
const LATERAL_FLOOR_COUNTS: f64 = 12.0;

impl Gesture {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            phase: Phase::Idle,
            x: 0.0,
            y: 0.0,
            path_len: 0.0,
            max_distance: 0.0,
            max_lateral: 0.0,
            elapsed_ms: 0,
        }
    }

    pub fn press(&mut self) {
        if self.phase != Phase::Idle {
            return;
        }
        self.phase = Phase::Pending;
        self.x = 0.0;
        self.y = 0.0;
        self.path_len = 0.0;
        self.max_distance = 0.0;
        self.max_lateral = 0.0;
        self.elapsed_ms = 0;
    }

    /// 一帧已经合并的相对位移。向上为负 y。
    pub fn motion(&mut self, dx: f64, dy: f64) {
        if !matches!(self.phase, Phase::Pending | Phase::Drawing) {
            return;
        }
        self.x += dx;
        self.y += dy;
        self.path_len += dx.hypot(dy);
        self.max_distance = self.max_distance.max(self.x.hypot(self.y));
        self.max_lateral = self.max_lateral.max(self.x.abs());
        if self.phase == Phase::Pending && self.max_distance > self.limits.start_counts {
            self.phase = Phase::Drawing;
        }
        self.expire_if_needed();
    }

    /// 本次按下后的单调时间。倒退的时间戳忽略。已进入绘制后超时则保持取消。
    pub fn advance(&mut self, elapsed_ms: u64) {
        if self.phase == Phase::Idle {
            return;
        }
        if elapsed_ms >= self.elapsed_ms {
            self.elapsed_ms = elapsed_ms;
        }
        self.expire_if_needed();
    }

    /// 滚轮、额外按钮、丢帧或连接/会话失效。右键释放前保持取消。
    pub fn cancel(&mut self) {
        if self.phase != Phase::Idle {
            self.phase = Phase::CancelledUntilRelease;
        }
    }

    pub fn is_drawing(&self) -> bool {
        self.phase == Phase::Drawing
    }

    /// `elapsed_ms` 是本次右键按下起的单调时间差。
    pub fn release(&mut self, elapsed_ms: u64) -> Decision {
        if self.phase != Phase::Idle {
            self.advance(elapsed_ms);
        }
        let phase = self.phase;
        self.phase = Phase::Idle;
        match phase {
            Phase::Idle => Decision::None,
            Phase::CancelledUntilRelease => Decision::Cancel,
            Phase::Pending => {
                if self.max_distance <= self.limits.click_slop_counts {
                    Decision::RightClick
                } else {
                    Decision::Cancel
                }
            }
            Phase::Drawing => {
                if self.upward_copy() {
                    Decision::Copy
                } else {
                    Decision::Cancel
                }
            }
        }
    }

    fn expire_if_needed(&mut self) {
        if self.phase == Phase::Drawing && self.elapsed_ms > self.limits.max_duration_ms {
            self.phase = Phase::CancelledUntilRelease;
        }
    }

    fn upward_copy(&self) -> bool {
        if self.y > -self.limits.min_up_counts || self.path_len <= 0.0 {
            return false;
        }
        let net_up = -self.y;
        let lateral_limit = (net_up * LATERAL_RATIO).max(LATERAL_FLOOR_COUNTS);
        self.max_lateral <= lateral_limit && net_up / self.path_len >= MIN_STRAIGHTNESS
    }
}
