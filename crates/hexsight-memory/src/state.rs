// 全局状态管理
// 核心职责：
// - 管理单局 GameState 生命周期
// - 增量更新：接收 RecognizedFrame，对比差异，仅更新变化字段
// - 降噪门控：连续 approval_frames 帧一致才确认更新

use hexsight_core::{GameState, HexResult, RecognizedFrame, RiskLevel};

/// 降噪确认帧数阈值
const NOISE_GATE_FRAMES: u32 = 3;

/// 对局内存管理器
pub struct GameMemory {
    /// 当前对局状态
    state: GameState,
    /// 候选帧缓冲（降噪用）
    pending: Option<RecognizedFrame>,
    /// 候选帧连续一致计数
    pending_count: u32,
    /// 对局是否活跃
    active: bool,
}

impl GameMemory {
    /// 创建新对局记忆
    pub fn new() -> Self {
        Self {
            state: GameState::default(),
            pending: None,
            pending_count: 0,
            active: false,
        }
    }

    /// 喂入一帧识别结果
    /// 返回：如果状态确实更新，返回 Some(&GameState)
    pub fn feed(&mut self, frame: RecognizedFrame) -> HexResult<Option<&GameState>> {
        // 自动检测新对局开始
        if !self.active {
            self.begin_game();
        }

        // 降噪门控：连续 NOISE_GATE_FRAMES 帧一致才确认更新
        if let Some(ref pending) = self.pending {
            if self.frames_equal(pending, &frame) {
                self.pending_count += 1;
                if self.pending_count >= NOISE_GATE_FRAMES {
                    self.apply_frame(frame);
                    return Ok(Some(&self.state));
                }
            } else {
                // 帧不一致，重置候选
                self.pending = Some(frame);
                self.pending_count = 1;
            }
        } else {
            self.pending = Some(frame);
            self.pending_count = 1;
        }

        Ok(None)
    }

    /// 获取当前完整状态
    pub fn current_state(&self) -> &GameState {
        &self.state
    }

    /// 开始新对局
    pub fn begin_game(&mut self) {
        self.state = GameState::default();
        self.pending = None;
        self.pending_count = 0;
        self.active = true;
    }

    /// 重置记忆（对局结束/退出大厅）
    pub fn reset(&mut self) {
        self.state = GameState::default();
        self.pending = None;
        self.pending_count = 0;
        self.active = false;
    }

    /// 是否处于对局中
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// 获取风险等级
    pub fn risk_level(&self) -> RiskLevel {
        self.state.risk_level.clone()
    }

    /// 应用确认帧到状态
    fn apply_frame(&mut self, frame: RecognizedFrame) {
        // 存档上一帧到历史
        let prev = std::mem::replace(&mut self.state.current, frame);
        self.state.history.push(prev);
        self.state.frame_seq += 1;

        // 限制历史长度（保留最近200帧）
        if self.state.history.len() > 200 {
            self.state.history.remove(0);
        }
    }

    /// 两帧关键字段是否一致
    fn frames_equal(&self, a: &RecognizedFrame, b: &RecognizedFrame) -> bool {
        a.gold == b.gold
            && a.hp == b.hp
            && a.level == b.level
            && a.round == b.round
    }
}

impl Default for GameMemory {
    fn default() -> Self {
        Self::new()
    }
}
