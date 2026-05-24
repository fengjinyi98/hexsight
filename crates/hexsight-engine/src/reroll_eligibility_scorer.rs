// 赌狗资格评分器
// 核心职责：
// - 判断当前阵容是否适合赌狗路线
// - 评估核心牌数、经济、血量、同行
// - 输出赌狗评分、推荐节奏、停手/放弃条件

use hexsight_core::{LineupProfile, PlaystyleTag, RerollEligibility};

/// 赌狗资格评分器
pub struct RerollEligibilityScorer;

impl RerollEligibilityScorer {
    /// 评估某套阵容的赌狗资格
    pub fn assess(
        profile: &LineupProfile,
        core_hero_counts: &[(String, i32)], // (hero_id, 当前持有数)
        current_gold: i32,
        current_hp: i32,
        _current_level: i32,
        rival_count: i32,
        round_stage: f64, // e.g., 2.1, 3.2, 4.1
    ) -> RerollEligibility {
        let is_reroll = profile.playstyle_tags.iter().any(|t| {
            matches!(
                t,
                PlaystyleTag::Reroll1Cost | PlaystyleTag::Reroll2Cost | PlaystyleTag::Reroll3Cost
            )
        });

        if !is_reroll || profile.carry_costs.is_empty() {
            return RerollEligibility {
                score: 0,
                lineup_id: profile.lineup_id.clone(),
                core_cost: 0,
                current_core_count: 0,
                target_count: 9,
                recommended_level: 0,
                tempo: "不适合赌狗".into(),
                stop_conditions: vec![],
                abandon_conditions: vec!["非赌狗阵容".into()],
                rival_count,
                is_recommended: false,
            };
        }

        let core_cost = *profile.carry_costs.iter().min().unwrap_or(&1);
        let core_hero_ids: Vec<&str> = profile.carry_hero_ids.iter().map(|s| s.as_str()).collect();

        // 统计当前持有的核心牌数
        let current_core_count: i32 = core_hero_counts
            .iter()
            .filter(|(id, _)| core_hero_ids.contains(&id.as_str()))
            .map(|(_, count)| *count)
            .sum();

        // 基础评分：核心牌越多越好
        let mut score = (current_core_count as f64 / 9.0 * 50.0) as i32;

        // 经济加分
        if current_gold >= 30 {
            score += 15;
        } else if current_gold >= 20 {
            score += 10;
        } else if current_gold >= 10 {
            score += 5;
        }

        // 血量安全加分
        if current_hp >= 70 {
            score += 15;
        } else if current_hp >= 50 {
            score += 8;
        } else if current_hp < 35 {
            score -= 20;
        }

        // 同行扣分
        if rival_count == 0 {
            score += 10;
        } else if rival_count == 1 {
            score -= 5;
        } else if rival_count >= 2 {
            score -= 20;
        }

        // 阶段适配
        if core_cost == 1 && round_stage <= 3.5 {
            score += 10;
        } else if core_cost == 1 && round_stage > 3.5 {
            score -= 30;
        }
        if core_cost == 2 && round_stage <= 4.5 {
            score += 5;
        }
        if core_cost == 3 && (3.5..=5.5).contains(&round_stage) {
            score += 10;
        }

        // 确定推荐等级
        let recommended_level = match core_cost {
            1 => 5,
            2 => 6,
            _ => 7,
        };

        let tempo = match core_cost {
            1 => "3-1 卡 5 级大 D 或慢 D".into(),
            2 => "3-2 升 6 后卡 6 慢 D".into(),
            _ => "4-1/4-2 七级慢 D".into(),
        };

        let stop_conditions = vec![
            format!("主 C 三星（需 {} 张）", 9 - current_core_count),
            "金币低于 20".into(),
        ];

        let mut abandon_conditions = vec![
            format!("同行 ≥ 2 家（当前 {} 家）", rival_count),
            "核心牌少于 4 张且已过关键 D 牌阶段".into(),
            format!("血量低于 35（当前 {}）", current_hp),
        ];

        if core_cost == 1 && round_stage > 3.5 {
            abandon_conditions.push("已过 1 费赌狗窗口".into());
        }

        score = score.clamp(0, 100);
        let is_recommended = score >= 60;

        RerollEligibility {
            score,
            lineup_id: profile.lineup_id.clone(),
            core_cost,
            current_core_count,
            target_count: 9,
            recommended_level,
            tempo,
            stop_conditions,
            abandon_conditions,
            rival_count,
            is_recommended,
        }
    }

    /// 从阵容列表中筛选适合赌狗的阵容
    pub fn find_reroll_candidates(profiles: &[LineupProfile]) -> Vec<&LineupProfile> {
        profiles
            .iter()
            .filter(|p| {
                p.playstyle_tags.iter().any(|t| {
                    matches!(
                        t,
                        PlaystyleTag::Reroll1Cost
                            | PlaystyleTag::Reroll2Cost
                            | PlaystyleTag::Reroll3Cost
                    )
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_profile(carry_costs: Vec<i32>, tags: Vec<PlaystyleTag>) -> LineupProfile {
        LineupProfile {
            lineup_id: "test".into(),
            name: "测试阵容".into(),
            base_tier: 80,
            playstyle_tags: tags,
            final_hero_ids: vec![],
            carry_hero_ids: carry_costs.iter().map(|c| format!("hero_{}", c)).collect(),
            tank_hero_ids: vec![],
            core_equipment_ids: vec![],
            tank_equipment_ids: vec![],
            equipment_order_ids: vec![],
            recommended_hex_ids: vec![],
            replacement_hex_ids: vec![],
            early_hero_ids: vec![],
            mid_hero_ids: vec![],
            trait_targets: Default::default(),
            strategy_texts: Default::default(),
            mode_specific: serde_json::Value::Null,
            carry_costs,
            category: None,
        }
    }

    #[test]
    fn non_reroll_lineup_returns_zero() {
        let profile = make_profile(vec![4], vec![PlaystyleTag::Standard]);
        let result = RerollEligibilityScorer::assess(&profile, &[], 40, 100, 6, 0, 3.1);
        assert!(!result.is_recommended);
        assert_eq!(result.score, 0);
    }

    #[test]
    fn reroll_with_good_conditions() {
        let profile = make_profile(
            vec![1],
            vec![PlaystyleTag::Standard, PlaystyleTag::Reroll1Cost],
        );
        let counts = vec![("hero_1".into(), 5)];
        let result = RerollEligibilityScorer::assess(&profile, &counts, 40, 100, 5, 0, 2.5);
        assert!(result.is_recommended);
        assert!(result.score >= 60);
    }

    #[test]
    fn reroll_with_rival_and_low_hp() {
        let profile = make_profile(
            vec![1],
            vec![PlaystyleTag::Standard, PlaystyleTag::Reroll1Cost],
        );
        let counts = vec![("hero_1".into(), 3)];
        let result = RerollEligibilityScorer::assess(&profile, &counts, 15, 30, 5, 2, 3.5);
        assert!(!result.is_recommended);
    }
}
