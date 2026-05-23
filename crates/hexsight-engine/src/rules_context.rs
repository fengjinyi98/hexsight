// 规则上下文构建器
// 核心职责：
// - 将 LineupCardData + GameDataIndex 整理为规则引擎和 LLM 可消费的稳定输入
// - 输出对齐目标文档第七章 JSON Schema
// - 避免规则层直接依赖 UI 展示状态

use std::collections::HashMap;

use hexsight_core::{
    AugmentsContext, EquipmentContext, EquipRef, HexRef, LineupCardData, LineupContextInfo,
    ModeContextInfo, ModeSpecificContext, RuleEquipmentSnapshot, RuleHeroSnapshot,
    RuleHexSnapshot, RulesContextOutput,
};

use crate::GameDataIndex;

/// 支持的模式配置表
const SUPPORTED_MODES: &[(&str, &str, &str, &str, &[&str])] = &[
    ("17", "星神", "S18", "m18", &["standardLineup", "godRewards", "transitionContacts"]),
    ("16", "英雄联盟传奇", "S18", "m17", &["standardLineup", "unlockTasks", "transitionContacts"]),
    ("4", "天选福星", "S18", "m17", &["standardLineup", "chosenMechanics", "transitionContacts"]),
];

/// 规则上下文构建器
pub struct RulesContextBuilder;

impl RulesContextBuilder {
    /// 构建完整的 RulesContextOutput
    pub fn build(
        card: &LineupCardData,
        mode: &str,
        game_data: Option<&GameDataIndex>,
    ) -> RulesContextOutput {
        let detail = &card.detail;

        // 解析英雄、装备、符文
        let resolved_heroes = Self::resolve_heroes(&detail.final_heroes, game_data);
        let resolved_equip_order = Self::resolve_equipment_order(&detail.equipment_order_ids, game_data);
        let resolved_recommended_hexes = Self::resolve_hexes(&detail.recommended_hex_ids, game_data);
        let resolved_replacement_hexes = Self::resolve_hexes(&detail.replacement_hex_ids, game_data);

        // 羁绊
        let resolved_traits = game_data
            .map(|gd| gd.trait_summaries(&detail.final_heroes, &detail.official_traits))
            .unwrap_or_default();

        // 强化符文
        let augments = AugmentsContext {
            recommended: resolved_recommended_hexes.iter().map(|h| HexRef {
                id: h.id.clone(), name: h.name.clone(), level: h.level,
            }).collect(),
            replacement: resolved_replacement_hexes.iter().map(|h| HexRef {
                id: h.id.clone(), name: h.name.clone(), level: h.level,
            }).collect(),
        };

        // 装备顺序
        let equipment = EquipmentContext {
            order: resolved_equip_order.iter().map(|e| EquipRef {
                id: e.id.clone(), name: e.name.clone(),
            }).collect(),
        };

        // 模式特有上下文
        let mode_specific = ModeSpecificContext {
            god_rewards: detail.god_rewards.iter().map(|g| {
                serde_json::json!({
                    "stage": g.stage, "god_id": g.god_id, "wish_ids": g.wish_ids,
                })
            }).collect(),
            unlock_tasks: detail.unlock_tasks.iter().map(|t| {
                serde_json::json!({
                    "task_id": t.task_id, "hero_id": t.hero_id,
                })
            }).collect(),
            chosen: detail.chosen_contact.as_ref().map(|c| {
                serde_json::json!({ "id": c.id, "type": c.contact_type })
            }),
        };

        // 策略文本
        let mut strategy_texts = HashMap::new();
        insert_if_not_empty(&mut strategy_texts, "earlyInfo", &detail.early_info);
        insert_if_not_empty(&mut strategy_texts, "dTime", &detail.d_time);
        insert_if_not_empty(&mut strategy_texts, "locationInfo", &detail.location_info);
        insert_if_not_empty(&mut strategy_texts, "enemyInfo", &detail.enemy_info);
        insert_if_not_empty(&mut strategy_texts, "hexInfo", &detail.hex_info);
        insert_if_not_empty(&mut strategy_texts, "equipmentInfo", &detail.equipment_info);

        RulesContextOutput {
            mode: Self::mode_info(mode),
            lineup: LineupContextInfo {
                id: card.id.clone(),
                name: card.name.clone(),
                author: card.author.clone(),
                quality: card.quality.clone(),
                tags: card.tags.clone(),
            },
            final_heroes: resolved_heroes,
            traits: resolved_traits,
            augments,
            equipment,
            mode_specific,
            strategy_texts,
        }
    }

    /// 获取模式信息
    pub fn mode_info(mode: &str) -> ModeContextInfo {
        SUPPORTED_MODES.iter()
            .find(|(id, _, _, _, _)| *id == mode)
            .map(|(id, name, season, _version_path, capabilities)| ModeContextInfo {
                id: id.to_string(),
                name: name.to_string(),
                season: season.to_string(),
                capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
            })
            .unwrap_or_else(|| ModeContextInfo {
                id: mode.to_string(),
                name: "未知模式".to_string(),
                season: "S18".to_string(),
                capabilities: vec![],
            })
    }

    /// 获取所有支持的模式列表
    pub fn supported_modes() -> Vec<ModeContextInfo> {
        SUPPORTED_MODES.iter()
            .map(|(id, name, season, _version_path, capabilities)| ModeContextInfo {
                id: id.to_string(),
                name: name.to_string(),
                season: season.to_string(),
                capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
            })
            .collect()
    }

    fn resolve_heroes(
        pieces: &[hexsight_core::LineupPieceData],
        game_data: Option<&GameDataIndex>,
    ) -> Vec<RuleHeroSnapshot> {
        pieces.iter().map(|piece| {
            let (name, cost, picture) = if let Some(gd) = game_data {
                let hero = gd.hero(&piece.hero_id);
                (hero.map(|h| h.name.clone()).unwrap_or_else(|| piece.hero_id.clone()),
                 hero.map(|h| h.cost).unwrap_or(0),
                 hero.map(|h| h.picture.clone()).unwrap_or_default())
            } else {
                (piece.hero_id.clone(), 0, String::new())
            };
            let equipment_names: Vec<String> = piece.equipment_ids.iter().map(|eid| {
                game_data.map(|gd| gd.equip_name(eid).to_string())
                    .unwrap_or_else(|| eid.clone())
            }).collect();

            RuleHeroSnapshot {
                id: piece.hero_id.clone(),
                name,
                cost,
                picture,
                position: piece.location_key.clone(),
                is_carry: piece.is_carry_hero,
                equipment_ids: piece.equipment_ids.clone(),
                equipment_names,
            }
        }).collect()
    }

    fn resolve_equipment_order(
        ids: &[String],
        game_data: Option<&GameDataIndex>,
    ) -> Vec<RuleEquipmentSnapshot> {
        ids.iter().map(|id| {
            let (name, equip_type, picture) = if let Some(gd) = game_data {
                let equip = gd.equipment(id);
                (equip.map(|e| e.name.clone()).unwrap_or_else(|| id.clone()),
                 equip.map(|e| e.equip_type.clone()).unwrap_or_default(),
                 equip.map(|e| e.picture.clone()).unwrap_or_default())
            } else {
                (id.clone(), String::new(), String::new())
            };
            RuleEquipmentSnapshot { id: id.clone(), name, equip_type, picture }
        }).collect()
    }

    fn resolve_hexes(
        ids: &[String],
        game_data: Option<&GameDataIndex>,
    ) -> Vec<RuleHexSnapshot> {
        ids.iter().map(|id| {
            let (name, level, desc, icon) = if let Some(gd) = game_data {
                let hex = gd.hex(id);
                (hex.map(|h| h.name.clone()).unwrap_or_else(|| id.clone()),
                 hex.and_then(|h| h.level.parse().ok()).unwrap_or(0),
                 hex.map(|h| h.desc.clone()).unwrap_or_default(),
                 hex.map(|h| h.icon.clone()).unwrap_or_default())
            } else {
                (id.clone(), 0, String::new(), String::new())
            };
            RuleHexSnapshot { id: id.clone(), name, level, desc, icon }
        }).collect()
    }
}

fn insert_if_not_empty(map: &mut HashMap<String, String>, key: &str, value: &str) {
    if !value.trim().is_empty() {
        map.insert(key.to_string(), value.to_string());
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::{GameDataIndex, LineupAdapter, LineupLoader};

    fn test_config_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap().join("config")
    }

    #[test]
    fn build_context_mode17_snapshot() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let card = cards.iter().find(|c| c.name.contains("神谕龙王")).unwrap();
        let ctx = RulesContextBuilder::build(card, "17", Some(&index));

        assert_eq!(ctx.mode.name, "星神");
        assert!(ctx.mode.capabilities.contains(&"godRewards".to_string()));
        assert!(!ctx.final_heroes.is_empty());
        assert!(ctx.final_heroes.iter().any(|h| h.name == "超级机甲"));
        assert!(!ctx.traits.is_empty());
        assert!(ctx.traits.iter().any(|t| t.name == "牧羊人"));
        assert!(!ctx.equipment.order.is_empty());
    }

    #[test]
    fn build_context_all_modes() {
        for mode in &["17", "16", "4"] {
            let index = GameDataIndex::load(&test_config_root(), mode).unwrap();
            let raw = LineupLoader::load_cached_lineups(&test_config_root(), mode, "S18").unwrap();
            let cards = LineupAdapter::cards_from_raw_list(&raw, mode);
            assert!(!cards.is_empty());
            let ctx = RulesContextBuilder::build(&cards[0], mode, Some(&index));
            assert!(!ctx.final_heroes.is_empty());
            assert!(!ctx.lineup.name.is_empty());
        }
    }

    #[test]
    fn json_output_valid() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let ctx = RulesContextBuilder::build(&cards[0], "17", Some(&index));
        let json = serde_json::to_string_pretty(&ctx).unwrap();
        assert!(json.contains("\"mode\""));
        assert!(json.contains("\"lineup\""));
        assert!(json.contains("\"finalHeroes\""));
        assert!(json.contains("\"traits\""));
        assert!(json.contains("\"augments\""));
        assert!(json.contains("\"equipment\""));
        assert!(json.contains("\"modeSpecific\""));
        assert!(json.contains("\"strategyTexts\""));
    }

    #[test]
    fn supported_modes_non_empty() {
        let modes = RulesContextBuilder::supported_modes();
        assert_eq!(modes.len(), 3);
    }
}
