// 备战席转移评分器
// 核心职责：
// - 根据目标阵容主 C / 主坦先验定位最终承载者
// - 计算当前临时承载者的卖出成本
// - 判断装备应立即给当前棋子还是等待目标棋子

use std::collections::HashMap;

use hexsight_core::{ChampionCapability, ChampionRole, LineupProfile};

use crate::holder_scorer::{HolderRole, HolderUnit};

/// 装备转移判断结果
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BenchTransitionDecision {
    pub item_id: String,
    pub final_holder_id: String,
    pub final_holder_name: String,
    pub current_sell_cost: i32,
    pub should_wait_for_target: bool,
    pub reasons: Vec<String>,
}

/// 备战席转移评分器
pub struct BenchTransitionScorer;

impl BenchTransitionScorer {
    pub fn score(
        item_id: &str,
        board_units: &[HolderUnit],
        bench_units: &[HolderUnit],
        target_lineups: &[LineupProfile],
        champions: &[ChampionCapability],
        holder_role: HolderRole,
    ) -> Option<BenchTransitionDecision> {
        let champion_by_id: HashMap<&str, &ChampionCapability> = champions
            .iter()
            .map(|champion| (champion.hero_id.as_str(), champion))
            .collect();
        let final_holder_id = Self::final_holder_id(item_id, target_lineups, holder_role)?;
        let final_holder = champion_by_id.get(final_holder_id.as_str())?;
        let current = board_units
            .iter()
            .max_by_key(|unit| Self::unit_transfer_cost(unit, &champion_by_id));
        let current_sell_cost = current
            .map(|unit| Self::unit_transfer_cost(unit, &champion_by_id))
            .unwrap_or(0);
        let target_on_bench = bench_units
            .iter()
            .any(|unit| unit.hero_id == final_holder_id);
        let target_on_board = board_units
            .iter()
            .any(|unit| unit.hero_id == final_holder_id);
        let current_is_low_value = current
            .map(|unit| unit.star_level <= 1 && current_sell_cost <= 2)
            .unwrap_or(true);
        let should_wait_for_target = target_on_bench || target_on_board || current_is_low_value;

        let mut reasons = Vec::new();
        if target_on_bench {
            reasons.push("目标主C已在备战席，保留装备给最终承载者".into());
        } else if target_on_board {
            reasons.push("目标承载者已上场，优先直接给目标棋子".into());
        } else if current_is_low_value {
            reasons.push("当前临时承载者星级低，等待目标棋子收益更高".into());
        } else {
            reasons.push("目标棋子未到，当前承载者可先稳血".into());
        }

        Some(BenchTransitionDecision {
            item_id: item_id.into(),
            final_holder_id,
            final_holder_name: final_holder.name.clone(),
            current_sell_cost,
            should_wait_for_target,
            reasons,
        })
    }

    pub fn score_for_candidate(
        item_id: &str,
        candidate_unit: &HolderUnit,
        board_units: &[HolderUnit],
        bench_units: &[HolderUnit],
        target_lineups: &[LineupProfile],
        champions: &[ChampionCapability],
        holder_role: HolderRole,
    ) -> Option<BenchTransitionDecision> {
        let champion_by_id: HashMap<&str, &ChampionCapability> = champions
            .iter()
            .map(|champion| (champion.hero_id.as_str(), champion))
            .collect();
        let final_holder_id = Self::final_holder_id(item_id, target_lineups, holder_role)?;
        let final_holder = champion_by_id.get(final_holder_id.as_str())?;
        let current_sell_cost = Self::unit_transfer_cost(candidate_unit, &champion_by_id);
        let target_on_bench = bench_units
            .iter()
            .any(|unit| unit.hero_id == final_holder_id);
        let target_on_board = board_units
            .iter()
            .any(|unit| unit.hero_id == final_holder_id);
        let current_is_low_value = candidate_unit.star_level <= 1 && current_sell_cost <= 2;
        let should_wait_for_target = target_on_bench || target_on_board || current_is_low_value;

        let mut reasons = Vec::new();
        if target_on_bench {
            reasons.push("目标主C已在备战席，保留装备给最终承载者".into());
        } else if target_on_board {
            reasons.push("目标承载者已上场，优先直接给目标棋子".into());
        } else if current_is_low_value {
            reasons.push("当前临时承载者星级低，等待目标棋子收益更高".into());
        } else {
            reasons.push("目标棋子未到，当前承载者可先稳血".into());
        }

        Some(BenchTransitionDecision {
            item_id: item_id.into(),
            final_holder_id,
            final_holder_name: final_holder.name.clone(),
            current_sell_cost,
            should_wait_for_target,
            reasons,
        })
    }

    pub(crate) fn final_holder_id(
        _item_id: &str,
        target_lineups: &[LineupProfile],
        holder_role: HolderRole,
    ) -> Option<String> {
        for profile in target_lineups {
            let preferred = match holder_role {
                HolderRole::Carry => &profile.carry_hero_ids,
                HolderRole::Tank => &profile.tank_hero_ids,
                HolderRole::Utility => &profile.final_hero_ids,
            };
            if let Some(hero_id) = preferred.first() {
                return Some(hero_id.clone());
            }
        }
        None
    }

    pub(crate) fn unit_transfer_cost(
        unit: &HolderUnit,
        champion_by_id: &HashMap<&str, &ChampionCapability>,
    ) -> i32 {
        let base_cost = champion_by_id
            .get(unit.hero_id.as_str())
            .map(|champion| champion.cost.max(1))
            .unwrap_or(1);
        base_cost * star_multiplier(unit.star_level)
    }
}

pub(crate) fn infer_holder_role(
    item_id: &str,
    item_tags: &[String],
    item_damage_fit: &[String],
    target_lineups: &[LineupProfile],
) -> HolderRole {
    if target_lineups
        .iter()
        .any(|profile| profile.tank_equipment_ids.iter().any(|id| id == item_id))
        || item_damage_fit.iter().any(|fit| fit == "tank")
        || item_tags.iter().any(|tag| {
            matches!(
                tag.as_str(),
                "shield_survival" | "sustain" | "damage_reduction" | "aura_team_buff"
            )
        })
    {
        return HolderRole::Tank;
    }
    if target_lineups
        .iter()
        .any(|profile| profile.core_equipment_ids.iter().any(|id| id == item_id))
        || item_damage_fit.iter().any(|fit| fit == "ad" || fit == "ap")
    {
        return HolderRole::Carry;
    }
    HolderRole::Utility
}

pub(crate) fn role_matches_holder(champion: &ChampionCapability, role: HolderRole) -> bool {
    match role {
        HolderRole::Carry => {
            matches!(
                champion.role,
                ChampionRole::PrimaryCarry | ChampionRole::SecondaryCarry
            ) || champion.position_role == "backline"
        }
        HolderRole::Tank => {
            matches!(
                champion.role,
                ChampionRole::MainTank | ChampionRole::OffTank
            ) || champion.position_role == "frontline"
                || champion.damage_profile == "tank"
        }
        HolderRole::Utility => true,
    }
}

fn star_multiplier(star_level: i32) -> i32 {
    match star_level {
        level if level <= 1 => 1,
        2 => 3,
        _ => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holder_scorer::HolderUnitSource;
    use hexsight_core::{ItemPreferences, PlaystyleTag, PowerSpike};

    fn champion(hero_id: &str, cost: i32) -> ChampionCapability {
        ChampionCapability {
            hero_id: hero_id.into(),
            name: hero_id.into(),
            cost,
            traits: vec![],
            role: ChampionRole::PrimaryCarry,
            damage_profile: "ad".into(),
            damage_pattern: vec![],
            cast_pattern: "attack_based".into(),
            scaling_stats: vec!["attack_damage".into()],
            position_role: "backline".into(),
            item_strictness: "medium".into(),
            power_spikes: vec![PowerSpike {
                spike_type: "star".into(),
                value: 2,
                impact: 20,
            }],
            risk_profile: vec![],
            item_preferences: ItemPreferences::default(),
            confidence: 1.0,
            needs_override: false,
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
            core_equipment_ids: vec!["item".into()],
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
    fn two_star_one_cost_sell_cost_is_three() {
        let champions = vec![champion("early", 1), champion("late", 4)];
        let current = HolderUnit::new("early", 2, HolderUnitSource::Board);
        let target = HolderUnit::new("late", 1, HolderUnitSource::Bench);
        let decision = BenchTransitionScorer::score(
            "item",
            &[current],
            &[target],
            &[profile()],
            &champions,
            HolderRole::Carry,
        )
        .unwrap();

        assert_eq!(decision.current_sell_cost, 3);
        assert!(decision.should_wait_for_target);
    }
}
