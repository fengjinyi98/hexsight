// 海克斯适配评分器
// 核心职责：
// - 评估海克斯与候选阵容的匹配度
// - 判断是否锁阵容方向，优先评估弹性和玩法开关收益
// - 输出第一个海克斯的推荐、风险、后续观察条件

use std::collections::HashMap;

use crate::augment_effect_interpreter::AugmentEffect;
use hexsight_core::{AugmentDecision, LineupProfile};

/// 海克斯评分结果
#[derive(Debug, Clone)]
pub struct AugmentScore {
    pub augment_id: String,
    pub augment_name: String,
    pub total_score: i32,
    /// 通用战力收益 (0-100)
    pub combat_power: i32,
    /// 阵容覆盖度（支持多少套候选阵容）
    pub lineup_coverage: i32,
    /// 锁方向风险 (0-100, 越高越危险)
    pub lock_risk: i32,
    /// 玩法开关价值 (0-100)
    pub playstyle_enable_value: i32,
    /// 是否推荐
    pub is_recommended: bool,
    /// 标签
    pub tags: Vec<String>,
    /// 理由
    pub reasons: Vec<String>,
    /// 支持的阵容 ID 列表
    pub supported_lineup_ids: Vec<String>,
}

/// 海克斯适配评分器
pub struct AugmentFitScorer;

impl AugmentFitScorer {
    /// 评估候选海克斯列表与阵容的匹配度
    pub fn score(
        candidate_augments: &[(String, String)], // (augment_id, augment_name)
        profiles: &[LineupProfile],
        _current_lineup_scores: &HashMap<String, i32>,
        is_first_augment: bool,
        opening_is_strong: bool,
        opening_is_weak: bool,
    ) -> Vec<AugmentScore> {
        candidate_augments
            .iter()
            .map(|(aug_id, aug_name)| {
                let mut reasons = Vec::new();
                let mut tags = Vec::new();
                let mut supported_ids = Vec::new();

                // 计算阵容覆盖度
                let coverage = profiles
                    .iter()
                    .filter(|p| {
                        p.recommended_hex_ids.contains(aug_id)
                            || p.replacement_hex_ids.contains(aug_id)
                    })
                    .count();

                for p in profiles {
                    if p.recommended_hex_ids.contains(aug_id) {
                        supported_ids.push(p.lineup_id.clone());
                        reasons.push(format!("{} 推荐此海克斯", p.name));
                        tags.push("trait_commit".to_string());
                    } else if p.replacement_hex_ids.contains(aug_id) {
                        supported_ids.push(p.lineup_id.clone());
                        reasons.push(format!("{} 备选此海克斯", p.name));
                    }
                }

                let lineup_coverage =
                    (coverage as f64 / profiles.len().max(1) as f64 * 100.0) as i32;

                // 锁方向风险评估
                let lock_risk = if supported_ids.len() <= 1 {
                    if is_first_augment {
                        75
                    } else {
                        40
                    }
                } else if supported_ids.len() <= 3 {
                    if is_first_augment {
                        45
                    } else {
                        20
                    }
                } else {
                    15
                };

                // 玩法开关价值
                let playstyle_value = if coverage >= 2 {
                    70
                } else if coverage == 1 {
                    50
                } else {
                    10
                };

                // 通用战力（根据当前局势判断）
                let combat_power = if opening_is_strong {
                    60
                } else if opening_is_weak {
                    40
                } else {
                    50
                };

                if coverage >= 2 {
                    tags.push("generic_combat".to_string());
                }
                if opening_is_weak {
                    reasons.push("当前开局较弱，优先通用经济/装备海克斯".to_string());
                    tags.push("generic_econ".to_string());
                }

                let total_score = if is_first_augment {
                    // 第一个海克斯：弹性优先
                    (lineup_coverage * 2 + (100 - lock_risk) + combat_power) / 3
                } else {
                    // 后续海克斯：阵容匹配优先
                    (lineup_coverage * 3 + playstyle_value + combat_power) / 3
                };

                let is_recommended = total_score >= 50;

                AugmentScore {
                    augment_id: aug_id.clone(),
                    augment_name: aug_name.clone(),
                    total_score: total_score.clamp(0, 100),
                    combat_power,
                    lineup_coverage,
                    lock_risk,
                    playstyle_enable_value: playstyle_value,
                    is_recommended,
                    tags,
                    reasons,
                    supported_lineup_ids: supported_ids,
                }
            })
            .collect()
    }

    /// 使用 AugmentEffect（来自 AugmentEffectInterpreter）的评分
    /// 复用实时标签、锁方向风险、即时战力
    pub fn score_with_effects(
        effects: &[AugmentEffect],
        profiles: &[LineupProfile],
        _current_lineup_scores: &HashMap<String, i32>,
        is_first_augment: bool,
        opening_is_strong: bool,
        opening_is_weak: bool,
    ) -> Vec<AugmentScore> {
        effects
            .iter()
            .map(|effect| {
                let mut reasons = effect.reason.clone();
                let tags = effect.tags.clone();
                let mut supported_ids = Vec::new();

                // 计算阵容覆盖度
                let coverage = profiles
                    .iter()
                    .filter(|p| {
                        p.recommended_hex_ids.contains(&effect.augment_id)
                            || p.replacement_hex_ids.contains(&effect.augment_id)
                    })
                    .count();

                for p in profiles {
                    if p.recommended_hex_ids.contains(&effect.augment_id)
                        || p.replacement_hex_ids.contains(&effect.augment_id)
                    {
                        supported_ids.push(p.lineup_id.clone());
                    }
                }

                let lineup_coverage =
                    (coverage as f64 / profiles.len().max(1) as f64 * 100.0) as i32;

                // 使用 Interpreter 的锁方向风险，结合阵容覆盖度修正
                let lock_risk = if lineup_coverage >= 60 {
                    (effect.lock_risk as f64 * 0.6) as i32
                } else if lineup_coverage >= 30 {
                    effect.lock_risk
                } else {
                    (effect.lock_risk + 15).min(100)
                };

                // 玩法开关价值：赌狗标签 + 覆盖度
                let playstyle_value = if tags.contains(&"reroll_enable".to_string()) {
                    effect.reroll_value.clamp(0, 100)
                } else if coverage >= 2 {
                    70
                } else if coverage == 1 {
                    50
                } else {
                    10
                };

                // 即时战力 = Interpreter 输出 + 开局修正
                let combat_power = if opening_is_strong {
                    (effect.immediate_power + 10).min(100)
                } else if opening_is_weak {
                    (effect.immediate_power - 5).max(0)
                } else {
                    effect.immediate_power
                };

                if coverage >= 2 && !tags.contains(&"hero_commit".to_string()) {
                    reasons.push("支持多套阵容，弹性好".to_string());
                } else {
                    reasons.push(format!("覆盖 {} 套阵容", coverage));
                }

                let total_score = if is_first_augment {
                    (lineup_coverage * 2 + (100 - lock_risk) + combat_power) / 3
                } else {
                    (lineup_coverage * 3 + playstyle_value + combat_power) / 3
                };

                AugmentScore {
                    augment_id: effect.augment_id.clone(),
                    augment_name: effect.name.clone(),
                    total_score: total_score.clamp(0, 100),
                    combat_power,
                    lineup_coverage,
                    lock_risk,
                    playstyle_enable_value: playstyle_value,
                    is_recommended: total_score >= 50,
                    tags,
                    reasons,
                    supported_lineup_ids: supported_ids,
                }
            })
            .collect()
    }

    /// 第一个海克斯的特殊决策
    pub fn first_augment_decision(scores: &[AugmentScore], current_hp: i32) -> AugmentDecision {
        if scores.is_empty() {
            return AugmentDecision {
                action: "take_fallback".into(),
                recommended: "无候选海克斯".into(),
                lock_lineup: false,
                follow_up: "等待后续海克斯选择".into(),
            };
        }

        let best = scores.iter().max_by_key(|s| s.total_score).unwrap();

        let should_lock =
            best.total_score >= 80 && best.lock_risk < 40 && best.supported_lineup_ids.len() <= 2;

        let mut recommended = format!("{} ({}分)", best.augment_name, best.total_score);
        if should_lock && !best.supported_lineup_ids.is_empty() {
            recommended.push_str(&format!(
                " [可锁: {}]",
                best.supported_lineup_ids.join(", ")
            ));
        }

        let lock_lineup = should_lock;

        let follow_up = if lock_lineup {
            "方向明确，后续优先匹配此阵容装备和棋子".to_string()
        } else if current_hp < 50 {
            "血量偏低，优先保战力，3-2 再确认方向".to_string()
        } else {
            "3-2 前根据来牌和装备确认方向".to_string()
        };

        AugmentDecision {
            action: "take".into(),
            recommended,
            lock_lineup,
            follow_up,
        }
    }
}

// ============================================================

/// 经济规划器
/// 核心职责：
/// - 判断存钱、升级、搜牌、止血动作
/// - 结合玩法标签决定经济节奏
pub struct EconomyPlanner;

impl EconomyPlanner {
    /// 经济动作决策
    pub fn plan(
        current_gold: i32,
        current_hp: i32,
        current_level: i32,
        current_streak: i32,
        round_stage: f64,
        lineup_locked: bool,
        has_reroll_tag: bool,
        reroll_level: Option<i32>,
    ) -> EconomyDecisionResult {
        let mut reasons = Vec::new();

        // 血量危机 → 止血
        if current_hp < 35 {
            reasons.push(format!("血量仅 {}，必须止血", current_hp));
            return EconomyDecisionResult {
                action: "roll_down_to_stabilize".into(),
                label: "全力搜牌止血".into(),
                target_gold: 0,
                reasons,
            };
        }

        // 低血量
        if current_hp < 50 {
            reasons.push("血量偏低，减少贪经济".into());
        }

        // 赌狗节奏
        if has_reroll_tag {
            if let Some(rl) = reroll_level {
                if current_level > rl {
                    reasons.push(format!("已超过赌狗推荐等级 {}，应尽快 D 牌或放弃", rl));
                    return EconomyDecisionResult {
                        action: "reroll_or_pivot".into(),
                        label: format!("卡 {} 级 D 牌或考虑放弃", rl),
                        target_gold: 10,
                        reasons,
                    };
                } else if current_level == rl {
                    if current_gold >= 40 {
                        reasons.push(format!("卡 {} 级赌狗，金币充裕可慢 D", rl));
                        return EconomyDecisionResult {
                            action: "slow_roll".into(),
                            label: format!("卡 {} 级慢 D", rl),
                            target_gold: 30,
                            reasons,
                        };
                    } else {
                        reasons.push(format!("卡 {} 级赌狗，金币紧张需控制", rl));
                        return EconomyDecisionResult {
                            action: "slow_roll_tight".into(),
                            label: format!("卡 {} 级慢 D（经济紧张）", rl),
                            target_gold: 20,
                            reasons,
                        };
                    }
                } else {
                    reasons.push(format!("需要升到 {} 级开始 D 牌", rl));
                    return EconomyDecisionResult {
                        action: "level_to_reroll".into(),
                        label: format!("升到 {} 级", rl),
                        target_gold: 30,
                        reasons,
                    };
                }
            }
        }

        // 速 8/速 9 节奏
        if lineup_locked && current_level < 8 && round_stage >= 3.5 && current_gold >= 40 {
            reasons.push("阵容已锁，金币充裕，优先升人口".to_string());
            return EconomyDecisionResult {
                action: "level_up".into(),
                label: "升人口找核心牌".into(),
                target_gold: 20,
                reasons,
            };
        }

        // 连胜 → 可提前升人口
        if current_streak >= 2 && current_gold >= 30 {
            reasons.push(format!("{}-连胜，可提前升人口保连胜", current_streak));
            return EconomyDecisionResult {
                action: "level_for_streak".into(),
                label: "升人口保连胜".into(),
                target_gold: 10,
                reasons,
            };
        }

        // 连败 → 保利息
        if current_streak <= -2 {
            reasons.push(format!("{}-连败，保利息优先", current_streak.abs()));
            return EconomyDecisionResult {
                action: "save_interest".into(),
                label: "保利息".into(),
                target_gold: 40,
                reasons,
            };
        }

        // 常规运营
        if current_gold >= 40 {
            reasons.push("经济充裕，可选择性搜牌或升人口".to_string());
            EconomyDecisionResult {
                action: "flexible".into(),
                label: "弹性操作".into(),
                target_gold: 20,
                reasons,
            }
        } else if current_gold >= 20 {
            reasons.push("经济中等，优先保利息".to_string());
            EconomyDecisionResult {
                action: "save_interest".into(),
                label: "保利息".into(),
                target_gold: 30,
                reasons,
            }
        } else {
            reasons.push("经济紧张，不要 D 牌".to_string());
            EconomyDecisionResult {
                action: "save_all".into(),
                label: "全力存钱".into(),
                target_gold: 20,
                reasons,
            }
        }
    }
}

/// 经济决策结果
#[derive(Debug, Clone)]
pub struct EconomyDecisionResult {
    pub action: String,
    pub label: String,
    pub target_gold: i32,
    pub reasons: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexsight_core::PlaystyleTag;

    // ---- AugmentFitScorer ----

    fn make_profile(id: &str, rec_hex: &[&str], rep_hex: &[&str]) -> LineupProfile {
        LineupProfile {
            lineup_id: id.into(),
            name: format!("阵容{}", id),
            base_tier: 80,
            playstyle_tags: vec![PlaystyleTag::Standard],
            final_hero_ids: vec![],
            carry_hero_ids: vec![],
            tank_hero_ids: vec![],
            core_equipment_ids: vec![],
            tank_equipment_ids: vec![],
            equipment_order_ids: vec![],
            recommended_hex_ids: rec_hex.iter().map(|s| s.to_string()).collect(),
            replacement_hex_ids: rep_hex.iter().map(|s| s.to_string()).collect(),
            early_hero_ids: vec![],
            mid_hero_ids: vec![],
            trait_targets: Default::default(),
            strategy_texts: Default::default(),
            mode_specific: serde_json::Value::Null,
            carry_costs: vec![4],
            category: None,
        }
    }

    #[test]
    fn first_augment_prefers_flexibility() {
        let profiles = vec![
            make_profile("1", &["hex_a"], &[]),
            make_profile("2", &["hex_b"], &[]),
            make_profile("3", &["hex_a"], &["hex_c"]),
        ];
        let candidates = vec![
            ("hex_a".into(), "海克斯A".into()),
            ("hex_c".into(), "海克斯C".into()),
        ];
        let scores =
            AugmentFitScorer::score(&candidates, &profiles, &HashMap::new(), true, false, true);
        assert!(!scores.is_empty());
        // 覆盖度最高的排前面
        let best = &scores[0];
        assert!(best.lineup_coverage > 0);
    }

    #[test]
    fn first_augment_decision_outputs_lock_or_not() {
        let scores = vec![AugmentScore {
            augment_id: "hex_a".into(),
            augment_name: "海克斯A".into(),
            total_score: 85,
            combat_power: 70,
            lineup_coverage: 60,
            lock_risk: 25,
            playstyle_enable_value: 70,
            is_recommended: true,
            tags: vec!["trait_commit".into()],
            reasons: vec!["阵容1 推荐此海克斯".into()],
            supported_lineup_ids: vec!["1".into()],
        }];
        let decision = AugmentFitScorer::first_augment_decision(&scores, 100);
        assert!(decision.recommended.contains("海克斯A"));
    }

    // ---- EconomyPlanner ----

    #[test]
    fn low_hp_forces_stabilize() {
        let result = EconomyPlanner::plan(20, 30, 6, 0, 3.5, false, false, None);
        assert_eq!(result.action, "roll_down_to_stabilize");
    }

    #[test]
    fn streak_affects_decision() {
        let win = EconomyPlanner::plan(40, 100, 6, 3, 3.0, false, false, None);
        assert!(win.action.contains("level"));

        let loss = EconomyPlanner::plan(40, 100, 6, -3, 3.0, false, false, None);
        assert_eq!(loss.action, "save_interest");
    }

    #[test]
    fn reroll_rhythm() {
        let result = EconomyPlanner::plan(45, 80, 5, 0, 2.5, false, true, Some(5));
        assert!(result.action.contains("slow_roll"));
    }
}
