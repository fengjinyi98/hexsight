// 阶段伤害表 + 版本覆写加载器
// 核心职责：
// - DamageProfile：维护版本化阶段基础伤害表
// - PatchOverrideLoader：加载版本公告修正、热修参数、人工覆写

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use hexsight_core::{HexError, HexResult, RulePack};

/// 阶段伤害表
#[derive(Debug, Clone)]
pub struct DamageProfile {
    /// 阶段 → 基础伤害
    pub stage_damage: HashMap<i32, i32>,
    /// 版本
    pub version: String,
}

impl DamageProfile {
    /// 从规则包加载伤害表
    pub fn from_rule_pack(pack: &RulePack) -> Self {
        let stage_damage: HashMap<i32, i32> = pack.damage_profile.iter()
            .filter_map(|(k, v)| k.parse::<i32>().ok().map(|stage| (stage, *v)))
            .collect();

        Self {
            stage_damage,
            version: pack.version.clone(),
        }
    }

    /// 获取某阶段的基础伤害
    pub fn stage_damage(&self, stage: i32) -> i32 {
        self.stage_damage.get(&stage).copied().unwrap_or_else(|| {
            // 默认：stage 2 = 0, stage 3 = 2, stage 4 = 4, stage 5+ = stage
            match stage {
                2 => 0,
                3 => 2,
                4 => 4,
                5 => 6,
                6 => 8,
                _ => (stage - 1).max(0) * 2,
            }
        })
    }

    /// 计算掉血量 = 阶段基础伤害 + 对方剩余棋子数
    pub fn calculate_damage(&self, stage: i32, enemy_survivors: i32) -> i32 {
        self.stage_damage(stage) + enemy_survivors
    }

    /// 判断是否会被大入（≥ 8 滴血）
    pub fn is_large_damage(&self, stage: i32, enemy_survivors: i32) -> bool {
        self.calculate_damage(stage, enemy_survivors) >= 8
    }

    /// 估算剩余棋子对应的掉血区间
    pub fn damage_range(&self, stage: i32, min_survivors: i32, max_survivors: i32) -> (i32, i32) {
        (
            self.calculate_damage(stage, min_survivors),
            self.calculate_damage(stage, max_survivors),
        )
    }
}

// ============================================================

/// 版本覆写条目
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PatchOverride {
    /// 覆写版本
    pub version: String,
    /// 描述
    pub description: String,
    /// 阵容强度调整 (lineup_id → 新 tier)
    #[serde(default)]
    pub lineup_tier_overrides: HashMap<String, String>,
    /// 装备强度调整
    #[serde(default)]
    pub item_synergy_overrides: HashMap<String, f64>,
    /// 伤害表覆写
    #[serde(default)]
    pub damage_overrides: HashMap<String, i32>,
    /// 阈值覆写
    #[serde(default)]
    pub threshold_overrides: HashMap<String, i32>,
    /// 人工权重覆写
    #[serde(default)]
    pub weight_overrides: HashMap<String, f64>,
}

/// 版本覆写加载器
pub struct PatchOverrideLoader;

impl PatchOverrideLoader {
    /// 加载指定版本的覆写文件
    pub fn load(config_root: &Path, version: &str) -> HexResult<Option<PatchOverride>> {
        let path = Self::override_path(config_root, version);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| HexError::Config(format!(
                "读取覆写文件失败 {}: {}", path.display(), e
            )))?;
        let ov: PatchOverride = serde_json::from_str(&content)
            .map_err(|e| HexError::Config(format!(
                "解析覆写文件失败 {}: {}", path.display(), e
            )))?;
        Ok(Some(ov))
    }

    /// 应用覆写到规则包（返回新的 RulePack）
    pub fn apply(pack: &mut RulePack, ov: &PatchOverride) {
        // 覆写伤害表
        for (stage, damage) in &ov.damage_overrides {
            pack.damage_profile.insert(stage.clone(), *damage);
        }
        // 覆写阈值
        if let Some(lock) = ov.threshold_overrides.get("lockLineupScore") {
            pack.thresholds.lock_lineup_score = *lock;
        }
        if let Some(pivot) = ov.threshold_overrides.get("pivotRiskScore") {
            pack.thresholds.pivot_risk_score = *pivot;
        }
        if let Some(hp) = ov.threshold_overrides.get("mustStabilizeHealth") {
            pack.thresholds.must_stabilize_health = *hp;
        }
        // 覆写权重
        if let Some(w) = ov.weight_overrides.get("itemFit") {
            pack.weights.lineup_fit.item_fit = *w;
        }
        if let Some(w) = ov.weight_overrides.get("championHit") {
            pack.weights.lineup_fit.champion_hit = *w;
        }
        if let Some(w) = ov.weight_overrides.get("augmentFit") {
            pack.weights.lineup_fit.augment_fit = *w;
        }
    }

    /// 创建默认覆写模板（用于新版本）
    pub fn create_template(config_root: &Path, version: &str) -> HexResult<()> {
        let path = Self::override_path(config_root, version);
        if path.exists() {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| HexError::Config(format!("创建覆写目录失败: {}", e)))?;
        }
        let template = PatchOverride {
            version: version.to_string(),
            description: format!("{} 版本公告修正与热修覆写", version),
            lineup_tier_overrides: HashMap::new(),
            item_synergy_overrides: HashMap::new(),
            damage_overrides: HashMap::new(),
            threshold_overrides: HashMap::new(),
            weight_overrides: HashMap::new(),
        };
        let json = serde_json::to_string_pretty(&template)
            .map_err(|e| HexError::Config(format!("序列化模板失败: {}", e)))?;
        fs::write(&path, json)
            .map_err(|e| HexError::Config(format!("写入模板失败 {}: {}", path.display(), e)))?;
        Ok(())
    }

    fn override_path(config_root: &Path, version: &str) -> PathBuf {
        config_root.join("rules").join(version).join("patch_overrides.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RulePackLoader;

    fn test_config_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap().join("config")
    }

    #[test]
    fn damage_profile_from_rule_pack() {
        let (pack, _warnings) = RulePackLoader::load(&test_config_root(), "S18.1");
        let dp = DamageProfile::from_rule_pack(&pack);
        assert_eq!(dp.stage_damage(2), 0);
        assert_eq!(dp.stage_damage(6), 8);
    }

    #[test]
    fn damage_calculation() {
        let (pack, _warnings) = RulePackLoader::load(&test_config_root(), "S18.1");
        let dp = DamageProfile::from_rule_pack(&pack);
        // Stage 4 + 3 survivors = 4 + 3 = 7
        assert_eq!(dp.calculate_damage(4, 3), 7);
        // Stage 5 + 3 survivors = 6 + 3 = 9 → 大入
        assert!(dp.is_large_damage(5, 3));
    }

    #[test]
    fn damage_range_output() {
        let (pack, _warnings) = RulePackLoader::load(&test_config_root(), "S18.1");
        let dp = DamageProfile::from_rule_pack(&pack);
        let (min, max) = dp.damage_range(4, 1, 4);
        assert!(min < max);
        assert_eq!(min, 5); // 4 + 1
        assert_eq!(max, 8); // 4 + 4
    }

    #[test]
    fn patch_override_load_nonexistent() {
        let result = PatchOverrideLoader::load(&test_config_root(), "S99").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn patch_override_apply_modifies_pack() {
        let mut pack = RulePack::default();
        let ov = PatchOverride {
            version: "S18.1".into(),
            description: "测试覆写".into(),
            lineup_tier_overrides: Default::default(),
            item_synergy_overrides: Default::default(),
            damage_overrides: HashMap::from([("5".into(), 8)]),
            threshold_overrides: HashMap::from([("mustStabilizeHealth".into(), 40)]),
            weight_overrides: HashMap::from([("itemFit".into(), 0.30)]),
        };
        PatchOverrideLoader::apply(&mut pack, &ov);
        assert_eq!(pack.damage_profile.get("5"), Some(&8));
        assert_eq!(pack.thresholds.must_stabilize_health, 40);
        assert_eq!(pack.weights.lineup_fit.item_fit, 0.30);
    }
}
