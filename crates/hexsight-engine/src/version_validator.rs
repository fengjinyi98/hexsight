// 版本校验工具
// 核心职责：
// - 检查阵容中的 hero_id/equip_id 是否能在静态数据中找到
// - 报告缺失或不匹配的 ID
// - 校验阵容解析率

use std::path::Path;

use hexsight_core::HexResult;

use crate::{GameDataIndex, LineupAdapter, LineupLoader};

/// 版本校验工具
pub struct VersionValidator;

impl VersionValidator {
    /// 校验某模式的阵容数据与静态数据是否对齐
    pub fn validate(config_root: &Path, mode: &str) -> HexResult<ValidationReport> {
        let index = GameDataIndex::load(config_root, mode)?;
        let raw_items = LineupLoader::load_cached_lineups(config_root, mode, "S18")?;
        let cards = LineupAdapter::cards_from_raw_list(&raw_items, mode);

        let mut missing_hero_ids = Vec::new();
        let mut missing_equip_ids = Vec::new();
        let mut seen_hero_ids = std::collections::HashSet::new();
        let mut seen_equip_ids = std::collections::HashSet::new();
        let mut unparseable_lineups = Vec::new();

        for raw in &raw_items {
            if raw.detail.is_empty() {
                unparseable_lineups.push(raw.id.clone());
            }
        }

        for card in &cards {
            for piece in &card.detail.final_heroes {
                seen_hero_ids.insert(piece.hero_id.clone());
                if index.hero(&piece.hero_id).is_none() {
                    missing_hero_ids.push(format!("{} (阵容: {})", piece.hero_id, card.name));
                }
                for eid in &piece.equipment_ids {
                    if eid == "0" {
                        continue;
                    }
                    seen_equip_ids.insert(eid.clone());
                    if index.equipment(eid).is_none() {
                        missing_equip_ids.push(format!("{} (阵容: {})", eid, card.name));
                    }
                }
            }
        }

        let is_aligned = missing_hero_ids.is_empty() && missing_equip_ids.is_empty();

        Ok(ValidationReport {
            mode: mode.to_string(),
            total_lineups: raw_items.len(),
            parseable_lineups: cards.len(),
            unparseable_lineups: raw_items.len() - cards.len(),
            unique_hero_ids_in_lineups: seen_hero_ids.len(),
            unique_equip_ids_in_lineups: seen_equip_ids.len(),
            total_hero_ids_in_static: index.heroes_by_id.len(),
            total_equip_ids_in_static: index.equipment_by_id.len(),
            missing_hero_ids,
            missing_equip_ids,
            is_aligned,
        })
    }
}

/// 校验报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// 模式 ID
    pub mode: String,
    /// 阵容总数
    pub total_lineups: usize,
    /// 可解析的阵容数
    pub parseable_lineups: usize,
    /// 无法解析的阵容数（detail 为空）
    pub unparseable_lineups: usize,
    /// 阵容中引用的独立 hero_id 数
    pub unique_hero_ids_in_lineups: usize,
    /// 阵容中引用的独立 equip_id 数
    pub unique_equip_ids_in_lineups: usize,
    /// 静态数据中的英雄总数
    pub total_hero_ids_in_static: usize,
    /// 静态数据中的装备总数
    pub total_equip_ids_in_static: usize,
    /// 在静态数据中缺失的 hero_id
    pub missing_hero_ids: Vec<String>,
    /// 在静态数据中缺失的 equip_id
    pub missing_equip_ids: Vec<String>,
    /// 数据是否对齐
    pub is_aligned: bool,
}

impl ValidationReport {
    /// 转换为 JSON（对齐文档验收格式）
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "mode": self.mode,
            "total_lineups": self.total_lineups,
            "parseable_lineups": self.parseable_lineups,
            "unparseable_lineups": self.unparseable_lineups,
            "unique_hero_ids_in_lineups": self.unique_hero_ids_in_lineups,
            "unique_equip_ids_in_lineups": self.unique_equip_ids_in_lineups,
            "total_hero_ids_in_static": self.total_hero_ids_in_static,
            "total_equip_ids_in_static": self.total_equip_ids_in_static,
            "missing_hero_ids": self.missing_hero_ids,
            "missing_equip_ids": self.missing_equip_ids,
            "is_aligned": self.is_aligned,
        })
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
    fn validate_mode17() {
        let report = VersionValidator::validate(&test_config_root(), "17").unwrap();
        assert!(report.total_lineups > 0);
        assert!(report.parseable_lineups > 0);
        // 验证有 JSON 输出
        let json = report.to_json();
        assert_eq!(json["mode"].as_str().unwrap(), "17");
        assert!(json["total_lineups"].as_u64().unwrap() > 0);
    }

    #[test]
    fn validate_all_modes() {
        for mode in &["17", "16", "4"] {
            let report = VersionValidator::validate(&test_config_root(), mode)
                .unwrap_or_else(|_| panic!("mode {} 校验失败", mode));
            assert!(report.total_lineups > 0, "mode {} 无阵容数据", mode);
            assert!(report.parseable_lineups > 0, "mode {} 全不可解析", mode);
        }
    }
}
