// 环境修正评分器
// 核心职责：
// - 将对手回复、厚前排、爆发等环境信号转为装备收益修正
// - 输出环境加分和简短解释
// - 从版本化规则配置加载环境权重

use std::path::Path;

use hexsight_core::{ConflictEnvironment, ItemValueProfile};
use serde::{Deserialize, Serialize};

/// 环境修正权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentWeights {
    #[serde(rename = "healHeavyAntiHeal")]
    pub heal_heavy_anti_heal: i32,
    #[serde(rename = "frontlineThickArmorShred")]
    pub frontline_thick_armor_shred: i32,
    #[serde(rename = "frontlineThickMrShred")]
    pub frontline_thick_mr_shred: i32,
    #[serde(rename = "burstHeavySurvival")]
    pub burst_heavy_survival: i32,
    #[serde(rename = "ownAdArmorShred")]
    pub own_ad_armor_shred: i32,
    #[serde(rename = "ownApMrShred")]
    pub own_ap_mr_shred: i32,
}

impl Default for EnvironmentWeights {
    fn default() -> Self {
        Self {
            heal_heavy_anti_heal: 18,
            frontline_thick_armor_shred: 16,
            frontline_thick_mr_shred: 16,
            burst_heavy_survival: 12,
            own_ad_armor_shred: 8,
            own_ap_mr_shred: 8,
        }
    }
}

impl EnvironmentWeights {
    /// 从规则目录加载环境权重配置
    pub fn load(config_root: &Path, version: &str) -> Result<Self, String> {
        let path = config_root
            .join("rules")
            .join(version)
            .join("environment_weights.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取环境权重配置失败 {}: {}", path.display(), e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析环境权重配置失败: {}", e))
    }
}

/// 单件装备环境修正结果
#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentItemModifier {
    pub score: i32,
    pub reasons: Vec<String>,
}

/// 环境修正评分器
pub struct EnvironmentModifierScorer {
    weights: EnvironmentWeights,
}

impl EnvironmentModifierScorer {
    pub fn new(weights: EnvironmentWeights) -> Self {
        Self { weights }
    }

    /// 计算单件装备在当前环境下的额外价值
    pub fn score_item(
        &self,
        item: &ItemValueProfile,
        environment: &ConflictEnvironment,
    ) -> EnvironmentItemModifier {
        let mut score = 0;
        let mut reasons = Vec::new();

        if environment.opponent_heal_heavy && has_tag(item, "anti_heal") {
            score += self.weights.heal_heavy_anti_heal;
            reasons.push("对手回复多，重伤价值提高".to_string());
        }

        if environment.opponent_frontline_thick && has_tag(item, "armor_shred") {
            score += self.weights.frontline_thick_armor_shred;
            reasons.push("对手前排厚，破甲价值提高".to_string());
        }

        if environment.own_ad_heavy && has_tag(item, "armor_shred") {
            score += self.weights.own_ad_armor_shred;
            reasons.push("己方物理输出占比高，破甲收益提高".to_string());
        }

        if environment.opponent_frontline_thick && has_tag(item, "mr_shred") {
            score += self.weights.frontline_thick_mr_shred;
            reasons.push("对手前排厚，魔抗击碎价值提高".to_string());
        }

        if environment.own_ap_heavy && has_tag(item, "mr_shred") {
            score += self.weights.own_ap_mr_shred;
            reasons.push("己方法系输出占比高，魔抗击碎收益提高".to_string());
        }

        if environment.opponent_burst_heavy
            && (has_tag(item, "shield_survival") || has_tag(item, "sustain"))
        {
            score += self.weights.burst_heavy_survival;
            reasons.push("对手爆发高，生存装备价值提高".to_string());
        }

        EnvironmentItemModifier { score, reasons }
    }
}

fn has_tag(item: &ItemValueProfile, tag: &str) -> bool {
    item.effect_tags.iter().any(|value| value == tag)
        || item.conflict_group == tag
        || item.replacement_group == tag
}
