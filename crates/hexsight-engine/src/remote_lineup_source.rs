// 远端阵容数据源
// 核心职责：
// - 根据模式配置拼装官方 CDN URL
// - 发起 HTTP 请求拉取远端 lineup_detail_total.json
// - 校验并写入本地缓存

use std::path::Path;

use hexsight_core::HexResult;

use crate::LineupLoader;

/// 支持的模式远端配置
const MODE_REMOTE_CONFIGS: &[(&str, &str, &str)] =
    &[("17", "m18", "11"), ("16", "m17", "11"), ("4", "m17", "11")];

/// 远端阵容数据源
pub struct RemoteLineupSource;

impl RemoteLineupSource {
    /// 获取某模式的 CDN URL
    pub fn cdn_url(mode: &str) -> Option<String> {
        MODE_REMOTE_CONFIGS.iter()
            .find(|(id, _, _)| *id == mode)
            .map(|(id, version_path, channel)| {
                format!(
                    "https://game.gtimg.cn/images/lol/act/jkzlkauto/json/lineupJson/{}/{}/{}/lineup_detail_total.json",
                    version_path, channel, id
                )
            })
    }

    /// 从远端拉取阵容数据，写入本地缓存
    pub fn refresh(config_root: &Path, mode: &str) -> HexResult<usize> {
        let url = Self::cdn_url(mode)
            .ok_or_else(|| hexsight_core::HexError::Config(format!("模式 {} 无远端配置", mode)))?;

        let response = ureq::get(&url)
            .call()
            .map_err(|e| hexsight_core::HexError::Config(format!("远端请求失败 {}: {}", url, e)))?;

        let raw_json = response
            .into_body()
            .read_to_string()
            .map_err(|e| hexsight_core::HexError::Config(format!("读取响应失败: {}", e)))?;

        // 校验 JSON 合法性
        let container: hexsight_core::LineupListContainer = serde_json::from_str(&raw_json)
            .map_err(|e| {
                hexsight_core::HexError::Config(format!("远端返回 JSON 格式错误: {}", e))
            })?;

        let count = container.lineup_list.len();
        LineupLoader::save_remote_cache(config_root, mode, "S18", &raw_json)?;
        Ok(count)
    }

    /// 是否支持该模式的远端刷新
    pub fn supports_remote(mode: &str) -> bool {
        MODE_REMOTE_CONFIGS.iter().any(|(id, _, _)| *id == mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdn_url_for_mode17() {
        let url = RemoteLineupSource::cdn_url("17");
        assert!(url.is_some());
        assert!(url.unwrap().contains("lineup_detail_total.json"));
    }

    #[test]
    fn cdn_urls_for_all_modes() {
        for (mode, _, _) in MODE_REMOTE_CONFIGS {
            assert!(RemoteLineupSource::cdn_url(mode).is_some());
        }
    }

    #[test]
    fn supports_remote_all_modes() {
        assert!(RemoteLineupSource::supports_remote("17"));
        assert!(RemoteLineupSource::supports_remote("16"));
        assert!(RemoteLineupSource::supports_remote("4"));
        assert!(!RemoteLineupSource::supports_remote("99"));
    }
}
