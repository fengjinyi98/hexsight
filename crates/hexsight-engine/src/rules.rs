// 固定决策规则引擎
// 核心职责：
// - 开局选阵规则（装备散件 → 体系划分 → 推荐阵容）
// - 过渡运营规则（人口节奏、D牌规则、打工匹配）
// - 同行博弈规则（0-1/2/≥3家对应策略）
// - 装备劣势补救规则
// - 海克斯基础匹配规则

use hexsight_core::{Equipment, HexResult, RiskLevel, RecognizedFrame};

/// 固定规则引擎
pub struct RulesEngine;

impl RulesEngine {
    pub fn new() -> Self {
        Self
    }

    /// 判定同行风险等级
    /// 输入：同体系对手数量
    pub fn assess_rival_risk(&self, rival_count: u32) -> RiskLevel {
        match rival_count {
            0..=1 => RiskLevel::Low,
            2 => RiskLevel::Medium,
            _ => RiskLevel::High,
        }
    }

    /// 开局选阵决策
    /// 输入：3件散件 + 初始来牌
    /// 输出：推荐体系（物理/法系/重装）
    pub fn classify_opening(&self, _equipment: &[Equipment], _heroes: &[String]) -> HexResult<String> {
        // TODO: 按装备分类：攻击散件→物理，大棒水滴→法系，锁子甲腰带→重装
        Ok("物理".to_string())
    }

    /// 人口节奏决策
    /// 输入：当前回合、血量、经济
    /// 输出：是否升人口建议
    pub fn level_advice(&self, _state: &RecognizedFrame) -> HexResult<String> {
        // TODO: 判断当前回合 → 建议升/不升人口
        // 2-5 升6 / 4-1 强升8
        Ok("保持当前人口".to_string())
    }

    /// D牌决策
    /// 输入：血量、经济、同行数量
    /// 输出：D牌策略
    pub fn roll_advice(&self, hp: u32, _gold: u32, _rival_count: u32) -> HexResult<String> {
        if hp >= 50 {
            Ok("不D牌，保50利息".to_string())
        } else if hp >= 30 {
            Ok("小幅D牌提质量".to_string())
        } else {
            Ok("全力D牌保命".to_string())
        }
    }

    /// 装备劣势补救建议
    pub fn equip_remedy(&self, _state: &RecognizedFrame) -> HexResult<String> {
        // TODO: 检测装备冲突 → 推荐补救方案
        Ok("".to_string())
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}
