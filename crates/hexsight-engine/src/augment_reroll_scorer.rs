// 海克斯刷新决策评分器
// 核心职责：
// - 对每轮三个候选海克斯排序
// - 根据刷新机会输出 take / reroll / take_fallback
// - 保留推荐海克斯、兜底海克斯、锁方向风险和短解释

use std::collections::HashMap;
use std::path::Path;

use hexsight_core::{HexError, HexResult};

/// 海克斯轮次
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AugmentStage {
    #[serde(rename = "first")]
    First,
    #[serde(rename = "second")]
    Second,
    #[serde(rename = "third")]
    Third,
}

/// 海克斯决策动作
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AugmentDecisionAction {
    #[serde(rename = "take")]
    Take,
    #[serde(rename = "reroll")]
    Reroll,
    #[serde(rename = "take_fallback")]
    TakeFallback,
}

/// 单个海克斯排序结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedAugmentOption {
    pub augment_id: String,
    pub augment_name: String,
    pub total_score: i32,
    pub current_value: i32,
    pub lineup_coverage: i32,
    pub lock_risk: i32,
    pub fallback_score: i32,
    pub tags: Vec<String>,
    pub supported_lineup_ids: Vec<String>,
    pub reason: Vec<String>,
}

/// 海克斯局势上下文
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AugmentSituationContext {
    pub current_hp: i32,
    pub current_gold: i32,
    pub item_gap_level: i32,
    pub core_hero_count: i32,
    pub locked_lineup_id: Option<String>,
    #[serde(default)]
    pub existing_effect_tags: Vec<String>,
    #[serde(default)]
    pub required_effect_tags: Vec<String>,
    #[serde(default)]
    pub preferred_damage_profile: Option<String>,
}

/// 海克斯刷新决策输入
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AugmentRerollContext {
    pub ranked_options: Vec<RankedAugmentOption>,
    pub has_reroll: bool,
    pub stage: AugmentStage,
    pub current_hp: i32,
    pub current_gold: i32,
    pub locked_lineup_id: Option<String>,
}

/// 海克斯刷新决策输出
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AugmentRerollDecision {
    pub action: AugmentDecisionAction,
    pub recommended_augment_id: Option<String>,
    pub recommended_augment_name: Option<String>,
    pub fallback_augment_id: Option<String>,
    pub lock_risk: i32,
    pub expected_reroll_gain: i32,
    pub reason: Vec<String>,
    pub ranked_options: Vec<RankedAugmentOption>,
}

/// 海克斯刷新阈值配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AugmentThresholds {
    pub version: String,
    #[serde(default = "AugmentThresholds::default_first_min_take_score")]
    pub first_min_take_score: i32,
    #[serde(default = "AugmentThresholds::default_later_min_take_score")]
    pub later_min_take_score: i32,
    #[serde(default = "AugmentThresholds::default_fallback_min_score")]
    pub fallback_min_score: i32,
    #[serde(default = "AugmentThresholds::default_reroll_expected_gain")]
    pub reroll_expected_gain: i32,
    #[serde(default = "AugmentThresholds::default_low_hp_take_bonus")]
    pub low_hp_take_bonus: i32,
    #[serde(default = "AugmentThresholds::default_low_hp_threshold")]
    pub low_hp_threshold: i32,
}

impl AugmentThresholds {
    fn default_first_min_take_score() -> i32 {
        65
    }

    fn default_later_min_take_score() -> i32 {
        58
    }

    fn default_fallback_min_score() -> i32 {
        35
    }

    fn default_reroll_expected_gain() -> i32 {
        12
    }

    fn default_low_hp_take_bonus() -> i32 {
        8
    }

    fn default_low_hp_threshold() -> i32 {
        45
    }

    fn min_take_score(&self, stage: AugmentStage, current_hp: i32) -> i32 {
        let base = match stage {
            AugmentStage::First => self.first_min_take_score,
            AugmentStage::Second | AugmentStage::Third => self.later_min_take_score,
        };
        if current_hp <= self.low_hp_threshold {
            (base - self.low_hp_take_bonus).max(self.fallback_min_score)
        } else {
            base
        }
    }
}

impl Default for AugmentThresholds {
    fn default() -> Self {
        Self {
            version: "default".into(),
            first_min_take_score: Self::default_first_min_take_score(),
            later_min_take_score: Self::default_later_min_take_score(),
            fallback_min_score: Self::default_fallback_min_score(),
            reroll_expected_gain: Self::default_reroll_expected_gain(),
            low_hp_take_bonus: Self::default_low_hp_take_bonus(),
            low_hp_threshold: Self::default_low_hp_threshold(),
        }
    }
}

/// 海克斯类型权重配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AugmentTypeWeights {
    pub version: String,
    #[serde(default)]
    pub weights: HashMap<String, i32>,
}

impl Default for AugmentTypeWeights {
    fn default() -> Self {
        Self {
            version: "default".into(),
            weights: HashMap::from([
                ("generic_combat".into(), 8),
                ("generic_econ".into(), 5),
                ("generic_item".into(), 10),
                ("trait_commit".into(), -6),
                ("hero_commit".into(), -18),
                ("reroll_enable".into(), 6),
                ("tempo".into(), 7),
                ("late_cap".into(), 2),
                ("mode_special".into(), -4),
                ("high_risk".into(), -10),
            ]),
        }
    }
}

/// P4 配置加载器
pub struct AugmentRerollConfigLoader;

impl AugmentRerollConfigLoader {
    pub fn load_thresholds(path: &Path) -> HexResult<AugmentThresholds> {
        if !path.exists() {
            return Ok(AugmentThresholds::default());
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            HexError::Config(format!("读取海克斯阈值配置失败 {}: {}", path.display(), e))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            HexError::Config(format!("解析海克斯阈值配置失败 {}: {}", path.display(), e))
        })
    }

    pub fn load_type_weights(path: &Path) -> HexResult<AugmentTypeWeights> {
        if !path.exists() {
            return Ok(AugmentTypeWeights::default());
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            HexError::Config(format!(
                "读取海克斯类型权重配置失败 {}: {}",
                path.display(),
                e
            ))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            HexError::Config(format!(
                "解析海克斯类型权重配置失败 {}: {}",
                path.display(),
                e
            ))
        })
    }
}

/// 海克斯刷新决策器
pub struct AugmentRerollScorer;

impl AugmentRerollScorer {
    pub fn decide(context: &AugmentRerollContext) -> AugmentRerollDecision {
        Self::decide_with_thresholds(context, &AugmentThresholds::default())
    }

    pub fn decide_with_thresholds(
        context: &AugmentRerollContext,
        thresholds: &AugmentThresholds,
    ) -> AugmentRerollDecision {
        if context.ranked_options.is_empty() {
            return AugmentRerollDecision {
                action: AugmentDecisionAction::TakeFallback,
                recommended_augment_id: None,
                recommended_augment_name: None,
                fallback_augment_id: None,
                lock_risk: 0,
                expected_reroll_gain: 0,
                reason: vec!["未识别到候选海克斯".into()],
                ranked_options: vec![],
            };
        }

        let mut ranked = context.ranked_options.clone();
        ranked.sort_by(|a, b| {
            b.total_score
                .cmp(&a.total_score)
                .then_with(|| b.fallback_score.cmp(&a.fallback_score))
                .then_with(|| a.lock_risk.cmp(&b.lock_risk))
        });

        let best = ranked.first().expect("ranked is not empty");
        let fallback = ranked
            .iter()
            .max_by(|a, b| {
                a.fallback_score
                    .cmp(&b.fallback_score)
                    .then_with(|| a.lineup_coverage.cmp(&b.lineup_coverage))
                    .then_with(|| b.lock_risk.cmp(&a.lock_risk))
            })
            .expect("ranked is not empty");
        let min_take_score = thresholds.min_take_score(context.stage, context.current_hp);
        let expected_reroll_gain =
            Self::expected_reroll_gain(best, context.stage, context.current_hp);

        if best.total_score >= min_take_score {
            return AugmentRerollDecision {
                action: AugmentDecisionAction::Take,
                recommended_augment_id: Some(best.augment_id.clone()),
                recommended_augment_name: Some(best.augment_name.clone()),
                fallback_augment_id: Some(fallback.augment_id.clone()),
                lock_risk: best.lock_risk,
                expected_reroll_gain,
                reason: vec![format!(
                    "{} 达到当前阶段最低可接受分 {}",
                    best.augment_name, min_take_score
                )],
                ranked_options: ranked,
            };
        }

        if context.has_reroll && expected_reroll_gain >= thresholds.reroll_expected_gain {
            return AugmentRerollDecision {
                action: AugmentDecisionAction::Reroll,
                recommended_augment_id: None,
                recommended_augment_name: None,
                fallback_augment_id: Some(fallback.augment_id.clone()),
                lock_risk: best.lock_risk,
                expected_reroll_gain,
                reason: vec![format!(
                    "三个候选都低于最低可接受分 {}，保留兜底 {}，建议刷新",
                    min_take_score, fallback.augment_name
                )],
                ranked_options: ranked,
            };
        }

        AugmentRerollDecision {
            action: AugmentDecisionAction::TakeFallback,
            recommended_augment_id: Some(fallback.augment_id.clone()),
            recommended_augment_name: Some(fallback.augment_name.clone()),
            fallback_augment_id: Some(fallback.augment_id.clone()),
            lock_risk: fallback.lock_risk,
            expected_reroll_gain,
            reason: vec![format!(
                "刷新机会不足或收益不足，兜底选择 {}，覆盖 {} 分且锁方向风险 {}",
                fallback.augment_name, fallback.lineup_coverage, fallback.lock_risk
            )],
            ranked_options: ranked,
        }
    }

    fn expected_reroll_gain(
        best: &RankedAugmentOption,
        stage: AugmentStage,
        current_hp: i32,
    ) -> i32 {
        let target = match stage {
            AugmentStage::First => 68,
            AugmentStage::Second | AugmentStage::Third => 62,
        };
        let hp_adjust = if current_hp <= 45 { -6 } else { 0 };
        (target + hp_adjust - best.total_score).max(0)
    }
}
