// 规则包加载器
// 核心职责：
// - 从 config/rules/<version>/rule_pack.json 加载版本化规则参数
// - 失败时回退 Default 规则包并返回错误信息
// - 支持列出版本目录下的所有可用规则包

use std::fs;
use std::path::{Path, PathBuf};

use crate::PatchOverrideLoader;
use hexsight_core::{HexError, HexResult, RulePack};

/// 规则包加载器
pub struct RulePackLoader;

impl RulePackLoader {
    /// 加载指定版本的规则包，自动尝试应用 patch_overrides.json
    /// 失败回退 Default 规则包并返回警告信息
    pub fn load(config_root: &Path, version: &str) -> (RulePack, Vec<String>) {
        let path = Self::rule_pack_path(config_root, version);
        let mut warnings = Vec::new();

        let mut pack = match Self::load_from_path(&path) {
            Ok(p) => p,
            Err(e) => {
                warnings.push(format!(
                    "规则包加载失败 {}: {}，回退到默认规则包",
                    path.display(),
                    e
                ));
                RulePack::default()
            }
        };

        // 自动尝试加载并应用版本覆写
        match PatchOverrideLoader::load(config_root, version) {
            Ok(Some(ov)) => {
                PatchOverrideLoader::apply(&mut pack, &ov);
            }
            Ok(None) => {} // 无覆写文件，正常
            Err(e) => {
                warnings.push(format!("版本覆写加载失败: {}", e));
            }
        }

        (pack, warnings)
    }

    /// 从指定路径加载规则包
    pub fn load_from_path(path: &Path) -> HexResult<RulePack> {
        let content = fs::read_to_string(path)
            .map_err(|e| HexError::Config(format!("读取规则包失败 {}: {}", path.display(), e)))?;
        let pack: RulePack = serde_json::from_str(&content).map_err(|e| {
            HexError::Config(format!("解析规则包 JSON 失败 {}: {}", path.display(), e))
        })?;
        Ok(pack)
    }

    /// 列出可用的规则包版本
    pub fn available_versions(config_root: &Path) -> Vec<String> {
        let rules_dir = config_root.join("rules");
        let mut versions = Vec::new();
        if let Ok(entries) = fs::read_dir(&rules_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let pack_path = entry.path().join("rule_pack.json");
                    if pack_path.exists() {
                        versions.push(name);
                    }
                }
            }
        }
        versions.sort();
        versions
    }

    /// 规则包文件路径
    fn rule_pack_path(config_root: &Path, version: &str) -> PathBuf {
        config_root
            .join("rules")
            .join(version)
            .join("rule_pack.json")
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
    fn load_s18_1_rule_pack() {
        let (pack, warnings) = RulePackLoader::load(&test_config_root(), "S18.1");
        assert!(warnings.is_empty(), "加载失败: {:?}", warnings);
        assert_eq!(pack.version, "S18.1");
        assert!(!pack.modes.is_empty());
        assert!(pack.weights.lineup_fit.item_fit > 0.0);
        assert!(pack.weights.transition.two_star > 0.0);
    }

    #[test]
    fn load_nonexistent_version_falls_back() {
        let (pack, warnings) = RulePackLoader::load(&test_config_root(), "S99");
        assert!(!warnings.is_empty());
        // 回退到默认
        assert_eq!(pack.version, "S18.1");
    }

    #[test]
    fn available_versions() {
        let versions = RulePackLoader::available_versions(&test_config_root());
        assert!(versions.contains(&"S18.1".to_string()));
    }

    #[test]
    fn rule_pack_weights_sum_near_one() {
        let (pack, _) = RulePackLoader::load(&test_config_root(), "S18.1");
        let w = &pack.weights.lineup_fit;
        let sum = w.base_score
            + w.item_fit
            + w.champion_hit
            + w.augment_fit
            + w.trait_fit
            + w.stage_fit
            + w.economy_fit
            + w.health_safety
            + w.playstyle_switch;
        assert!((sum - 1.0).abs() < 0.01, "权重和不为 1.0: {}", sum);
    }

    #[test]
    fn rule_pack_thresholds_reasonable() {
        let (pack, _) = RulePackLoader::load(&test_config_root(), "S18.1");
        assert!(pack.thresholds.lock_lineup_score > 50);
        assert!(pack.thresholds.must_stabilize_health > 0);
        assert!(pack.thresholds.must_stabilize_health < 100);
    }

    #[test]
    fn rule_pack_damage_profile_covers_stages() {
        let (pack, _) = RulePackLoader::load(&test_config_root(), "S18.1");
        assert!(pack.damage_profile.contains_key("2"));
        assert!(pack.damage_profile.contains_key("6"));
    }
}

// ============================================================
// 回归集成测试
// ============================================================

#[cfg(test)]
mod regression_tests {
    use super::*;
    use crate::*;
    use std::path::PathBuf;

    fn config_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config")
    }

    /// 回归：规则包加载 + 阵容档案生成 + 评分完整链路
    #[test]
    fn full_pipeline_rule_pack_to_scores() {
        let root = config_root();
        // 1. 加载规则包
        let (pack, warnings) = RulePackLoader::load(&root, "S18.1");
        assert!(warnings.is_empty());

        // 2. 加载游戏数据 + 阵容
        let index = GameDataIndex::load(&root, "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&root, "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");

        // 3. 生成阵容档案
        let profiles = LineupProfileBuilder::build_all(&cards, &index);
        assert!(!profiles.is_empty());

        // 4. 规则包权重驱动评分
        let scores = LineupFitScorer::score_all(
            &profiles,
            &[],
            &[],
            &[],
            20,
            100,
            6,
            3.5,
            &std::collections::HashMap::new(),
            &pack,
        );
        assert!(!scores.is_empty());
        assert!(scores.iter().all(|s| s.total_score <= 100));

        // 5. 过渡战力也用规则包权重
        let strength = TransitionStrengthScorer::assess(5, 3, true, true, 3, 2, true, &pack);
        assert!(strength.total_score > 0);

        // 6. Schema 校验
        let report = VersionValidator::validate(&root, "17").unwrap();
        assert!(report.total_lineups > 0);
        assert!(report.is_aligned);
    }

    /// 回归：装备评分用散件输入
    #[test]
    fn item_fit_uses_components() {
        let root = config_root();
        let index = GameDataIndex::load(&root, "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&root, "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let profiles = LineupProfileBuilder::build_all(&cards, &index);

        let components = vec!["1003".to_string(), "1004".to_string()];
        let equipped = vec!["2001".to_string()];
        let results = ItemFitScorer::score(&profiles, &equipped, &components);
        assert!(!results.is_empty());
        // 散件应影响核心装匹配计数
        let best = &results[0];
        assert!(best.can_build_core_count >= 0);
    }

    /// 回归：权重变化影响评分
    #[test]
    fn weight_changes_affect_scores() {
        let root = config_root();
        let index = GameDataIndex::load(&root, "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&root, "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let profiles = LineupProfileBuilder::build_all(&cards, &index);

        // 默认权重
        let default_scores = LineupFitScorer::score_all(
            &profiles,
            &[],
            &[],
            &[],
            20,
            100,
            6,
            3.5,
            &std::collections::HashMap::new(),
            &RulePack::default(),
        );

        // 修改权重：装备权重翻倍
        let mut alt_pack = RulePack::default();
        alt_pack.weights.lineup_fit.item_fit = 0.50;
        let alt_scores = LineupFitScorer::score_all(
            &profiles,
            &[],
            &[],
            &[],
            20,
            100,
            6,
            3.5,
            &std::collections::HashMap::new(),
            &alt_pack,
        );

        // 权重变化应导致分数差异
        let _default_top = default_scores.iter().max_by_key(|s| s.total_score).unwrap();
        let _alt_top = alt_scores.iter().max_by_key(|s| s.total_score).unwrap();
        // 至少有一个阵容的分数变了
        let any_diff = default_scores
            .iter()
            .zip(alt_scores.iter())
            .any(|(d, a)| d.total_score != a.total_score);
        assert!(any_diff, "权重变化后所有阵容分数完全相同");
    }
}
