// 规则决策规划器
// 核心职责：
// - 整合开局路线、阵容评分、装备匹配、赌狗资格、过渡战力
// - 输出结构化 RuleOutput JSON（对齐文档第十五章）
// - 为每个推荐生成理由、风险和转向条件

use hexsight_core::{
    AugmentDecision, EconomyDecision, FightOutcome, ItemDecision, LineupRecommendation,
    LineupScore, OpeningRouteResult, RerollEligibility, RuleOutput, TransitionDecision,
};
use crate::augment_economy_planner::AugmentScore;
use crate::augment_economy_planner::EconomyDecisionResult;
use crate::augment_reroll_scorer::{AugmentDecisionAction, AugmentRerollDecision};
use crate::transition_risk_scorer::{RiskReport, TransitionMatch};

/// 规则决策规划器
pub struct DecisionPlanner;

impl DecisionPlanner {
    /// 生成完整规则输出
    #[deprecated(note = "使用 plan() 代替，旧 plan 不集成 P1 模块")]
    pub fn plan_legacy(
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
            action: "take".into(),
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

    /// 完整规则输出（集成 P1/P2 模块结果）—— 主入口
    pub fn plan(
        opening: &OpeningRouteResult,
        lineup_scores: &[LineupScore],
        item_fit_direction: &str,
        economy_result: &EconomyDecisionResult,
        augment_scores: &[AugmentScore],
        is_first_augment: bool,
        risk_report: &RiskReport,
        reroll_candidates: &[RerollEligibility],
        transition_matches: &[TransitionMatch],
        fight_outcome: Option<FightOutcome>,
        _current_hp: i32,
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
                risk: {
                    let mut r = s.risks.clone();
                    r.extend(risk_report.details.iter().cloned());
                    r
                },
            }
        }).collect();

        let strategy = match opening.route {
            hexsight_core::OpeningRoute::WinStreak => "连胜".to_string(),
            hexsight_core::OpeningRoute::LossStreak => "精致连败".to_string(),
            hexsight_core::OpeningRoute::Mixed => "混合过渡".to_string(),
        };

        // 经济决策（来自 EconomyPlanner）
        let economy_action = EconomyDecision {
            action: economy_result.action.clone(),
            reason: economy_result.reasons.join("; "),
        };

        // 装备决策（来自 ItemFitScorer 方向）
        let item_action = ItemDecision {
            action: match item_fit_direction {
                "AD" => "build_ad".into(),
                "AP" => "build_ap".into(),
                "坦" => "build_tank".into(),
                _ => "hold_flexible".into(),
            },
            reason: format!("当前装备方向: {}", item_fit_direction),
        };

        // 海克斯决策（来自 AugmentFitScorer）
        let (augment_action, lock_lineup, follow_up) = if let Some(best) = augment_scores.iter()
            .filter(|s| s.is_recommended)
            .max_by_key(|s| s.total_score)
        {
            let lf = if best.supported_lineup_ids.len() <= 2 {
                format!("锁定阵容: {}", best.supported_lineup_ids.join(", "))
            } else {
                "继续观察".to_string()
            };
            let lock = best.supported_lineup_ids.len() <= 2 && best.total_score >= 70;
            (format!("{} ({}分)", best.augment_name, best.total_score), lock, lf)
        } else if is_first_augment {
            ("通用经济/装备海克斯".into(), false, "3-2 前根据来牌确认方向".into())
        } else {
            ("优先补阵容匹配海克斯".into(), false, "根据当前阵容补强".into())
        };

        let augment_action = AugmentDecision {
            action: "take".into(),
            recommended: augment_action,
            lock_lineup,
            follow_up,
        };

        // 过渡决策
        let best_transition = transition_matches.iter()
            .max_by_key(|m| m.transition_score);
        let transition_action = if let Some(tm) = best_transition {
            TransitionDecision {
                route: format!("匹配 {}", tm.lineup_name),
                target_damage_per_round: match risk_report.overall {
                    crate::transition_risk_scorer::RiskLevel::Low => "0-3".into(),
                    crate::transition_risk_scorer::RiskLevel::Medium => "3-6".into(),
                    crate::transition_risk_scorer::RiskLevel::High => "5-8".into(),
                    crate::transition_risk_scorer::RiskLevel::Critical => "立即止血".into(),
                },
                stop_loss_stage: match opening.route {
                    hexsight_core::OpeningRoute::LossStreak => "3-2".into(),
                    _ => "3-5".into(),
                },
            }
        } else {
            TransitionDecision {
                route: "无明确过渡匹配".into(),
                target_damage_per_round: "2-6".into(),
                stop_loss_stage: "3-2".into(),
            }
        };

        // 转向条件（整合阵容风险 + 赌狗放弃条件）
        let mut pivot_conditions: Vec<String> = risk_report.pivot_reasons.clone();
        for reroll in reroll_candidates {
            if !reroll.is_recommended {
                pivot_conditions.extend(reroll.abandon_conditions.clone());
            }
        }
        if pivot_conditions.is_empty() && lineup_recommendations.first().map(|r| r.score).unwrap_or(0) < 50 {
            pivot_conditions.push("当前最佳阵容评分偏低，继续观察".to_string());
        }

        RuleOutput {
            strategy,
            lineup_recommendations,
            economy_action,
            item_action,
            augment_action,
            transition_action,
            pivot_conditions,
            fight_outcome,
            generated_at: chrono_now(),
            engine_version: "0.3.0".into(),
        }
    }

    /// 完整规则输出（集成 P4 海克斯刷新决策）—— P4 主入口
    pub fn plan_with_augment_reroll(
        opening: &OpeningRouteResult,
        lineup_scores: &[LineupScore],
        item_fit_direction: &str,
        economy_result: &EconomyDecisionResult,
        augment_scores: &[AugmentScore],
        is_first_augment: bool,
        augment_reroll: Option<&AugmentRerollDecision>,
        risk_report: &RiskReport,
        reroll_candidates: &[RerollEligibility],
        transition_matches: &[TransitionMatch],
        fight_outcome: Option<FightOutcome>,
        current_hp: i32,
    ) -> RuleOutput {
        let mut output = Self::plan(
            opening,
            lineup_scores,
            item_fit_direction,
            economy_result,
            augment_scores,
            is_first_augment,
            risk_report,
            reroll_candidates,
            transition_matches,
            fight_outcome,
            current_hp,
        );

        if let Some(decision) = augment_reroll {
            output.augment_action = Self::augment_decision_from_reroll(decision);
            output.engine_version = "0.4.0".into();
        }

        output
    }

    fn augment_decision_from_reroll(decision: &AugmentRerollDecision) -> AugmentDecision {
        let action = match decision.action {
            AugmentDecisionAction::Take => "take",
            AugmentDecisionAction::Reroll => "reroll",
            AugmentDecisionAction::TakeFallback => "take_fallback",
        }.to_string();

        let recommended = match decision.action {
            AugmentDecisionAction::Take => decision.recommended_augment_name.as_ref()
                .map(|name| format!("拿 {}", name))
                .unwrap_or_else(|| "拿当前最高分海克斯".into()),
            AugmentDecisionAction::Reroll => "刷新当前三个海克斯".into(),
            AugmentDecisionAction::TakeFallback => decision.recommended_augment_name.as_ref()
                .map(|name| format!("兜底拿 {}", name))
                .unwrap_or_else(|| "兜底拿锁方向风险最低的海克斯".into()),
        };

        let lock_lineup = decision.action == AugmentDecisionAction::Take
            && decision.lock_risk <= 35
            && decision.ranked_options.first()
                .map(|option| option.supported_lineup_ids.len() <= 2)
                .unwrap_or(false);

        AugmentDecision {
            action,
            recommended,
            lock_lineup,
            follow_up: decision.reason.join("; "),
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
    use hexsight_core::OpeningRouteResult;

    #[test]
    fn plan_full_integrates_all_p1_outputs() {
        use crate::augment_economy_planner::{AugmentScore, EconomyDecisionResult};
        use crate::transition_risk_scorer::{RiskLevel, RiskReport, TransitionMatch};

        let opening = OpeningRouteResult {
            route: hexsight_core::OpeningRoute::Mixed,
            confidence: 0.65,
            reasons: vec!["状态中等".into()],
            two_star_count: 2,
            frontline_quality: 50,
            can_build_combat_item: true,
            recommended_actions: vec!["保血量".into()],
        };

        let scores = vec![LineupScore {
            lineup_id: "4514".into(), name: "神谕龙王".into(),
            total_score: 82, base_score: 80,
            item_fit_score: 70, champion_hit_score: 60,
            augment_fit_score: 50, trait_fit_score: 60,
            stage_fit_score: 70, economy_fit_score: 75,
            health_safety_score: 90, playstyle_switch_score: 50,
            rival_penalty: 0, difficulty_penalty: 0,
            reasons: vec!["装备匹配".into()], risks: vec![],
            requires_augment: false,
        }];

        let economy = EconomyDecisionResult {
            action: "save_interest".into(), label: "保利息".into(),
            target_gold: 30,
            reasons: vec!["经济中等".into()],
        };

        let augment_scores = vec![AugmentScore {
            augment_id: "hex_a".into(), augment_name: "通用战力".into(),
            total_score: 65, combat_power: 70, lineup_coverage: 60,
            lock_risk: 20, playstyle_enable_value: 50,
            is_recommended: true, tags: vec!["generic_combat".into()],
            reasons: vec!["提供即时战力".into()],
            supported_lineup_ids: vec!["4514".into(), "other".into()],
        }];

        let risk = RiskReport {
            overall: RiskLevel::Low,
            hp_risk: RiskLevel::Low, rival_risk: RiskLevel::Low,
            item_risk: RiskLevel::Low, completion_risk: RiskLevel::Low,
            economy_risk: RiskLevel::Low, lock_risk: RiskLevel::Low,
            details: vec!["当前局势稳定".into()],
            priorities: vec!["保经济".into()],
            should_pivot: false, pivot_reasons: vec![],
        };

        let transitions = vec![TransitionMatch {
            lineup_id: "4514".into(), lineup_name: "神谕龙王".into(),
            early_hits: 2, mid_hits: 1, transition_score: 65,
            keep_hero_ids: vec!["h1".into()],
            transition_traits: vec!["牧羊人 x3".into()],
        }];

        let output = DecisionPlanner::plan(
            &opening, &scores, "AD",
            &economy, &augment_scores, true,
            &risk, &[], &transitions, None, 100,
        );

        assert_eq!(output.strategy, "混合过渡");
        assert_eq!(output.lineup_recommendations.len(), 1);
        assert!(output.economy_action.action.contains("save"));
        assert!(output.augment_action.recommended.contains("通用战力"));
        assert!(!output.augment_action.lock_lineup);
        assert!(output.pivot_conditions.is_empty());
        assert!(output.fight_outcome.is_none());

        let json = serde_json::to_string_pretty(&output).unwrap();
        assert!(json.contains("lineupRecommendations"));
        assert!(json.contains("economyAction"));
        assert!(json.contains("augmentAction"));
        assert!(json.contains("transitionAction"));
        assert!(json.contains("pivotConditions"));
    }

    #[test]
    fn p2_fight_outcome_in_rule_output() {
        use crate::board_power_fight::{BoardPowerScorer, FightOutcomeEstimator};
        use crate::DamageProfile;
        use crate::augment_economy_planner::{AugmentScore, EconomyDecisionResult};
        use crate::transition_risk_scorer::{RiskLevel, RiskReport, TransitionMatch};
        use hexsight_core::ChampionCombatProfile;
        use hexsight_core::RulePack;

        let pack = RulePack::default();
        let make = |hp, ad, armor, tank| -> ChampionCombatProfile {
            let mut roles = vec!["副C".to_string()];
            if tank { roles.push("主坦".to_string()); }
            ChampionCombatProfile {
                hero_id: "t".into(), name: "t".into(), cost: 3, role_tags: roles,
                damage_type: "物理".into(), attack_pattern: vec!["单体".into()],
                targeting: "当前目标".into(), cast_tempo: "中启动".into(),
                special_tags: vec![], base_hp: hp, base_armor: armor,
                base_mr: 30, base_ad: ad, base_as: 0.7,
                effective_hp: (hp as f64 * (1.0 + armor as f64 / 100.0)) as i32,
            }
        };

        let our_power = BoardPowerScorer::compute(
            &[make(900, 50, 60, true), make(600, 85, 30, false)],
            2, 1, false, &pack,
        );
        let enemy_power = BoardPowerScorer::compute(
            &[make(700, 80, 35, true), make(550, 90, 25, false)],
            2, 2, true, &pack,
        );
        let dp = DamageProfile::from_rule_pack(&pack);
        let outcome = FightOutcomeEstimator::predict(&our_power, &enemy_power, 80, 4, &dp);

        let opening = OpeningRouteResult {
            route: hexsight_core::OpeningRoute::Mixed,
            confidence: 0.65, reasons: vec![], two_star_count: 2, frontline_quality: 50,
            can_build_combat_item: true, recommended_actions: vec![],
        };
        let scores = vec![LineupScore {
            lineup_id: "x".into(), name: "x".into(), total_score: 80, base_score: 80,
            item_fit_score: 70, champion_hit_score: 60, augment_fit_score: 50,
            trait_fit_score: 60, stage_fit_score: 70, economy_fit_score: 75,
            health_safety_score: 90, playstyle_switch_score: 50,
            rival_penalty: 0, difficulty_penalty: 0,
            reasons: vec![], risks: vec![], requires_augment: false,
        }];
        let econ = EconomyDecisionResult { action: "s".into(), label: "s".into(), target_gold: 30, reasons: vec![] };
        let aug = vec![AugmentScore {
            augment_id: "x".into(), augment_name: "x".into(), total_score: 60,
            combat_power: 50, lineup_coverage: 50, lock_risk: 20,
            playstyle_enable_value: 50, is_recommended: true,
            tags: vec![], reasons: vec![], supported_lineup_ids: vec![],
        }];
        let risk = RiskReport {
            overall: RiskLevel::Low, hp_risk: RiskLevel::Low, rival_risk: RiskLevel::Low,
            item_risk: RiskLevel::Low, completion_risk: RiskLevel::Low,
            economy_risk: RiskLevel::Low, lock_risk: RiskLevel::Low,
            details: vec![], priorities: vec![], should_pivot: false, pivot_reasons: vec![],
        };
        let tm = vec![TransitionMatch {
            lineup_id: "x".into(), lineup_name: "x".into(),
            early_hits: 1, mid_hits: 0, transition_score: 50,
            keep_hero_ids: vec![], transition_traits: vec![],
        }];

        let output = DecisionPlanner::plan(
            &opening, &scores, "混合", &econ, &aug, false,
            &risk, &[], &tm, Some(outcome), 100,
        );
        assert!(output.fight_outcome.is_some());
        let json = serde_json::to_string_pretty(&output).unwrap();
        assert!(json.contains("fightOutcome"));
    }
}
