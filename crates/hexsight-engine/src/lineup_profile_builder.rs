// 阵容规则档案构建器
// 核心职责：
// - 从 LineupCardData + GameDataIndex 生成稳定的 LineupProfile
// - 自动推断玩法标签（赌狗/速8/装备严格/专属依赖等）
// - 提取主C/主坦、核心装备、羁绊目标、过渡棋子

use std::collections::HashMap;

use hexsight_core::{LineupCardData, LineupProfile, PlaystyleTag};

use crate::GameDataIndex;

/// 阵容规则档案构建器
pub struct LineupProfileBuilder;

impl LineupProfileBuilder {
    /// 从阵容卡片和游戏数据构建规则档案
    pub fn build(card: &LineupCardData, index: &GameDataIndex) -> LineupProfile {
        let detail = &card.detail;

        // 提取主 C/主坦
        let carry_ids: Vec<String> = detail
            .final_heroes
            .iter()
            .filter(|p| p.is_carry_hero)
            .map(|p| p.hero_id.clone())
            .collect();
        let tank_ids: Vec<String> = detail
            .final_heroes
            .iter()
            .filter(|p| {
                if p.is_carry_hero {
                    return false;
                }
                // 前排棋子通常 row <= 2
                let hero = index.hero(&p.hero_id);
                let is_frontline = p.row <= 2;
                let has_defense = hero
                    .map(|h| {
                        let armor = h.armor.parse::<i32>().unwrap_or(0);
                        let hp = h.initHP.parse::<i32>().unwrap_or(0);
                        armor > 30 || hp > 700
                    })
                    .unwrap_or(false);
                is_frontline && has_defense
            })
            .map(|p| p.hero_id.clone())
            .collect();

        // 核心装备（主 C 身上的装备）
        let core_equipment_ids: Vec<String> = detail
            .final_heroes
            .iter()
            .filter(|p| p.is_carry_hero)
            .flat_map(|p| p.equipment_ids.clone())
            .collect();

        // 主坦装备（前排坦克身上的装备）
        let tank_equipment_ids: Vec<String> = detail
            .final_heroes
            .iter()
            .filter(|p| !p.is_carry_hero && p.row <= 2 && !p.equipment_ids.is_empty())
            .flat_map(|p| p.equipment_ids.clone())
            .collect();

        // 羁绊目标：优先官方 contacts，否则从棋子反推
        let trait_targets: HashMap<String, i32> = if !detail.official_traits.is_empty() {
            detail
                .official_traits
                .iter()
                .map(|t| (t.id.clone(), t.count))
                .collect()
        } else {
            let summaries = index.trait_summaries(&detail.final_heroes, &[]);
            summaries
                .into_iter()
                .map(|s| (s.trait_id, s.count))
                .collect()
        };

        // 主 C 费用
        let carry_costs: Vec<i32> = carry_ids
            .iter()
            .filter_map(|id| index.hero(id).map(|h| h.cost))
            .collect();

        // 基础强度
        let base_tier = match card.quality.as_str() {
            "S" => 100,
            "A" => 80,
            "B" => 60,
            _ => 40,
        };

        // 策略文本
        let mut strategy_texts = HashMap::new();
        if !detail.early_info.is_empty() {
            strategy_texts.insert("early".into(), detail.early_info.clone());
        }
        if !detail.d_time.is_empty() {
            strategy_texts.insert("roll_timing".into(), detail.d_time.clone());
        }
        if !detail.location_info.is_empty() {
            strategy_texts.insert("positioning".into(), detail.location_info.clone());
        }
        if !detail.equipment_info.is_empty() {
            strategy_texts.insert("equipment".into(), detail.equipment_info.clone());
        }
        if !detail.hex_info.is_empty() {
            strategy_texts.insert("augment".into(), detail.hex_info.clone());
        }

        // 推断玩法标签
        let playstyle_tags = Self::infer_playstyle_tags(
            &carry_costs,
            card.quality.as_str(),
            &core_equipment_ids,
            &detail.recommended_hex_ids,
            card.category.as_deref(),
        );

        LineupProfile {
            lineup_id: card.id.clone(),
            name: card.name.clone(),
            base_tier,
            playstyle_tags,
            final_hero_ids: detail
                .final_heroes
                .iter()
                .map(|p| p.hero_id.clone())
                .collect(),
            carry_hero_ids: carry_ids,
            tank_hero_ids: tank_ids,
            core_equipment_ids,
            tank_equipment_ids,
            equipment_order_ids: detail.equipment_order_ids.clone(),
            recommended_hex_ids: detail.recommended_hex_ids.clone(),
            replacement_hex_ids: detail.replacement_hex_ids.clone(),
            early_hero_ids: detail
                .early_heroes
                .iter()
                .map(|p| p.hero_id.clone())
                .collect(),
            mid_hero_ids: detail
                .mid_heroes
                .iter()
                .map(|p| p.hero_id.clone())
                .collect(),
            trait_targets,
            strategy_texts,
            mode_specific: serde_json::json!({
                "god_rewards": detail.god_rewards,
                "unlock_tasks": detail.unlock_tasks,
                "chosen_contact": detail.chosen_contact,
            }),
            carry_costs,
            category: card.category.clone(),
        }
    }

    /// 批量构建
    pub fn build_all(cards: &[LineupCardData], index: &GameDataIndex) -> Vec<LineupProfile> {
        cards.iter().map(|c| Self::build(c, index)).collect()
    }

    /// 推断玩法标签
    fn infer_playstyle_tags(
        carry_costs: &[i32],
        quality: &str,
        core_equipment_ids: &[String],
        recommended_hex_ids: &[String],
        category: Option<&str>,
    ) -> Vec<PlaystyleTag> {
        let mut tags = Vec::new();

        if carry_costs.is_empty() {
            tags.push(PlaystyleTag::Standard);
            return tags;
        }

        let max_cost = carry_costs.iter().max().copied().unwrap_or(4);
        let min_cost = carry_costs.iter().min().copied().unwrap_or(1);
        let avg_cost = carry_costs.iter().sum::<i32>() as f64 / carry_costs.len() as f64;

        // 常规运营
        tags.push(PlaystyleTag::Standard);

        // 赌狗判断：主C 是低费且需要三星
        if min_cost == 1 && carry_costs.iter().any(|&c| c == 1) {
            tags.push(PlaystyleTag::Reroll1Cost);
        }
        if carry_costs.iter().any(|&c| c == 2) && avg_cost <= 2.5 {
            tags.push(PlaystyleTag::Reroll2Cost);
        }
        if carry_costs.iter().any(|&c| c == 3) && avg_cost <= 3.5 && max_cost <= 4 {
            tags.push(PlaystyleTag::Reroll3Cost);
        }

        // 速 8/速 9：主 C 以 4/5 费为主
        if max_cost >= 4 && min_cost >= 3 {
            tags.push(PlaystyleTag::Fast8);
        }
        if max_cost >= 5 && min_cost >= 4 {
            tags.push(PlaystyleTag::Fast9);
        }

        // 装备判断
        if core_equipment_ids.len() >= 3 {
            tags.push(PlaystyleTag::ItemStrict);
        } else {
            tags.push(PlaystyleTag::ItemFlexible);
        }

        // 经济开局
        if quality == "A" || quality == "B" {
            tags.push(PlaystyleTag::EconOpen);
        }

        // 后期上限
        if max_cost >= 5 || quality == "S" {
            tags.push(PlaystyleTag::LateCap);
        }

        // 专属海克斯依赖
        if !recommended_hex_ids.is_empty() && recommended_hex_ids.len() <= 2 {
            tags.push(PlaystyleTag::AugmentEnabled);
        }

        // 分类标签
        if category == Some("趣味娱乐") {
            tags.push(PlaystyleTag::ModeSpecial);
        }

        tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameDataIndex, LineupAdapter, LineupLoader};
    use std::path::PathBuf;

    fn test_config_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config")
    }

    #[test]
    fn build_profile_for_mode17() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let profiles = LineupProfileBuilder::build_all(&cards, &index);

        assert!(!profiles.is_empty());
        for profile in &profiles {
            assert!(!profile.lineup_id.is_empty());
            assert!(!profile.name.is_empty());
            assert!(profile.base_tier > 0);
            assert!(!profile.playstyle_tags.is_empty());
            assert!(!profile.final_hero_ids.is_empty());
        }
    }

    #[test]
    fn profile_has_carry_and_tank() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        // 找有主 C 标记的阵容
        let card_with_carry = cards
            .iter()
            .find(|c| c.detail.final_heroes.iter().any(|p| p.is_carry_hero));
        if let Some(card) = card_with_carry {
            let profile = LineupProfileBuilder::build(card, &index);
            assert!(
                !profile.carry_hero_ids.is_empty(),
                "有 is_carry_hero 但 carry_hero_ids 为空"
            );
        }
    }

    #[test]
    fn build_profiles_all_modes() {
        for mode in &["17", "16", "4"] {
            let index = GameDataIndex::load(&test_config_root(), mode).unwrap();
            let raw = LineupLoader::load_cached_lineups(&test_config_root(), mode, "S18").unwrap();
            let cards = LineupAdapter::cards_from_raw_list(&raw, mode);
            let profiles = LineupProfileBuilder::build_all(&cards, &index);
            assert!(!profiles.is_empty(), "mode {} 无 profiles", mode);
        }
    }

    #[test]
    fn playstyle_tags_inferred() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let profile = &LineupProfileBuilder::build_all(&cards, &index)[0];
        // 至少要有 Standard 标签
        assert!(profile.playstyle_tags.contains(&PlaystyleTag::Standard));
    }
}
