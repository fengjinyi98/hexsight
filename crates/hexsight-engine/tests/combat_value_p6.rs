use hexsight_core::{
    ChampionCapability, ChampionRole, ConflictEnvironment, ItemPreferences, ItemStat,
    ItemValueProfile,
};
use hexsight_engine::{
    CombatValueEstimator, CombatValueWeights, EnvironmentModifierScorer, EnvironmentWeights,
};
use std::path::PathBuf;

fn config_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config")
}

fn champion() -> ChampionCapability {
    ChampionCapability {
        hero_id: "carry_ap".into(),
        name: "法系主C".into(),
        cost: 4,
        traits: vec!["mage".into()],
        role: ChampionRole::PrimaryCarry,
        damage_profile: "ap".into(),
        damage_pattern: vec!["sustained".into(), "aoe".into()],
        cast_pattern: "mana_cast".into(),
        scaling_stats: vec!["ability_power".into(), "mana".into()],
        position_role: "backline".into(),
        item_strictness: "medium".into(),
        power_spikes: vec![],
        risk_profile: vec!["burst_damage".into()],
        item_preferences: ItemPreferences::default(),
        confidence: 1.0,
        needs_override: false,
    }
}

fn item(
    id: &str,
    name: &str,
    stats: Vec<(&str, f64)>,
    tags: Vec<&str>,
    fit: Vec<&str>,
) -> ItemValueProfile {
    ItemValueProfile {
        item_id: id.into(),
        name: name.into(),
        item_type: "成型装备".into(),
        stats: stats
            .into_iter()
            .map(|(name, value)| ItemStat {
                name: name.into(),
                value,
            })
            .collect(),
        effect_tags: tags.into_iter().map(String::from).collect(),
        best_for_profiles: vec![],
        bad_for_profiles: vec![],
        replacement_group: "".into(),
        conflict_group: "".into(),
        stage_bias: "mid".into(),
        tier: None,
        damage_type_fit: fit.into_iter().map(String::from).collect(),
    }
}

#[test]
fn compares_same_carry_item_sets_by_simplified_dps_and_ehp() {
    let carry = champion();
    let output_set = vec![
        item("ap", "大帽", vec![("ap", 45.0)], vec![], vec!["ap"]),
        item(
            "mana",
            "青龙刀",
            vec![("mana", 15.0)],
            vec!["mana_engine"],
            vec!["ap"],
        ),
    ];
    let defensive_set = vec![item(
        "guard",
        "防御装",
        vec![("hp", 300.0), ("armor", 30.0)],
        vec!["shield_survival"],
        vec!["tank"],
    )];

    let estimator = CombatValueEstimator::new(CombatValueWeights::default());
    let output_result = estimator.estimate(&carry, &output_set, &ConflictEnvironment::default());
    let defensive_result =
        estimator.estimate(&carry, &defensive_set, &ConflictEnvironment::default());

    assert!(output_result.simplified_dps > defensive_result.simplified_dps);
    assert!(defensive_result.simplified_ehp > output_result.simplified_ehp);
    assert!(output_result
        .reasons
        .iter()
        .any(|reason| reason.contains("法强") || reason.contains("启动")));
}

#[test]
fn compare_item_sets_reports_winner_and_value_diff() {
    let carry = champion();
    let output_set = vec![
        item("ap", "大帽", vec![("ap", 45.0)], vec![], vec!["ap"]),
        item(
            "mana",
            "青龙刀",
            vec![("mana", 15.0)],
            vec!["mana_engine"],
            vec!["ap"],
        ),
    ];
    let defensive_set = vec![item(
        "guard",
        "防御装",
        vec![("hp", 300.0), ("armor", 30.0)],
        vec!["shield_survival"],
        vec!["tank"],
    )];

    let estimator = CombatValueEstimator::new(CombatValueWeights::default());
    let diff = estimator.compare_item_sets(
        &carry,
        &output_set,
        &defensive_set,
        &ConflictEnvironment::default(),
    );

    assert_eq!(diff.winner, "left");
    assert!(diff.score_diff > 0);
    assert!(diff.dps_diff > 0.0);
}

#[test]
fn real_s18_config_loads_combat_and_environment_weights() {
    let root = config_root();

    let combat_weights =
        CombatValueWeights::load(&root, "S18.1").expect("应加载 S18.1 简化收益权重");
    let environment_weights =
        EnvironmentWeights::load(&root, "S18.1").expect("应加载 S18.1 环境修正权重");

    assert!(combat_weights.base_dps > 0.0);
    assert!(combat_weights.mana_per_point > 0.0);
    assert!(environment_weights.heal_heavy_anti_heal > 0);
    assert!(environment_weights.own_ap_mr_shred > 0);
}

#[test]
fn heal_heavy_environment_increases_anti_heal_value() {
    let red_buff = item(
        "anti_heal",
        "红霸符",
        vec![("attack_speed", 20.0)],
        vec!["anti_heal", "burn"],
        vec!["ad", "ap"],
    );
    let scorer = EnvironmentModifierScorer::new(EnvironmentWeights::default());

    let normal = scorer.score_item(&red_buff, &ConflictEnvironment::default());
    let heal_heavy = scorer.score_item(
        &red_buff,
        &ConflictEnvironment {
            opponent_heal_heavy: true,
            ..ConflictEnvironment::default()
        },
    );

    assert!(heal_heavy.score > normal.score);
    assert!(heal_heavy
        .reasons
        .iter()
        .any(|reason| reason.contains("回复") && reason.contains("重伤")));
}

#[test]
fn thick_frontline_environment_increases_shred_value_for_matching_damage_type() {
    let last_whisper = item(
        "armor_shred",
        "轻语",
        vec![("crit", 20.0)],
        vec!["armor_shred"],
        vec!["ad"],
    );
    let scorer = EnvironmentModifierScorer::new(EnvironmentWeights::default());

    let normal = scorer.score_item(&last_whisper, &ConflictEnvironment::default());
    let thick_frontline = scorer.score_item(
        &last_whisper,
        &ConflictEnvironment {
            opponent_frontline_thick: true,
            own_ad_heavy: true,
            ..ConflictEnvironment::default()
        },
    );

    assert!(thick_frontline.score > normal.score);
    assert!(thick_frontline
        .reasons
        .iter()
        .any(|reason| reason.contains("前排") && reason.contains("破甲")));
}

#[test]
fn burst_heavy_environment_increases_survival_item_value() {
    let edge_of_night = item(
        "survival",
        "夜刃",
        vec![("ad", 10.0)],
        vec!["shield_survival"],
        vec!["ad"],
    );
    let scorer = EnvironmentModifierScorer::new(EnvironmentWeights::default());

    let normal = scorer.score_item(&edge_of_night, &ConflictEnvironment::default());
    let burst_heavy = scorer.score_item(
        &edge_of_night,
        &ConflictEnvironment {
            opponent_burst_heavy: true,
            ..ConflictEnvironment::default()
        },
    );

    assert!(burst_heavy.score > normal.score);
    assert!(burst_heavy
        .reasons
        .iter()
        .any(|reason| reason.contains("爆发") && reason.contains("生存")));
}

#[test]
fn ap_heavy_environment_increases_mr_shred_value() {
    let spark = item(
        "mr_shred",
        "离子火花",
        vec![("ap", 10.0)],
        vec!["mr_shred"],
        vec!["ap"],
    );
    let scorer = EnvironmentModifierScorer::new(EnvironmentWeights::default());

    let normal = scorer.score_item(&spark, &ConflictEnvironment::default());
    let ap_heavy = scorer.score_item(
        &spark,
        &ConflictEnvironment {
            own_ap_heavy: true,
            opponent_frontline_thick: true,
            ..ConflictEnvironment::default()
        },
    );

    assert!(ap_heavy.score > normal.score);
    assert!(ap_heavy
        .reasons
        .iter()
        .any(|reason| reason.contains("魔抗击碎")));
}
