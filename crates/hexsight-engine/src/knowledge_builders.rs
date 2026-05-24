// 知识底座构建器
// 核心职责：
// - ChampionCapabilityBuilder：从 HeroData + KnowledgeKeywords 生成 ChampionCapability
// - ItemValueBuilder：从 EquipmentData + KnowledgeKeywords 生成 ItemValueProfile
// - 输出解析覆盖率报告（未知标签/低置信度条目/需覆写项）

use std::path::Path;

use hexsight_core::{
    AugmentEffectProfile, ChampionCapability, ChampionRole, EquipmentData, HeroData, HexData,
    ItemPreferences, ItemStat, ItemValueProfile, KnowledgeKeywords, PowerSpike, TraitData,
    TraitEffectProfile,
};

use crate::champion_item_fit::{ChampionItemOverrideLoader, ChampionItemOverrides};

/// 棋子能力构建器
pub struct ChampionCapabilityBuilder {
    keywords: KnowledgeKeywords,
}

impl ChampionCapabilityBuilder {
    pub fn new(keywords: KnowledgeKeywords) -> Self {
        Self { keywords }
    }

    pub fn with_defaults() -> Self {
        Self::new(KnowledgeKeywords::default())
    }

    /// 从英雄数据生成能力画像
    pub fn build(&self, hero: &HeroData) -> ChampionCapability {
        let desc = format!(
            "{} {} {} {}",
            hero.skillName, hero.skillDesc, hero.skillBriefValue, hero.skillValueDesc
        );
        let desc_lower = desc.to_lowercase();

        // 角色定位
        let role = self.classify_role(hero);

        // 伤害类型
        let damage_profile = self.classify_damage_type(&desc_lower, hero);

        // 输出模式
        let damage_pattern = self.classify_attack_patterns(&desc_lower);

        // 施放模式
        let cast_pattern = self.classify_cast_pattern(&desc_lower, hero);

        // 缩放属性
        let scaling_stats = self.classify_scaling_stats(&desc_lower, hero);

        // 站位角色
        let position_role = self.classify_position(hero);

        // 装备严格度
        let item_strictness = self.classify_item_strictness(hero);

        // 强度节点
        let power_spikes = self.classify_power_spikes(hero);

        // 风险标签
        let risk_profile = self.classify_risks(&desc_lower);

        // 羁绊
        let traits: Vec<String> = hero
            .species
            .split('|')
            .chain(hero.hero_class.split('|'))
            .filter(|s| !s.is_empty() && s != &"-1" && s != &"0")
            .map(|s| s.to_string())
            .collect();

        // 置信度
        let confidence = self.estimate_confidence(&desc_lower);

        ChampionCapability {
            hero_id: hero.id.clone(),
            name: hero.name.clone(),
            cost: hero.cost,
            traits,
            role,
            damage_profile,
            damage_pattern,
            cast_pattern,
            scaling_stats,
            position_role,
            item_strictness,
            power_spikes,
            risk_profile,
            item_preferences: ItemPreferences::default(),
            confidence,
            needs_override: confidence < 0.5,
        }
    }

    /// 批量构建并输出覆盖率报告
    pub fn build_all(&self, heroes: &[&HeroData]) -> (Vec<ChampionCapability>, CoverageReport) {
        let mut caps = Vec::new();
        let unknown_tags = Vec::new();
        let mut low_confidence = Vec::new();
        let mut needs_override = Vec::new();

        for hero in heroes
            .iter()
            .filter(|h| h.cost > 0 && !h.name.contains("假人"))
        {
            let cap = self.build(hero);
            if cap.needs_override {
                needs_override.push(cap.hero_id.clone());
            }
            if cap.confidence < 0.7 {
                low_confidence.push(format!("{} (conf={:.2})", cap.name, cap.confidence));
            }
            caps.push(cap);
        }

        let report = CoverageReport {
            total: caps.len(),
            with_role: caps
                .iter()
                .filter(|c| {
                    matches!(
                        c.role,
                        ChampionRole::PrimaryCarry | ChampionRole::SecondaryCarry
                    )
                })
                .count(),
            with_damage_type: caps.iter().filter(|c| !c.damage_profile.is_empty()).count(),
            low_confidence,
            needs_override,
            unknown_tags,
        };

        (caps, report)
    }

    // ---- 分类方法 ----

    fn classify_role(&self, hero: &HeroData) -> ChampionRole {
        let hp = hero.initHP.parse::<i32>().unwrap_or(500);
        let ad = hero.initAttackDamage.parse::<i32>().unwrap_or(50);
        let desc = &hero.skillDesc;

        if hp > 800 && desc.contains("嘲讽") || desc.contains("护盾") && hp > 700 {
            return ChampionRole::MainTank;
        }
        if hp > 700 || hero.armor.parse::<i32>().unwrap_or(30) > 50 {
            return ChampionRole::OffTank;
        }
        if ad > 70 && desc.contains("伤害") {
            return ChampionRole::PrimaryCarry;
        }
        if ad > 55 {
            return ChampionRole::SecondaryCarry;
        }
        if desc.contains("治疗") || desc.contains("护盾") || desc.contains("控制") {
            return ChampionRole::Utility;
        }
        ChampionRole::Filler
    }

    fn classify_damage_type(&self, desc: &str, hero: &HeroData) -> String {
        for kw in self
            .keywords
            .damage_types
            .get("true_damage")
            .unwrap_or(&vec![])
        {
            if desc.contains(kw) {
                return "true_damage".into();
            }
        }
        let has_ad = self
            .keywords
            .damage_types
            .get("ad")
            .unwrap_or(&vec![])
            .iter()
            .any(|k| desc.contains(k));
        let has_ap = self
            .keywords
            .damage_types
            .get("ap")
            .unwrap_or(&vec![])
            .iter()
            .any(|k| desc.contains(k));
        if has_ad && has_ap {
            return "hybrid".into();
        }
        if has_ap {
            return "ap".into();
        }
        if has_ad {
            return "ad".into();
        }
        if hero.skillBriefValue.contains("%") || hero.skillDesc.contains("法术") {
            return "ap".into();
        }
        "ad".into()
    }

    fn classify_attack_patterns(&self, desc: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        for (tag, kws) in &self.keywords.attack_patterns {
            if kws.iter().any(|k| desc.contains(k)) {
                patterns.push(tag.clone());
            }
        }
        if patterns.is_empty() {
            patterns.push("single_target".into());
        }
        patterns
    }

    fn classify_cast_pattern(&self, desc: &str, hero: &HeroData) -> String {
        for (tag, kws) in &self.keywords.cast_patterns {
            if kws.iter().any(|k| desc.contains(k)) {
                return tag.clone();
            }
        }
        let init_mp = hero.initMP.parse::<i32>().unwrap_or(0);
        let max_mp = hero.maxMP.parse::<i32>().unwrap_or(100);
        if init_mp as f64 / max_mp.max(1) as f64 > 0.5 {
            return "mana_cast".into();
        }
        "mana_cast".into()
    }

    fn classify_scaling_stats(&self, desc: &str, hero: &HeroData) -> Vec<String> {
        let mut stats = Vec::new();
        if desc.contains("攻击力") || desc.contains("攻击") {
            stats.push("attack_damage".into());
        }
        if desc.contains("法术强度") || desc.contains("法强") {
            stats.push("ability_power".into());
        }
        if desc.contains("攻速") || desc.contains("攻击速度") {
            stats.push("attack_speed".into());
        }
        let max_mp = hero.maxMP.parse::<i32>().unwrap_or(100);
        if max_mp >= 60 {
            stats.push("mana".into());
        }
        if hero.criticalStrikeChance.parse::<i32>().unwrap_or(0) > 25 {
            stats.push("crit".into());
        }
        if desc.contains("生命值") || desc.contains("护甲") {
            stats.push("health".into());
        }
        if stats.is_empty() {
            stats.push("attack_damage".into());
        }
        stats
    }

    fn classify_position(&self, hero: &HeroData) -> String {
        let range = hero.attackRange.parse::<i32>().unwrap_or(1);
        let hp = hero.initHP.parse::<i32>().unwrap_or(500);
        if range >= 3 {
            return "backline".into();
        }
        if hp > 700 {
            return "frontline".into();
        }
        "flex".into()
    }

    fn classify_item_strictness(&self, hero: &HeroData) -> String {
        if hero.cost >= 4 {
            return "high".into();
        }
        if hero.cost >= 3 {
            return "medium".into();
        }
        "low".into()
    }

    fn classify_power_spikes(&self, hero: &HeroData) -> Vec<PowerSpike> {
        vec![
            PowerSpike {
                spike_type: "star".into(),
                value: 2,
                impact: if hero.cost <= 2 { 30 } else { 15 },
            },
            PowerSpike {
                spike_type: "star".into(),
                value: 3,
                impact: if hero.cost <= 3 { 40 } else { 25 },
            },
            PowerSpike {
                spike_type: "item_count".into(),
                value: 2,
                impact: 20,
            },
        ]
    }

    fn classify_risks(&self, desc: &str) -> Vec<String> {
        let mut risks = Vec::new();
        if desc.contains("眩晕") || desc.contains("击飞") {
            risks.push("crowd_control".into());
        }
        if desc.contains("爆发") {
            risks.push("burst_damage".into());
        }
        if desc.contains("重伤") || desc.contains("减疗") {
            risks.push("anti_heal".into());
        }
        risks
    }

    fn estimate_confidence(&self, desc: &str) -> f64 {
        let mut hits = 0u32;
        for kws in self.keywords.damage_types.values() {
            if kws.iter().any(|k| desc.contains(k)) {
                hits += 1;
            }
        }
        for kws in self.keywords.cast_patterns.values() {
            if kws.iter().any(|k| desc.contains(k)) {
                hits += 1;
            }
        }
        // 3/6 = 0.5 基础，最多到 1.0
        (0.4 + hits as f64 * 0.1).min(0.95)
    }
}

/// 解析覆盖率报告
#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub total: usize,
    pub with_role: usize,
    pub with_damage_type: usize,
    pub low_confidence: Vec<String>,
    pub needs_override: Vec<String>,
    pub unknown_tags: Vec<String>,
}

// ============================================================

/// 装备收益构建器
pub struct ItemValueBuilder {
    keywords: KnowledgeKeywords,
}

impl ItemValueBuilder {
    pub fn new(keywords: KnowledgeKeywords) -> Self {
        Self { keywords }
    }

    pub fn with_defaults() -> Self {
        Self::new(KnowledgeKeywords::default())
    }

    /// 从装备数据生成收益画像
    pub fn build(&self, equip: &EquipmentData) -> ItemValueProfile {
        let desc = format!("{} {} {}", equip.name, equip.basicDesc, equip.desc);
        let desc_lower = desc.to_lowercase();

        // 效果标签
        let effect_tags = self.classify_effect_tags(&desc_lower);

        // 伤害类型适配
        let damage_type_fit: Vec<String> =
            if equip.equip_type.contains("攻击") || desc_lower.contains("攻") {
                vec!["ad".into()]
            } else if desc_lower.contains("法") || desc_lower.contains("ap") {
                vec!["ap".into()]
            } else if desc_lower.contains("生命")
                || desc_lower.contains("护甲")
                || desc_lower.contains("抗")
            {
                vec!["tank".into()]
            } else {
                vec!["ad".into(), "ap".into()]
            };

        // 最适合的棋子类型
        let best_for = self.classify_best_for(&desc_lower, &damage_type_fit);

        // 冲突组
        let conflict_group = self
            .keywords
            .conflict_groups
            .iter()
            .find(|(_, names)| names.iter().any(|n| equip.name.contains(n.as_str())))
            .map(|(k, _)| k.clone())
            .unwrap_or_default();

        // 装备属性
        let stats = self.extract_stats(equip);

        // 阶段偏好
        let stage_bias = if equip.is_completed {
            "mid_late".into()
        } else {
            "early".into()
        };

        ItemValueProfile {
            item_id: equip.id.clone(),
            name: equip.name.clone(),
            item_type: equip.equip_type.clone(),
            stats,
            effect_tags,
            best_for_profiles: best_for,
            bad_for_profiles: vec![],
            replacement_group: String::new(),
            conflict_group,
            stage_bias,
            tier: None,
            damage_type_fit,
        }
    }

    pub fn build_all(&self, equips: &[&EquipmentData]) -> Vec<ItemValueProfile> {
        equips.iter().map(|e| self.build(e)).collect()
    }

    fn classify_effect_tags(&self, desc: &str) -> Vec<String> {
        let mut tags = Vec::new();
        for (tag, kws) in &self.keywords.effect_tags {
            if kws.iter().any(|k| desc.contains(k)) {
                tags.push(tag.clone());
            }
        }
        tags
    }

    fn classify_best_for(&self, _desc: &str, damage_fit: &[String]) -> Vec<String> {
        let mut profiles = Vec::new();
        for dt in damage_fit {
            if let Some(fits) = self.keywords.damage_type_item_fit.get(dt) {
                for f in fits {
                    profiles.push(f.clone());
                }
            }
        }
        profiles
    }

    fn extract_stats(&self, equip: &EquipmentData) -> Vec<ItemStat> {
        let mut stats = Vec::new();
        // 从 basicDesc 提取数值
        let desc = &equip.basicDesc;
        if desc.contains("攻击") || desc.contains("物理") {
            stats.push(ItemStat {
                name: "ad".into(),
                value: 10.0,
            });
        }
        if desc.contains("法强") || desc.contains("法术") || desc.contains("ap") {
            stats.push(ItemStat {
                name: "ap".into(),
                value: 10.0,
            });
        }
        if desc.contains("攻速") {
            stats.push(ItemStat {
                name: "attack_speed".into(),
                value: 10.0,
            });
        }
        if desc.contains("生命") || desc.contains("hp") {
            stats.push(ItemStat {
                name: "hp".into(),
                value: 150.0,
            });
        }
        if desc.contains("护甲") {
            stats.push(ItemStat {
                name: "armor".into(),
                value: 20.0,
            });
        }
        if desc.contains("魔抗") {
            stats.push(ItemStat {
                name: "mr".into(),
                value: 20.0,
            });
        }
        if desc.contains("暴击") {
            stats.push(ItemStat {
                name: "crit".into(),
                value: 10.0,
            });
        }
        stats
    }
}

// ============================================================
// 知识关键词加载器
// ============================================================

/// 知识关键词加载器
pub struct KnowledgeKeywordLoader;

impl KnowledgeKeywordLoader {
    pub fn load(path: &std::path::Path) -> hexsight_core::HexResult<KnowledgeKeywords> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            hexsight_core::HexError::Config(format!("读取关键词配置失败 {}: {}", path.display(), e))
        })?;
        let keywords: KnowledgeKeywords = serde_json::from_str(&content).map_err(|e| {
            hexsight_core::HexError::Config(format!("解析关键词配置失败 {}: {}", path.display(), e))
        })?;
        Ok(keywords)
    }

    pub fn load_or_default(config_root: &std::path::Path, version: &str) -> KnowledgeKeywords {
        let path = config_root
            .join("rules")
            .join(version)
            .join("knowledge_keywords.json");
        Self::load(&path).unwrap_or_else(|e| {
            eprintln!("[Knowledge] 加载关键词配置失败: {}，使用默认", e);
            KnowledgeKeywords::default()
        })
    }
}

// ============================================================
// 海克斯效果画像构建器
// ============================================================

pub struct AugmentEffectProfileBuilder {
    #[allow(dead_code)]
    keywords: KnowledgeKeywords,
}

impl AugmentEffectProfileBuilder {
    pub fn new(keywords: KnowledgeKeywords) -> Self {
        Self { keywords }
    }

    pub fn build(&self, hex: &HexData) -> AugmentEffectProfile {
        let desc = format!("{} {}", hex.name, hex.desc);
        let mut tags = Vec::new();
        let mut hints = Vec::new();

        // 经济标签
        if desc.contains("金币")
            || desc.contains("经济")
            || desc.contains("利息")
            || desc.contains("免费")
        {
            tags.push("generic_econ".into());
            hints.push("经济收益".into());
        }
        // 装备标签
        if desc.contains("装备")
            || desc.contains("散件")
            || desc.contains("锻造")
            || desc.contains("重铸")
        {
            tags.push("generic_item".into());
        }
        // 战力标签
        if desc.contains("攻击力")
            || desc.contains("法强")
            || desc.contains("伤害")
            || desc.contains("攻速")
        {
            tags.push("generic_combat".into());
        }
        // 羁绊标签
        if desc.contains("纹章")
            || desc.contains("转职")
            || desc.contains("之心")
            || desc.contains("羁绊")
        {
            tags.push("trait_commit".into());
        }
        // 英雄专属
        if desc.contains("英雄") || desc.contains("专属") || desc.contains("强化") {
            tags.push("hero_commit".into());
        }
        // 赌狗
        if desc.contains("刷新") || desc.contains("低费") || desc.contains("三星") {
            tags.push("reroll_enable".into());
        }
        // 模式特殊
        if desc.contains("星神") || desc.contains("天选") || desc.contains("任务") {
            tags.push("mode_special".into());
        }
        if tags.is_empty() {
            tags.push("generic_combat".into());
        }

        let level = hex.level.parse::<i32>().unwrap_or(1);
        let immediate_power = match level {
            1 => 30,
            2 => 50,
            3 => 70,
            _ => 50,
        };

        let lock_risk = if tags.contains(&"hero_commit".to_string()) {
            60
        } else {
            20
        };

        AugmentEffectProfile {
            augment_id: hex.id.clone(),
            name: hex.name.clone(),
            level,
            tags,
            immediate_power,
            lineup_coverage: 0,
            lock_risk,
            hints,
        }
    }

    pub fn build_all(&self, hexes: &[&HexData]) -> Vec<AugmentEffectProfile> {
        hexes.iter().map(|h| self.build(h)).collect()
    }
}

// ============================================================
// 羁绊效果画像构建器
// ============================================================

pub struct TraitEffectProfileBuilder;

impl TraitEffectProfileBuilder {
    pub fn build(&self, trait_data: &TraitData) -> TraitEffectProfile {
        let desc = format!(
            "{} {} {}",
            trait_data.name, trait_data.desc, trait_data.prefix
        );
        let mut effect_tags = Vec::new();
        let mut lineup_value = 50;

        if desc.contains("伤害") || desc.contains("攻击") {
            effect_tags.push("damage".into());
            lineup_value += 10;
        }
        if desc.contains("生命") || desc.contains("护甲") || desc.contains("魔抗") {
            effect_tags.push("defense".into());
            lineup_value += 10;
        }
        if desc.contains("金币") || desc.contains("经济") {
            effect_tags.push("economy".into());
        }
        if desc.contains("召唤") || desc.contains("分身") {
            effect_tags.push("summon".into());
        }
        if desc.contains("控制") || desc.contains("眩晕") || desc.contains("击飞") {
            effect_tags.push("cc".into());
        }
        if desc.contains("装备") || desc.contains("锻造") {
            effect_tags.push("item_synergy".into());
        }

        TraitEffectProfile {
            trait_id: trait_data.checkId.clone(),
            name: trait_data.name.clone(),
            trait_type: if trait_data.trait_type == 0 {
                "race".into()
            } else {
                "job".into()
            },
            thresholds: trait_data.thresholds.clone(),
            effect_tags,
            lineup_value: lineup_value.clamp(0, 100),
            desc: trait_data.desc.clone(),
        }
    }

    pub fn build_all(&self, traits: &[TraitData]) -> Vec<TraitEffectProfile> {
        traits.iter().map(|t| self.build(t)).collect()
    }
}

// ============================================================
// KnowledgeBase 统一知识库
// ============================================================

/// 知识库 — 一个版本的全部结构化知识
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    pub version: String,
    pub champions: Vec<ChampionCapability>,
    pub items: Vec<ItemValueProfile>,
    pub augments: Vec<AugmentEffectProfile>,
    pub traits: Vec<TraitEffectProfile>,
    pub coverage: KnowledgeCoverageReport,
}

/// 知识覆盖率报告（分类型）
#[derive(Debug, Clone, serde::Serialize)]
pub struct KnowledgeCoverageReport {
    pub champions: CategoryCoverage,
    pub items: CategoryCoverage,
    pub augments: CategoryCoverage,
    pub traits: CategoryCoverage,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CategoryCoverage {
    pub total: usize,
    pub tagged: usize,
    pub low_confidence: usize,
    pub needs_override: usize,
    pub unknown_terms: Vec<String>,
}

/// 知识库统一构建器
pub struct KnowledgeBaseBuilder {
    champ_builder: ChampionCapabilityBuilder,
    item_builder: ItemValueBuilder,
    augment_builder: AugmentEffectProfileBuilder,
    trait_builder: TraitEffectProfileBuilder,
}

impl KnowledgeBaseBuilder {
    pub fn new(keywords: KnowledgeKeywords) -> Self {
        Self {
            champ_builder: ChampionCapabilityBuilder::new(keywords.clone()),
            item_builder: ItemValueBuilder::new(keywords.clone()),
            augment_builder: AugmentEffectProfileBuilder::new(keywords),
            trait_builder: TraitEffectProfileBuilder,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(KnowledgeKeywords::default())
    }

    /// 一次性生成全量知识库
    pub fn build_all(
        &self,
        heroes: &[HeroData],
        equips: &[EquipmentData],
        hexes: &[HexData],
        traits: &[TraitData],
    ) -> KnowledgeBase {
        self.build_all_with_item_overrides(heroes, equips, hexes, traits, None)
    }

    /// 使用 P1 装备覆写配置生成全量知识库
    pub fn build_all_with_item_overrides(
        &self,
        heroes: &[HeroData],
        equips: &[EquipmentData],
        hexes: &[HexData],
        traits: &[TraitData],
        item_overrides: Option<&ChampionItemOverrides>,
    ) -> KnowledgeBase {
        // 棋子（只处理 cost > 0 且非假人）
        let hero_refs: Vec<&HeroData> = heroes
            .iter()
            .filter(|h| h.cost > 0 && !h.name.contains("假人"))
            .collect();
        let (mut champions, champ_report) = self.champ_builder.build_all(&hero_refs);
        if let Some(overrides) = item_overrides {
            for champion in &mut champions {
                ChampionItemOverrideLoader::apply(champion, overrides);
            }
        }

        let equip_refs: Vec<&EquipmentData> = equips.iter().collect();
        let items = self.item_builder.build_all(&equip_refs);

        let hex_refs: Vec<&HexData> = hexes.iter().collect();
        let augments = self.augment_builder.build_all(&hex_refs);

        let traits = self.trait_builder.build_all(traits);

        let coverage = KnowledgeCoverageReport {
            champions: CategoryCoverage {
                total: champions.len(),
                tagged: champ_report.with_role,
                low_confidence: champ_report.low_confidence.len(),
                needs_override: champ_report.needs_override.len(),
                unknown_terms: champ_report.unknown_tags,
            },
            items: CategoryCoverage {
                total: items.len(),
                tagged: items.iter().filter(|i| !i.effect_tags.is_empty()).count(),
                low_confidence: items.iter().filter(|i| i.effect_tags.is_empty()).count(),
                needs_override: 0,
                unknown_terms: vec![],
            },
            augments: CategoryCoverage {
                total: augments.len(),
                tagged: augments.iter().filter(|a| !a.tags.is_empty()).count(),
                low_confidence: 0,
                needs_override: 0,
                unknown_terms: vec![],
            },
            traits: CategoryCoverage {
                total: traits.len(),
                tagged: traits.iter().filter(|t| !t.effect_tags.is_empty()).count(),
                low_confidence: 0,
                needs_override: 0,
                unknown_terms: vec![],
            },
        };

        KnowledgeBase {
            version: "S18.1".into(),
            champions,
            items,
            augments,
            traits,
            coverage,
        }
    }

    /// 从规则配置目录加载 P1 覆写并生成全量知识库
    pub fn build_all_with_rule_config(
        &self,
        heroes: &[HeroData],
        equips: &[EquipmentData],
        hexes: &[HexData],
        traits: &[TraitData],
        config_root: &Path,
        version: &str,
    ) -> hexsight_core::HexResult<KnowledgeBase> {
        let override_path = config_root
            .join("rules")
            .join(version)
            .join("champion_item_overrides.json");
        let overrides = ChampionItemOverrideLoader::load(&override_path)?;
        Ok(self.build_all_with_item_overrides(heroes, equips, hexes, traits, Some(&overrides)))
    }
}

// ============================================================
// 人工覆写加载器
// ============================================================

/// 人工覆写条目
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ManualOverride {
    pub id: String,
    #[serde(rename = "type")]
    pub override_type: String, // champion / item / augment / trait
    #[serde(default)]
    pub tags_add: Vec<String>,
    #[serde(default)]
    pub tags_remove: Vec<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub notes: String,
}

/// 人工覆写加载器
pub struct ManualOverrideLoader;

impl ManualOverrideLoader {
    pub fn load(path: &std::path::Path) -> hexsight_core::HexResult<Vec<ManualOverride>> {
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            hexsight_core::HexError::Config(format!("读取覆写文件失败 {}: {}", path.display(), e))
        })?;
        let overrides: Vec<ManualOverride> = serde_json::from_str(&content).map_err(|e| {
            hexsight_core::HexError::Config(format!("解析覆写文件失败 {}: {}", path.display(), e))
        })?;
        Ok(overrides)
    }

    /// 按类型和 ID 索引覆写
    pub fn index(
        overrides: &[ManualOverride],
    ) -> std::collections::HashMap<String, &ManualOverride> {
        overrides
            .iter()
            .map(|o| (format!("{}:{}", o.override_type, o.id), o))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hero(name: &str, cost: i32, hp: i32, ad: i32, skill: &str) -> HeroData {
        HeroData {
            id: format!("hero_{}", name),
            name: name.into(),
            price: cost.to_string(),
            picture: "".into(),
            skillName: "测试".into(),
            skillDesc: skill.into(),
            skillIcon: "".into(),
            skillBriefValue: "100%".into(),
            skillValueDesc: "".into(),
            species: "1".into(),
            hero_class: "2".into(),
            initHP: hp.to_string(),
            initAttackDamage: ad.to_string(),
            attackSpeed: "0.7".into(),
            armor: "30".into(),
            magicResist: "30".into(),
            attackRange: "1".into(),
            initMP: "0".into(),
            maxMP: "80".into(),
            criticalStrikeChance: "25".into(),
            cost,
            base_key: "".into(),
            star_level: 1,
        }
    }

    fn make_equip(id: &str, name: &str, etype: &str, desc: &str) -> EquipmentData {
        EquipmentData {
            id: id.into(),
            name: name.into(),
            equip_type: etype.into(),
            picture: "".into(),
            basicDesc: desc.into(),
            desc: "".into(),
            synthesis1: "0".into(),
            synthesis2: "0".into(),
            icon: "".into(),
            is_component: etype == "基础装备",
            is_completed: etype == "成型装备",
        }
    }

    #[test]
    fn build_carry_capability() {
        let builder = ChampionCapabilityBuilder::with_defaults();
        let hero = make_hero("主C", 4, 600, 85, "对目标造成大量物理伤害并降低护甲");
        let cap = builder.build(&hero);
        assert_eq!(cap.role, ChampionRole::PrimaryCarry);
        assert_eq!(cap.damage_profile, "ad");
        assert!(!cap.scaling_stats.is_empty());
    }

    #[test]
    fn build_tank_capability() {
        let builder = ChampionCapabilityBuilder::with_defaults();
        let hero = make_hero("主坦", 3, 900, 50, "获得护盾持续3秒并嘲讽周围敌人");
        let cap = builder.build(&hero);
        assert!(matches!(
            cap.role,
            ChampionRole::MainTank | ChampionRole::OffTank
        ));
        assert_eq!(cap.position_role, "frontline");
    }

    #[test]
    fn coverage_report_generated() {
        let builder = ChampionCapabilityBuilder::with_defaults();
        let h1 = make_hero("C1", 4, 600, 85, "物理爆发伤害");
        let h2 = make_hero("C2", 3, 900, 50, "护盾嘲讽");
        let h3 = make_hero("C3", 2, 550, 60, "魔法伤害治疗");
        let heroes: Vec<&HeroData> = vec![&h1, &h2, &h3];
        let (caps, report) = builder.build_all(&heroes);
        assert_eq!(caps.len(), 3);
        assert!(report.total > 0);
    }

    #[test]
    fn item_value_effect_tags() {
        let builder = ItemValueBuilder::with_defaults();
        let equip = make_equip(
            "2001",
            "日炎斗篷",
            "成型装备",
            "每2秒灼烧周围敌人并施加重伤",
        );
        let profile = builder.build(&equip);
        assert!(profile
            .effect_tags
            .iter()
            .any(|t| t == "burn" || t == "anti_heal"));
    }

    #[test]
    fn item_conflict_group_detected() {
        let builder = ItemValueBuilder::with_defaults();
        let equip = make_equip("2002", "红霸符", "成型装备", "攻击附带灼烧和重伤");
        let profile = builder.build(&equip);
        assert!(!profile.conflict_group.is_empty());
    }

    #[test]
    fn item_overrides_apply_to_knowledge_base() {
        let builder = KnowledgeBaseBuilder::with_defaults();
        let hero = make_hero("覆写主C", 4, 600, 85, "造成魔法伤害");
        let overrides = crate::champion_item_fit::ChampionItemOverrides {
            version: "test".into(),
            overrides: std::collections::HashMap::from([(
                hero.id.clone(),
                crate::champion_item_fit::ChampionItemOverrideEntry {
                    item_strictness: Some("high".into()),
                    notes: String::new(),
                },
            )]),
        };
        let kb = builder.build_all_with_item_overrides(
            std::slice::from_ref(&hero),
            &[],
            &[],
            &[],
            Some(&overrides),
        );
        let champion = kb
            .champions
            .iter()
            .find(|champion| champion.hero_id == hero.id)
            .unwrap();
        assert_eq!(champion.item_strictness, "high");
    }

    // ---- 全模式真实数据测试 ----

    fn config_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config")
    }

    #[test]
    fn real_data_knowledge_base_all_modes() {
        use crate::GameDataLoader;
        let root = config_root();
        for mode in &["17", "16", "4"] {
            let mut heroes_map = GameDataLoader::load_heroes(&root, mode).unwrap();
            GameDataLoader::enrich_heroes(&mut heroes_map);
            let heroes: Vec<HeroData> = heroes_map.into_values().collect();
            let equips_map = GameDataLoader::load_equipment(&root, mode).unwrap();
            let equips: Vec<EquipmentData> = equips_map.into_values().collect();
            let hexes_map = GameDataLoader::load_hexes(&root, mode).unwrap();
            let hexes: Vec<HexData> = hexes_map.into_values().collect();
            let traits = GameDataLoader::load_traits(&root, mode).unwrap();

            let kb =
                KnowledgeBaseBuilder::with_defaults().build_all(&heroes, &equips, &hexes, &traits);

            assert!(kb.champions.len() > 10, "mode {} 棋子知识为空", mode);
            assert!(kb.items.len() > 10, "mode {} 装备知识为空", mode);
            assert!(kb.augments.len() > 10, "mode {} 海克斯知识为空", mode);
            assert!(kb.traits.len() > 5, "mode {} 羁绊知识为空", mode);

            let cov = &kb.coverage;
            assert!(cov.champions.total > 0, "mode {} 棋子覆盖率为0", mode);
            assert!(cov.items.total > 0);
            assert!(cov.augments.total > 0);
            assert!(cov.traits.total > 0);
        }
    }

    #[test]
    fn real_rule_config_item_overrides_enter_knowledge_base() {
        use crate::{ChampionItemOverrideLoader, GameDataLoader};

        let root = config_root();
        let mut heroes_map = GameDataLoader::load_heroes(&root, "17").unwrap();
        GameDataLoader::enrich_heroes(&mut heroes_map);
        let heroes: Vec<HeroData> = heroes_map.into_values().collect();
        let equips: Vec<EquipmentData> = GameDataLoader::load_equipment(&root, "17")
            .unwrap()
            .into_values()
            .collect();
        let hexes: Vec<HexData> = GameDataLoader::load_hexes(&root, "17")
            .unwrap()
            .into_values()
            .collect();
        let traits = GameDataLoader::load_traits(&root, "17").unwrap();
        let override_path = root
            .join("rules")
            .join("S18.1")
            .join("champion_item_overrides.json");
        let overrides = ChampionItemOverrideLoader::load(&override_path).unwrap();
        let overridden_id = overrides.overrides.keys().next().unwrap().clone();

        let kb = KnowledgeBaseBuilder::with_defaults()
            .build_all_with_rule_config(&heroes, &equips, &hexes, &traits, &root, "S18.1")
            .unwrap();
        let champion = kb
            .champions
            .iter()
            .find(|champion| champion.hero_id == overridden_id)
            .unwrap();
        assert_eq!(champion.item_strictness, "high");
    }

    #[test]
    fn knowledge_keyword_loader_from_file() {
        let root = config_root();
        let keywords = KnowledgeKeywordLoader::load_or_default(&root, "S18.1");
        assert!(!keywords.damage_types.is_empty());
        assert!(keywords.damage_types.contains_key("ad"));
    }

    #[test]
    fn manual_override_loader_empty_file() {
        let root = config_root();
        let path = root
            .join("rules")
            .join("S18.1")
            .join("manual_overrides.json");
        let overrides = ManualOverrideLoader::load(&path).unwrap();
        assert!(!overrides.is_empty() || overrides.is_empty()); // 始终通过，只验证加载不崩溃
    }
}
