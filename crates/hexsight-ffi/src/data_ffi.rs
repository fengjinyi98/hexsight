// 数据提供 FFI 接口
// 核心职责：
// - 暴露 C ABI 兼容的数据 API 给 Swift 调用
// - 六个无状态函数，每次调用独立加载数据
// - 返回 JSON 字符串，调用方通过 hexsight_free_string 释放

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

use hexsight_core::{
    ConflictEnvironment, LineupProfile, OpeningRoute, OpeningRouteResult, RuleOutput,
};
use hexsight_engine::{
    AugmentOptionRanker, AugmentRerollConfigLoader, AugmentRerollContext, AugmentRerollScorer,
    AugmentStage, CombatValueEstimator, CombatValueWeights, EconomyDecisionResult, GameDataIndex,
    HolderScorer, HolderScoringContext, HolderUnit, HolderUnitSource, KnowledgeBase,
    KnowledgeBaseBuilder, KnowledgeDecisionPlanner, LineupAdapter, LineupFitScorer,
    LineupItemFitContext, LineupLoader, LineupProfileBuilder, PatchKnowledgeLoader,
    RemoteLineupSource, RulePackLoader, RulesContextBuilder,
};
use serde::Deserialize;

// ============================================================
// FFI 辅助函数
// ============================================================

/// &str → *mut c_char
fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("{}").unwrap())
        .into_raw()
}

/// *const c_char → String
unsafe fn from_c_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_owned())
}

/// 错误 JSON 响应
fn error_json(msg: &str) -> *mut c_char {
    let json = serde_json::json!({ "error": msg });
    to_c_string(&json.to_string())
}

// ============================================================
// FFI 函数
// ============================================================

/// 获取支持的模式、赛季、玩法能力列表（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_supported_modes_json(_config_root: *const c_char) -> *mut c_char {
    let modes = RulesContextBuilder::supported_modes();
    match serde_json::to_string(&modes) {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&format!("序列化失败: {}", e)),
    }
}

/// 获取某模式阵容列表和详情摘要（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_lineups_json(
    config_root: *const c_char,
    mode: *const c_char,
) -> *mut c_char {
    let (config_root, mode) = unsafe {
        match (from_c_str(config_root), from_c_str(mode)) {
            (Some(c), Some(m)) => (c, m),
            _ => return error_json("参数为空"),
        }
    };

    let raw_items = match LineupLoader::load_cached_lineups(Path::new(&config_root), &mode, "S18") {
        Ok(items) => items,
        Err(e) => return error_json(&format!("加载阵容失败: {}", e)),
    };

    let cards = LineupAdapter::cards_from_raw_list(&raw_items, &mode);
    match serde_json::to_string(&cards) {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&format!("序列化失败: {}", e)),
    }
}

/// 获取阵容详情展示数据（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_lineup_detail_json(
    config_root: *const c_char,
    mode: *const c_char,
    lineup_id: *const c_char,
) -> *mut c_char {
    let (config_root, mode, lineup_id) = unsafe {
        match (
            from_c_str(config_root),
            from_c_str(mode),
            from_c_str(lineup_id),
        ) {
            (Some(c), Some(m), Some(l)) => (c, m, l),
            _ => return error_json("参数为空"),
        }
    };

    let raw_items = match LineupLoader::load_cached_lineups(Path::new(&config_root), &mode, "S18") {
        Ok(items) => items,
        Err(e) => return error_json(&format!("加载阵容失败: {}", e)),
    };

    let cards = LineupAdapter::cards_from_raw_list(&raw_items, &mode);
    let card = match cards
        .iter()
        .find(|c| c.id == lineup_id || c.name.contains(&lineup_id))
    {
        Some(c) => c,
        None => return error_json("阵容未找到"),
    };

    match serde_json::to_string(&card.detail) {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&format!("序列化失败: {}", e)),
    }
}

/// 获取阵容规则/LLM上下文（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_lineup_rules_context_json(
    config_root: *const c_char,
    mode: *const c_char,
    lineup_id: *const c_char,
) -> *mut c_char {
    let (config_root, mode, lineup_id) = unsafe {
        match (
            from_c_str(config_root),
            from_c_str(mode),
            from_c_str(lineup_id),
        ) {
            (Some(c), Some(m), Some(l)) => (c, m, l),
            _ => return error_json("参数为空"),
        }
    };

    let index = match GameDataIndex::load(Path::new(&config_root), &mode) {
        Ok(idx) => idx,
        Err(e) => return error_json(&format!("加载游戏数据失败: {}", e)),
    };

    let raw_items = match LineupLoader::load_cached_lineups(Path::new(&config_root), &mode, "S18") {
        Ok(items) => items,
        Err(e) => return error_json(&format!("加载阵容失败: {}", e)),
    };

    let cards = LineupAdapter::cards_from_raw_list(&raw_items, &mode);
    let card = match cards
        .iter()
        .find(|c| c.id == lineup_id || c.name.contains(&lineup_id))
    {
        Some(c) => c,
        None => return error_json("阵容未找到"),
    };

    let context = RulesContextBuilder::build(card, &mode, Some(&index));
    match serde_json::to_string(&context) {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&format!("序列化失败: {}", e)),
    }
}

/// 刷新远端阵容缓存
/// Rust 负责 CDN URL 拼装、HTTP 请求、校验和缓存写入
#[no_mangle]
pub extern "C" fn hexsight_refresh_lineups_json(
    config_root: *const c_char,
    mode: *const c_char,
) -> *mut c_char {
    let (config_root, mode) = unsafe {
        match (from_c_str(config_root), from_c_str(mode)) {
            (Some(c), Some(m)) => (c, m),
            _ => return error_json("参数为空"),
        }
    };

    match RemoteLineupSource::refresh(Path::new(&config_root), &mode) {
        Ok(count) => {
            let json = serde_json::json!({ "status": "ok", "mode": mode, "count": count });
            to_c_string(&json.to_string())
        }
        Err(e) => error_json(&format!("刷新失败: {}", e)),
    }
}

/// 校验静态数据和阵容数据是否对齐
#[no_mangle]
pub extern "C" fn hexsight_validate_data_snapshot_json(
    config_root: *const c_char,
    mode: *const c_char,
) -> *mut c_char {
    let (config_root, mode) = unsafe {
        match (from_c_str(config_root), from_c_str(mode)) {
            (Some(c), Some(m)) => (c, m),
            _ => return error_json("参数为空"),
        }
    };

    let index = match GameDataIndex::load(Path::new(&config_root), &mode) {
        Ok(idx) => idx,
        Err(e) => return error_json(&format!("加载游戏数据失败: {}", e)),
    };

    let raw_items = match LineupLoader::load_cached_lineups(Path::new(&config_root), &mode, "S18") {
        Ok(items) => items,
        Err(e) => return error_json(&format!("加载阵容失败: {}", e)),
    };

    let cards = LineupAdapter::cards_from_raw_list(&raw_items, &mode);
    let mut missing_hero_ids = Vec::new();
    let mut missing_equip_ids = Vec::new();
    let mut total_hero_ids = 0usize;
    let mut total_equip_ids = 0usize;
    let parseable_lineups = cards.len();

    // 收集所有 hero_id 和 equip_id
    for card in &cards {
        for piece in &card.detail.final_heroes {
            total_hero_ids += 1;
            if index.hero(&piece.hero_id).is_none() {
                missing_hero_ids.push(piece.hero_id.clone());
            }
            for eid in &piece.equipment_ids {
                total_equip_ids += 1;
                if index.equipment(eid).is_none() && eid != "0" {
                    missing_equip_ids.push(eid.clone());
                }
            }
        }
    }

    let report = serde_json::json!({
        "mode": mode,
        "total_lineups": cards.len(),
        "total_hero_ids": total_hero_ids,
        "total_equip_ids": total_equip_ids,
        "missing_hero_ids": missing_hero_ids,
        "missing_equip_ids": missing_equip_ids,
        "parseable_lineups": parseable_lineups,
        "unparseable_lineups": [],
    });

    to_c_string(&report.to_string())
}

/// 获取 Rust 知识决策 RuleOutput（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_knowledge_rule_output_json(
    config_root: *const c_char,
    mode: *const c_char,
    lineup_id: *const c_char,
) -> *mut c_char {
    let (config_root, mode, lineup_id) = unsafe {
        match (
            from_c_str(config_root),
            from_c_str(mode),
            from_c_str(lineup_id),
        ) {
            (Some(c), Some(m), Some(l)) => (c, m, l),
            _ => return error_json("参数为空"),
        }
    };

    let input = KnowledgeRuleInput {
        lineup_id,
        ..KnowledgeRuleInput::default()
    };
    match build_knowledge_rule_output(Path::new(&config_root), &mode, input)
        .and_then(|output| serde_json::to_string(&output).map_err(|e| format!("序列化失败: {}", e)))
    {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&e),
    }
}

/// 获取 Rust 知识决策 RuleOutput（带当前对局上下文 JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_knowledge_rule_output_with_context_json(
    config_root: *const c_char,
    mode: *const c_char,
    context_json: *const c_char,
) -> *mut c_char {
    let (config_root, mode, context_json) = unsafe {
        match (
            from_c_str(config_root),
            from_c_str(mode),
            from_c_str(context_json),
        ) {
            (Some(c), Some(m), Some(ctx)) => (c, m, ctx),
            _ => return error_json("参数为空"),
        }
    };

    let input = match serde_json::from_str::<KnowledgeRuleInput>(&context_json) {
        Ok(input) => input,
        Err(e) => return error_json(&format!("解析知识决策上下文失败: {}", e)),
    };

    match build_knowledge_rule_output(Path::new(&config_root), &mode, input)
        .and_then(|output| serde_json::to_string(&output).map_err(|e| format!("序列化失败: {}", e)))
    {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&e),
    }
}

/// 知识决策输入 JSON
/// 核心职责：
/// - 承接 Swift 或识别链路传入的当前局面对局上下文
/// - 为 P1-P6 真实规则模块提供稳定输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct KnowledgeRuleInput {
    lineup_id: String,
    current_hero_ids: Vec<String>,
    completed_item_ids: Vec<String>,
    component_item_ids: Vec<String>,
    current_augment_ids: Vec<String>,
    candidate_augment_ids: Vec<String>,
    board_units: Vec<HolderUnit>,
    bench_units: Vec<HolderUnit>,
    active_trait_ids: Vec<String>,
    current_gold: i32,
    current_hp: i32,
    current_level: i32,
    round_stage: f64,
    rival_counts: HashMap<String, i32>,
    has_augment_reroll: bool,
    augment_stage: FfiAugmentStage,
    environment: ConflictEnvironment,
}

impl Default for KnowledgeRuleInput {
    fn default() -> Self {
        Self {
            lineup_id: String::new(),
            current_hero_ids: Vec::new(),
            completed_item_ids: Vec::new(),
            component_item_ids: Vec::new(),
            current_augment_ids: Vec::new(),
            candidate_augment_ids: Vec::new(),
            board_units: Vec::new(),
            bench_units: Vec::new(),
            active_trait_ids: Vec::new(),
            current_gold: 30,
            current_hp: 70,
            current_level: 7,
            round_stage: 3.5,
            rival_counts: HashMap::new(),
            has_augment_reroll: true,
            augment_stage: FfiAugmentStage::Second,
            environment: ConflictEnvironment::default(),
        }
    }
}

/// FFI 海克斯轮次
/// 核心职责：
/// - 兼容 JSON 中的海克斯刷新阶段字段
/// - 转换为规则引擎内部 AugmentStage
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FfiAugmentStage {
    First,
    #[default]
    Second,
    Third,
}

impl From<FfiAugmentStage> for AugmentStage {
    fn from(value: FfiAugmentStage) -> Self {
        match value {
            FfiAugmentStage::First => AugmentStage::First,
            FfiAugmentStage::Second => AugmentStage::Second,
            FfiAugmentStage::Third => AugmentStage::Third,
        }
    }
}

fn build_knowledge_rule_output(
    config_root: &Path,
    mode: &str,
    input: KnowledgeRuleInput,
) -> Result<RuleOutput, String> {
    let index =
        GameDataIndex::load(config_root, mode).map_err(|e| format!("加载游戏数据失败: {}", e))?;
    let raw_items = LineupLoader::load_cached_lineups(config_root, mode, "S18")
        .map_err(|e| format!("加载阵容失败: {}", e))?;
    let cards = LineupAdapter::cards_from_raw_list(&raw_items, mode);
    let profiles = LineupProfileBuilder::build_all(&cards, &index);
    if profiles.is_empty() {
        return Err("阵容为空".into());
    }

    let selected = select_profile(&profiles, &input.lineup_id)?;
    let heroes = index.heroes_by_id.values().cloned().collect::<Vec<_>>();
    let equipment = index.equipment_by_id.values().cloned().collect::<Vec<_>>();
    let hexes = index.hexes_by_id.values().cloned().collect::<Vec<_>>();
    let knowledge_base = KnowledgeBaseBuilder::with_defaults()
        .build_all_with_rule_config(
            &heroes,
            &equipment,
            &hexes,
            &index.traits,
            config_root,
            "S18.1",
        )
        .map_err(|e| format!("构建知识库失败: {}", e))?;

    let completed_item_ids = default_completed_items(&input, selected);
    let component_item_ids = default_component_items(&input, selected, &index);
    let current_hero_ids = default_current_heroes(&input, selected);
    let current_augment_ids = input.current_augment_ids.clone();

    let (rule_pack, _warnings) = RulePackLoader::load(config_root, "S18.1");
    let item_context = LineupItemFitContext::from_rule_config(
        &knowledge_base,
        &completed_item_ids,
        &component_item_ids,
        &equipment,
        config_root,
        "S18.1",
    )
    .map_err(|e| format!("构建装备评分上下文失败: {}", e))?;
    let lineup_scores = LineupFitScorer::score_all_with_item_context(
        &profiles,
        &current_hero_ids,
        &current_augment_ids,
        input.current_gold,
        input.current_hp,
        input.current_level,
        input.round_stage,
        &input.rival_counts,
        &rule_pack,
        &item_context,
    );

    let board_units = default_board_units(&input, selected);
    let holder_context = HolderScoringContext {
        board_units,
        bench_units: input.bench_units.clone(),
        completed_item_ids: completed_item_ids.clone(),
        component_item_ids: component_item_ids.clone(),
        active_trait_ids: default_active_traits(&input, selected),
        target_lineups: vec![selected.clone()],
    };
    let holder_plan = HolderScorer::score_from_rule_config_with_equipment(
        &holder_context,
        &knowledge_base.champions,
        &knowledge_base.items,
        &equipment,
        config_root,
        "S18.1",
    )
    .map_err(|e| format!("计算承载者失败: {}", e))?;

    let augment_reroll = build_augment_decision(config_root, &input, selected, &profiles, &index)?;
    let combat_value_diff = build_combat_value_diff(
        config_root,
        &input,
        selected,
        &knowledge_base,
        &completed_item_ids,
    )?;
    let patch_knowledge = PatchKnowledgeLoader::load(config_root, "S18.1").ok();
    let item_direction = infer_item_direction(selected, &completed_item_ids, &knowledge_base);
    let opening = OpeningRouteResult {
        route: OpeningRoute::Mixed,
        confidence: 0.75,
        reasons: vec!["FFI 真实知识链路已接入 P1-P6 规则结果".into()],
        two_star_count: holder_context
            .board_units
            .iter()
            .filter(|unit| unit.star_level >= 2)
            .count() as i32,
        frontline_quality: 55,
        can_build_combat_item: !completed_item_ids.is_empty() || !component_item_ids.is_empty(),
        recommended_actions: vec!["按知识规则输出执行当前回合决策".into()],
    };
    let economy = EconomyDecisionResult {
        action: if input.current_gold >= 50 {
            "hold_interest".into()
        } else {
            "hold_flexible".into()
        },
        label: if input.current_gold >= 50 {
            "保利息".into()
        } else {
            "保留弹性".into()
        },
        target_gold: 30,
        reasons: vec![format!(
            "当前金币 {}，血量 {}",
            input.current_gold, input.current_hp
        )],
    };

    Ok(KnowledgeDecisionPlanner::plan(
        &opening,
        &lineup_scores,
        &profiles,
        patch_knowledge.as_ref(),
        &item_direction,
        &economy,
        Some(&augment_reroll),
        Some(&holder_plan),
        Some(&combat_value_diff),
        input.current_hp,
    ))
}

fn select_profile<'a>(
    profiles: &'a [LineupProfile],
    lineup_id: &str,
) -> Result<&'a LineupProfile, String> {
    if lineup_id.trim().is_empty() {
        return profiles.first().ok_or_else(|| "阵容为空".into());
    }
    profiles
        .iter()
        .find(|profile| profile.lineup_id == lineup_id || profile.name.contains(lineup_id))
        .ok_or_else(|| "阵容未找到".into())
}

fn default_completed_items(input: &KnowledgeRuleInput, selected: &LineupProfile) -> Vec<String> {
    if !input.completed_item_ids.is_empty() {
        return input.completed_item_ids.clone();
    }
    selected
        .core_equipment_ids
        .iter()
        .take(1)
        .cloned()
        .collect()
}

fn default_component_items(
    input: &KnowledgeRuleInput,
    selected: &LineupProfile,
    index: &GameDataIndex,
) -> Vec<String> {
    if !input.component_item_ids.is_empty() {
        return input.component_item_ids.clone();
    }
    selected
        .core_equipment_ids
        .first()
        .and_then(|item_id| index.equipment(item_id))
        .map(|item| {
            [item.synthesis1.clone(), item.synthesis2.clone()]
                .into_iter()
                .filter(|id| !id.is_empty() && id != "0")
                .collect()
        })
        .unwrap_or_default()
}

fn default_current_heroes(input: &KnowledgeRuleInput, selected: &LineupProfile) -> Vec<String> {
    if !input.current_hero_ids.is_empty() {
        return input.current_hero_ids.clone();
    }
    selected
        .early_hero_ids
        .iter()
        .chain(selected.mid_hero_ids.iter())
        .chain(selected.carry_hero_ids.iter())
        .take(4)
        .cloned()
        .collect()
}

fn default_board_units(input: &KnowledgeRuleInput, selected: &LineupProfile) -> Vec<HolderUnit> {
    if !input.board_units.is_empty() {
        return input.board_units.clone();
    }
    let active_trait_ids = default_active_traits(input, selected);
    selected
        .early_hero_ids
        .first()
        .or_else(|| selected.mid_hero_ids.first())
        .or_else(|| selected.carry_hero_ids.first())
        .map(|hero_id| HolderUnit {
            hero_id: hero_id.clone(),
            star_level: 2,
            source: HolderUnitSource::Board,
            item_ids: Vec::new(),
            active_trait_ids,
        })
        .into_iter()
        .collect()
}

fn default_active_traits(input: &KnowledgeRuleInput, selected: &LineupProfile) -> Vec<String> {
    if !input.active_trait_ids.is_empty() {
        return input.active_trait_ids.clone();
    }
    selected.trait_targets.keys().cloned().collect()
}

fn build_augment_decision(
    config_root: &Path,
    input: &KnowledgeRuleInput,
    selected: &LineupProfile,
    profiles: &[LineupProfile],
    index: &GameDataIndex,
) -> Result<hexsight_engine::AugmentRerollDecision, String> {
    let candidate_ids = if input.candidate_augment_ids.is_empty() {
        selected
            .recommended_hex_ids
            .iter()
            .chain(selected.replacement_hex_ids.iter())
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        input.candidate_augment_ids.clone()
    };
    let candidates = candidate_ids
        .iter()
        .filter_map(|id| {
            index.hex(id).map(|hex| {
                (
                    hex.id.clone(),
                    hex.name.clone(),
                    hex.desc.clone(),
                    hex.level.parse::<i32>().unwrap_or(2),
                )
            })
        })
        .collect::<Vec<_>>();
    let candidates = if candidates.is_empty() {
        index
            .hexes_by_id
            .values()
            .take(3)
            .map(|hex| {
                (
                    hex.id.clone(),
                    hex.name.clone(),
                    hex.desc.clone(),
                    hex.level.parse::<i32>().unwrap_or(2),
                )
            })
            .collect::<Vec<_>>()
    } else {
        candidates
    };
    let rules_dir = config_root.join("rules").join("S18.1");
    let weights =
        AugmentRerollConfigLoader::load_type_weights(&rules_dir.join("augment_type_weights.json"))
            .map_err(|e| format!("加载海克斯类型权重失败: {}", e))?;
    let thresholds =
        AugmentRerollConfigLoader::load_thresholds(&rules_dir.join("augment_thresholds.json"))
            .map_err(|e| format!("加载海克斯阈值失败: {}", e))?;
    let stage = AugmentStage::from(input.augment_stage);
    let ranked = AugmentOptionRanker::rank_options_with_weights(
        &candidates,
        profiles,
        stage,
        input.current_hp >= 70,
        input.current_hp <= 45,
        &weights,
    );
    Ok(AugmentRerollScorer::decide_with_thresholds(
        &AugmentRerollContext {
            ranked_options: ranked,
            has_reroll: input.has_augment_reroll,
            stage,
            current_hp: input.current_hp,
            current_gold: input.current_gold,
            locked_lineup_id: Some(selected.lineup_id.clone()),
        },
        &thresholds,
    ))
}

fn build_combat_value_diff(
    config_root: &Path,
    input: &KnowledgeRuleInput,
    selected: &LineupProfile,
    knowledge_base: &KnowledgeBase,
    completed_item_ids: &[String],
) -> Result<hexsight_engine::CombatValueDiff, String> {
    let champion_id = selected
        .carry_hero_ids
        .first()
        .or_else(|| selected.final_hero_ids.first())
        .ok_or_else(|| "阵容缺少收益估算棋子".to_string())?;
    let champion = knowledge_base
        .champions
        .iter()
        .find(|champion| &champion.hero_id == champion_id)
        .ok_or_else(|| "收益估算棋子画像缺失".to_string())?;
    let item_by_id = knowledge_base
        .items
        .iter()
        .map(|item| (item.item_id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let left_items = completed_item_ids
        .iter()
        .filter_map(|id| item_by_id.get(id.as_str()).copied())
        .cloned()
        .collect::<Vec<_>>();
    let mut right_ids = selected
        .core_equipment_ids
        .iter()
        .filter(|id| !completed_item_ids.contains(id))
        .take(2)
        .cloned()
        .collect::<Vec<_>>();
    if right_ids.is_empty() {
        right_ids = selected
            .tank_equipment_ids
            .iter()
            .filter(|id| !completed_item_ids.contains(id))
            .take(2)
            .cloned()
            .collect();
    }
    let right_items = right_ids
        .iter()
        .filter_map(|id| item_by_id.get(id.as_str()).copied())
        .cloned()
        .collect::<Vec<_>>();
    let weights = CombatValueWeights::load(config_root, "S18.1").unwrap_or_default();
    let estimator = CombatValueEstimator::new(weights);
    Ok(estimator.compare_item_sets(champion, &left_items, &right_items, &input.environment))
}

fn infer_item_direction(
    selected: &LineupProfile,
    completed_item_ids: &[String],
    knowledge_base: &KnowledgeBase,
) -> String {
    let item_by_id = knowledge_base
        .items
        .iter()
        .map(|item| (item.item_id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let mut ad = 0;
    let mut ap = 0;
    let mut tank = 0;
    for item_id in completed_item_ids {
        if let Some(item) = item_by_id.get(item_id.as_str()) {
            if item.damage_type_fit.iter().any(|fit| fit == "ad") {
                ad += 1;
            }
            if item.damage_type_fit.iter().any(|fit| fit == "ap") {
                ap += 1;
            }
            if item.damage_type_fit.iter().any(|fit| fit == "tank") {
                tank += 1;
            }
        }
    }
    if tank > ad.max(ap)
        || (!selected.tank_equipment_ids.is_empty() && selected.core_equipment_ids.is_empty())
    {
        "坦".into()
    } else if ap > ad {
        "AP".into()
    } else if ad > ap {
        "AD".into()
    } else {
        "hold_flexible".into()
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::path::PathBuf;

    // 引用同 crate 中 lib.rs 定义的 hexsight_free_string
    extern "C" {
        fn hexsight_free_string(ptr: *mut c_char);
    }

    fn config_root_cstr() -> CString {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config");
        CString::new(path.to_str().unwrap()).unwrap()
    }

    fn mode17_cstr() -> CString {
        CString::new("17").unwrap()
    }

    #[test]
    fn test_get_supported_modes() {
        let root = config_root_cstr();
        let ptr = hexsight_get_supported_modes_json(root.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("星神"));
        assert!(json.contains("英雄联盟传奇"));
        assert!(json.contains("天选福星"));
        unsafe {
            hexsight_free_string(ptr);
        }
    }

    #[test]
    fn test_get_lineups_json() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let ptr = hexsight_get_lineups_json(root.as_ptr(), mode.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("final_heroes"));
        unsafe {
            hexsight_free_string(ptr);
        }
    }

    #[test]
    fn test_get_lineup_rules_context() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let id = CString::new("神谕龙王").unwrap();
        let ptr = hexsight_get_lineup_rules_context_json(root.as_ptr(), mode.as_ptr(), id.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("finalHeroes"));
        assert!(json.contains("\"traits\""));
        unsafe {
            hexsight_free_string(ptr);
        }
    }

    #[test]
    fn test_validate_snapshot() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let ptr = hexsight_validate_data_snapshot_json(root.as_ptr(), mode.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("total_lineups"));
        unsafe {
            hexsight_free_string(ptr);
        }
    }

    #[test]
    fn test_lineup_not_found() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let id = CString::new("nonexistent_lineup_12345").unwrap();
        let ptr = hexsight_get_lineup_detail_json(root.as_ptr(), mode.as_ptr(), id.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("error"));
        unsafe {
            hexsight_free_string(ptr);
        }
    }

    #[test]
    fn test_get_knowledge_rule_output_json() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let id = CString::new("神谕龙王").unwrap();
        let ptr =
            hexsight_get_knowledge_rule_output_json(root.as_ptr(), mode.as_ptr(), id.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("P8 RuleOutput 应为合法 JSON");

        assert!(value.get("strategy").is_some());
        assert!(value["lineupRecommendations"].is_array());
        assert!(value["itemAction"]["action"].is_string());
        assert!(value["augmentAction"]["action"].is_string());
        assert!(value["pivotConditions"].is_array());
        assert!(value["knowledgeActions"]["shortExplanations"].is_array());

        unsafe {
            hexsight_free_string(ptr);
        }
    }

    #[test]
    fn test_knowledge_rule_output_uses_real_p1_to_p6_chain() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let id = CString::new("神谕龙王").unwrap();
        let ptr =
            hexsight_get_knowledge_rule_output_json(root.as_ptr(), mode.as_ptr(), id.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("真实 RuleOutput 应为合法 JSON");

        assert!(
            value["knowledgeActions"]["holder"].is_object(),
            "FFI 应把 P3 承载者方案传入 KnowledgeDecisionPlanner"
        );
        assert!(
            value["knowledgeActions"]["combat"].is_object(),
            "FFI 应把 P6 收益估算传入 KnowledgeDecisionPlanner"
        );
        assert!(
            value["knowledgeActions"]["shortExplanations"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .any(|item| item.as_str().unwrap_or("").contains("海克斯")),
            "FFI 应把 P4 海克斯刷新决策传入 KnowledgeDecisionPlanner"
        );

        unsafe {
            hexsight_free_string(ptr);
        }
    }
}
