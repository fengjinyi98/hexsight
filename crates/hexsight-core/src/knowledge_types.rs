// 知识底座核心类型
// 核心职责：
// - ChampionCapability：棋子机制画像（输出方式/技能模式/装备偏好/风险标签）
// - ItemValueProfile：装备收益画像（属性/效果标签/适配类型/替代关系/冲突组）
// - AugmentEffectProfile：海克斯效果画像（类型/战力/阵容覆盖/玩法开关）
// - TraitEffectProfile：羁绊效果画像（即时收益/阵容价值/断点信息）
// - PatchEntry：结构化版本公告改动
// - 所有类型支持 serde，供配置文件和解析器使用

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 棋子能力模型（对齐文档第四章）
// ============================================================

/// 棋子能力画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionCapability {
    #[serde(rename = "heroId")]
    pub hero_id: String,
    pub name: String,
    pub cost: i32,
    /// 种族/职业羁绊 ID 列表
    pub traits: Vec<String>,
    /// 角色定位
    pub role: ChampionRole,
    /// 伤害类型
    #[serde(rename = "damageProfile")]
    pub damage_profile: String,
    /// 输出模式
    #[serde(rename = "damagePattern")]
    pub damage_pattern: Vec<String>,
    /// 施放模式
    #[serde(rename = "castPattern")]
    pub cast_pattern: String,
    /// 核心缩放属性
    #[serde(rename = "scalingStats")]
    pub scaling_stats: Vec<String>,
    /// 站位角色
    #[serde(rename = "positionRole")]
    pub position_role: String,
    /// 装备严格度
    #[serde(rename = "itemStrictness")]
    pub item_strictness: String,
    /// 强度节点
    #[serde(rename = "powerSpikes")]
    pub power_spikes: Vec<PowerSpike>,
    /// 风险标签
    #[serde(rename = "riskProfile")]
    pub risk_profile: Vec<String>,
    /// 装备偏好（来自官网推荐 + 机制推断）
    #[serde(rename = "itemPreferences", default)]
    pub item_preferences: ItemPreferences,
    /// 置信度 (0.0-1.0)，基于关键词匹配 vs 覆写的比例
    pub confidence: f64,
    /// 是否需要人工覆写
    #[serde(rename = "needsOverride")]
    pub needs_override: bool,
}

/// 棋子角色
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChampionRole {
    #[serde(rename = "primary_carry")]
    PrimaryCarry,
    #[serde(rename = "secondary_carry")]
    SecondaryCarry,
    #[serde(rename = "main_tank")]
    MainTank,
    #[serde(rename = "off_tank")]
    OffTank,
    #[serde(rename = "utility")]
    Utility,
    #[serde(rename = "filler")]
    Filler,
}

/// 强度节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSpike {
    #[serde(rename = "type")]
    pub spike_type: String,  // star / item_count / trait
    pub value: i32,
    pub impact: i32,
}

/// 装备偏好分级
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemPreferences {
    pub core: Vec<String>,
    pub strong: Vec<String>,
    pub acceptable: Vec<String>,
    pub emergency: Vec<String>,
    pub bad: Vec<String>,
}

// ============================================================
// 装备收益模型（对齐文档第五章）
// ============================================================

/// 装备收益画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemValueProfile {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub name: String,
    /// 基础装备 / 成型装备 / 神器 / 光明 / 特殊
    #[serde(rename = "type")]
    pub item_type: String,
    /// 属性列表
    pub stats: Vec<ItemStat>,
    /// 效果标签
    #[serde(rename = "effectTags")]
    pub effect_tags: Vec<String>,
    /// 最适合的棋子机制
    #[serde(rename = "bestForProfiles")]
    pub best_for_profiles: Vec<String>,
    /// 低收益的棋子机制
    #[serde(rename = "badForProfiles")]
    pub bad_for_profiles: Vec<String>,
    /// 可替代装备组
    #[serde(rename = "replacementGroup")]
    pub replacement_group: String,
    /// 收益冲突组
    #[serde(rename = "conflictGroup")]
    pub conflict_group: String,
    /// 阶段偏好
    #[serde(rename = "stageBias")]
    pub stage_bias: String,
    /// 装备等级（core/strong/acceptable/emergency/bad）
    pub tier: Option<String>,
    /// 伤害类型适配
    #[serde(rename = "damageTypeFit")]
    pub damage_type_fit: Vec<String>,
}

/// 装备属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStat {
    pub name: String,
    pub value: f64,
}

// ============================================================
// 海克斯效果画像
// ============================================================

/// 海克斯效果画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AugmentEffectProfile {
    #[serde(rename = "augmentId")]
    pub augment_id: String,
    pub name: String,
    pub level: i32,
    /// 类型标签
    pub tags: Vec<String>,
    /// 即时战力
    #[serde(rename = "immediatePower")]
    pub immediate_power: i32,
    /// 阵容覆盖度
    #[serde(rename = "lineupCoverage")]
    pub lineup_coverage: i32,
    /// 锁方向风险
    #[serde(rename = "lockRisk")]
    pub lock_risk: i32,
    /// 策略提示
    pub hints: Vec<String>,
}

// ============================================================
// 羁绊效果画像
// ============================================================

/// 羁绊效果画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitEffectProfile {
    #[serde(rename = "traitId")]
    pub trait_id: String,
    pub name: String,
    /// race / job
    #[serde(rename = "type")]
    pub trait_type: String,
    /// 断点列表
    pub thresholds: Vec<i32>,
    /// 效果标签
    #[serde(rename = "effectTags")]
    pub effect_tags: Vec<String>,
    /// 阵容价值
    #[serde(rename = "lineupValue")]
    pub lineup_value: i32,
    /// 描述
    pub desc: String,
}

// ============================================================
// 版本公告知识
// ============================================================

/// 版本公告改动条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchEntry {
    #[serde(rename = "patchVersion")]
    pub patch_version: String,
    #[serde(rename = "publishedAt", default)]
    pub published_at: String,
    #[serde(rename = "targetType")]
    pub target_type: String,  // champion / trait / item / augment / system
    #[serde(rename = "targetId")]
    pub target_id: String,
    #[serde(rename = "targetName")]
    pub target_name: String,
    #[serde(rename = "changeType")]
    pub change_type: String,  // buff / nerf / adjust / rework
    #[serde(rename = "changedStats", default)]
    pub changed_stats: HashMap<String, f64>,
    #[serde(rename = "impactScore")]
    pub impact_score: i32,
    #[serde(rename = "affectedLineups", default)]
    pub affected_lineups: Vec<String>,
    #[serde(default)]
    pub reason: String,
}

// ============================================================
// 知识关键词配置
// ============================================================

/// 知识关键词配置（从 config 加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeKeywords {
    pub version: String,
    /// 伤害类型关键词
    #[serde(rename = "damageTypes")]
    pub damage_types: HashMap<String, Vec<String>>,
    /// 施放模式关键词
    #[serde(rename = "castPatterns")]
    pub cast_patterns: HashMap<String, Vec<String>>,
    /// 攻击模式关键词
    #[serde(rename = "attackPatterns")]
    pub attack_patterns: HashMap<String, Vec<String>>,
    /// 装备效果标签关键词
    #[serde(rename = "effectTags")]
    pub effect_tags: HashMap<String, Vec<String>>,
    /// 装备冲突组
    #[serde(rename = "conflictGroups")]
    pub conflict_groups: HashMap<String, Vec<String>>,
    /// 伤害类型→装备适配
    #[serde(rename = "damageTypeItemFit")]
    pub damage_type_item_fit: HashMap<String, Vec<String>>,
}

impl Default for KnowledgeKeywords {
    fn default() -> Self {
        Self {
            version: "S18.1".into(),
            damage_types: HashMap::from([
                ("ad".into(), vec!["攻击力".into(), "物理伤害".into(), "攻击".into()]),
                ("ap".into(), vec!["法术强度".into(), "魔法伤害".into(), "法强".into()]),
                ("true_damage".into(), vec!["真实伤害".into(), "最大生命值".into()]),
                ("tank".into(), vec!["护甲".into(), "魔抗".into(), "生命值".into(), "减伤".into()]),
                ("utility".into(), vec!["护盾".into(), "治疗".into(), "控制".into()]),
            ]),
            cast_patterns: HashMap::from([
                ("mana_cast".into(), vec!["法力值".into(), "施放".into(), "蓝量".into()]),
                ("attack_based".into(), vec!["每次攻击".into(), "普通攻击".into(), "普攻".into()]),
                ("passive".into(), vec!["被动".into()]),
                ("transform".into(), vec!["变身".into(), "变形".into()]),
                ("summon".into(), vec!["召唤".into(), "分身".into()]),
            ]),
            attack_patterns: HashMap::from([
                ("burst".into(), vec!["爆发".into(), "一次性".into()]),
                ("sustained".into(), vec!["持续".into(), "每秒".into()]),
                ("execute".into(), vec!["处决".into(), "最大生命值".into()]),
                ("aoe".into(), vec!["范围".into(), "周围".into(), "全场".into()]),
                ("single_target".into(), vec!["单体".into(), "目标".into()]),
            ]),
            effect_tags: HashMap::from([
                ("anti_heal".into(), vec!["重伤".into(), "减疗".into(), "治疗降低".into()]),
                ("armor_shred".into(), vec!["护甲击碎".into(), "降低护甲".into(), "破甲".into()]),
                ("mr_shred".into(), vec!["魔抗击碎".into(), "降低魔抗".into()]),
                ("burn".into(), vec!["灼烧".into(), "燃烧".into()]),
                ("mana_engine".into(), vec!["回蓝".into(), "法力值回复".into(), "启动".into()]),
                ("sustain".into(), vec!["吸血".into(), "全能吸血".into(), "回复".into()]),
                ("crit_enable".into(), vec!["暴击".into(), "技能暴击".into()]),
                ("shield_survival".into(), vec!["护盾".into(), "保命".into(), "夜刃".into()]),
                ("aura_team_buff".into(), vec!["光环".into(), "团队".into()]),
            ]),
            conflict_groups: HashMap::from([
                ("anti_heal".into(), vec!["日炎".into(), "红霸符".into(), "鬼书".into()]),
                ("armor_shred".into(), vec!["轻语".into(), "薄暮".into()]),
                ("mr_shred".into(), vec!["离子".into(), "虚空".into()]),
                ("burn".into(), vec!["日炎".into(), "红霸符".into()]),
            ]),
            damage_type_item_fit: HashMap::from([
                ("ad".into(), vec!["攻击力".into(), "攻速".into(), "暴击".into(), "穿甲".into()]),
                ("ap".into(), vec!["法强".into(), "回蓝".into(), "暴击".into(), "破防".into()]),
                ("tank".into(), vec!["生命".into(), "双抗".into(), "减伤".into(), "回复".into()]),
            ]),
        }
    }
}
