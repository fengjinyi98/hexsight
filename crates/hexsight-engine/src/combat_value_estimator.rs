// 简化收益估算器
// 核心职责：
// - 基于棋子机制和装备画像估算简化 DPS/EHP
// - 比较同一棋子不同装备组合的收益差
// - 合并环境修正分，为装备建议和转阵容判断提供依据

use std::path::Path;

use hexsight_core::{ChampionCapability, ConflictEnvironment, ItemValueProfile};
use serde::{Deserialize, Serialize};

use crate::environment_modifier::{EnvironmentModifierScorer, EnvironmentWeights};

/// 简化收益估算权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatValueWeights {
    #[serde(rename = "baseDps")]
    pub base_dps: f64,
    #[serde(rename = "baseEhp")]
    pub base_ehp: f64,
    #[serde(rename = "damageTypeFitMultiplier")]
    pub damage_type_fit_multiplier: f64,
    #[serde(rename = "abilityPowerPerPoint")]
    pub ability_power_per_point: f64,
    #[serde(rename = "attackDamagePerPoint")]
    pub attack_damage_per_point: f64,
    #[serde(rename = "attackSpeedPerPoint")]
    pub attack_speed_per_point: f64,
    #[serde(rename = "critPerPoint")]
    pub crit_per_point: f64,
    #[serde(rename = "manaPerPoint")]
    pub mana_per_point: f64,
    #[serde(rename = "manaEngineMultiplier")]
    pub mana_engine_multiplier: f64,
    #[serde(rename = "hpPerPoint")]
    pub hp_per_point: f64,
    #[serde(rename = "armorPerPoint")]
    pub armor_per_point: f64,
    #[serde(rename = "mrPerPoint")]
    pub mr_per_point: f64,
    #[serde(rename = "survivalTagMultiplier")]
    pub survival_tag_multiplier: f64,
}

impl Default for CombatValueWeights {
    fn default() -> Self {
        Self {
            base_dps: 100.0,
            base_ehp: 1000.0,
            damage_type_fit_multiplier: 1.12,
            ability_power_per_point: 1.15,
            attack_damage_per_point: 1.1,
            attack_speed_per_point: 0.95,
            crit_per_point: 0.75,
            mana_per_point: 0.9,
            mana_engine_multiplier: 1.18,
            hp_per_point: 1.0,
            armor_per_point: 8.0,
            mr_per_point: 8.0,
            survival_tag_multiplier: 1.12,
        }
    }
}

impl CombatValueWeights {
    /// 从规则目录加载收益估算权重配置
    pub fn load(config_root: &Path, version: &str) -> Result<Self, String> {
        let path = config_root
            .join("rules")
            .join(version)
            .join("combat_value_weights.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取收益估算权重配置失败 {}: {}", path.display(), e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析收益估算权重配置失败: {}", e))
    }
}

/// 单次收益估算结果
#[derive(Debug, Clone, Serialize)]
pub struct CombatValueEstimate {
    #[serde(rename = "simplifiedDps")]
    pub simplified_dps: f64,
    #[serde(rename = "simplifiedEhp")]
    pub simplified_ehp: f64,
    #[serde(rename = "itemValueScore")]
    pub item_value_score: i32,
    #[serde(rename = "environmentScore")]
    pub environment_score: i32,
    pub reasons: Vec<String>,
}

/// 装备组合收益差
#[derive(Debug, Clone, Serialize)]
pub struct CombatValueDiff {
    #[serde(rename = "dpsDiff")]
    pub dps_diff: f64,
    #[serde(rename = "ehpDiff")]
    pub ehp_diff: f64,
    #[serde(rename = "scoreDiff")]
    pub score_diff: i32,
    pub winner: String,
    pub reasons: Vec<String>,
}

/// 简化收益估算器
pub struct CombatValueEstimator {
    weights: CombatValueWeights,
    environment_scorer: EnvironmentModifierScorer,
}

impl CombatValueEstimator {
    pub fn new(weights: CombatValueWeights) -> Self {
        Self {
            weights,
            environment_scorer: EnvironmentModifierScorer::new(EnvironmentWeights::default()),
        }
    }

    pub fn new_with_environment_weights(
        weights: CombatValueWeights,
        environment_weights: EnvironmentWeights,
    ) -> Self {
        Self {
            weights,
            environment_scorer: EnvironmentModifierScorer::new(environment_weights),
        }
    }

    /// 估算棋子携带一组装备后的 DPS/EHP 与收益分
    pub fn estimate(
        &self,
        champion: &ChampionCapability,
        items: &[ItemValueProfile],
        environment: &ConflictEnvironment,
    ) -> CombatValueEstimate {
        let mut dps = self.weights.base_dps + champion.cost.max(1) as f64 * 8.0;
        let mut ehp = self.weights.base_ehp + champion.cost.max(1) as f64 * 120.0;
        let mut environment_score = 0;
        let mut reasons = Vec::new();

        if champion.role == hexsight_core::ChampionRole::MainTank
            || champion.position_role == "frontline"
        {
            ehp *= 1.15;
        }

        for item in items {
            if self.damage_type_matches(champion, item) {
                dps *= self.weights.damage_type_fit_multiplier;
                reasons.push(format!("{} 伤害类型匹配", item.name));
            }

            for stat in &item.stats {
                match stat.name.as_str() {
                    "ap" => {
                        dps += stat.value * self.weights.ability_power_per_point;
                        if champion
                            .scaling_stats
                            .iter()
                            .any(|value| value == "ability_power")
                        {
                            reasons.push(format!("{} 提供法强收益", item.name));
                        }
                    }
                    "ad" | "attack_damage" => {
                        dps += stat.value * self.weights.attack_damage_per_point;
                    }
                    "attack_speed" => {
                        dps += stat.value * self.weights.attack_speed_per_point;
                    }
                    "crit" => {
                        dps += stat.value * self.weights.crit_per_point;
                    }
                    "mana" => {
                        if champion.cast_pattern == "mana_cast" {
                            dps += stat.value * self.weights.mana_per_point;
                            reasons.push(format!("{} 提供启动收益", item.name));
                        }
                    }
                    "hp" => {
                        ehp += stat.value * self.weights.hp_per_point;
                    }
                    "armor" => {
                        ehp += stat.value * self.weights.armor_per_point;
                    }
                    "mr" | "magic_resist" => {
                        ehp += stat.value * self.weights.mr_per_point;
                    }
                    _ => {}
                }
            }

            if champion.cast_pattern == "mana_cast" && has_tag(item, "mana_engine") {
                dps *= self.weights.mana_engine_multiplier;
                reasons.push(format!("{} 提高技能释放频率", item.name));
            }

            if has_tag(item, "shield_survival") || has_tag(item, "sustain") {
                ehp *= self.weights.survival_tag_multiplier;
                reasons.push(format!("{} 提供生存收益", item.name));
            }

            let modifier = self.environment_scorer.score_item(item, environment);
            environment_score += modifier.score;
            reasons.extend(modifier.reasons);
        }

        let item_value_score = ((dps / self.weights.base_dps) * 50.0
            + (ehp / self.weights.base_ehp) * 20.0
            + environment_score as f64)
            .round() as i32;

        CombatValueEstimate {
            simplified_dps: round_one(dps),
            simplified_ehp: round_one(ehp),
            item_value_score,
            environment_score,
            reasons,
        }
    }

    /// 比较两组装备在同一棋子上的收益差
    pub fn compare_item_sets(
        &self,
        champion: &ChampionCapability,
        left_items: &[ItemValueProfile],
        right_items: &[ItemValueProfile],
        environment: &ConflictEnvironment,
    ) -> CombatValueDiff {
        let left = self.estimate(champion, left_items, environment);
        let right = self.estimate(champion, right_items, environment);
        let score_diff = left.item_value_score - right.item_value_score;
        let winner = if score_diff >= 0 { "left" } else { "right" }.to_string();
        let reasons = if score_diff >= 0 {
            left.reasons.clone()
        } else {
            right.reasons.clone()
        };

        CombatValueDiff {
            dps_diff: round_one(left.simplified_dps - right.simplified_dps),
            ehp_diff: round_one(left.simplified_ehp - right.simplified_ehp),
            score_diff,
            winner,
            reasons,
        }
    }

    fn damage_type_matches(&self, champion: &ChampionCapability, item: &ItemValueProfile) -> bool {
        item.damage_type_fit.iter().any(|fit| {
            champion.damage_profile == *fit
                || (fit == "ad"
                    && champion
                        .scaling_stats
                        .iter()
                        .any(|value| value == "attack_damage"))
                || (fit == "ap"
                    && champion
                        .scaling_stats
                        .iter()
                        .any(|value| value == "ability_power"))
                || (fit == "tank" && champion.position_role == "frontline")
        })
    }
}

fn has_tag(item: &ItemValueProfile, tag: &str) -> bool {
    item.effect_tags.iter().any(|value| value == tag)
        || item.conflict_group == tag
        || item.replacement_group == tag
}

fn round_one(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
