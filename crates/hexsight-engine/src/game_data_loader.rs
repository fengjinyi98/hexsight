// 游戏数据加载器
// 核心职责：
// - 从 config/game_data/mode{id}/ 加载英雄、装备、羁绊、符文等静态数据
// - 处理 object-of-objects JSON 格式 {"data": {"key1": {...}, "key2": {...}}}
// - 为 GameDataIndex 提供原始数据加载能力

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use hexsight_core::{
    EquipmentData, GodData, GodStage, GodWishEntry, HeroData, HexData, MissionData, TraitData,
};
use hexsight_core::{HexError, HexResult};

/// 游戏数据加载器
pub struct GameDataLoader;

impl GameDataLoader {
    /// 构建模式游戏数据目录路径
    fn mode_dir(config_root: &Path, mode: &str) -> PathBuf {
        config_root.join("game_data").join(format!("mode{}", mode))
    }

    // ---- 英雄 ----

    /// 加载英雄数据，返回 ID → HeroData 映射
    pub fn load_heroes(config_root: &Path, mode: &str) -> HexResult<HashMap<String, HeroData>> {
        let path = Self::mode_dir(config_root, mode).join("chess.json");
        let raw: HashMap<String, serde_json::Value> = load_data_map(&path)?;
        let mut map = HashMap::new();
        for (key, value) in raw {
            if let Ok(hero) = serde_json::from_value::<HeroData>(value) {
                map.insert(key, hero);
            }
        }
        Ok(map)
    }

    /// 预计算英雄派生字段（cost, base_key, star_level）
    pub fn enrich_heroes(heroes: &mut HashMap<String, HeroData>) {
        for hero in heroes.values_mut() {
            hero.cost = hero.price.parse::<i32>().unwrap_or(0);
            hero.star_level = hero
                .id
                .chars()
                .next()
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1) as i32;
            hero.base_key = format!("{}_{}", hero.name, hero.price);
        }
    }

    // ---- 装备 ----

    /// 加载装备数据，返回 ID → EquipmentData 映射
    pub fn load_equipment(
        config_root: &Path,
        mode: &str,
    ) -> HexResult<HashMap<String, EquipmentData>> {
        let path = Self::mode_dir(config_root, mode).join("equip.json");
        let raw: HashMap<String, serde_json::Value> = load_data_map(&path)?;
        let mut map = HashMap::new();
        for (key, value) in raw {
            if let Ok(mut equip) = serde_json::from_value::<EquipmentData>(value) {
                equip.is_component = equip.equip_type == "基础装备";
                equip.is_completed = equip.equip_type == "成型装备";
                map.insert(key, equip);
            }
        }
        Ok(map)
    }

    // ---- 羁绊 ----

    /// 加载羁绊数据，返回 Vec<TraitData>
    pub fn load_traits(config_root: &Path, mode: &str) -> HexResult<Vec<TraitData>> {
        let path = Self::mode_dir(config_root, mode).join("trait.json");
        let raw: HashMap<String, serde_json::Value> = load_data_map(&path)?;
        let mut traits: Vec<TraitData> = raw
            .into_values()
            .filter_map(|v| serde_json::from_value::<TraitData>(v).ok())
            .collect();
        // 预计算阈值
        for t in &mut traits {
            t.thresholds = t
                .numList
                .split('|')
                .filter_map(|s| s.trim().parse::<i32>().ok())
                .collect();
        }
        Ok(traits)
    }

    // ---- 强化符文 ----

    /// 加载强化符文数据，返回 ID → HexData 映射
    pub fn load_hexes(config_root: &Path, mode: &str) -> HexResult<HashMap<String, HexData>> {
        let path = Self::mode_dir(config_root, mode).join("hex.json");
        let raw: HashMap<String, serde_json::Value> = load_data_map(&path)?;
        let mut map = HashMap::new();
        for (key, value) in raw {
            if let Ok(hex) = serde_json::from_value::<HexData>(value) {
                map.insert(key, hex);
            }
        }
        Ok(map)
    }

    // ---- 种族/职业名称映射 ----

    /// 加载种族 ID → 名称 映射
    pub fn load_race_names(config_root: &Path, mode: &str) -> HexResult<HashMap<String, String>> {
        let path = Self::mode_dir(config_root, mode).join("race.json");
        load_name_map(&path)
    }

    /// 加载职业 ID → 名称 映射
    pub fn load_job_names(config_root: &Path, mode: &str) -> HexResult<HashMap<String, String>> {
        let path = Self::mode_dir(config_root, mode).join("job.json");
        load_name_map(&path)
    }

    /// 加载装备 ID → 名称 映射
    pub fn load_equip_names(config_root: &Path, mode: &str) -> HexResult<HashMap<String, String>> {
        let path = Self::mode_dir(config_root, mode).join("equip.json");
        load_name_map(&path)
    }

    // ---- 模式特有数据 ----

    /// 加载解锁任务数据（mode16 特有）
    pub fn load_missions(
        config_root: &Path,
        mode: &str,
    ) -> HexResult<HashMap<String, MissionData>> {
        let path = Self::mode_dir(config_root, mode).join("mission.json");
        // majority.json 的 data 是 object-of-objects 格式
        let raw: HashMap<String, serde_json::Value> = load_data_map(&path)?;
        let mut map = HashMap::new();
        for group in raw.values() {
            let hero_id = value_to_string(&group["heroid"]);
            let missions = group["mission"].as_array();
            if let Some(missions) = missions {
                for mission in missions {
                    let id = value_to_string(&mission["id"]);
                    if id.is_empty() {
                        continue;
                    }
                    let task_tips = mission["tasktips"].as_str().unwrap_or("").to_string();
                    let desc = mission["desc"].as_str().unwrap_or("").to_string();
                    map.insert(
                        id.clone(),
                        MissionData {
                            id,
                            hero_id: hero_id.clone(),
                            task_tips,
                            desc,
                        },
                    );
                }
            }
        }
        Ok(map)
    }

    /// 加载星神奖励数据（mode17 特有）
    pub fn load_gods(config_root: &Path, mode: &str) -> HexResult<Vec<GodData>> {
        let path = Self::mode_dir(config_root, mode).join("god.json");
        let raw: Vec<serde_json::Value> = load_data_array(&path)?;
        let mut gods = Vec::new();
        for god_val in &raw {
            let god_id = value_to_string(&god_val["godId"]);
            let god_name = god_val["godName"].as_str().unwrap_or("").to_string();
            let mut stages = Vec::new();
            if let Some(stage_arr) = god_val["stages"].as_array() {
                for stage in stage_arr {
                    let num = value_to_string(&stage["num"]);
                    let wishes: Vec<GodWishEntry> = stage["wishes"]
                        .as_array()
                        .map(|w| {
                            w.iter()
                                .map(|wish| GodWishEntry {
                                    id: value_to_string(&wish["id"]),
                                    name: wish["name"].as_str().unwrap_or("").to_string(),
                                    desc: wish["desc"].as_str().unwrap_or("").to_string(),
                                    icon: wish["icon"].as_str().unwrap_or("").to_string(),
                                    god_id: god_id.clone(),
                                    god_name: god_name.clone(),
                                    stage: num.parse::<i32>().unwrap_or(0),
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    stages.push(GodStage { num, wishes });
                }
            }
            gods.push(GodData {
                god_id,
                god_name,
                stages,
            });
        }
        Ok(gods)
    }
}

// ============================================================
// 内部辅助函数
// ============================================================

/// 加载 object-of-objects 格式 JSON 文件，返回 "data" 字段下的 HashMap
/// 格式: {"version": "...", "data": {"key1": {...}, "key2": {...}}}
fn load_data_map(path: &Path) -> HexResult<HashMap<String, serde_json::Value>> {
    let content = fs::read_to_string(path)
        .map_err(|e| HexError::Config(format!("读取文件失败 {}: {}", path.display(), e)))?;
    let root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| HexError::Config(format!("JSON 解析失败 {}: {}", path.display(), e)))?;
    let data = root
        .get("data")
        .ok_or_else(|| HexError::Config(format!("缺少 data 字段: {}", path.display())))?;
    let map: HashMap<String, serde_json::Value> = serde_json::from_value(data.clone())
        .map_err(|e| HexError::Config(format!("data 格式错误 {}: {}", path.display(), e)))?;
    Ok(map)
}

/// 加载数组格式 JSON 文件，返回 "data" 字段下的 Vec
/// 格式: {"version": "...", "data": [{...}, {...}]}
fn load_data_array(path: &Path) -> HexResult<Vec<serde_json::Value>> {
    let content = fs::read_to_string(path)
        .map_err(|e| HexError::Config(format!("读取文件失败 {}: {}", path.display(), e)))?;
    let root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| HexError::Config(format!("JSON 解析失败 {}: {}", path.display(), e)))?;
    let data = root
        .get("data")
        .ok_or_else(|| HexError::Config(format!("缺少 data 字段: {}", path.display())))?;
    let arr: Vec<serde_json::Value> = serde_json::from_value(data.clone())
        .map_err(|e| HexError::Config(format!("data 数组格式错误 {}: {}", path.display(), e)))?;
    Ok(arr)
}

/// 加载 ID → 名称 映射
fn load_name_map(path: &Path) -> HexResult<HashMap<String, String>> {
    let raw: HashMap<String, serde_json::Value> = load_data_map(path)?;
    let mut map = HashMap::new();
    for (key, value) in raw {
        if let Some(name) = value.get("name").and_then(|n| n.as_str()) {
            map.insert(key, name.to_string());
        }
    }
    Ok(map)
}

/// 将 serde_json::Value 转为字符串
fn value_to_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config_root() -> PathBuf {
        // CARGO_MANIFEST_DIR = crates/hexsight-engine
        // 向上 2 级到项目根
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config")
    }

    #[test]
    fn load_heroes_mode17() {
        let heroes = GameDataLoader::load_heroes(&test_config_root(), "17").unwrap();
        assert!(heroes.len() > 50, "mode17 英雄数量不足: {}", heroes.len());
        // 使用 mode17 中实际存在的英雄
        let has_timo = heroes.values().any(|h| h.name.contains("提莫"));
        assert!(has_timo, "mode17 英雄中未找到提莫");
    }

    #[test]
    fn load_equipment_mode17() {
        let equip = GameDataLoader::load_equipment(&test_config_root(), "17").unwrap();
        assert!(equip.len() > 30, "mode17 装备数量不足: {}", equip.len());
        assert!(
            equip.values().any(|e| e.equip_type == "基础装备"),
            "缺少基础装备"
        );
        assert!(
            equip.values().any(|e| e.equip_type == "成型装备"),
            "缺少成型装备"
        );
    }

    #[test]
    fn load_traits_mode17() {
        let traits = GameDataLoader::load_traits(&test_config_root(), "17").unwrap();
        assert!(!traits.is_empty(), "mode17 羁绊数量为 0");
        // 验证种族(type=0)和职业(type=1)都存在
        let has_race = traits.iter().any(|t| t.trait_type == 0);
        let has_job = traits.iter().any(|t| t.trait_type == 1);
        assert!(has_race || has_job, "羁绊中既无种族也无职业");
    }

    #[test]
    fn load_hexes_mode17() {
        let hexes = GameDataLoader::load_hexes(&test_config_root(), "17").unwrap();
        assert!(hexes.len() > 50, "mode17 强化符文数量不足: {}", hexes.len());
    }

    #[test]
    fn load_race_job_names() {
        let races = GameDataLoader::load_race_names(&test_config_root(), "17").unwrap();
        let jobs = GameDataLoader::load_job_names(&test_config_root(), "17").unwrap();
        assert!(!races.is_empty(), "种族名称为空");
        assert!(!jobs.is_empty(), "职业名称为空");
    }

    #[test]
    fn load_all_three_modes() {
        for mode in &["17", "16", "4"] {
            let heroes = GameDataLoader::load_heroes(&test_config_root(), mode)
                .unwrap_or_else(|_| panic!("mode {} 加载英雄失败", mode));
            assert!(
                heroes.len() > 10,
                "mode {} 英雄数量不足: {}",
                mode,
                heroes.len()
            );

            let traits = GameDataLoader::load_traits(&test_config_root(), mode)
                .unwrap_or_else(|_| panic!("mode {} 加载羁绊失败", mode));
            assert!(
                !traits.is_empty(),
                "mode {} 羁绊数量不足: {}",
                mode,
                traits.len()
            );

            let equip = GameDataLoader::load_equipment(&test_config_root(), mode)
                .unwrap_or_else(|_| panic!("mode {} 加载装备失败", mode));
            assert!(
                equip.len() > 10,
                "mode {} 装备数量不足: {}",
                mode,
                equip.len()
            );

            let hexes = GameDataLoader::load_hexes(&test_config_root(), mode)
                .unwrap_or_else(|_| panic!("mode {} 加载符文失败", mode));
            assert!(
                hexes.len() > 10,
                "mode {} 符文数量不足: {}",
                mode,
                hexes.len()
            );
        }
    }

    #[test]
    fn load_missions_mode16() {
        let missions = GameDataLoader::load_missions(&test_config_root(), "16").unwrap();
        assert!(!missions.is_empty(), "mode16 任务数据为空");
    }

    #[test]
    fn load_gods_mode17() {
        let gods = GameDataLoader::load_gods(&test_config_root(), "17").unwrap();
        assert!(!gods.is_empty(), "mode17 神明数据为空");
        assert!(
            gods.iter().any(|g| !g.stages.is_empty()),
            "神明阶段数据为空"
        );
    }

    #[test]
    fn enrich_heroes_works() {
        let mut heroes = GameDataLoader::load_heroes(&test_config_root(), "17").unwrap();
        GameDataLoader::enrich_heroes(&mut heroes);
        for hero in heroes.values() {
            assert!(!hero.base_key.is_empty(), "英雄 {} base_key 为空", hero.id);
            assert!(hero.star_level >= 1, "英雄 {} star_level 无效", hero.id);
        }
    }
}
