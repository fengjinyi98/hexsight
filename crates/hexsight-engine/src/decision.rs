// 决策输出引擎
// 核心职责：
// - 整合规则引擎 + 阵容库 → 输出最终 Decision
// - 判定是否需要触发 LLM 推理（复杂对局场景）
// - 格式化自然语言建议

use hexsight_core::{Decision, DecisionSource, GameState};

use super::lineup::LineupDB;
use super::rules::RulesEngine;

/// 决策引擎 —— 整合规则与阵容，输出最终建议
pub struct DecisionEngine {
    #[allow(dead_code)]
    rules: RulesEngine,
    lineups: LineupDB,
    /// 是否需要 LLM 增强
    llm_triggered: bool,
}

impl DecisionEngine {
    pub fn new() -> Self {
        Self {
            rules: RulesEngine::new(),
            lineups: LineupDB::new(),
            llm_triggered: false,
        }
    }

    /// 初始化：加载阵容库
    pub fn init(&mut self, lineup_dir: &str) -> hexsight_core::HexResult<()> {
        self.lineups.load_from_dir(lineup_dir)?;
        Ok(())
    }

    /// 核心决策：根据对局状态输出建议
    pub fn decide(&mut self, _state: &GameState) -> Decision {
        let mut decision = Decision {
            source: DecisionSource::RuleEngine,
            ..Default::default()
        };

        // 判定是否需要 LLM
        if self.should_trigger_llm(_state) {
            decision.source = DecisionSource::Hybrid;
            self.llm_triggered = true;
        }

        // TODO: 实际决策逻辑
        // 1. 阵容推荐（lineups.recommend）
        // 2. 操作建议（rules.level_advice / roll_advice）
        // 3. 同行风险（rules.assess_rival_risk）
        // 4. 装备路线（lineup.carry_items）

        decision
    }

    /// 获取阵容库引用
    pub fn lineups(&self) -> &LineupDB {
        &self.lineups
    }

    /// 获取阵容库可变引用
    pub fn lineups_mut(&mut self) -> &mut LineupDB {
        &mut self.lineups
    }

    /// 判定是否应触发 LLM 推理
    fn should_trigger_llm(&self, state: &GameState) -> bool {
        // 触发条件（任一满足）：
        // 1. 同行 ≥ 3
        // 2. 血量 < 30（危急）
        // 3. 装备极差（需进一步判定）
        state.rival_count >= 3 || state.current.hp < 30
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}
