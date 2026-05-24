// 装备匹配评分器
// 核心职责：
// - 判断当前装备最适合哪些阵容
// - 计算装备方向（AD/AP/坦/混合）
// - 输出装备动作建议

use hexsight_core::{ItemAction, ItemFitResult, LineupProfile};

/// 装备匹配评分器
pub struct ItemFitScorer;

impl ItemFitScorer {
    /// 根据当前装备散件/成装，计算每套阵容的装备匹配分
    pub fn score(
        profiles: &[LineupProfile],
        current_equipment_ids: &[String],
        current_components: &[String],
    ) -> Vec<ItemFitResult> {
        let equip_set: Vec<&str> = current_equipment_ids.iter().map(|s| s.as_str()).collect();
        let mut all_available = equip_set.clone();
        for c in current_components {
            all_available.push(c.as_str());
        }

        let mut results: Vec<ItemFitResult> = profiles
            .iter()
            .map(|profile| {
                let core_match = profile
                    .core_equipment_ids
                    .iter()
                    .filter(|eid| all_available.contains(&eid.as_str()))
                    .count();
                let equipment_match = profile
                    .equipment_order_ids
                    .iter()
                    .filter(|eid| all_available.contains(&eid.as_str()))
                    .count();
                let total_core = profile.core_equipment_ids.len().max(1);
                let match_ratio = (core_match as f64 / total_core as f64).min(1.0);

                let (action, reasons) = if match_ratio >= 0.6 {
                    let reasons = vec![format!("核心装备匹配 {}/{}", core_match, total_core)];
                    (ItemAction::BuildCombatNow, reasons)
                } else if match_ratio >= 0.3 {
                    let reasons = vec![format!(
                        "核心装备部分匹配 {}/{}，可继续观察",
                        core_match, total_core
                    )];
                    (ItemAction::WaitCore, reasons)
                } else if equipment_match > 0 {
                    let reasons = vec![format!("装备优先级部分匹配 {} 件", equipment_match)];
                    (ItemAction::BuildGeneric, reasons)
                } else {
                    let reasons = vec!["当前装备不匹配此阵容核心装".to_string()];
                    (ItemAction::HoldComponents, reasons)
                };

                let flexibility = if profile
                    .playstyle_tags
                    .iter()
                    .any(|t| matches!(t, hexsight_core::PlaystyleTag::ItemFlexible))
                {
                    80
                } else {
                    40
                };

                ItemFitResult {
                    best_fit_lineup_ids: vec![profile.lineup_id.clone()],
                    direction: Self::classify_direction(&profile.core_equipment_ids),
                    core_component_count: profile.core_equipment_ids.len() as i32,
                    can_build_core_count: core_match as i32,
                    flexibility,
                    recommended_action: action,
                    reasons,
                }
            })
            .collect();

        // 按核心装匹配数排序
        results.sort_by(|a, b| b.can_build_core_count.cmp(&a.can_build_core_count));
        results
    }

    /// 获取全局最佳装备方向
    pub fn best_direction(results: &[ItemFitResult]) -> String {
        let mut ad_count = 0;
        let mut ap_count = 0;
        let mut tank_count = 0;
        for r in results.iter().take(5) {
            match r.direction.as_str() {
                "AD" => ad_count += 1,
                "AP" => ap_count += 1,
                "坦" => tank_count += 1,
                _ => {}
            }
        }
        if ad_count > ap_count && ad_count > tank_count {
            "AD".into()
        } else if ap_count > ad_count && ap_count > tank_count {
            "AP".into()
        } else if tank_count > ad_count && tank_count > ap_count {
            "坦".into()
        } else {
            "混合".into()
        }
    }

    /// 根据装备 ID 判断方向（使用简单 ID 前缀规则）
    fn classify_direction(equipment_ids: &[String]) -> String {
        let mut ad = 0;
        let mut ap = 0;
        let mut tank = 0;
        for eid in equipment_ids {
            let id = eid.parse::<i32>().unwrap_or(0);
            match id {
                // 攻击散件
                1001 | 1006 | 1008 | 1019 | 1021 => ad += 1,
                // 法系散件
                1003 | 1004 | 1005 | 1010 => ap += 1,
                // 防御散件
                1002 | 1007 | 1009 | 1011 => tank += 1,
                _ => {}
            }
        }
        if ad > ap && ad > tank {
            "AD".into()
        } else if ap > ad && ap > tank {
            "AP".into()
        } else if tank > ad && tank > ap {
            "坦".into()
        } else {
            "混合".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameDataIndex, LineupAdapter, LineupLoader, LineupProfileBuilder};
    use std::path::PathBuf;

    fn load_profiles(mode: &str) -> Vec<LineupProfile> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config");
        let index = GameDataIndex::load(&root, mode).unwrap();
        let raw = LineupLoader::load_cached_lineups(&root, mode, "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, mode);
        LineupProfileBuilder::build_all(&cards, &index)
    }

    #[test]
    fn score_mode17_lineups() {
        let profiles = load_profiles("17");
        let components = vec!["1003".to_string(), "1009".to_string()];
        let results = ItemFitScorer::score(&profiles, &[], &components);
        assert!(!results.is_empty());
        // 至少有一条有方向
        assert!(results.iter().any(|r| !r.direction.is_empty()));
    }

    #[test]
    fn item_direction_classified() {
        let profiles = load_profiles("17");
        let empty: Vec<String> = vec![];
        let results = ItemFitScorer::score(&profiles, &[], &empty);
        let direction = ItemFitScorer::best_direction(&results);
        assert!(!direction.is_empty());
    }
}
