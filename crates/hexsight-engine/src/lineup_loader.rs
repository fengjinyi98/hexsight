// 阵容数据加载器
// 核心职责：
// - 从 config/lineups/ 加载官方阵容缓存 JSON
// - 解析 lineup_detail_total.json 格式
// - 提供本地缓存和远端刷新入口

use std::fs;
use std::path::{Path, PathBuf};

use hexsight_core::{HexError, HexResult};
use hexsight_core::{LineupListContainer, LineupRawItem};

/// 阵容数据加载器
pub struct LineupLoader;

impl LineupLoader {
    /// 加载本地缓存的阵容列表
    /// 文件格式: {config_root}/lineups/mode{mode}_{season}.json
    pub fn load_cached_lineups(
        config_root: &Path,
        mode: &str,
        season: &str,
    ) -> HexResult<Vec<LineupRawItem>> {
        let path = Self::cache_path(config_root, mode, season);
        let content = fs::read_to_string(&path)
            .map_err(|e| HexError::Config(format!("读取阵容缓存失败 {}: {}", path.display(), e)))?;
        let container: LineupListContainer = serde_json::from_str(&content).map_err(|e| {
            HexError::Config(format!("解析阵容 JSON 失败 {}: {}", path.display(), e))
        })?;
        Ok(container.lineup_list)
    }

    /// 从远端数据写入缓存（网络数据由 Swift 侧通过 URLSession 获取后传入）
    pub fn save_remote_cache(
        config_root: &Path,
        mode: &str,
        season: &str,
        raw_json: &str,
    ) -> HexResult<()> {
        // 验证 JSON 合法性
        let _container: LineupListContainer = serde_json::from_str(raw_json)
            .map_err(|e| HexError::Config(format!("远端数据 JSON 格式错误: {}", e)))?;

        let path = Self::cache_path(config_root, mode, season);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| HexError::Config(format!("创建缓存目录失败: {}", e)))?;
        }
        fs::write(&path, raw_json)
            .map_err(|e| HexError::Config(format!("写入缓存失败 {}: {}", path.display(), e)))?;
        Ok(())
    }

    /// 从内存中的 JSON 字符串解析阵容（用于 Swift 传入远端拉取结果）
    pub fn parse_from_json(raw_json: &str) -> HexResult<Vec<LineupRawItem>> {
        let container: LineupListContainer = serde_json::from_str(raw_json)
            .map_err(|e| HexError::Config(format!("JSON 解析失败: {}", e)))?;
        Ok(container.lineup_list)
    }

    /// 缓存文件路径
    fn cache_path(config_root: &Path, mode: &str, season: &str) -> PathBuf {
        config_root
            .join("lineups")
            .join(format!("mode{}_{}.json", mode, season))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config")
    }

    #[test]
    fn load_cached_lineups_mode17() {
        let items = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        assert!(!items.is_empty(), "mode17 阵容缓存为空");
        // 验证 detail 字段是 JSON 字符串
        assert!(items.iter().any(|i| !i.detail.is_empty()), "无 detail 字段");
    }

    #[test]
    fn load_all_three_modes_lineups() {
        for mode in &["17", "16", "4"] {
            let items = LineupLoader::load_cached_lineups(&test_config_root(), mode, "S18")
                .unwrap_or_else(|_| panic!("mode {} 阵容加载失败", mode));
            assert!(!items.is_empty(), "mode {} 阵容为空", mode);
        }
    }

    #[test]
    fn parse_from_json_roundtrip() {
        let items = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        let container = LineupListContainer { lineup_list: items };
        let json = serde_json::to_string(&container).unwrap();
        let parsed = LineupLoader::parse_from_json(&json).unwrap();
        assert!(!parsed.is_empty());
    }
}
