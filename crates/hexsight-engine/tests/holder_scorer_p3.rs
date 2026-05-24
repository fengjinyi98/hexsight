use hexsight_core::{
    ChampionCapability, ChampionRole, EquipmentData, ItemStat, ItemValueProfile, LineupProfile,
    PlaystyleTag, PowerSpike,
};
use hexsight_engine::{
    BenchTransitionScorer, HolderAction, HolderRole, HolderScorer, HolderScoringContext,
    HolderUnit, HolderUnitSource, ItemSynthesisIndex,
};

fn champion(
    hero_id: &str,
    name: &str,
    cost: i32,
    role: ChampionRole,
    damage_profile: &str,
    position_role: &str,
    scaling_stats: Vec<&str>,
) -> ChampionCapability {
    ChampionCapability {
        hero_id: hero_id.into(),
        name: name.into(),
        cost,
        traits: vec![],
        role,
        damage_profile: damage_profile.into(),
        damage_pattern: vec!["sustained".into()],
        cast_pattern: "attack_based".into(),
        scaling_stats: scaling_stats.into_iter().map(String::from).collect(),
        position_role: position_role.into(),
        item_strictness: "medium".into(),
        power_spikes: vec![PowerSpike {
            spike_type: "star".into(),
            value: 2,
            impact: 20,
        }],
        risk_profile: vec![],
        item_preferences: Default::default(),
        confidence: 0.9,
        needs_override: false,
    }
}

fn item(
    item_id: &str,
    name: &str,
    damage_fit: Vec<&str>,
    stats: Vec<&str>,
    tags: Vec<&str>,
) -> ItemValueProfile {
    ItemValueProfile {
        item_id: item_id.into(),
        name: name.into(),
        item_type: "成型装备".into(),
        stats: stats
            .into_iter()
            .map(|name| ItemStat {
                name: name.into(),
                value: 30.0,
            })
            .collect(),
        effect_tags: tags.into_iter().map(String::from).collect(),
        best_for_profiles: damage_fit.iter().map(|v| v.to_string()).collect(),
        bad_for_profiles: vec![],
        replacement_group: String::new(),
        conflict_group: String::new(),
        stage_bias: "early_mid".into(),
        tier: None,
        damage_type_fit: damage_fit.into_iter().map(String::from).collect(),
    }
}

fn profile(
    carry_ids: Vec<&str>,
    tank_ids: Vec<&str>,
    core_items: Vec<&str>,
    tank_items: Vec<&str>,
) -> LineupProfile {
    LineupProfile {
        lineup_id: "lineup_1".into(),
        name: "测试阵容".into(),
        base_tier: 90,
        playstyle_tags: vec![PlaystyleTag::Tempo],
        final_hero_ids: carry_ids
            .iter()
            .chain(tank_ids.iter())
            .map(|id| id.to_string())
            .collect(),
        carry_hero_ids: carry_ids.into_iter().map(String::from).collect(),
        tank_hero_ids: tank_ids.into_iter().map(String::from).collect(),
        core_equipment_ids: core_items.into_iter().map(String::from).collect(),
        tank_equipment_ids: tank_items.into_iter().map(String::from).collect(),
        equipment_order_ids: vec![],
        recommended_hex_ids: vec![],
        replacement_hex_ids: vec![],
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
fn recommends_two_star_ad_workhorse_as_temporary_carry() {
    let champions = vec![
        champion(
            "early_ad",
            "二星打工射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
        champion(
            "weak_utility",
            "一星挂件",
            1,
            ChampionRole::Utility,
            "utility",
            "backline",
            vec![],
        ),
        champion(
            "late_carry",
            "最终主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
    ];
    let items = vec![item(
        "ad_item",
        "物理成装",
        vec!["ad"],
        vec!["ad", "attack_speed"],
        vec![],
    )];
    let context = HolderScoringContext {
        board_units: vec![
            HolderUnit::new("early_ad", 2, HolderUnitSource::Board),
            HolderUnit::new("weak_utility", 1, HolderUnitSource::Board),
        ],
        bench_units: vec![],
        completed_item_ids: vec!["ad_item".into()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
    };

    let plan = HolderScorer::score(&context, &champions, &items);

    let best = plan.best.expect("应给出最佳临时承载者");
    assert_eq!(best.temporary_holder_id, "early_ad");
    assert_eq!(best.final_holder_id, Some("late_carry".into()));
    assert_eq!(best.holder_role, HolderRole::Carry);
    assert_eq!(best.action, HolderAction::EquipNow);
    assert!(best.sell_cost > 0);
    assert!(plan.summary.contains("给二星打工射手"));
}

#[test]
fn recommends_frontline_tank_as_temporary_tank_holder() {
    let champions = vec![
        champion(
            "front_tank",
            "二星前排",
            2,
            ChampionRole::OffTank,
            "tank",
            "frontline",
            vec!["health"],
        ),
        champion(
            "back_carry",
            "后排输出",
            2,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage"],
        ),
        champion(
            "late_tank",
            "最终主坦",
            4,
            ChampionRole::MainTank,
            "tank",
            "frontline",
            vec!["health"],
        ),
    ];
    let items = vec![item(
        "tank_item",
        "肉装",
        vec!["tank"],
        vec!["hp", "armor"],
        vec!["shield_survival"],
    )];
    let context = HolderScoringContext {
        board_units: vec![
            HolderUnit::new("front_tank", 2, HolderUnitSource::Board),
            HolderUnit::new("back_carry", 2, HolderUnitSource::Board),
        ],
        bench_units: vec![],
        completed_item_ids: vec!["tank_item".into()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile(
            vec![],
            vec!["late_tank"],
            vec![],
            vec!["tank_item"],
        )],
    };

    let plan = HolderScorer::score(&context, &champions, &items);

    let best = plan.best.expect("应给出最佳临时承载者");
    assert_eq!(best.temporary_holder_id, "front_tank");
    assert_eq!(best.final_holder_id, Some("late_tank".into()));
    assert_eq!(best.holder_role, HolderRole::Tank);
    assert_eq!(best.action, HolderAction::EquipNow);
}

#[test]
fn waits_for_target_carry_when_target_is_available_and_current_holder_is_one_star() {
    let champions = vec![
        champion(
            "early_ad",
            "一星临时输出",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage"],
        ),
        champion(
            "late_carry",
            "目标主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
    ];
    let items = vec![item(
        "ad_item",
        "物理成装",
        vec!["ad"],
        vec!["ad", "attack_speed"],
        vec![],
    )];
    let context = HolderScoringContext {
        board_units: vec![HolderUnit::new("early_ad", 1, HolderUnitSource::Board)],
        bench_units: vec![HolderUnit::new("late_carry", 1, HolderUnitSource::Bench)],
        completed_item_ids: vec!["ad_item".into()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
    };

    let plan = HolderScorer::score(&context, &champions, &items);

    let best = plan.best.expect("应给出目标主C等待建议");
    assert_eq!(best.final_holder_id, Some("late_carry".into()));
    assert_eq!(best.action, HolderAction::WaitForTarget);
    assert!(best
        .reasons
        .iter()
        .any(|reason| reason.contains("目标主C已在备战席")));
}

#[test]
fn bench_transition_scorer_outputs_final_holder_and_sell_cost() {
    let current = HolderUnit::new("early_ad", 2, HolderUnitSource::Board);
    let target = HolderUnit::new("late_carry", 1, HolderUnitSource::Bench);
    let champions = vec![
        champion(
            "early_ad",
            "二星打工射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage"],
        ),
        champion(
            "late_carry",
            "目标主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage"],
        ),
    ];
    let decision = BenchTransitionScorer::score(
        "ad_item",
        &[current],
        &[target],
        &[profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
        &champions,
        HolderRole::Carry,
    )
    .expect("应给出转移对象");

    assert_eq!(decision.final_holder_id, "late_carry");
    assert_eq!(decision.current_sell_cost, 3);
    assert!(decision.should_wait_for_target);
}

#[test]
fn returns_backup_holders_for_same_completed_item() {
    let champions = vec![
        champion(
            "best_ad",
            "最佳二星射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
        champion(
            "backup_ad",
            "备用射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage"],
        ),
        champion(
            "late_carry",
            "最终主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
    ];
    let items = vec![item(
        "ad_item",
        "物理成装",
        vec!["ad"],
        vec!["ad", "attack_speed"],
        vec![],
    )];
    let context = HolderScoringContext {
        board_units: vec![
            HolderUnit::new("best_ad", 2, HolderUnitSource::Board),
            HolderUnit::new("backup_ad", 2, HolderUnitSource::Board),
        ],
        bench_units: vec![],
        completed_item_ids: vec!["ad_item".into()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
    };

    let plan = HolderScorer::score(&context, &champions, &items);

    assert_eq!(plan.best.unwrap().temporary_holder_id, "best_ad");
    assert!(
        plan.alternatives
            .iter()
            .any(|candidate| candidate.temporary_holder_id == "backup_ad"),
        "单件装备也应输出备用承载者"
    );
}

#[test]
fn real_s18_config_loads_holder_rules_and_scores_plan() {
    use hexsight_engine::{
        GameDataIndex, KnowledgeBaseBuilder, LineupAdapter, LineupLoader, LineupProfileBuilder,
    };

    let config_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config");
    let index = GameDataIndex::load(&config_root, "17").unwrap();
    let raw = LineupLoader::load_cached_lineups(&config_root, "17", "S18").unwrap();
    let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
    let profiles = LineupProfileBuilder::build_all(&cards, &index);
    let profile = profiles
        .iter()
        .find(|profile| {
            !profile.core_equipment_ids.is_empty() && !profile.carry_hero_ids.is_empty()
        })
        .expect("真实阵容应包含主C和核心装")
        .clone();
    let kb = KnowledgeBaseBuilder::with_defaults().build_all(
        &index.heroes_by_id.values().cloned().collect::<Vec<_>>(),
        &index.equipment_by_id.values().cloned().collect::<Vec<_>>(),
        &index.hexes_by_id.values().cloned().collect::<Vec<_>>(),
        &index.traits,
    );
    let board_hero_id = profile
        .early_hero_ids
        .first()
        .or_else(|| profile.mid_hero_ids.first())
        .or_else(|| profile.carry_hero_ids.first())
        .expect("真实阵容应有可用棋子")
        .clone();
    let context = HolderScoringContext {
        board_units: vec![HolderUnit::new(&board_hero_id, 2, HolderUnitSource::Board)],
        bench_units: vec![],
        completed_item_ids: vec![profile.core_equipment_ids[0].clone()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile],
    };

    let plan = HolderScorer::score_from_rule_config(
        &context,
        &kb.champions,
        &kb.items,
        &config_root,
        "S18.1",
    )
    .expect("真实 P3 配置应可加载");

    assert!(plan.best.is_some(), "真实配置应能输出承载者建议");
    assert!(
        plan.summary.contains("给") || plan.summary.contains("等"),
        "P3 输出应是短操作建议"
    );
}

#[test]
fn component_bench_can_build_combat_item_and_recommend_holder() {
    let champions = vec![
        champion(
            "early_ad",
            "二星打工射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
        champion(
            "late_carry",
            "最终主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
    ];
    let items = vec![item(
        "ad_item",
        "物理成装",
        vec!["ad"],
        vec!["ad", "attack_speed"],
        vec![],
    )];
    let synthesis = ItemSynthesisIndex::from_routes(vec![(
        "ad_item".to_string(),
        ["bf_sword".to_string(), "bow".to_string()],
    )]);
    let context = HolderScoringContext {
        board_units: vec![HolderUnit::new("early_ad", 2, HolderUnitSource::Board)],
        bench_units: vec![],
        completed_item_ids: vec![],
        component_item_ids: vec!["bf_sword".into(), "bow".into()],
        active_trait_ids: vec![],
        target_lineups: vec![profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
    };

    let plan = HolderScorer::score_with_synthesis_index(&context, &champions, &items, &synthesis);

    let best = plan.best.expect("散件可合成战力装时应输出承载者");
    assert_eq!(best.item_id, "ad_item");
    assert_eq!(best.temporary_holder_id, "early_ad");
    assert!(best
        .reasons
        .iter()
        .any(|reason| reason.contains("装备席散件可合成")));
    assert!(plan.summary.contains("合物理成装"));
}

#[test]
fn recommends_two_star_bench_workhorse_as_temporary_holder() {
    let champions = vec![
        champion(
            "board_filler",
            "场上一星挂件",
            1,
            ChampionRole::Utility,
            "utility",
            "backline",
            vec![],
        ),
        champion(
            "bench_ad",
            "备战席二星射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
        champion(
            "late_carry",
            "最终主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
    ];
    let items = vec![item(
        "ad_item",
        "物理成装",
        vec!["ad"],
        vec!["ad", "attack_speed"],
        vec![],
    )];
    let context = HolderScoringContext {
        board_units: vec![HolderUnit::new("board_filler", 1, HolderUnitSource::Board)],
        bench_units: vec![HolderUnit::new("bench_ad", 2, HolderUnitSource::Bench)],
        completed_item_ids: vec!["ad_item".into()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
    };

    let plan = HolderScorer::score(&context, &champions, &items);

    let best = plan.best.expect("备战席强打工牌应参与承载者候选");
    assert_eq!(best.temporary_holder_id, "bench_ad");
    assert!(best
        .reasons
        .iter()
        .any(|reason| reason.contains("备战席可上场承载")));
}

#[test]
fn filters_full_item_holder_from_recommendations() {
    let champions = vec![
        champion(
            "full_ad",
            "满装射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
        champion(
            "open_ad",
            "空装备射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage"],
        ),
        champion(
            "late_carry",
            "最终主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage"],
        ),
    ];
    let mut full = HolderUnit::new("full_ad", 2, HolderUnitSource::Board);
    full.item_ids = vec!["i1".into(), "i2".into(), "i3".into()];
    let context = HolderScoringContext {
        board_units: vec![full, HolderUnit::new("open_ad", 2, HolderUnitSource::Board)],
        bench_units: vec![],
        completed_item_ids: vec!["ad_item".into()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
    };

    let plan = HolderScorer::score(
        &context,
        &champions,
        &[item(
            "ad_item",
            "物理成装",
            vec!["ad"],
            vec!["ad", "attack_speed"],
            vec![],
        )],
    );

    let all_ids = plan
        .best
        .iter()
        .chain(plan.alternatives.iter())
        .map(|candidate| candidate.temporary_holder_id.as_str())
        .collect::<Vec<_>>();
    assert!(
        !all_ids.contains(&"full_ad"),
        "满 3 件棋子不应被推荐继续承载"
    );
    assert!(all_ids.contains(&"open_ad"));
}

#[test]
fn default_config_entry_builds_synthesis_index_from_equipment_data() {
    let champions = vec![
        champion(
            "early_ad",
            "二星打工射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
        champion(
            "late_carry",
            "最终主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
    ];
    let items = vec![item(
        "ad_item",
        "物理成装",
        vec!["ad"],
        vec!["ad", "attack_speed"],
        vec![],
    )];
    let equipment = vec![EquipmentData {
        id: "ad_item".into(),
        name: "物理成装".into(),
        equip_type: "成型装备".into(),
        picture: String::new(),
        basicDesc: String::new(),
        desc: String::new(),
        synthesis1: "bf_sword".into(),
        synthesis2: "bow".into(),
        icon: String::new(),
        is_component: false,
        is_completed: true,
    }];
    let config_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config");
    let context = HolderScoringContext {
        board_units: vec![HolderUnit::new("early_ad", 2, HolderUnitSource::Board)],
        bench_units: vec![],
        completed_item_ids: vec![],
        component_item_ids: vec!["bf_sword".into(), "bow".into()],
        active_trait_ids: vec![],
        target_lineups: vec![profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
    };

    let plan = HolderScorer::score_from_rule_config_with_equipment(
        &context,
        &champions,
        &items,
        &equipment,
        &config_root,
        "S18.1",
    )
    .expect("配置入口应构建合成索引");

    let best = plan.best.expect("默认配置入口应消费散件合成");
    assert_eq!(best.item_id, "ad_item");
    assert!(plan.summary.contains("合物理成装"));
}

#[test]
fn sell_cost_uses_actual_recommended_holder() {
    let champions = vec![
        champion(
            "expensive_bad",
            "昂贵前排",
            5,
            ChampionRole::MainTank,
            "tank",
            "frontline",
            vec!["health"],
        ),
        champion(
            "cheap_good",
            "便宜射手",
            1,
            ChampionRole::SecondaryCarry,
            "ad",
            "backline",
            vec!["attack_damage", "attack_speed"],
        ),
        champion(
            "late_carry",
            "最终主C",
            4,
            ChampionRole::PrimaryCarry,
            "ad",
            "backline",
            vec!["attack_damage"],
        ),
    ];
    let context = HolderScoringContext {
        board_units: vec![
            HolderUnit::new("expensive_bad", 2, HolderUnitSource::Board),
            HolderUnit::new("cheap_good", 2, HolderUnitSource::Board),
        ],
        bench_units: vec![],
        completed_item_ids: vec!["ad_item".into()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile(vec!["late_carry"], vec![], vec!["ad_item"], vec![])],
    };

    let plan = HolderScorer::score(
        &context,
        &champions,
        &[item(
            "ad_item",
            "物理成装",
            vec!["ad"],
            vec!["ad", "attack_speed"],
            vec![],
        )],
    );

    let best = plan.best.expect("应推荐实际适配的便宜射手");
    assert_eq!(best.temporary_holder_id, "cheap_good");
    assert_eq!(best.sell_cost, 3);
}

#[test]
fn real_mode16_workhorse_override_enters_holder_reasons() {
    use hexsight_engine::{
        GameDataIndex, HolderConfigLoader, KnowledgeBaseBuilder, LineupAdapter, LineupLoader,
        LineupProfileBuilder,
    };

    let config_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config");
    let index = GameDataIndex::load(&config_root, "16").unwrap();
    let raw = LineupLoader::load_cached_lineups(&config_root, "16", "S18").unwrap();
    let cards = LineupAdapter::cards_from_raw_list(&raw, "16");
    let profiles = LineupProfileBuilder::build_all(&cards, &index);
    let mut profile = profiles
        .iter()
        .find(|profile| {
            !profile.core_equipment_ids.is_empty() && !profile.carry_hero_ids.is_empty()
        })
        .expect("mode16 真实阵容应有核心装")
        .clone();
    let kb = KnowledgeBaseBuilder::with_defaults().build_all(
        &index.heroes_by_id.values().cloned().collect::<Vec<_>>(),
        &index.equipment_by_id.values().cloned().collect::<Vec<_>>(),
        &index.hexes_by_id.values().cloned().collect::<Vec<_>>(),
        &index.traits,
    );
    let override_id = "13331";
    let champ = kb
        .champions
        .iter()
        .find(|champion| champion.hero_id == override_id)
        .expect("workhorse_overrides 中的 mode16 棋子应存在");
    let item = kb
        .items
        .iter()
        .find(|item| item.item_id == "2010")
        .cloned()
        .expect("真实知识库应有鬼索的狂暴之刃");
    profile.core_equipment_ids = vec![item.item_id.clone()];
    let context = HolderScoringContext {
        board_units: vec![HolderUnit::new(&champ.hero_id, 2, HolderUnitSource::Board)],
        bench_units: vec![],
        completed_item_ids: vec![item.item_id.clone()],
        component_item_ids: vec![],
        active_trait_ids: vec![],
        target_lineups: vec![profile],
    };
    let rules = HolderConfigLoader::load_rules(
        &config_root
            .join("rules")
            .join("S18.1")
            .join("holder_rules.json"),
    )
    .unwrap();
    let overrides = HolderConfigLoader::load_workhorse_overrides(
        &config_root
            .join("rules")
            .join("S18.1")
            .join("workhorse_overrides.json"),
    )
    .unwrap();
    let with_override = HolderScorer::score_with_rules(
        &context,
        std::slice::from_ref(champ),
        std::slice::from_ref(&item),
        &rules,
        &overrides,
    );

    assert!(
        with_override
            .best
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason.contains("三费过渡输出强度高")),
        "真实 workhorse_overrides 应进入承载者推荐原因"
    );
}
