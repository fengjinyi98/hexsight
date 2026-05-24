// 打工承载者评分器
// 核心职责：
// - 根据当前棋盘、备战席、装备席推荐临时装备承载者
// - 区分打工 C、打工坦克和功能承载者
// - 输出短操作建议、最终转移对象和卖出成本

use std::collections::{HashMap, HashSet};
use std::path::Path;

use hexsight_core::{ChampionCapability, EquipmentData, ItemValueProfile, LineupProfile};

use crate::bench_transition_scorer::{
    infer_holder_role, role_matches_holder, BenchTransitionScorer,
};
use crate::champion_item_fit::{ChampionItemFitScorer, ItemSynthesisIndex};

/// 承载者来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HolderUnitSource {
    #[serde(rename = "board")]
    Board,
    #[serde(rename = "bench")]
    Bench,
}

/// 当前棋子快照
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HolderUnit {
    pub hero_id: String,
    pub star_level: i32,
    pub source: HolderUnitSource,
    #[serde(default)]
    pub item_ids: Vec<String>,
    #[serde(default)]
    pub active_trait_ids: Vec<String>,
}

impl HolderUnit {
    pub fn new(hero_id: &str, star_level: i32, source: HolderUnitSource) -> Self {
        Self {
            hero_id: hero_id.into(),
            star_level,
            source,
            item_ids: vec![],
            active_trait_ids: vec![],
        }
    }
}

/// 承载者定位
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HolderRole {
    #[serde(rename = "carry")]
    Carry,
    #[serde(rename = "tank")]
    Tank,
    #[serde(rename = "utility")]
    Utility,
}

/// 装备承载动作
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HolderAction {
    #[serde(rename = "equip_now")]
    EquipNow,
    #[serde(rename = "wait_for_target")]
    WaitForTarget,
}

/// 打工承载者评分输入
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HolderScoringContext {
    pub board_units: Vec<HolderUnit>,
    pub bench_units: Vec<HolderUnit>,
    pub completed_item_ids: Vec<String>,
    pub component_item_ids: Vec<String>,
    pub active_trait_ids: Vec<String>,
    pub target_lineups: Vec<LineupProfile>,
}

/// 单件装备承载建议
#[derive(Debug, Clone, serde::Serialize)]
pub struct HolderRecommendation {
    pub item_id: String,
    pub item_name: String,
    pub temporary_holder_id: String,
    pub temporary_holder_name: String,
    pub holder_role: HolderRole,
    pub final_holder_id: Option<String>,
    pub final_holder_name: Option<String>,
    pub score: i32,
    pub sell_cost: i32,
    pub action: HolderAction,
    pub reasons: Vec<String>,
}

/// 承载者推荐方案
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct HolderPlan {
    pub best: Option<HolderRecommendation>,
    pub alternatives: Vec<HolderRecommendation>,
    pub summary: String,
}

/// 承载者评分规则
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HolderRules {
    pub version: String,
    #[serde(default)]
    pub two_star_bonus: i32,
    #[serde(default)]
    pub board_bonus: i32,
    #[serde(default)]
    pub role_match_bonus: i32,
    #[serde(default)]
    pub trait_match_bonus: i32,
    #[serde(default)]
    pub wait_score_gap: i32,
    #[serde(default)]
    pub sell_cost_penalty_per_gold: i32,
}

impl Default for HolderRules {
    fn default() -> Self {
        Self {
            version: "default".into(),
            two_star_bonus: 18,
            board_bonus: 8,
            role_match_bonus: 16,
            trait_match_bonus: 4,
            wait_score_gap: 12,
            sell_cost_penalty_per_gold: 2,
        }
    }
}

/// 打工棋子强度覆写
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkhorseOverrides {
    pub version: String,
    #[serde(default)]
    pub overrides: HashMap<String, WorkhorseOverride>,
}

/// 单棋子打工强度覆写
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkhorseOverride {
    #[serde(default)]
    pub bonus: i32,
    #[serde(default)]
    pub roles: Vec<HolderRole>,
    #[serde(default)]
    pub notes: String,
}

/// P3 配置加载器
pub struct HolderConfigLoader;

impl HolderConfigLoader {
    pub fn load_rules(path: &Path) -> hexsight_core::HexResult<HolderRules> {
        if !path.exists() {
            return Ok(HolderRules::default());
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            hexsight_core::HexError::Config(format!(
                "读取承载者规则配置失败 {}: {}",
                path.display(),
                e
            ))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            hexsight_core::HexError::Config(format!(
                "解析承载者规则配置失败 {}: {}",
                path.display(),
                e
            ))
        })
    }

    pub fn load_workhorse_overrides(path: &Path) -> hexsight_core::HexResult<WorkhorseOverrides> {
        if !path.exists() {
            return Ok(WorkhorseOverrides {
                version: String::new(),
                overrides: HashMap::new(),
            });
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            hexsight_core::HexError::Config(format!(
                "读取打工棋子覆写配置失败 {}: {}",
                path.display(),
                e
            ))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            hexsight_core::HexError::Config(format!(
                "解析打工棋子覆写配置失败 {}: {}",
                path.display(),
                e
            ))
        })
    }
}

/// 打工承载者评分器
pub struct HolderScorer;

impl HolderScorer {
    pub fn score(
        context: &HolderScoringContext,
        champions: &[ChampionCapability],
        items: &[ItemValueProfile],
    ) -> HolderPlan {
        Self::score_with_rules_and_synthesis(
            context,
            champions,
            items,
            &HolderRules::default(),
            &WorkhorseOverrides {
                version: String::new(),
                overrides: HashMap::new(),
            },
            None,
        )
    }

    pub fn score_with_synthesis_index(
        context: &HolderScoringContext,
        champions: &[ChampionCapability],
        items: &[ItemValueProfile],
        synthesis_index: &ItemSynthesisIndex,
    ) -> HolderPlan {
        Self::score_with_rules_and_synthesis(
            context,
            champions,
            items,
            &HolderRules::default(),
            &WorkhorseOverrides {
                version: String::new(),
                overrides: HashMap::new(),
            },
            Some(synthesis_index),
        )
    }

    pub fn score_from_rule_config(
        context: &HolderScoringContext,
        champions: &[ChampionCapability],
        items: &[ItemValueProfile],
        config_root: &Path,
        version: &str,
    ) -> hexsight_core::HexResult<HolderPlan> {
        let rules_dir = config_root.join("rules").join(version);
        let rules = HolderConfigLoader::load_rules(&rules_dir.join("holder_rules.json"))?;
        let overrides = HolderConfigLoader::load_workhorse_overrides(
            &rules_dir.join("workhorse_overrides.json"),
        )?;
        Ok(Self::score_with_rules_and_synthesis(
            context, champions, items, &rules, &overrides, None,
        ))
    }

    pub fn score_from_rule_config_with_equipment(
        context: &HolderScoringContext,
        champions: &[ChampionCapability],
        items: &[ItemValueProfile],
        equipment: &[EquipmentData],
        config_root: &Path,
        version: &str,
    ) -> hexsight_core::HexResult<HolderPlan> {
        let rules_dir = config_root.join("rules").join(version);
        let rules = HolderConfigLoader::load_rules(&rules_dir.join("holder_rules.json"))?;
        let overrides = HolderConfigLoader::load_workhorse_overrides(
            &rules_dir.join("workhorse_overrides.json"),
        )?;
        let synthesis_index = ItemSynthesisIndex::from_equipment(equipment);
        Ok(Self::score_with_rules_and_synthesis(
            context,
            champions,
            items,
            &rules,
            &overrides,
            Some(&synthesis_index),
        ))
    }

    pub fn score_with_rules(
        context: &HolderScoringContext,
        champions: &[ChampionCapability],
        items: &[ItemValueProfile],
        rules: &HolderRules,
        overrides: &WorkhorseOverrides,
    ) -> HolderPlan {
        Self::score_with_rules_and_synthesis(context, champions, items, rules, overrides, None)
    }

    fn score_with_rules_and_synthesis(
        context: &HolderScoringContext,
        champions: &[ChampionCapability],
        items: &[ItemValueProfile],
        rules: &HolderRules,
        overrides: &WorkhorseOverrides,
        synthesis_index: Option<&ItemSynthesisIndex>,
    ) -> HolderPlan {
        let champion_by_id: HashMap<&str, &ChampionCapability> = champions
            .iter()
            .map(|champion| (champion.hero_id.as_str(), champion))
            .collect();
        let item_by_id: HashMap<&str, &ItemValueProfile> = items
            .iter()
            .map(|item| (item.item_id.as_str(), item))
            .collect();

        let mut recommendations = Vec::new();
        let completed_item_ids: HashSet<&str> = context
            .completed_item_ids
            .iter()
            .map(String::as_str)
            .collect();
        let mut candidate_items: Vec<(&ItemValueProfile, bool)> = Vec::new();
        for item_id in &context.completed_item_ids {
            let Some(item) = item_by_id.get(item_id.as_str()) else {
                continue;
            };
            candidate_items.push((item, false));
        }
        if let Some(synthesis_index) = synthesis_index {
            for item in items {
                if completed_item_ids.contains(item.item_id.as_str()) {
                    continue;
                }
                if item.item_type != "成型装备" {
                    continue;
                }
                if synthesis_index.can_synthesize(&item.item_id, &context.component_item_ids) {
                    candidate_items.push((item, true));
                }
            }
        }

        for (item, synthesized_from_components) in candidate_items {
            let holder_role = infer_holder_role(
                &item.item_id,
                &item.effect_tags,
                &item.damage_type_fit,
                &context.target_lineups,
            );
            recommendations.extend(Self::candidates_for_item(
                item,
                holder_role,
                context,
                &champion_by_id,
                champions,
                rules,
                overrides,
                synthesized_from_components,
            ));
        }

        recommendations.sort_by(|a, b| b.score.cmp(&a.score));
        let best = recommendations.first().cloned();
        let alternatives = recommendations.iter().skip(1).take(3).cloned().collect();
        let summary = best
            .as_ref()
            .map(Self::summary_for)
            .unwrap_or_else(|| "暂无可用承载者".into());
        HolderPlan {
            best,
            alternatives,
            summary,
        }
    }

    fn candidates_for_item(
        item: &ItemValueProfile,
        holder_role: HolderRole,
        context: &HolderScoringContext,
        champion_by_id: &HashMap<&str, &ChampionCapability>,
        champions: &[ChampionCapability],
        rules: &HolderRules,
        overrides: &WorkhorseOverrides,
        synthesized_from_components: bool,
    ) -> Vec<HolderRecommendation> {
        let active_traits: HashSet<&str> = context
            .active_trait_ids
            .iter()
            .map(String::as_str)
            .collect();
        let mut candidates = Vec::new();
        for unit in context.board_units.iter().chain(context.bench_units.iter()) {
            if unit.item_ids.len() >= 3 {
                continue;
            }
            let Some(champion) = champion_by_id.get(unit.hero_id.as_str()) else {
                continue;
            };
            let fit_score = ChampionItemFitScorer::score_item(champion, item);
            let mut score = fit_score;
            let mut reasons = vec![format!("{}适配{}", champion.name, item.name)];
            if synthesized_from_components {
                reasons.push("装备席散件可合成".into());
            }

            if unit.star_level >= 2 {
                score += rules.two_star_bonus;
                reasons.push("二星质量可先稳血".into());
            }
            if unit.source == HolderUnitSource::Board {
                score += rules.board_bonus;
                reasons.push("当前已上场".into());
            } else {
                reasons.push("备战席可上场承载".into());
            }
            if role_matches_holder(champion, holder_role) {
                score += rules.role_match_bonus;
                reasons.push(match holder_role {
                    HolderRole::Carry => "输出定位匹配".into(),
                    HolderRole::Tank => "前排定位匹配".into(),
                    HolderRole::Utility => "功能定位可承载".into(),
                });
            }
            let matched_traits = champion
                .traits
                .iter()
                .filter(|trait_id| active_traits.contains(trait_id.as_str()))
                .count() as i32;
            if matched_traits > 0 {
                score += matched_traits * rules.trait_match_bonus;
                reasons.push("当前羁绊有贡献".into());
            }
            if let Some(override_entry) = overrides.overrides.get(&unit.hero_id) {
                let role_allowed =
                    override_entry.roles.is_empty() || override_entry.roles.contains(&holder_role);
                if role_allowed {
                    score += override_entry.bonus;
                    if !override_entry.notes.is_empty() {
                        reasons.push(override_entry.notes.clone());
                    }
                }
            }

            let sell_cost = BenchTransitionScorer::unit_transfer_cost(unit, champion_by_id);
            score -= sell_cost * rules.sell_cost_penalty_per_gold;

            let mut final_holder_id = None;
            let mut final_holder_name = None;
            let mut action = HolderAction::EquipNow;
            if let Some(decision) = BenchTransitionScorer::score_for_candidate(
                &item.item_id,
                unit,
                &context.board_units,
                &context.bench_units,
                &context.target_lineups,
                champions,
                holder_role,
            ) {
                final_holder_id = Some(decision.final_holder_id.clone());
                final_holder_name = Some(decision.final_holder_name.clone());
                let target_available = context
                    .bench_units
                    .iter()
                    .chain(context.board_units.iter())
                    .any(|candidate| candidate.hero_id == decision.final_holder_id);
                if target_available && unit.hero_id != decision.final_holder_id {
                    let target_score = champion_by_id
                        .get(decision.final_holder_id.as_str())
                        .map(|target_champion| {
                            ChampionItemFitScorer::score_item(target_champion, item)
                        })
                        .unwrap_or(0);
                    if unit.star_level <= 1 || target_score + rules.wait_score_gap >= score {
                        action = HolderAction::WaitForTarget;
                        reasons.extend(decision.reasons);
                    }
                }
            }

            candidates.push(HolderRecommendation {
                item_id: item.item_id.clone(),
                item_name: item.name.clone(),
                temporary_holder_id: unit.hero_id.clone(),
                temporary_holder_name: champion.name.clone(),
                holder_role,
                final_holder_id,
                final_holder_name,
                score: score.clamp(0, 140),
                sell_cost,
                action,
                reasons,
            });
        }

        candidates.sort_by(|a, b| b.score.cmp(&a.score));
        for candidate in &mut candidates {
            if candidate.action == HolderAction::WaitForTarget {
                if let Some(target_id) = &candidate.final_holder_id {
                    if let Some(target_champion) = champion_by_id.get(target_id.as_str()) {
                        candidate.temporary_holder_id = target_champion.hero_id.clone();
                        candidate.temporary_holder_name = target_champion.name.clone();
                        candidate.sell_cost = 0;
                    }
                }
            }
        }
        candidates
    }

    fn summary_for(recommendation: &HolderRecommendation) -> String {
        let follow_up = recommendation
            .final_holder_name
            .as_ref()
            .map(|name| format!("，后续给{}", name))
            .unwrap_or_default();
        match recommendation.action {
            HolderAction::EquipNow => format!(
                "合{}，给{}{}",
                recommendation.item_name, recommendation.temporary_holder_name, follow_up
            ),
            HolderAction::WaitForTarget => format!(
                "先留{}，等{}",
                recommendation.item_name,
                recommendation
                    .final_holder_name
                    .as_deref()
                    .unwrap_or(recommendation.temporary_holder_name.as_str())
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexsight_core::{ChampionRole, ItemStat, PlaystyleTag, PowerSpike};

    fn champion(hero_id: &str, role: ChampionRole, position: &str) -> ChampionCapability {
        ChampionCapability {
            hero_id: hero_id.into(),
            name: hero_id.into(),
            cost: 1,
            traits: vec![],
            role,
            damage_profile: "ad".into(),
            damage_pattern: vec![],
            cast_pattern: "attack_based".into(),
            scaling_stats: vec!["attack_damage".into()],
            position_role: position.into(),
            item_strictness: "medium".into(),
            power_spikes: vec![PowerSpike {
                spike_type: "star".into(),
                value: 2,
                impact: 20,
            }],
            risk_profile: vec![],
            item_preferences: Default::default(),
            confidence: 1.0,
            needs_override: false,
        }
    }

    fn ad_item() -> ItemValueProfile {
        ItemValueProfile {
            item_id: "ad_item".into(),
            name: "物理装".into(),
            item_type: "成型装备".into(),
            stats: vec![ItemStat {
                name: "ad".into(),
                value: 30.0,
            }],
            effect_tags: vec![],
            best_for_profiles: vec!["ad".into()],
            bad_for_profiles: vec![],
            replacement_group: String::new(),
            conflict_group: String::new(),
            stage_bias: "early_mid".into(),
            tier: None,
            damage_type_fit: vec!["ad".into()],
        }
    }

    fn profile() -> LineupProfile {
        LineupProfile {
            lineup_id: "p".into(),
            name: "p".into(),
            base_tier: 100,
            playstyle_tags: vec![PlaystyleTag::Tempo],
            final_hero_ids: vec!["late".into()],
            carry_hero_ids: vec!["late".into()],
            tank_hero_ids: vec![],
            core_equipment_ids: vec!["ad_item".into()],
            tank_equipment_ids: vec![],
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
    fn score_prefers_two_star_board_carry() {
        let champions = vec![
            champion("early", ChampionRole::SecondaryCarry, "backline"),
            champion("late", ChampionRole::PrimaryCarry, "backline"),
        ];
        let context = HolderScoringContext {
            board_units: vec![HolderUnit::new("early", 2, HolderUnitSource::Board)],
            bench_units: vec![],
            completed_item_ids: vec!["ad_item".into()],
            component_item_ids: vec![],
            active_trait_ids: vec![],
            target_lineups: vec![profile()],
        };

        let plan = HolderScorer::score(&context, &champions, &[ad_item()]);

        assert_eq!(plan.best.unwrap().temporary_holder_id, "early");
    }
}
