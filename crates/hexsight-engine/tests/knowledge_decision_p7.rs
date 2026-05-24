use std::collections::HashMap;

use hexsight_core::{LineupProfile, LineupScore, OpeningRoute, OpeningRouteResult, PlaystyleTag};
use hexsight_engine::{
    AugmentDecisionAction, AugmentRerollDecision, CombatValueDiff, EconomyDecisionResult,
    HolderAction, HolderPlan, HolderRecommendation, HolderRole, KnowledgeDecisionPlanner,
    KnowledgeRegression, PatchKnowledge,
};

fn score(id: &str, total: i32, item_fit: i32) -> LineupScore {
    LineupScore {
        lineup_id: id.into(),
        name: format!("阵容{}", id),
        total_score: total,
        base_score: total,
        item_fit_score: item_fit,
        champion_hit_score: 60,
        augment_fit_score: 50,
        trait_fit_score: 60,
        stage_fit_score: 70,
        economy_fit_score: 75,
        health_safety_score: 90,
        playstyle_switch_score: 50,
        rival_penalty: 0,
        difficulty_penalty: 0,
        reasons: vec!["初始评分".into()],
        risks: vec![],
        requires_augment: false,
    }
}

fn profile(id: &str) -> LineupProfile {
    LineupProfile {
        lineup_id: id.into(),
        name: format!("阵容{}", id),
        base_tier: 80,
        playstyle_tags: vec![PlaystyleTag::Standard],
        final_hero_ids: vec!["hero_carry".into()],
        carry_hero_ids: vec!["hero_carry".into()],
        tank_hero_ids: vec!["hero_tank".into()],
        core_equipment_ids: vec!["item_core".into()],
        tank_equipment_ids: vec![],
        equipment_order_ids: vec!["item_core".into()],
        recommended_hex_ids: vec!["hex_good".into()],
        replacement_hex_ids: vec![],
        early_hero_ids: vec![],
        mid_hero_ids: vec![],
        trait_targets: HashMap::new(),
        strategy_texts: HashMap::new(),
        mode_specific: serde_json::Value::Null,
        carry_costs: vec![4],
        category: None,
    }
}

#[test]
fn p7_integrates_knowledge_outputs_into_rule_output() {
    let opening = OpeningRouteResult {
        route: OpeningRoute::Mixed,
        confidence: 0.7,
        reasons: vec!["局势中等".into()],
        two_star_count: 2,
        frontline_quality: 55,
        can_build_combat_item: true,
        recommended_actions: vec!["稳血".into()],
    };

    let economy = EconomyDecisionResult {
        action: "save_interest".into(),
        label: "保利息".into(),
        target_gold: 30,
        reasons: vec!["经济线健康".into()],
    };

    let holder = HolderPlan {
        best: Some(HolderRecommendation {
            item_id: "item_core".into(),
            item_name: "核心输出装".into(),
            temporary_holder_id: "workhorse".into(),
            temporary_holder_name: "二星打工C".into(),
            holder_role: HolderRole::Carry,
            final_holder_id: Some("hero_carry".into()),
            final_holder_name: Some("主C".into()),
            score: 88,
            sell_cost: 2,
            action: HolderAction::EquipNow,
            reasons: vec!["二星质量高".into(), "装备机制匹配".into()],
        }),
        alternatives: vec![],
        summary: "当前装给二星打工C，后续转给主C".into(),
    };

    let augment_reroll = AugmentRerollDecision {
        action: AugmentDecisionAction::Reroll,
        recommended_augment_id: None,
        recommended_augment_name: None,
        fallback_augment_id: Some("fallback_hex".into()),
        lock_risk: 60,
        expected_reroll_gain: 18,
        reason: vec!["三个候选都低于当前阶段阈值".into()],
        ranked_options: vec![],
    };

    let patch = PatchKnowledge {
        version: "S18.1".into(),
        entries: vec![hexsight_core::PatchEntry {
            patch_version: "S18.1".into(),
            published_at: "2026-05-25".into(),
            target_type: "champion".into(),
            target_id: "hero_carry".into(),
            target_name: "主C".into(),
            change_type: "nerf".into(),
            changed_stats: HashMap::new(),
            impact_score: -12,
            affected_lineups: vec!["line_a".into()],
            reason: "核心输出削弱".into(),
        }],
    };

    let combat = CombatValueDiff {
        dps_diff: -18.5,
        ehp_diff: 6.0,
        score_diff: -16,
        winner: "right".into(),
        reasons: vec!["替代装备收益更高".into()],
    };

    let output = KnowledgeDecisionPlanner::plan(
        &opening,
        &[score("line_a", 72, 38), score("line_b", 65, 70)],
        &[profile("line_a"), profile("line_b")],
        Some(&patch),
        "AP",
        &economy,
        Some(&augment_reroll),
        Some(&holder),
        Some(&combat),
        42,
    );

    assert_eq!(output.engine_version, "0.7.0");
    assert_eq!(output.augment_action.action, "reroll");
    assert!(output.item_action.reason.contains("二星打工C"));
    assert!(output.item_action.reason.contains("后续转给主C"));
    assert!(output.item_action.reason.contains("-16"));
    assert!(output
        .pivot_conditions
        .iter()
        .any(|condition| condition.contains("装备适配偏低")));
    assert!(output
        .pivot_conditions
        .iter()
        .any(|condition| condition.contains("收益估算显示")));
    assert!(output
        .lineup_recommendations
        .iter()
        .any(|lineup| lineup.risk.iter().any(|risk| risk.contains("核心输出削弱"))));

    let json = serde_json::to_value(&output).expect("应能序列化 P7 RuleOutput");
    assert_eq!(
        json["knowledgeActions"]["holder"]["temporaryHolder"],
        "二星打工C"
    );
    assert_eq!(json["knowledgeActions"]["combat"]["scoreDiff"], -16);
    assert_eq!(json["knowledgeActions"]["combat"]["winner"], "right");
    assert!(json["knowledgeActions"]["shortExplanations"]
        .as_array()
        .expect("短解释应为数组")
        .iter()
        .any(|value| value.as_str().unwrap_or("").contains("刷新")));
}

#[test]
fn p7_regression_cases_are_loadable() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config");

    let cases = KnowledgeRegression::load_cases(&root, "S18.1").expect("应能加载 P7 回归样例");

    assert!(cases
        .iter()
        .any(|case| case.id == "p7_knowledge_decision_smoke"));
    assert!(cases.iter().any(|case| case
        .expected_actions
        .iter()
        .any(|action| action == "reroll")));
}

#[test]
fn p7_regression_cases_replay_planner_outputs() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config");

    let case = KnowledgeRegression::load_cases(&root, "S18.1")
        .expect("应能加载 P7 回归样例")
        .into_iter()
        .find(|case| case.id == "p7_knowledge_decision_smoke")
        .expect("应存在 P7 smoke 样例");
    assert_eq!(case.fixture.as_deref(), Some("p7_smoke"));

    let output = p7_smoke_output();
    let json = serde_json::to_value(&output).expect("RuleOutput 应能序列化");

    for expected in &case.expected_actions {
        assert!(
            action_present(&json, expected),
            "回归样例动作未命中: {expected}"
        );
    }

    let text = serde_json::to_string(&json).expect("RuleOutput JSON 应能转字符串");
    for expected in &case.expected_explanations {
        assert!(text.contains(expected), "回归样例解释未命中: {expected}");
    }
}

fn p7_smoke_output() -> hexsight_core::RuleOutput {
    let opening = OpeningRouteResult {
        route: OpeningRoute::Mixed,
        confidence: 0.7,
        reasons: vec!["局势中等".into()],
        two_star_count: 2,
        frontline_quality: 55,
        can_build_combat_item: true,
        recommended_actions: vec!["稳血".into()],
    };
    let economy = EconomyDecisionResult {
        action: "save_interest".into(),
        label: "保利息".into(),
        target_gold: 30,
        reasons: vec!["经济线健康".into()],
    };
    let holder = HolderPlan {
        best: Some(HolderRecommendation {
            item_id: "item_core".into(),
            item_name: "核心输出装".into(),
            temporary_holder_id: "workhorse".into(),
            temporary_holder_name: "二星打工C".into(),
            holder_role: HolderRole::Carry,
            final_holder_id: Some("hero_carry".into()),
            final_holder_name: Some("主C".into()),
            score: 88,
            sell_cost: 2,
            action: HolderAction::EquipNow,
            reasons: vec!["二星质量高".into(), "装备机制匹配".into()],
        }),
        alternatives: vec![],
        summary: "当前装给二星打工C，后续转给主C".into(),
    };
    let augment_reroll = AugmentRerollDecision {
        action: AugmentDecisionAction::Reroll,
        recommended_augment_id: None,
        recommended_augment_name: None,
        fallback_augment_id: Some("fallback_hex".into()),
        lock_risk: 60,
        expected_reroll_gain: 18,
        reason: vec!["三个候选都低于当前阶段阈值".into()],
        ranked_options: vec![],
    };
    let patch = PatchKnowledge {
        version: "S18.1".into(),
        entries: vec![hexsight_core::PatchEntry {
            patch_version: "S18.1".into(),
            published_at: "2026-05-25".into(),
            target_type: "champion".into(),
            target_id: "hero_carry".into(),
            target_name: "主C".into(),
            change_type: "nerf".into(),
            changed_stats: HashMap::new(),
            impact_score: -12,
            affected_lineups: vec!["line_a".into()],
            reason: "核心输出削弱".into(),
        }],
    };
    let combat = CombatValueDiff {
        dps_diff: -18.5,
        ehp_diff: 6.0,
        score_diff: -16,
        winner: "right".into(),
        reasons: vec!["替代装备收益更高".into()],
    };

    KnowledgeDecisionPlanner::plan(
        &opening,
        &[score("line_a", 72, 38), score("line_b", 65, 70)],
        &[profile("line_a"), profile("line_b")],
        Some(&patch),
        "AP",
        &economy,
        Some(&augment_reroll),
        Some(&holder),
        Some(&combat),
        42,
    )
}

fn action_present(json: &serde_json::Value, expected: &str) -> bool {
    match expected {
        "reroll" => json["augmentAction"]["action"] == "reroll",
        "equip_now" => {
            json["itemAction"]["action"] == "equip_now"
                || json["knowledgeActions"]["holder"]["action"] == "equip_now"
        }
        "pivot_condition" => json["pivotConditions"]
            .as_array()
            .map(|conditions| !conditions.is_empty())
            .unwrap_or(false),
        "combat_diff" => json["knowledgeActions"]["combat"]["scoreDiff"].is_number(),
        _ => false,
    }
}
