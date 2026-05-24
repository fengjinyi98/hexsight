// 过渡阵容匹配器 + 风险评分器
// 核心职责：
// - TransitionLineupMatcher：匹配官网过渡阵容，输出前期/中期推荐棋子
// - RiskScorer：输出同行、血量、装备、成型风险与转向条件

// HashMap used in tests

use hexsight_core::LineupProfile;

/// 过渡匹配结果
#[derive(Debug, Clone)]
pub struct TransitionMatch {
    pub lineup_id: String,
    pub lineup_name: String,
    /// 前期过渡英雄命中数
    pub early_hits: i32,
    /// 中期过渡英雄命中数
    pub mid_hits: i32,
    /// 过渡匹配度 (0-100)
    pub transition_score: i32,
    /// 推荐保留英雄
    pub keep_hero_ids: Vec<String>,
    /// 推荐过渡羁绊
    pub transition_traits: Vec<String>,
}

/// 过渡阵容匹配器
pub struct TransitionLineupMatcher;

impl TransitionLineupMatcher {
    /// 根据当前英雄列表匹配官网过渡阵容
    pub fn match_transitions(
        profiles: &[LineupProfile],
        current_hero_ids: &[String],
        round_stage: f64, // 当前阶段
    ) -> Vec<TransitionMatch> {
        profiles
            .iter()
            .map(|profile| {
                let early_hits = profile
                    .early_hero_ids
                    .iter()
                    .filter(|id| current_hero_ids.contains(id))
                    .count() as i32;
                let mid_hits = profile
                    .mid_hero_ids
                    .iter()
                    .filter(|id| current_hero_ids.contains(id))
                    .count() as i32;

                // 前期优先 early，中后期优先 mid
                let (effective_hits, _weight) = if round_stage <= 3.5 {
                    (early_hits * 2 + mid_hits, "early")
                } else {
                    (early_hits + mid_hits * 2, "mid")
                };

                let keep_ids: Vec<String> = current_hero_ids
                    .iter()
                    .filter(|id| {
                        profile.early_hero_ids.contains(id) || profile.mid_hero_ids.contains(id)
                    })
                    .cloned()
                    .collect();

                // 提取过渡羁绊（从 trait_targets 中取前 3 个）
                let mut traits: Vec<(&String, &i32)> = profile.trait_targets.iter().collect();
                traits.sort_by(|a, b| b.1.cmp(a.1));
                let transition_traits: Vec<String> = traits
                    .iter()
                    .take(3)
                    .map(|(id, count)| format!("{} x{}", id, count))
                    .collect();

                let max_possible =
                    (profile.early_hero_ids.len() + profile.mid_hero_ids.len()).max(1) as f64;
                let transition_score =
                    (effective_hits as f64 / max_possible * 100.0).min(100.0) as i32;

                TransitionMatch {
                    lineup_id: profile.lineup_id.clone(),
                    lineup_name: profile.name.clone(),
                    early_hits,
                    mid_hits,
                    transition_score,
                    keep_hero_ids: keep_ids,
                    transition_traits,
                }
            })
            .collect()
    }

    /// 获取过渡最匹配的阵容
    pub fn best_transition(matches: &[TransitionMatch], min_hits: i32) -> Option<&TransitionMatch> {
        matches
            .iter()
            .filter(|m| m.early_hits + m.mid_hits >= min_hits)
            .max_by_key(|m| m.transition_score)
    }
}

// ============================================================

/// 风险等级
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 风险报告
#[derive(Debug, Clone)]
pub struct RiskReport {
    /// 总体风险等级
    pub overall: RiskLevel,
    /// 血量风险
    pub hp_risk: RiskLevel,
    /// 同行风险
    pub rival_risk: RiskLevel,
    /// 装备风险（不匹配阵容）
    pub item_risk: RiskLevel,
    /// 成型风险（三星/高费/专属依赖）
    pub completion_risk: RiskLevel,
    /// 经济风险
    pub economy_risk: RiskLevel,
    /// 锁阵容风险
    pub lock_risk: RiskLevel,
    /// 风险明细
    pub details: Vec<String>,
    /// 建议优先级
    pub priorities: Vec<String>,
    /// 是否需要转向
    pub should_pivot: bool,
    /// 转向条件
    pub pivot_reasons: Vec<String>,
}

/// 风险评分器
pub struct RiskScorer;

impl RiskScorer {
    /// 综合风险评估
    pub fn assess(
        current_hp: i32,
        current_gold: i32,
        current_level: i32,
        lineup_locked: bool,
        rival_count: i32,
        item_match_ratio: f64,      // 装备匹配率 (0.0-1.0)
        _core_hero_hits: i32,       // 核心英雄命中数
        target_carry_costs: &[i32], // 目标阵容的核心费用
        requires_augment: bool,     // 需要专属海克斯
        has_required_augment: bool, // 已拿到关键海克斯
        _round_stage: f64,
    ) -> RiskReport {
        let mut details = Vec::new();
        let mut priorities = Vec::new();
        let mut pivot_reasons = Vec::new();

        // 血量风险
        let hp_risk = if current_hp < 25 {
            details.push(format!("血量仅 {}，极度危险", current_hp));
            priorities.push("立即止血".into());
            RiskLevel::Critical
        } else if current_hp < 35 {
            details.push(format!("血量 {}，进入危险区", current_hp));
            priorities.push("优先保血".into());
            RiskLevel::High
        } else if current_hp < 50 {
            details.push(format!("血量 {}，偏低", current_hp));
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // 同行风险
        let rival_risk = if rival_count >= 3 {
            details.push(format!("同行 {} 家，竞争激烈", rival_count));
            priorities.push("考虑转阵容".into());
            pivot_reasons.push(format!("同行 {} 家", rival_count));
            RiskLevel::High
        } else if rival_count >= 2 {
            details.push(format!("同行 {} 家", rival_count));
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // 装备风险
        let item_risk = if item_match_ratio < 0.2 {
            details.push("装备严重不匹配当前阵容".into());
            priorities.push("优先拿核心装或考虑转阵容".into());
            pivot_reasons.push("装备不匹配".into());
            RiskLevel::High
        } else if item_match_ratio < 0.5 {
            details.push(format!("装备匹配率 {:.0}%", item_match_ratio * 100.0));
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // 成型风险
        let completion_risk = {
            let mut risk = RiskLevel::Low;
            if requires_augment && !has_required_augment {
                details.push("需要专属海克斯但未拿到".into());
                risk = RiskLevel::High;
                pivot_reasons.push("缺少关键海克斯".into());
            }
            if target_carry_costs.iter().any(|&c| c >= 5) && current_level < 7 {
                details.push("核心高费但等级不足".into());
                if risk == RiskLevel::Low {
                    risk = RiskLevel::Medium;
                }
            }
            if target_carry_costs.iter().any(|&c| c >= 5) && current_gold < 20 {
                details.push("核心高费但经济不足".into());
                risk = RiskLevel::High;
            }
            risk
        };

        // 经济风险
        let economy_risk = if current_gold < 10 && current_hp < 50 {
            details.push("经济见底且血量偏低".into());
            RiskLevel::Critical
        } else if current_gold < 10 {
            details.push("经济见底".into());
            RiskLevel::High
        } else if current_gold < 20 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // 锁阵容风险
        let lock_risk = if lineup_locked {
            if item_match_ratio < 0.3 && current_hp < 50 {
                details.push("阵容已锁但装备不匹配，血量低".into());
                priorities.push("考虑放弃硬玩".into());
                pivot_reasons.push("装备不匹配且血量低".into());
                RiskLevel::High
            } else if rival_count >= 2 {
                details.push("阵容已锁且有同行".into());
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            }
        } else {
            RiskLevel::Low
        };

        // 总体风险
        let risk_values = [
            (&hp_risk, 2),
            (&rival_risk, 1),
            (&item_risk, 1),
            (&completion_risk, 1),
            (&economy_risk, 1),
            (&lock_risk, 1),
        ];
        let total_score: i32 = risk_values.iter().map(|(r, w)| risk_to_int(r) * w).sum();
        let overall = if total_score >= 24 {
            RiskLevel::Critical
        } else if total_score >= 16 {
            RiskLevel::High
        } else if total_score >= 10 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let should_pivot = overall == RiskLevel::Critical
            || (overall == RiskLevel::High && pivot_reasons.len() >= 2);

        RiskReport {
            overall,
            hp_risk,
            rival_risk,
            item_risk,
            completion_risk,
            economy_risk,
            lock_risk,
            details,
            priorities,
            should_pivot,
            pivot_reasons,
        }
    }
}

fn risk_to_int(r: &RiskLevel) -> i32 {
    match r {
        RiskLevel::Critical => 4,
        RiskLevel::High => 3,
        RiskLevel::Medium => 2,
        RiskLevel::Low => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn transition_matches_early_stage() {
        let profile = LineupProfile {
            lineup_id: "test".into(),
            name: "测试阵容".into(),
            base_tier: 80,
            playstyle_tags: vec![],
            final_hero_ids: vec![],
            carry_hero_ids: vec![],
            tank_hero_ids: vec![],
            core_equipment_ids: vec![],
            tank_equipment_ids: vec![],
            equipment_order_ids: vec![],
            recommended_hex_ids: vec![],
            replacement_hex_ids: vec![],
            early_hero_ids: vec!["h1".into(), "h2".into()],
            mid_hero_ids: vec!["h3".into()],
            trait_targets: HashMap::from([("t1".into(), 3)]),
            strategy_texts: Default::default(),
            mode_specific: serde_json::Value::Null,
            carry_costs: vec![3],
            category: None,
        };

        let current = vec!["h1".to_string(), "h4".to_string()];
        let matches = TransitionLineupMatcher::match_transitions(&[profile], &current, 2.5);
        assert_eq!(matches[0].early_hits, 1);
        assert_eq!(matches[0].mid_hits, 0);
        assert_eq!(matches[0].keep_hero_ids, vec!["h1"]);
    }

    #[test]
    fn risk_low_when_everything_good() {
        let report = RiskScorer::assess(100, 50, 6, false, 0, 0.8, 5, &[3], false, false, 3.0);
        assert_eq!(report.overall, RiskLevel::Low);
        assert!(!report.should_pivot);
    }

    #[test]
    fn risk_critical_when_hp_low_and_no_items() {
        let report = RiskScorer::assess(20, 5, 5, true, 3, 0.1, 1, &[5], true, false, 4.5);
        assert_eq!(report.overall, RiskLevel::Critical);
        assert!(report.should_pivot);
    }

    #[test]
    fn risk_medium_with_rival() {
        let report = RiskScorer::assess(80, 30, 6, false, 2, 0.6, 4, &[4], false, false, 4.0);
        assert_eq!(report.rival_risk, RiskLevel::Medium);
    }
}
