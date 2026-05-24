// 游戏数据索引
// 核心职责：
// - 从已加载的原始数据构建 ID → 名称/图标/描述的快速索引
// - 提供羁绊反推能力（从棋子 species/class 计算激活羁绊）
// - 为 RulesContext 和 LineupAdapter 提供统一的数据查询接口

use std::collections::HashMap;
use std::path::Path;

use hexsight_core::HexResult;
use hexsight_core::{
    EquipmentData, GodData, HeroData, HexData, LineupPieceData, MissionData, RuleTraitSnapshot,
    TraitContactData, TraitData,
};

use crate::GameDataLoader;

/// 游戏数据索引 —— ID 到可读信息的映射
pub struct GameDataIndex {
    /// 当前模式
    pub mode: String,
    /// hero_id → HeroData
    pub heroes_by_id: HashMap<String, HeroData>,
    /// equip_id → EquipmentData
    pub equipment_by_id: HashMap<String, EquipmentData>,
    /// hex_id → HexData
    pub hexes_by_id: HashMap<String, HexData>,
    /// 羁绊列表（按 level 排序）
    pub traits: Vec<TraitData>,
    /// 种族 ID → 名称
    pub race_names: HashMap<String, String>,
    /// 职业 ID → 名称
    pub job_names: HashMap<String, String>,
    /// 装备 ID → 名称
    pub equip_names: HashMap<String, String>,
    /// 英雄名 → 头像 URL
    pub hero_pictures_by_name: HashMap<String, String>,
    /// mission_id → MissionData（mode16 特有）
    pub missions_by_id: HashMap<String, MissionData>,
    /// 神明奖励数据（mode17 特有）
    pub gods: Vec<GodData>,
}

impl GameDataIndex {
    /// 从 config 目录加载某模式的完整索引
    pub fn load(config_root: &Path, mode: &str) -> HexResult<Self> {
        let mut heroes = GameDataLoader::load_heroes(config_root, mode)?;
        GameDataLoader::enrich_heroes(&mut heroes);

        let equipment = GameDataLoader::load_equipment(config_root, mode)?;
        let traits = GameDataLoader::load_traits(config_root, mode)?;
        let hexes = GameDataLoader::load_hexes(config_root, mode)?;
        let race_names = GameDataLoader::load_race_names(config_root, mode).unwrap_or_default();
        let job_names = GameDataLoader::load_job_names(config_root, mode).unwrap_or_default();
        let equip_names = GameDataLoader::load_equip_names(config_root, mode).unwrap_or_default();
        let missions_by_id = GameDataLoader::load_missions(config_root, mode).unwrap_or_default();
        let gods = GameDataLoader::load_gods(config_root, mode).unwrap_or_default();

        let hero_pictures_by_name: HashMap<String, String> = heroes
            .values()
            .filter(|h| !h.name.is_empty() && !h.picture.is_empty())
            .map(|h| (h.name.clone(), h.picture.clone()))
            .collect();

        Ok(Self {
            mode: mode.to_string(),
            heroes_by_id: heroes,
            equipment_by_id: equipment,
            hexes_by_id: hexes,
            traits,
            race_names,
            job_names,
            equip_names,
            hero_pictures_by_name,
            missions_by_id,
            gods,
        })
    }

    // ---- 查询方法 ----

    /// 按 ID 查询英雄
    pub fn hero(&self, id: &str) -> Option<&HeroData> {
        self.heroes_by_id.get(id)
    }

    /// 按 ID 查询装备
    pub fn equipment(&self, id: &str) -> Option<&EquipmentData> {
        self.equipment_by_id.get(id)
    }

    /// 按 ID 查询强化符文
    pub fn hex(&self, id: &str) -> Option<&HexData> {
        self.hexes_by_id.get(id)
    }

    /// 按名称查询英雄头像
    pub fn hero_picture(&self, name: &str) -> Option<&str> {
        self.hero_pictures_by_name.get(name).map(|s| s.as_str())
    }

    /// 种族名称
    pub fn race_name(&self, species_id: &str) -> &str {
        self.race_names
            .get(species_id)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// 职业名称
    pub fn job_name(&self, class_id: &str) -> &str {
        self.job_names
            .get(class_id)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// 装备名称（返回 owned String 以处理缺省回退）
    pub fn equip_name(&self, equip_id: &str) -> String {
        self.equip_names
            .get(equip_id)
            .cloned()
            .unwrap_or_else(|| equip_id.to_string())
    }

    /// 获取羁绊的所有等级配置
    pub fn trait_levels(&self, check_id: &str) -> Vec<&TraitData> {
        let mut levels: Vec<&TraitData> = self
            .traits
            .iter()
            .filter(|t| t.checkId == check_id)
            .collect();
        levels.sort_by_key(|t| t.level);
        levels
    }

    /// 获取某羁绊包含的英雄
    pub fn heroes_for_trait(&self, trait_id: &str, is_race: bool) -> Vec<&HeroData> {
        self.heroes_by_id
            .values()
            .filter(|h| h.cost > 0)
            .filter(|h| {
                let ids = if is_race { &h.species } else { &h.hero_class };
                split_trait_ids(ids).iter().any(|id| *id == trait_id)
            })
            .collect()
    }

    // ---- 羁绊反推 ----

    /// 计算阵容的羁绊快照列表
    /// 优先使用官方 contacts，若无则从棋子反推
    pub fn trait_summaries(
        &self,
        pieces: &[LineupPieceData],
        official_contacts: &[TraitContactData],
    ) -> Vec<RuleTraitSnapshot> {
        if !official_contacts.is_empty()
            && official_contacts.iter().any(|c| c.color > 0 || c.count > 0)
        {
            return self.summaries_from_contacts(official_contacts);
        }
        self.summaries_from_pieces(pieces)
    }

    /// 从官方 contact 数据构建羁绊快照
    fn summaries_from_contacts(&self, contacts: &[TraitContactData]) -> Vec<RuleTraitSnapshot> {
        let mut summaries: Vec<RuleTraitSnapshot> = contacts
            .iter()
            .filter(|c| c.color > 0 || c.count > 0)
            .filter_map(|contact| {
                let matched = self
                    .traits
                    .iter()
                    .find(|t| t.checkId == contact.id && trait_matches(&contact.contact_type, t))
                    .or_else(|| self.traits.iter().find(|t| t.checkId == contact.id));

                matched.map(|t| RuleTraitSnapshot {
                    id: format!("{}-{}", contact.contact_type, contact.id),
                    trait_id: contact.id.clone(),
                    trait_type: contact.contact_type.clone(),
                    name: t.name.clone(),
                    count: contact.count,
                    color: contact.color,
                    level: contact.level,
                    picture: t.picture.clone(),
                })
            })
            .collect();

        // 按 color(降序) > count(降序) > name(升序) 排序
        summaries.sort_by(|a, b| {
            b.color
                .cmp(&a.color)
                .then_with(|| b.count.cmp(&a.count))
                .then_with(|| a.name.cmp(&b.name))
        });
        summaries
    }

    /// 从棋子 species/class 反推羁绊
    fn summaries_from_pieces(&self, pieces: &[LineupPieceData]) -> Vec<RuleTraitSnapshot> {
        let mut counts: HashMap<String, i32> = HashMap::new();

        for piece in pieces {
            if piece.chess_type != "hero" {
                continue;
            }
            if let Some(hero) = self.hero(&piece.hero_id) {
                for id in split_trait_ids(&hero.species) {
                    *counts.entry(format!("race:{}", id)).or_default() += 1;
                }
                for id in split_trait_ids(&hero.hero_class) {
                    *counts.entry(format!("job:{}", id)).or_default() += 1;
                }
            }
        }

        let mut summaries: Vec<RuleTraitSnapshot> = counts
            .into_iter()
            .filter_map(|(key, count)| {
                let parts: Vec<&str> = key.splitn(2, ':').collect();
                if parts.len() != 2 {
                    return None;
                }
                let trait_type = parts[0];
                let check_id = parts[1].to_string();
                let key_owned = key.clone();

                let candidates: Vec<&TraitData> = self
                    .traits
                    .iter()
                    .filter(|t| t.checkId == check_id && trait_matches(trait_type, t))
                    .collect();

                let active = candidates
                    .iter()
                    .filter(|t| {
                        let required = t.num.parse::<i32>().unwrap_or(i32::MAX);
                        count >= required
                    })
                    .max_by_key(|t| t.level)?;

                Some(RuleTraitSnapshot {
                    id: key_owned,
                    trait_id: check_id.clone(),
                    trait_type: trait_type.to_string(),
                    name: active.name.clone(),
                    count,
                    color: active.color.parse::<i32>().unwrap_or(0),
                    level: active.level,
                    picture: active.picture.clone(),
                })
            })
            .collect();

        summaries.sort_by(|a, b| {
            b.color
                .cmp(&a.color)
                .then_with(|| b.count.cmp(&a.count))
                .then_with(|| a.name.cmp(&b.name))
        });
        summaries
    }
}

// ============================================================
// 辅助函数
// ============================================================

/// 拆分 | 分隔的羁绊 ID 字符串
fn split_trait_ids(raw: &str) -> Vec<&str> {
    raw.split('|')
        .filter(|s| !s.is_empty() && *s != "0" && *s != "-1")
        .collect()
}

/// 判断羁绊类型是否匹配
fn trait_matches(trait_type: &str, trait_data: &TraitData) -> bool {
    match trait_type {
        "race" => trait_data.trait_type == 0,
        "job" => trait_data.trait_type == 1,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn load_index_mode17() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        assert_eq!(index.mode, "17");
        assert!(index.heroes_by_id.len() > 50);
        assert!(index.equipment_by_id.len() > 30);
        assert!(!index.traits.is_empty());
        assert!(!index.hexes_by_id.is_empty());
        assert!(!index.race_names.is_empty());
        assert!(!index.job_names.is_empty());
    }

    #[test]
    fn load_index_all_modes() {
        for mode in &["17", "16", "4"] {
            let index = GameDataIndex::load(&test_config_root(), mode)
                .unwrap_or_else(|_| panic!("mode {} 索引加载失败", mode));
            assert!(index.heroes_by_id.len() > 10);
            assert!(index.equipment_by_id.len() > 10);
        }
    }

    #[test]
    fn hero_lookup() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        // 找一个实际存在的英雄
        if let Some((id, _)) = index.heroes_by_id.iter().find(|(_, h)| h.cost > 0) {
            let hero = index.hero(id);
            assert!(hero.is_some());
            assert!(!hero.unwrap().name.is_empty());
        }
    }

    #[test]
    fn equipment_lookup() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        if let Some((id, _)) = index.equipment_by_id.iter().next() {
            let equip = index.equipment(id);
            assert!(equip.is_some());
            assert!(!equip.unwrap().name.is_empty());
        }
    }

    #[test]
    fn trait_lookup() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        if let Some(t) = index.traits.first() {
            let levels = index.trait_levels(&t.checkId);
            assert!(!levels.is_empty());
        }
    }
}
