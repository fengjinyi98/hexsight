// P7 知识决策规划器
// 核心职责：
// - 汇总 P0-P6 的知识解析、版本修正、海克斯刷新、承载者和收益估算结果
// - 生成最终 RuleOutput 的即时动作、转向条件和短解释
// - 保持 Swift 展示层只消费结构化结论

use hexsight_core::{
    AugmentDecision, CombatActionSummary, EconomyDecision, FightOutcome, HolderActionSummary,
    ItemDecision, KnowledgeActions, LineupProfile, LineupRecommendation, LineupScore, OpeningRoute,
    OpeningRouteResult, RuleOutput, TransitionDecision,
};

use crate::augment_economy_planner::EconomyDecisionResult;
use crate::augment_reroll_scorer::{AugmentDecisionAction, AugmentRerollDecision};
use crate::combat_value_estimator::CombatValueDiff;
use crate::holder_scorer::{HolderAction, HolderPlan};
use crate::patch_knowledge::PatchKnowledge;
use crate::patch_modifier::PatchModifier;

/// P7 知识决策规划器
pub struct KnowledgeDecisionPlanner;

impl KnowledgeDecisionPlanner {
    #[allow(clippy::too_many_arguments)]
    pub fn plan(
        opening: &OpeningRouteResult,
        lineup_scores: &[LineupScore],
        profiles: &[LineupProfile],
        patch_knowledge: Option<&PatchKnowledge>,
        item_fit_direction: &str,
        economy_result: &EconomyDecisionResult,
        augment_reroll: Option<&AugmentRerollDecision>,
        holder_plan: Option<&HolderPlan>,
        combat_value_diff: Option<&CombatValueDiff>,
        current_hp: i32,
    ) -> RuleOutput {
        let adjusted_scores = patch_knowledge
            .map(|knowledge| PatchModifier::apply_to_scores(lineup_scores, profiles, knowledge))
            .unwrap_or_else(|| lineup_scores.to_vec());

        let mut sorted = adjusted_scores.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.total_score.cmp(&a.total_score));

        let lineup_recommendations = sorted
            .into_iter()
            .take(3)
            .map(|score| LineupRecommendation {
                lineup_id: score.lineup_id.clone(),
                name: score.name.clone(),
                score: score.total_score,
                reason: score.reasons.clone(),
                risk: score.risks.clone(),
            })
            .collect::<Vec<_>>();

        let strategy = match opening.route {
            OpeningRoute::WinStreak => "连胜".to_string(),
            OpeningRoute::LossStreak => "精致连败".to_string(),
            OpeningRoute::Mixed => "混合过渡".to_string(),
        };

        let economy_action = EconomyDecision {
            action: economy_result.action.clone(),
            reason: economy_result.reasons.join("; "),
        };

        let item_action = Self::item_action(item_fit_direction, holder_plan, combat_value_diff);
        let augment_action = augment_reroll
            .map(Self::augment_decision)
            .unwrap_or_else(|| AugmentDecision {
                action: "take".into(),
                recommended: "选择当前最高分海克斯".into(),
                lock_lineup: false,
                follow_up: "保留阵容弹性".into(),
            });

        let transition_action = TransitionDecision {
            route: holder_plan
                .and_then(|plan| plan.best.as_ref())
                .map(|best| format!("{} 承载过渡", best.temporary_holder_name))
                .unwrap_or_else(|| "按当前最高分阵容过渡".into()),
            target_damage_per_round: if current_hp < 45 {
                "立即止血".into()
            } else {
                "2-6".into()
            },
            stop_loss_stage: if current_hp < 45 {
                "当前回合"
            } else {
                "3-2"
            }
            .into(),
        };

        let knowledge_actions = Self::knowledge_actions(
            patch_knowledge,
            augment_reroll,
            holder_plan,
            combat_value_diff,
            &lineup_recommendations,
        );
        let pivot_conditions =
            Self::pivot_conditions(current_hp, &adjusted_scores, holder_plan, combat_value_diff);

        RuleOutput {
            strategy,
            lineup_recommendations,
            economy_action,
            item_action,
            augment_action,
            transition_action,
            pivot_conditions,
            fight_outcome: None::<FightOutcome>,
            knowledge_actions: Some(knowledge_actions),
            generated_at: chrono_now(),
            engine_version: "0.7.0".into(),
        }
    }

    fn item_action(
        item_fit_direction: &str,
        holder_plan: Option<&HolderPlan>,
        combat_value_diff: Option<&CombatValueDiff>,
    ) -> ItemDecision {
        if let Some(best) = holder_plan.and_then(|plan| plan.best.as_ref()) {
            let action = match best.action {
                HolderAction::EquipNow => "equip_now",
                HolderAction::WaitForTarget => "wait_for_target",
            };
            let final_holder = best
                .final_holder_name
                .as_ref()
                .map(|name| format!("，后续转给{}", name))
                .unwrap_or_default();
            let mut reason = format!(
                "{}给{}{}；{}",
                best.item_name,
                best.temporary_holder_name,
                final_holder,
                best.reasons.join("; ")
            );
            if let Some(diff) = combat_value_diff {
                reason.push_str(&format!("；收益比较 {} 分", diff.score_diff));
            }
            return ItemDecision {
                action: action.into(),
                reason,
            };
        }

        ItemDecision {
            action: match item_fit_direction {
                "AD" => "build_ad",
                "AP" => "build_ap",
                "坦" => "build_tank",
                _ => "hold_flexible",
            }
            .into(),
            reason: format!("当前装备方向: {}", item_fit_direction),
        }
    }

    fn augment_decision(decision: &AugmentRerollDecision) -> AugmentDecision {
        let action = match decision.action {
            AugmentDecisionAction::Take => "take",
            AugmentDecisionAction::Reroll => "reroll",
            AugmentDecisionAction::TakeFallback => "take_fallback",
        }
        .to_string();

        let recommended = match decision.action {
            AugmentDecisionAction::Take => decision
                .recommended_augment_name
                .as_ref()
                .map(|name| format!("拿 {}", name))
                .unwrap_or_else(|| "拿当前最高分海克斯".into()),
            AugmentDecisionAction::Reroll => "刷新当前三个海克斯".into(),
            AugmentDecisionAction::TakeFallback => decision
                .recommended_augment_name
                .as_ref()
                .map(|name| format!("兜底拿 {}", name))
                .unwrap_or_else(|| "兜底拿锁方向风险最低的海克斯".into()),
        };

        AugmentDecision {
            action,
            recommended,
            lock_lineup: decision.lock_risk <= 35,
            follow_up: decision.reason.join("; "),
        }
    }

    fn knowledge_actions(
        patch_knowledge: Option<&PatchKnowledge>,
        augment_reroll: Option<&AugmentRerollDecision>,
        holder_plan: Option<&HolderPlan>,
        combat_value_diff: Option<&CombatValueDiff>,
        lineups: &[LineupRecommendation],
    ) -> KnowledgeActions {
        let holder =
            holder_plan
                .and_then(|plan| plan.best.as_ref())
                .map(|best| HolderActionSummary {
                    item: best.item_name.clone(),
                    temporary_holder: best.temporary_holder_name.clone(),
                    final_holder: best.final_holder_name.clone(),
                    action: match best.action {
                        HolderAction::EquipNow => "equip_now",
                        HolderAction::WaitForTarget => "wait_for_target",
                    }
                    .into(),
                    reason: best.reasons.join("; "),
                });

        let combat = combat_value_diff.map(|diff| CombatActionSummary {
            winner: diff.winner.clone(),
            score_diff: diff.score_diff,
            reason: diff.reasons.join("; "),
        });

        let patch = patch_knowledge
            .map(|knowledge| {
                knowledge
                    .entries
                    .iter()
                    .map(|entry| {
                        format!(
                            "{} {} {}：{}",
                            entry.target_name, entry.change_type, entry.impact_score, entry.reason
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut short_explanations = Vec::new();
        if let Some(decision) = augment_reroll {
            short_explanations.push(match decision.action {
                AugmentDecisionAction::Take => "当前海克斯达到阈值，直接拿".into(),
                AugmentDecisionAction::Reroll => "当前三个海克斯低于阈值，建议刷新".into(),
                AugmentDecisionAction::TakeFallback => "刷新价值不足，兜底拿覆盖最高项".into(),
            });
        }
        if let Some(holder) = &holder {
            short_explanations.push(format!("{} 先给 {}", holder.item, holder.temporary_holder));
        }
        if let Some(best) = lineups.first() {
            short_explanations.push(format!("当前优先阵容 {}，评分 {}", best.name, best.score));
        }

        KnowledgeActions {
            holder,
            combat,
            patch,
            short_explanations,
        }
    }

    fn pivot_conditions(
        current_hp: i32,
        scores: &[LineupScore],
        holder_plan: Option<&HolderPlan>,
        combat_value_diff: Option<&CombatValueDiff>,
    ) -> Vec<String> {
        let mut conditions = Vec::new();
        if current_hp < 45 {
            conditions.push("血量低于 45，优先止血".into());
        }
        if scores.iter().any(|score| score.item_fit_score < 45) {
            conditions.push("装备适配偏低，考虑转向更吃当前装备的阵容".into());
        }
        if holder_plan
            .and_then(|plan| plan.best.as_ref())
            .map(|best| best.action == HolderAction::WaitForTarget)
            .unwrap_or(false)
        {
            conditions.push("当前承载者收益不足，等待目标主C或主坦".into());
        }
        if combat_value_diff
            .map(|diff| diff.score_diff < -10)
            .unwrap_or(false)
        {
            conditions.push("收益估算显示当前装备组合落后，优先改合替代装".into());
        }
        conditions
    }
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".into())
}
