// 规则决策规划器
// 核心职责：
// - 整合开局路线、阵容评分、装备匹配、赌狗资格、过渡战力
// - 输出结构化 RuleOutput JSON（对齐文档第十五章）
// - 为每个推荐生成理由、风险和转向条件

use hexsight_core::{
    AugmentDecision, EconomyDecision, ItemDecision, LineupRecommendation,
    LineupScore, OpeningRouteResult, RerollEligibility, RuleOutput, TransitionDecision,
};

/// 规则决策规划器
pub struct DecisionPlanner;

impl DecisionPlanner {
    /// 生成完整规则输出
    pub fn plan(
        opening: &OpeningRouteResult,
        lineup_scores: &[LineupScore],
        economy_action: (&str, &str),
        item_action: (&str, &str),
        reroll_candidates: &[RerollEligibility],
        current_hp: i32,
    ) -> RuleOutput {
        // Top 3 阵容
        let mut sorted: Vec<&LineupScore> = lineup_scores.iter().collect();
        sorted.sort_by(|a, b| b.total_score.cmp(&a.total_score));
        let top3: Vec<&LineupScore> = sorted.into_iter().take(3).collect();

        let lineup_recommendations: Vec<LineupRecommendation> = top3.iter().map(|s| {
            LineupRecommendation {
                lineup_id: s.lineup_id.clone(),
                name: s.name.clone(),
                score: s.total_score,
                reason: s.reasons.clone(),
                risk: s.risks.clone(),
            }
        }).collect();

        // 策略描述
        let strategy = match opening.route {
            hexsight_core::OpeningRoute::WinStreak => "连胜".to_string(),
            hexsight_core::OpeningRoute::LossStreak => "精致连败".to_string(),
            hexsight_core::OpeningRoute::Mixed => "混合过渡".to_string(),
        };

        // 经济决策
        let economy_action = EconomyDecision {
            action: economy_action.0.to_string(),
            reason: economy_action.1.to_string(),
        };

        // 装备决策
        let item_action = ItemDecision {
            action: item_action.0.to_string(),
            reason: item_action.1.to_string(),
        };

        // 海克斯决策
        let best_lineup = lineup_recommendations.first();
        let (augment_action, lock_lineup, follow_up) = if let Some(top) = best_lineup {
            if top.score >= 75 {
                (
                    format!("优先选择 {} 的推荐海克斯", top.name),
                    true,
                    "阵容方向明确，拿专属强化后锁定阵容".to_string(),
                )
            } else {
                (
                    "通用经济海克斯或通用战力海克斯".to_string(),
                    false,
                    "3-2 前根据来牌和装备确认方向".to_string(),
                )
            }
        } else {
            (
                "通用经济海克斯".to_string(),
                false,
                "当前无明确方向，保留弹性".to_string(),
            )
        };

        let augment_action = AugmentDecision {
            recommended: augment_action,
            lock_lineup,
            follow_up,
        };

        // 过渡决策
        let transition_action = match opening.route {
            hexsight_core::OpeningRoute::WinStreak => TransitionDecision {
                route: "win_streak_maintain".into(),
                target_damage_per_round: "0-2".into(),
                stop_loss_stage: "N/A".into(),
            },
            hexsight_core::OpeningRoute::LossStreak => TransitionDecision {
                route: "loss_streak_control".into(),
                target_damage_per_round: "4-8".into(),
                stop_loss_stage: "3-2".into(),
            },
            hexsight_core::OpeningRoute::Mixed => TransitionDecision {
                route: "mixed_transition".into(),
                target_damage_per_round: "2-5".into(),
                stop_loss_stage: "3-5".into(),
            },
        };

        // 转向条件
        let mut pivot_conditions = Vec::new();
        if current_hp < 45 {
            pivot_conditions.push("血量低于 45，停止贪经济".to_string());
        }
        if current_hp < 35 {
            pivot_conditions.push("血量低于 35，立即止血".to_string());
        }
        if let Some(top) = best_lineup {
            if top.score < 60 {
                pivot_conditions.push("当前最佳阵容评分低于 60，继续观察".to_string());
            }
        }
        // 赌狗转向条件
        for reroll in reroll_candidates {
            if reroll.is_recommended {
                for cond in &reroll.abandon_conditions {
                    pivot_conditions.push(format!("[{}] {}", reroll.lineup_id, cond));
                }
            }
        }

        RuleOutput {
            strategy,
            lineup_recommendations,
            economy_action,
            item_action,
            augment_action,
            transition_action,
            pivot_conditions,
            fight_outcome: None,
            generated_at: chrono_now(),
            engine_version: "0.2.0".into(),
        }
    }
}

/// 简易时间戳（不依赖 chrono crate）
fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexsight_core::{OpeningRoute, OpeningRouteResult};

    #[test]
    fn plan_produces_valid_json() {
        let opening = OpeningRouteResult {
            route: OpeningRoute::Mixed,
            confidence: 0.6,
            reasons: vec!["状态中等".into()],
            two_star_count: 1,
            frontline_quality: 45,
            can_build_combat_item: false,
            recommended_actions: vec!["保血量".into()],
        };

        let scores = vec![
            LineupScore {
                lineup_id: "4514".into(), name: "神谕龙王".into(),
                total_score: 86, base_score: 80,
                item_fit_score: 70, champion_hit_score: 60,
                augment_fit_score: 50, trait_fit_score: 60,
                stage_fit_score: 70, economy_fit_score: 75,
                health_safety_score: 90, playstyle_switch_score: 50,
                rival_penalty: 0, difficulty_penalty: 0,
                reasons: vec!["装备匹配".into()],
                risks: vec![],
                requires_augment: false,
            },
        ];

        let output = DecisionPlanner::plan(
            &opening, &scores,
            (&"save_interest", &"血量安全，保利息"),
            (&"hold_or_wait", &"未定阵容，等待方向"),
            &[], 100,
        );

        assert_eq!(output.strategy, "混合过渡");
        assert_eq!(output.lineup_recommendations.len(), 1);
        assert_eq!(output.lineup_recommendations[0].name, "神谕龙王");
        assert!(output.pivot_conditions.is_empty());

        // 验证 JSON 可序列化
        let json = serde_json::to_string_pretty(&output).unwrap();
        assert!(json.contains("lineupRecommendations"));
        assert!(json.contains("economyAction"));
        assert!(json.contains("augmentAction"));
    }
}
