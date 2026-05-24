// 棋子装备适配评分器
// 核心职责：
// - 基于棋子能力画像 + 装备收益画像计算装备适配分
// - 输出 core/strong/acceptable/emergency/bad 分级
// - 缺核心装时推荐替代装并降权阵容评分

use std::collections::{HashMap, HashSet};

use hexsight_core::{ChampionCapability, ConflictEnvironment, EquipmentData, ItemValueProfile};

use crate::item_conflict_scorer::ItemConflictScorer;

/// 装备适配等级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum ItemTier {
    #[serde(rename = "core")]
    Core,
    #[serde(rename = "strong")]
    Strong,
    #[serde(rename = "acceptable")]
    Acceptable,
    #[serde(rename = "emergency")]
    Emergency,
    #[serde(rename = "bad")]
    Bad,
}

/// 单件装备对棋子的适配结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChampionItemFit {
    pub item_id: String,
    pub item_name: String,
    /// 适配分 (0-100)
    pub score: i32,
    /// 适配等级
    pub tier: ItemTier,
    /// 适配原因
    pub reasons: Vec<String>,
    /// 不匹配原因
    pub penalties: Vec<String>,
}

/// 棋子装备适配评分器
pub struct ChampionItemFitScorer;

/// 装备缺失影响
#[derive(Debug, Clone)]
pub struct ItemGapImpact {
    /// 缺核心装惩罚分
    pub core_missing_penalty: i32,
    /// 推荐替代装
    pub alternatives: Vec<String>,
    /// 是否建议转向
    pub should_pivot: bool,
    /// 解释
    pub explanation: String,
}

/// 装备合成索引
/// 核心职责：
/// - 从静态装备数据提取成装到散件的合成路线
/// - 判断当前散件是否能合成指定成装
#[derive(Debug, Clone, Default)]
pub struct ItemSynthesisIndex {
    routes: HashMap<String, Vec<String>>,
}

impl ItemSynthesisIndex {
    pub fn from_equipment(equipment: &[EquipmentData]) -> Self {
        let routes = equipment
            .iter()
            .filter(|equip| equip.is_completed || equip.equip_type == "成型装备")
            .filter_map(|equip| {
                let components: Vec<String> = [&equip.synthesis1, &equip.synthesis2]
                    .into_iter()
                    .filter(|id| !id.is_empty() && *id != "0")
                    .map(|id| id.to_string())
                    .collect();
                if components.is_empty() {
                    None
                } else {
                    Some((equip.id.clone(), components))
                }
            })
            .collect();
        Self { routes }
    }

    pub fn from_routes<I, R>(routes: I) -> Self
    where
        I: IntoIterator<Item = (String, R)>,
        R: IntoIterator<Item = String>,
    {
        Self {
            routes: routes
                .into_iter()
                .map(|(item_id, components)| (item_id, components.into_iter().collect()))
                .collect(),
        }
    }

    pub fn can_synthesize(&self, item_id: &str, current_component_ids: &[String]) -> bool {
        let Some(route) = self.routes.get(item_id) else {
            return false;
        };
        if route.is_empty() {
            return false;
        }

        let mut available: HashMap<&str, usize> = HashMap::new();
        for component_id in current_component_ids {
            *available.entry(component_id.as_str()).or_insert(0) += 1;
        }

        for component_id in route {
            let Some(count) = available.get_mut(component_id.as_str()) else {
                return false;
            };
            if *count == 0 {
                return false;
            }
            *count -= 1;
        }
        true
    }
}

impl ChampionItemFitScorer {
    /// 计算某棋子对某装备的适配分
    pub fn score_item(champ: &ChampionCapability, item: &ItemValueProfile) -> i32 {
        let mut score = 40; // 基础分

        // 1. 伤害类型匹配 (权重 25)
        for dt in &item.damage_type_fit {
            if champ.damage_profile.contains(dt.as_str())
                || (dt == "ad" && champ.scaling_stats.contains(&"attack_damage".to_string()))
                || (dt == "ap" && champ.scaling_stats.contains(&"ability_power".to_string()))
                || (dt == "tank" && champ.position_role == "frontline")
            {
                score += 25;
                break;
            }
        }

        // 2. 缩放属性匹配 (权重 20)
        for stat in &champ.scaling_stats {
            let stat_match = match stat.as_str() {
                "attack_damage" => {
                    item.effect_tags.iter().any(|t| t == "attack_damage")
                        || item.stats.iter().any(|s| s.name == "ad")
                }
                "ability_power" => item.stats.iter().any(|s| s.name == "ap"),
                "attack_speed" => item.stats.iter().any(|s| s.name == "attack_speed"),
                "mana" => item.effect_tags.contains(&"mana_engine".to_string()),
                "crit" => item.stats.iter().any(|s| s.name == "crit"),
                "health" => item
                    .stats
                    .iter()
                    .any(|s| s.name == "hp" || s.name == "armor"),
                _ => false,
            };
            if stat_match {
                score += 20;
                break;
            }
        }

        // 3. 施放模式匹配 (权重 15)
        if champ.cast_pattern == "mana_cast"
            && item.effect_tags.contains(&"mana_engine".to_string())
        {
            score += 15;
        } else if champ.cast_pattern == "attack_based"
            && item
                .stats
                .iter()
                .any(|s| s.name == "attack_speed" || s.name == "ad")
        {
            score += 15;
        }

        // 4. 站位修正 (权重 10)
        if champ.position_role == "frontline"
            && item
                .stats
                .iter()
                .any(|s| s.name == "hp" || s.name == "armor")
        {
            score += 10;
        } else if champ.position_role == "backline"
            && item.effect_tags.contains(&"mana_engine".to_string())
        {
            score += 10;
        }

        // 5. 风险适配 (权重 10)
        if champ.risk_profile.contains(&"crowd_control".to_string())
            && item.effect_tags.contains(&"shield_survival".to_string())
        {
            score += 10;
        }
        if champ.risk_profile.contains(&"burst_damage".to_string())
            && item.effect_tags.contains(&"shield_survival".to_string())
        {
            score += 10;
        }

        score.clamp(0, 100)
    }

    /// 计算棋子的全部装备分级
    pub fn score_all_items(
        champ: &ChampionCapability,
        items: &[ItemValueProfile],
    ) -> Vec<ChampionItemFit> {
        let mut fits: Vec<ChampionItemFit> = items
            .iter()
            .filter(|i| i.item_type == "成型装备")
            .map(|item| {
                let score = Self::score_item(champ, item);
                let tier = match score {
                    s if s >= 75 => ItemTier::Core,
                    s if s >= 60 => ItemTier::Strong,
                    s if s >= 40 => ItemTier::Acceptable,
                    s if s >= 25 => ItemTier::Emergency,
                    _ => ItemTier::Bad,
                };
                let mut reasons = Vec::new();
                let mut penalties = Vec::new();
                if tier == ItemTier::Core || tier == ItemTier::Strong {
                    reasons.push("机制高度匹配".into());
                }
                if score < 40 {
                    penalties.push("收益偏低".into());
                }
                ChampionItemFit {
                    item_id: item.item_id.clone(),
                    item_name: item.name.clone(),
                    score,
                    tier,
                    reasons,
                    penalties,
                }
            })
            .collect();
        fits.sort_by(|a, b| b.score.cmp(&a.score));
        fits
    }

    /// 获取棋子的装备偏好（按分级归类）
    pub fn item_preferences(
        champ: &ChampionCapability,
        items: &[ItemValueProfile],
    ) -> hexsight_core::ItemPreferences {
        let fits = Self::score_all_items(champ, items);
        let mut prefs = hexsight_core::ItemPreferences::default();
        for fit in &fits {
            match fit.tier {
                ItemTier::Core => prefs.core.push(fit.item_name.clone()),
                ItemTier::Strong => prefs.strong.push(fit.item_name.clone()),
                ItemTier::Acceptable => prefs.acceptable.push(fit.item_name.clone()),
                ItemTier::Emergency => prefs.emergency.push(fit.item_name.clone()),
                ItemTier::Bad => prefs.bad.push(fit.item_name.clone()),
            }
        }
        prefs
    }

    /// 检查装备缺口影响
    #[deprecated(note = "使用 score_with_item_context 以接入推荐装、替代组和合成表")]
    pub fn assess_item_gaps(
        champ: &ChampionCapability,
        recommended_items: &[String],
        current_items: &[String],
        all_items: &[ItemValueProfile],
    ) -> ItemGapImpact {
        let fits = Self::score_all_items(champ, all_items);
        let missing_core: Vec<&String> = recommended_items
            .iter()
            .filter(|id| !current_items.contains(id))
            .collect();

        let mut alternatives = Vec::new();
        for fit in &fits {
            if fit.tier <= ItemTier::Strong && !recommended_items.contains(&fit.item_id) {
                alternatives.push(fit.item_name.clone());
                if alternatives.len() >= 3 {
                    break;
                }
            }
        }

        let core_missing_penalty = (missing_core.len() as i32 * 20).min(60);
        let should_pivot = core_missing_penalty >= 40 && alternatives.len() < 2;

        let explanation = if missing_core.is_empty() {
            "核心装备齐全".into()
        } else if should_pivot {
            format!(
                "缺 {} 件核心装且替代不足，建议考虑转阵容",
                missing_core.len()
            )
        } else {
            format!(
                "缺 {} 件核心装，可用替代: {}",
                missing_core.len(),
                alternatives.join(", ")
            )
        };

        ItemGapImpact {
            core_missing_penalty,
            alternatives,
            should_pivot,
            explanation,
        }
    }

    /// 基于推荐装备强先验 + 机制评分，返回完整分级
    /// recommended_ids: 官网阵容推荐的主 C 装备 ID 列表
    pub fn score_with_recommended(
        champ: &ChampionCapability,
        items: &[ItemValueProfile],
        recommended_ids: &[String],
        current_component_ids: &[String], // 当前散件 ID 列表
    ) -> (Vec<ChampionItemFit>, ItemGapImpact) {
        Self::score_with_item_context(
            champ,
            items,
            recommended_ids,
            &[],
            current_component_ids,
            None,
            None,
        )
    }

    /// 基于推荐装、当前成装、当前散件、替代组和合成表计算装备适配
    pub fn score_with_item_context(
        champ: &ChampionCapability,
        items: &[ItemValueProfile],
        recommended_ids: &[String],
        current_completed_item_ids: &[String],
        current_component_ids: &[String],
        replacement_groups: Option<&ItemReplacementGroups>,
        synthesis_index: Option<&ItemSynthesisIndex>,
    ) -> (Vec<ChampionItemFit>, ItemGapImpact) {
        let mut fits = Self::score_all_items(champ, items);

        // 官网推荐装强先验：标记为 core 候选并大幅加分
        for fit in &mut fits {
            if recommended_ids.contains(&fit.item_id) {
                fit.score = (fit.score + 30).min(100);
                fit.tier = if fit.score >= 75 {
                    ItemTier::Core
                } else {
                    ItemTier::Strong
                };
                fit.reasons.push("官网推荐装".into());
            }
        }
        fits.sort_by(|a, b| b.score.cmp(&a.score));

        let completed: HashSet<&str> = current_completed_item_ids
            .iter()
            .map(|id| id.as_str())
            .collect();
        let can_synthesize_ids: HashSet<&str> = recommended_ids
            .iter()
            .filter(|id| {
                synthesis_index
                    .map(|index| index.can_synthesize(id, current_component_ids))
                    .unwrap_or(false)
            })
            .map(|id| id.as_str())
            .collect();
        let can_synthesize = can_synthesize_ids.len() as i32;

        let missing_core: Vec<&String> = recommended_ids
            .iter()
            .filter(|id| !completed.contains(id.as_str()))
            .collect();

        let alternatives =
            Self::gap_alternatives(&fits, &missing_core, recommended_ids, replacement_groups);

        let unsynth_missing = missing_core
            .iter()
            .filter(|id| !can_synthesize_ids.contains(id.as_str()))
            .count() as i32;
        let strictness_penalty = if champ.item_strictness == "high" {
            unsynth_missing * 5
        } else {
            0
        };
        let penalty = ((missing_core.len() as i32 * 20) - (can_synthesize * 10)
            + strictness_penalty)
            .clamp(0, 60);

        let should_pivot = penalty >= 40 && alternatives.len() < 2;
        let explanation = if missing_core.is_empty() {
            format!("核心装备齐全（可合成 {} 件）", can_synthesize)
        } else if should_pivot {
            format!(
                "缺 {} 件核心装且替代不足（可合成 {} 件），建议转向",
                missing_core.len(),
                can_synthesize
            )
        } else {
            format!(
                "缺 {} 件核心装，可用替代: {}（可合成 {} 件）",
                missing_core.len(),
                alternatives.join(", "),
                can_synthesize
            )
        };

        let impact = ItemGapImpact {
            core_missing_penalty: penalty,
            alternatives,
            should_pivot,
            explanation,
        };

        (fits, impact)
    }

    /// 基于装备上下文和冲突评分器计算装备适配
    pub fn score_with_item_context_and_conflicts(
        champ: &ChampionCapability,
        items: &[ItemValueProfile],
        recommended_ids: &[String],
        current_completed_item_ids: &[String],
        current_component_ids: &[String],
        replacement_groups: Option<&ItemReplacementGroups>,
        synthesis_index: Option<&ItemSynthesisIndex>,
        conflict_scorer: Option<&ItemConflictScorer>,
        environment: &ConflictEnvironment,
    ) -> (Vec<ChampionItemFit>, ItemGapImpact) {
        let (mut fits, impact) = Self::score_with_item_context(
            champ,
            items,
            recommended_ids,
            current_completed_item_ids,
            current_component_ids,
            replacement_groups,
            synthesis_index,
        );

        let Some(conflict_scorer) = conflict_scorer else {
            return (fits, impact);
        };

        for fit in &mut fits {
            if current_completed_item_ids.contains(&fit.item_id) {
                continue;
            }

            let candidate_groups: HashSet<String> = conflict_scorer
                .get_conflict_groups(&fit.item_id)
                .into_iter()
                .flat_map(|group| {
                    group
                        .conflicts
                        .iter()
                        .filter(|entry| entry.item_id == fit.item_id)
                        .map(|_| group.label.clone())
                        .collect::<Vec<_>>()
                })
                .collect();
            if candidate_groups.is_empty() {
                continue;
            }

            let mut candidate_items = current_completed_item_ids.to_vec();
            candidate_items.push(fit.item_id.clone());
            let warnings =
                conflict_scorer.detect_conflicts_with_environment(&candidate_items, environment);
            let matching_penalty: i32 = warnings
                .iter()
                .filter(|warning| candidate_groups.contains(&warning.group_label))
                .map(|warning| warning.penalty_score)
                .sum();
            if matching_penalty >= 0 {
                continue;
            }

            let score_penalty = (-matching_penalty / 2).max(1);
            fit.score = (fit.score - score_penalty).clamp(0, 100);
            fit.tier = Self::tier_for_score(fit.score);
            for warning in warnings {
                if candidate_groups.contains(&warning.group_label) {
                    fit.penalties.push(format!(
                        "{}重复覆盖，{}",
                        warning.group_label, warning.remaining_value
                    ));
                }
            }
        }

        fits.sort_by(|a, b| b.score.cmp(&a.score));
        (fits, impact)
    }

    fn tier_for_score(score: i32) -> ItemTier {
        match score {
            s if s >= 75 => ItemTier::Core,
            s if s >= 60 => ItemTier::Strong,
            s if s >= 40 => ItemTier::Acceptable,
            s if s >= 25 => ItemTier::Emergency,
            _ => ItemTier::Bad,
        }
    }

    fn gap_alternatives(
        fits: &[ChampionItemFit],
        missing_core: &[&String],
        recommended_ids: &[String],
        replacement_groups: Option<&ItemReplacementGroups>,
    ) -> Vec<String> {
        let mut alternatives = Vec::new();
        let mut seen = HashSet::new();
        let fit_by_id: HashMap<&str, &ChampionItemFit> =
            fits.iter().map(|fit| (fit.item_id.as_str(), fit)).collect();
        let fit_by_name: HashMap<&str, &ChampionItemFit> = fits
            .iter()
            .map(|fit| (fit.item_name.as_str(), fit))
            .collect();

        if let Some(groups) = replacement_groups {
            for missing_id in missing_core {
                let missing_name = fit_by_id
                    .get(missing_id.as_str())
                    .map(|fit| fit.item_name.as_str())
                    .unwrap_or(missing_id.as_str());
                for group in groups.groups.values() {
                    let contains_missing = group
                        .configured_items()
                        .any(|item| item == *missing_id || item == missing_name);
                    if !contains_missing {
                        continue;
                    }
                    for item in group.configured_items() {
                        let Some(fit) = fit_by_id
                            .get(item.as_str())
                            .or_else(|| fit_by_name.get(item.as_str()))
                        else {
                            continue;
                        };
                        if recommended_ids.contains(&fit.item_id) {
                            continue;
                        }
                        if seen.insert(fit.item_name.clone()) {
                            alternatives.push(fit.item_name.clone());
                            if alternatives.len() >= 3 {
                                return alternatives;
                            }
                        }
                    }
                }
            }
        }

        for fit in fits {
            if fit.tier <= ItemTier::Acceptable
                && !recommended_ids.contains(&fit.item_id)
                && seen.insert(fit.item_name.clone())
            {
                alternatives.push(fit.item_name.clone());
                if alternatives.len() >= 3 {
                    break;
                }
            }
        }
        alternatives
    }
}

// ============================================================
// 配置加载器
// ============================================================

/// 装备替代组配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ItemReplacementGroups {
    pub version: String,
    pub groups: std::collections::HashMap<String, ReplacementGroup>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReplacementGroup {
    pub label: String,
    #[serde(default, rename = "itemIds")]
    pub item_ids: Vec<String>,
    #[serde(default, rename = "itemNames")]
    pub item_names: Vec<String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default, rename = "replacesFor", alias = "replaces_for")]
    pub replaces_for: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

impl ReplacementGroup {
    fn configured_items(&self) -> impl Iterator<Item = &String> {
        self.item_ids
            .iter()
            .chain(self.item_names.iter())
            .chain(self.items.iter())
    }
}

/// 装备替代组加载器
pub struct ItemReplacementGroupLoader;

impl ItemReplacementGroupLoader {
    pub fn load(path: &std::path::Path) -> hexsight_core::HexResult<ItemReplacementGroups> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            hexsight_core::HexError::Config(format!("读取替代组配置失败 {}: {}", path.display(), e))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            hexsight_core::HexError::Config(format!("解析替代组配置失败 {}: {}", path.display(), e))
        })
    }
}

/// 棋子装备覆写配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChampionItemOverrides {
    pub version: String,
    pub overrides: std::collections::HashMap<String, ChampionItemOverrideEntry>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChampionItemOverrideEntry {
    #[serde(rename = "itemStrictness")]
    pub item_strictness: Option<String>,
    #[serde(default)]
    pub notes: String,
}

/// 棋子装备覆写加载器
pub struct ChampionItemOverrideLoader;

impl ChampionItemOverrideLoader {
    pub fn load(path: &std::path::Path) -> hexsight_core::HexResult<ChampionItemOverrides> {
        if !path.exists() {
            return Ok(ChampionItemOverrides {
                version: "".into(),
                overrides: Default::default(),
            });
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            hexsight_core::HexError::Config(format!(
                "读取装备覆写配置失败 {}: {}",
                path.display(),
                e
            ))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            hexsight_core::HexError::Config(format!(
                "解析装备覆写配置失败 {}: {}",
                path.display(),
                e
            ))
        })
    }

    /// 应用覆写到棋子能力画像
    pub fn apply(champ: &mut ChampionCapability, overrides: &ChampionItemOverrides) {
        if let Some(ov) = overrides.overrides.get(&champ.hero_id) {
            if let Some(ref strictness) = ov.item_strictness {
                champ.item_strictness = strictness.clone();
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ItemConflictScorer;
    use hexsight_core::{
        ChampionRole, ConflictEntry, ConflictEnvironment, CoverageType, ItemConflictGroup,
        ItemConflictGroups, ItemStat, PowerSpike,
    };

    #[allow(dead_code)]
    fn make_ap_carry() -> ChampionCapability {
        ChampionCapability {
            hero_id: "ap1".into(),
            name: "AP主C".into(),
            cost: 4,
            traits: vec!["sorcerer".into()],
            role: ChampionRole::PrimaryCarry,
            damage_profile: "ap".into(),
            damage_pattern: vec!["burst".into()],
            cast_pattern: "mana_cast".into(),
            scaling_stats: vec!["ability_power".into(), "mana".into()],
            position_role: "backline".into(),
            item_strictness: "high".into(),
            power_spikes: vec![PowerSpike {
                spike_type: "star".into(),
                value: 2,
                impact: 20,
            }],
            risk_profile: vec!["burst_damage".into()],
            item_preferences: Default::default(),
            confidence: 0.8,
            needs_override: false,
        }
    }
    #[allow(dead_code)]
    fn make_ap_item(id: &str, name: &str, has_mana: bool) -> ItemValueProfile {
        let mut effect_tags = vec![];
        let mut stats = vec![ItemStat {
            name: "ap".into(),
            value: 30.0,
        }];
        if has_mana {
            effect_tags.push("mana_engine".into());
            stats.push(ItemStat {
                name: "mana".into(),
                value: 15.0,
            });
        }
        ItemValueProfile {
            item_id: id.into(),
            name: name.into(),
            item_type: "成型装备".into(),
            stats,
            effect_tags,
            best_for_profiles: vec!["ap".into()],
            bad_for_profiles: vec![],
            replacement_group: "".into(),
            conflict_group: "".into(),
            stage_bias: "mid_late".into(),
            tier: None,
            damage_type_fit: vec!["ap".into()],
        }
    }
    #[allow(dead_code)]
    fn make_ad_item(id: &str, name: &str) -> ItemValueProfile {
        ItemValueProfile {
            item_id: id.into(),
            name: name.into(),
            item_type: "成型装备".into(),
            stats: vec![ItemStat {
                name: "ad".into(),
                value: 30.0,
            }],
            effect_tags: vec![],
            best_for_profiles: vec!["ad".into()],
            bad_for_profiles: vec![],
            replacement_group: "".into(),
            conflict_group: "".into(),
            stage_bias: "mid_late".into(),
            tier: None,
            damage_type_fit: vec!["ad".into()],
        }
    }

    #[test]
    fn ap_carry_prefers_ap_items() {
        let champ = make_ap_carry();
        let items = vec![make_ap_item("i1", "法爆", true), make_ad_item("i2", "无尽")];
        let fits = ChampionItemFitScorer::score_all_items(&champ, &items);
        let best = &fits[0];
        assert_eq!(best.item_name, "法爆");
        assert!(best.score > 50);
    }

    #[test]
    fn mana_item_ranks_core_for_mana_cast() {
        let champ = make_ap_carry();
        let items = vec![
            make_ap_item("i1", "青龙刀", true), // AP + mana
            make_ap_item("i2", "帽子", false),  // AP only
        ];
        let fits = ChampionItemFitScorer::score_all_items(&champ, &items);
        assert!(fits[0].score > fits[1].score, "回蓝装应评分更高");
    }

    #[test]
    fn item_gap_detected() {
        let champ = make_ap_carry();
        let all_items = vec![
            make_ap_item("i1", "法爆", true),
            make_ap_item("i2", "帽子", false),
        ];
        let (_fits, impact) = ChampionItemFitScorer::score_with_item_context(
            &champ,
            &all_items,
            &["法爆".into(), "青龙刀".into()],
            &[],
            &[],
            None,
            None,
        );
        assert!(impact.core_missing_penalty > 0);
        assert!(!impact.alternatives.is_empty());
    }

    #[test]
    fn recommended_items_as_strong_prior() {
        let champ = make_ap_carry();
        let items = vec![
            make_ap_item("id_fb", "法爆", true),
            make_ad_item("id_ie", "无尽"),
        ];
        let (fits, _impact) =
            ChampionItemFitScorer::score_with_recommended(&champ, &items, &["id_fb".into()], &[]);
        // 推荐装应排在前面
        assert_eq!(fits[0].item_name, "法爆");
        assert!(_impact.core_missing_penalty >= 0);
    }

    #[test]
    fn replacement_group_drives_gap_alternatives() {
        let champ = make_ap_carry();
        let items = vec![
            make_ap_item("id_shojin", "青龙刀", true),
            make_ap_item("id_blue", "蓝霸符", true),
            make_ap_item("id_archangel", "大天使", false),
            make_ad_item("id_ie", "无尽"),
        ];
        let groups = ItemReplacementGroups {
            version: "test".into(),
            groups: std::collections::HashMap::from([(
                "ap_mana".into(),
                ReplacementGroup {
                    label: "AP回蓝装".into(),
                    item_ids: vec!["id_shojin".into(), "id_blue".into()],
                    item_names: vec!["大天使".into()],
                    items: vec![],
                    replaces_for: vec!["ap_carry_mana".into()],
                    notes: String::new(),
                },
            )]),
        };

        let (_fits, impact) = ChampionItemFitScorer::score_with_item_context(
            &champ,
            &items,
            &["id_shojin".into()],
            &[],
            &[],
            Some(&groups),
            None,
        );

        assert_eq!(impact.alternatives[0], "蓝霸符");
        assert_eq!(impact.alternatives[1], "大天使");
    }

    #[test]
    fn synthesis_components_reduce_missing_core_penalty() {
        let champ = make_ap_carry();
        let items = vec![make_ap_item("id_shojin", "青龙刀", true)];
        let synthesis = ItemSynthesisIndex::from_routes(vec![(
            "id_shojin".to_string(),
            ["bf_sword".to_string(), "tear".to_string()],
        )]);

        let (_fits, impact) = ChampionItemFitScorer::score_with_item_context(
            &champ,
            &items,
            &["id_shojin".into()],
            &[],
            &["bf_sword".into(), "tear".into()],
            None,
            Some(&synthesis),
        );

        assert_eq!(impact.core_missing_penalty, 10);
        assert!(impact.explanation.contains("可合成 1 件"));
    }

    #[test]
    fn conflict_context_lowers_repeated_effect_recommendation() {
        let champ = make_ap_carry();
        let items = vec![
            make_ap_item("2009", "红霸符", false),
            make_ap_item("2020", "莫雷洛秘典", false),
        ];
        let mut groups = HashMap::new();
        groups.insert(
            "anti_heal".into(),
            ItemConflictGroup {
                label: "重伤".into(),
                conflicts: vec![
                    ConflictEntry {
                        item_id: "2009".into(),
                        item_name: "红霸符".into(),
                        coverage: CoverageType::SingleTarget,
                    },
                    ConflictEntry {
                        item_id: "2029".into(),
                        item_name: "日炎斗篷".into(),
                        coverage: CoverageType::AoeAura,
                    },
                ],
                stacking_policy: hexsight_core::ConflictStackingPolicy {
                    first_source: 100,
                    second_source: 30,
                    third_plus_source: 10,
                },
                notes: String::new(),
            },
        );
        let scorer = ItemConflictScorer::new(&ItemConflictGroups {
            version: "test".into(),
            groups,
        });

        let (plain_fits, _) = ChampionItemFitScorer::score_with_item_context(
            &champ,
            &items,
            &["2009".into()],
            &["2029".into()],
            &[],
            None,
            None,
        );
        let (conflict_fits, _) = ChampionItemFitScorer::score_with_item_context_and_conflicts(
            &champ,
            &items,
            &["2009".into()],
            &["2029".into()],
            &[],
            None,
            None,
            Some(&scorer),
            &ConflictEnvironment::default(),
        );

        let plain_red = plain_fits.iter().find(|fit| fit.item_id == "2009").unwrap();
        let conflict_red = conflict_fits
            .iter()
            .find(|fit| fit.item_id == "2009")
            .unwrap();
        assert!(conflict_red.score < plain_red.score);
        assert!(conflict_red
            .penalties
            .iter()
            .any(|penalty| penalty.contains("重伤")));
    }

    #[test]
    fn real_data_item_fit_from_knowledge_base() {
        use crate::{GameDataIndex, KnowledgeBaseBuilder};

        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config");
        let index = GameDataIndex::load(&root, "17").unwrap();
        let kb = KnowledgeBaseBuilder::with_defaults().build_all(
            &index.heroes_by_id.values().cloned().collect::<Vec<_>>(),
            &index.equipment_by_id.values().cloned().collect::<Vec<_>>(),
            &index.hexes_by_id.values().cloned().collect::<Vec<_>>(),
            &index.traits,
        );

        // 找第一个主 C
        if let Some(carry) = kb
            .champions
            .iter()
            .find(|c| matches!(c.role, hexsight_core::ChampionRole::PrimaryCarry))
        {
            let (fits, _impact) =
                ChampionItemFitScorer::score_with_recommended(carry, &kb.items, &[], &[]);
            assert!(!fits.is_empty());
            // 至少有一个 high-score 装备
            assert!(
                fits.iter().any(|f| f.score > 40),
                "主C {} 无任何装备适配分>40",
                carry.name
            );
        }
    }
}
