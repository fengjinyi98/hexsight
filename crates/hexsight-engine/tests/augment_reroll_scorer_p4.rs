use hexsight_core::{LineupProfile, LineupScore, OpeningRouteResult, PlaystyleTag};
use hexsight_engine::augment_economy_planner::{AugmentScore, EconomyDecisionResult};
use hexsight_engine::transition_risk_scorer::{RiskLevel, RiskReport, TransitionMatch};
use hexsight_engine::{
    AugmentDecisionAction, AugmentOptionRanker, AugmentRerollConfigLoader, AugmentRerollContext,
    AugmentRerollDecision, AugmentRerollScorer, AugmentSituationContext, AugmentStage,
    DecisionPlanner, GameDataIndex, LineupAdapter, LineupLoader, LineupProfileBuilder,
    RankedAugmentOption,
};
use std::path::PathBuf;

fn profile(id: &str, recommended: &[&str], replacement: &[&str]) -> LineupProfile {
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
        recommended_hex_ids: recommended.iter().map(|id| id.to_string()).collect(),
        replacement_hex_ids: replacement.iter().map(|id| id.to_string()).collect(),
        early_hero_ids: vec![],
        mid_hero_ids: vec![],
        trait_targets: Default::default(),
        strategy_texts: Default::default(),
        mode_specific: serde_json::Value::Null,
        carry_costs: vec![4],
        category: None,
    }
}

fn option(
    id: &str,
    fallback_score: i32,
    lineup_coverage: i32,
    lock_risk: i32,
) -> RankedAugmentOption {
    RankedAugmentOption {
        augment_id: id.into(),
        augment_name: format!("海克斯{}", id),
        total_score: 30,
        current_value: 30,
        lineup_coverage,
        lock_risk,
        fallback_score,
        tags: vec![],
        supported_lineup_ids: vec![],
        reason: vec![],
    }
}

fn config_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config")
}

#[test]
fn recommends_reroll_when_three_options_are_below_stage_threshold() {
    let options = AugmentOptionRanker::rank_options(
        &[
            (
                "hero_lock".into(),
                "英雄专属".into(),
                "专属英雄强化，绑定特定英雄".into(),
                2,
            ),
            (
                "late_gamble".into(),
                "后期赌博".into(),
                "随机获得高风险后期奖励".into(),
                2,
            ),
            (
                "off_trait".into(),
                "无关纹章".into(),
                "获得一个未成型羁绊纹章".into(),
                2,
            ),
        ],
        &[profile("A", &["ideal"], &[])],
        AugmentStage::First,
        false,
        false,
    );

    let decision = AugmentRerollScorer::decide(&AugmentRerollContext {
        ranked_options: options,
        has_reroll: true,
        stage: AugmentStage::First,
        current_hp: 80,
        current_gold: 20,
        locked_lineup_id: None,
    });

    assert_eq!(decision.action, AugmentDecisionAction::Reroll);
    assert_eq!(decision.recommended_augment_id, None);
    assert!(decision
        .reason
        .iter()
        .any(|line| line.contains("低于最低可接受分")));
}

#[test]
fn takes_fallback_after_reroll_when_all_options_are_still_low() {
    let profiles = vec![
        profile("A", &[], &["flex_item"]),
        profile("B", &[], &["flex_item"]),
        profile("C", &[], &["small_combat"]),
    ];
    let options = AugmentOptionRanker::rank_options(
        &[
            (
                "hero_lock".into(),
                "英雄专属".into(),
                "专属英雄强化，绑定特定英雄".into(),
                2,
            ),
            (
                "flex_item".into(),
                "弹性站位".into(),
                "获得临时调整机会".into(),
                2,
            ),
            (
                "small_combat".into(),
                "小型战力".into(),
                "获得少量攻击力和法术强度".into(),
                2,
            ),
        ],
        &profiles,
        AugmentStage::Second,
        false,
        true,
    );

    let decision = AugmentRerollScorer::decide(&AugmentRerollContext {
        ranked_options: options,
        has_reroll: false,
        stage: AugmentStage::Second,
        current_hp: 80,
        current_gold: 10,
        locked_lineup_id: None,
    });

    assert_eq!(decision.action, AugmentDecisionAction::TakeFallback);
    assert_eq!(decision.recommended_augment_id, Some("flex_item".into()));
    assert!(decision.lock_risk <= 35);
    assert!(decision.reason.iter().any(|line| line.contains("兜底")));
}

#[test]
fn fallback_tiebreak_prefers_higher_lineup_coverage() {
    let decision = AugmentRerollScorer::decide(&AugmentRerollContext {
        ranked_options: vec![
            option("low_coverage", 40, 10, 20),
            option("high_coverage", 40, 80, 20),
        ],
        has_reroll: false,
        stage: AugmentStage::Second,
        current_hp: 80,
        current_gold: 20,
        locked_lineup_id: None,
    });

    assert_eq!(decision.action, AugmentDecisionAction::TakeFallback);
    assert_eq!(
        decision.recommended_augment_id,
        Some("high_coverage".into())
    );
}

#[test]
fn fallback_tiebreak_prefers_lower_lock_risk_after_equal_coverage() {
    let decision = AugmentRerollScorer::decide(&AugmentRerollContext {
        ranked_options: vec![
            option("high_risk", 40, 60, 80),
            option("low_risk", 40, 60, 15),
        ],
        has_reroll: false,
        stage: AugmentStage::Second,
        current_hp: 80,
        current_gold: 20,
        locked_lineup_id: None,
    });

    assert_eq!(decision.action, AugmentDecisionAction::TakeFallback);
    assert_eq!(decision.recommended_augment_id, Some("low_risk".into()));
    assert_eq!(decision.lock_risk, 15);
}

#[test]
fn real_s18_config_loads_p4_thresholds_and_type_weights() {
    let root = config_root().join("rules/S18.1");

    let thresholds =
        AugmentRerollConfigLoader::load_thresholds(&root.join("augment_thresholds.json"))
            .expect("应加载 P4 海克斯阈值配置");
    let weights =
        AugmentRerollConfigLoader::load_type_weights(&root.join("augment_type_weights.json"))
            .expect("应加载 P4 海克斯类型权重配置");

    assert_eq!(thresholds.version, "S18.1");
    assert!(thresholds.first_min_take_score > thresholds.fallback_min_score);
    assert_eq!(weights.version, "S18.1");
    assert!(weights.weights.contains_key("generic_item"));
    assert!(weights.weights.contains_key("hero_commit"));
}

#[test]
fn real_mode17_hex_and_lineup_recommendation_can_take() {
    let root = config_root();
    let index = GameDataIndex::load(&root, "17").expect("应加载 mode17 游戏数据");
    let raw =
        LineupLoader::load_cached_lineups(&root, "17", "S18").expect("应加载 mode17 官方阵容缓存");
    let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
    let profiles = LineupProfileBuilder::build_all(&cards, &index);
    let target_profile = profiles
        .iter()
        .find(|profile| {
            profile
                .recommended_hex_ids
                .iter()
                .any(|id| index.hex(id).is_some())
        })
        .expect("真实阵容应包含可解析的推荐海克斯");
    let recommended_hex_id = target_profile
        .recommended_hex_ids
        .iter()
        .find(|id| index.hex(id).is_some())
        .expect("推荐海克斯应存在于 hex.json");
    let recommended_hex = index.hex(recommended_hex_id).expect("推荐海克斯应可读取");
    let filler_hexes: Vec<_> = index
        .hexes_by_id
        .values()
        .filter(|hex| hex.id != *recommended_hex_id)
        .take(2)
        .collect();
    assert_eq!(filler_hexes.len(), 2, "真实 hex.json 应有足够候选");

    let candidates = vec![
        (
            recommended_hex.id.clone(),
            recommended_hex.name.clone(),
            recommended_hex.desc.clone(),
            recommended_hex.level.parse::<i32>().unwrap_or(2),
        ),
        (
            filler_hexes[0].id.clone(),
            filler_hexes[0].name.clone(),
            "绑定特定英雄的高风险英雄强化".into(),
            filler_hexes[0].level.parse::<i32>().unwrap_or(2),
        ),
        (
            filler_hexes[1].id.clone(),
            filler_hexes[1].name.clone(),
            "随机获得高风险后期奖励".into(),
            filler_hexes[1].level.parse::<i32>().unwrap_or(2),
        ),
    ];
    let context = AugmentSituationContext {
        current_hp: 78,
        current_gold: 20,
        item_gap_level: 0,
        core_hero_count: 1,
        locked_lineup_id: Some(target_profile.lineup_id.clone()),
        existing_effect_tags: vec![],
        required_effect_tags: vec![],
        preferred_damage_profile: None,
    };
    let options = AugmentOptionRanker::rank_options_with_context(
        &candidates,
        &profiles,
        AugmentStage::Second,
        &context,
    );
    let decision = AugmentRerollScorer::decide(&AugmentRerollContext {
        ranked_options: options,
        has_reroll: true,
        stage: AugmentStage::Second,
        current_hp: 78,
        current_gold: 20,
        locked_lineup_id: Some(target_profile.lineup_id.clone()),
    });

    assert_eq!(decision.action, AugmentDecisionAction::Take);
    assert_eq!(
        decision.recommended_augment_id,
        Some(recommended_hex.id.clone())
    );
    assert!(decision
        .ranked_options
        .first()
        .expect("应有排序结果")
        .reason
        .iter()
        .any(|line| line.contains("已锁阵容")));
}

#[test]
fn rule_output_contains_p4_reroll_action() {
    let opening = OpeningRouteResult {
        route: hexsight_core::OpeningRoute::Mixed,
        confidence: 0.65,
        reasons: vec![],
        two_star_count: 2,
        frontline_quality: 50,
        can_build_combat_item: true,
        recommended_actions: vec![],
    };
    let scores = vec![LineupScore {
        lineup_id: "lineup_a".into(),
        name: "阵容A".into(),
        total_score: 70,
        base_score: 80,
        item_fit_score: 50,
        champion_hit_score: 50,
        augment_fit_score: 20,
        trait_fit_score: 50,
        stage_fit_score: 60,
        economy_fit_score: 60,
        health_safety_score: 80,
        playstyle_switch_score: 40,
        rival_penalty: 0,
        difficulty_penalty: 0,
        reasons: vec![],
        risks: vec![],
        requires_augment: false,
    }];
    let economy = EconomyDecisionResult {
        action: "save_interest".into(),
        label: "保利息".into(),
        target_gold: 30,
        reasons: vec![],
    };
    let augment_scores = vec![AugmentScore {
        augment_id: "bad_a".into(),
        augment_name: "低分海克斯".into(),
        total_score: 35,
        combat_power: 30,
        lineup_coverage: 0,
        lock_risk: 80,
        playstyle_enable_value: 10,
        is_recommended: false,
        tags: vec![],
        reasons: vec![],
        supported_lineup_ids: vec![],
    }];
    let risk = RiskReport {
        overall: RiskLevel::Low,
        hp_risk: RiskLevel::Low,
        rival_risk: RiskLevel::Low,
        item_risk: RiskLevel::Low,
        completion_risk: RiskLevel::Low,
        economy_risk: RiskLevel::Low,
        lock_risk: RiskLevel::Low,
        details: vec![],
        priorities: vec![],
        should_pivot: false,
        pivot_reasons: vec![],
    };
    let transitions = vec![TransitionMatch {
        lineup_id: "lineup_a".into(),
        lineup_name: "阵容A".into(),
        early_hits: 1,
        mid_hits: 0,
        transition_score: 50,
        keep_hero_ids: vec![],
        transition_traits: vec![],
    }];
    let reroll_decision = AugmentRerollDecision {
        action: AugmentDecisionAction::Reroll,
        recommended_augment_id: None,
        recommended_augment_name: None,
        fallback_augment_id: Some("fallback_a".into()),
        lock_risk: 80,
        expected_reroll_gain: 20,
        reason: vec!["三个候选都低于最低可接受分，建议刷新".into()],
        ranked_options: vec![RankedAugmentOption {
            augment_id: "bad_a".into(),
            augment_name: "低分海克斯".into(),
            total_score: 35,
            current_value: 30,
            lineup_coverage: 0,
            lock_risk: 80,
            fallback_score: 25,
            tags: vec![],
            supported_lineup_ids: vec![],
            reason: vec![],
        }],
    };

    let output = DecisionPlanner::plan_with_augment_reroll(
        &opening,
        &scores,
        "混合",
        &economy,
        &augment_scores,
        true,
        Some(&reroll_decision),
        &risk,
        &[],
        &transitions,
        None,
        100,
    );

    assert_eq!(output.augment_action.action, "reroll");
    assert!(output.augment_action.recommended.contains("刷新"));
    assert!(!output.augment_action.lock_lineup);
}

#[test]
fn situation_context_prefers_economy_when_gold_is_high() {
    let profiles = vec![profile("A", &[], &[])];
    let context = AugmentSituationContext {
        current_hp: 80,
        current_gold: 70,
        item_gap_level: 0,
        core_hero_count: 0,
        locked_lineup_id: None,
        existing_effect_tags: vec![],
        required_effect_tags: vec![],
        preferred_damage_profile: None,
    };

    let options = AugmentOptionRanker::rank_options_with_context(
        &[
            (
                "item_help".into(),
                "装备补给".into(),
                "获得散件装备和重铸器".into(),
                2,
            ),
            (
                "rich".into(),
                "富有收益".into(),
                "获得金币、利息和后期高费奖励".into(),
                2,
            ),
        ],
        &profiles,
        AugmentStage::First,
        &context,
    );

    assert_eq!(options[0].augment_id, "rich");
    assert!(options[0]
        .reason
        .iter()
        .any(|line| line.contains("经济很好")));
}

#[test]
fn situation_context_prefers_locked_lineup_recommended_augment() {
    let profiles = vec![
        profile("locked", &["trait_a"], &[]),
        profile("flex", &["item_help"], &[]),
    ];
    let context = AugmentSituationContext {
        current_hp: 76,
        current_gold: 20,
        item_gap_level: 0,
        core_hero_count: 2,
        locked_lineup_id: Some("locked".into()),
        existing_effect_tags: vec![],
        required_effect_tags: vec![],
        preferred_damage_profile: None,
    };

    let options = AugmentOptionRanker::rank_options_with_context(
        &[
            (
                "trait_a".into(),
                "核心羁绊".into(),
                "获得一个羁绊纹章并强化当前体系".into(),
                2,
            ),
            (
                "item_help".into(),
                "装备补给".into(),
                "获得散件装备和重铸器".into(),
                2,
            ),
        ],
        &profiles,
        AugmentStage::Second,
        &context,
    );

    assert_eq!(options[0].augment_id, "trait_a");
    assert!(options[0]
        .reason
        .iter()
        .any(|line| line.contains("已锁阵容")));
}

#[test]
fn situation_context_penalizes_duplicate_effect_tags() {
    let profiles = vec![profile("A", &[], &[])];
    let context = AugmentSituationContext {
        current_hp: 80,
        current_gold: 20,
        item_gap_level: 0,
        core_hero_count: 0,
        locked_lineup_id: None,
        existing_effect_tags: vec!["anti_heal".into()],
        required_effect_tags: vec![],
        preferred_damage_profile: None,
    };

    let options = AugmentOptionRanker::rank_options_with_context(
        &[
            (
                "more_wound".into(),
                "重伤扩散".into(),
                "你的队伍获得重伤和减疗效果".into(),
                2,
            ),
            (
                "combat".into(),
                "通用战力".into(),
                "获得攻击力和法术强度".into(),
                2,
            ),
        ],
        &profiles,
        AugmentStage::Second,
        &context,
    );

    let duplicate = options
        .iter()
        .find(|option| option.augment_id == "more_wound")
        .expect("应保留重复效果候选");

    assert_eq!(options[0].augment_id, "combat");
    assert!(duplicate.current_value < options[0].current_value);
    assert!(duplicate
        .reason
        .iter()
        .any(|line| line.contains("重复效果收益下降")));
}

#[test]
fn required_effect_tags_reduce_duplicate_penalty() {
    let profiles = vec![profile("A", &[], &[])];
    let context = AugmentSituationContext {
        current_hp: 80,
        current_gold: 20,
        item_gap_level: 0,
        core_hero_count: 0,
        locked_lineup_id: None,
        existing_effect_tags: vec!["anti_heal".into()],
        required_effect_tags: vec!["anti_heal".into()],
        preferred_damage_profile: None,
    };

    let options = AugmentOptionRanker::rank_options_with_context(
        &[(
            "more_wound".into(),
            "重伤扩散".into(),
            "你的队伍获得重伤和减疗效果".into(),
            2,
        )],
        &profiles,
        AugmentStage::Second,
        &context,
    );

    assert!(options[0]
        .reason
        .iter()
        .any(|line| line.contains("当前局势需要")));
    assert!(!options[0]
        .reason
        .iter()
        .any(|line| line.contains("重复效果收益下降")));
}

#[test]
fn situation_context_penalizes_damage_profile_mismatch() {
    let profiles = vec![profile("ap_lineup", &["ap_boost"], &[])];
    let context = AugmentSituationContext {
        current_hp: 78,
        current_gold: 20,
        item_gap_level: 0,
        core_hero_count: 1,
        locked_lineup_id: Some("ap_lineup".into()),
        existing_effect_tags: vec![],
        required_effect_tags: vec![],
        preferred_damage_profile: Some("ap".into()),
    };

    let options = AugmentOptionRanker::rank_options_with_context(
        &[
            (
                "ad_boost".into(),
                "物理火力".into(),
                "获得攻击力、攻速和物理伤害".into(),
                2,
            ),
            (
                "ap_boost".into(),
                "法术涌动".into(),
                "获得法术强度和魔法伤害".into(),
                2,
            ),
        ],
        &profiles,
        AugmentStage::Second,
        &context,
    );

    let ad_option = options
        .iter()
        .find(|option| option.augment_id == "ad_boost")
        .expect("应保留 AD 候选");

    assert_eq!(options[0].augment_id, "ap_boost");
    assert!(ad_option
        .reason
        .iter()
        .any(|line| line.contains("当前局势不匹配")));
}
