// 游戏静态数据类型
// 核心职责：
// - 定义 config/game_data 中英雄、装备、羁绊、符文等纯数据模型
// - 定义 config/lineups 中阵容领域模型
// - 所有类型支持 serde 反序列化，字段使用 Option/#[serde(default)] 兼容版本变化
// - 与 types.rs（游戏运行时类型）完全分离

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 灵活 ID 反序列化（字符串或整数 → 字符串）
/// 处理官方 JSON 中 id 字段有时是 string 有时是 number 的情况
fn deserialize_flexible_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    struct FlexibleId;
    impl<'de> Visitor<'de> for FlexibleId {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("字符串或整数 ID")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<String, E> {
            Ok((v as i64).to_string())
        }
        fn visit_bool<E: de::Error>(self, v: bool) -> Result<String, E> {
            Ok(v.to_string())
        }
    }
    deserializer.deserialize_any(FlexibleId)
}

// ============================================================
// 游戏数据模型（game_data/mode*/）
// ============================================================

/// 数据容器 —— 所有 game_data JSON 的顶层结构 {"data": {...}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataContainer<T> {
    pub version: Option<String>,
    pub season: Option<String>,
    #[serde(rename = "setId")]
    pub set_id: Option<String>,
    pub time: Option<String>,
    pub data: T,
}

/// 英雄数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct HeroData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub picture: String,
    #[serde(default)]
    pub skillName: String,
    #[serde(default)]
    pub skillDesc: String,
    #[serde(default)]
    pub skillIcon: String,
    #[serde(default)]
    pub skillBriefValue: String,
    #[serde(default)]
    pub skillValueDesc: String,
    /// 种族 ID 列表（| 分隔）
    #[serde(default)]
    pub species: String,
    /// 职业 ID 列表（| 分隔）
    #[serde(default, rename = "class")]
    pub hero_class: String,
    #[serde(default)]
    pub initHP: String,
    #[serde(default)]
    pub initAttackDamage: String,
    #[serde(default)]
    pub attackSpeed: String,
    #[serde(default)]
    pub armor: String,
    #[serde(default)]
    pub magicResist: String,
    #[serde(default)]
    pub attackRange: String,
    #[serde(default)]
    pub initMP: String,
    #[serde(default)]
    pub maxMP: String,
    #[serde(default)]
    pub criticalStrikeChance: String,
    /// 费用（数值型）
    #[serde(default)]
    pub cost: i32,
    /// hero_id 去星级的 base key（同名+同费合并用）
    #[serde(default)]
    pub base_key: String,
    /// 星级
    #[serde(default)]
    pub star_level: i32,
}

/// 装备数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct EquipmentData {
    pub id: String,
    pub name: String,
    #[serde(default, rename = "type")]
    pub equip_type: String,
    #[serde(default)]
    pub picture: String,
    #[serde(default)]
    pub basicDesc: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub synthesis1: String,
    #[serde(default)]
    pub synthesis2: String,
    #[serde(default)]
    pub icon: String,
    /// 是否为基础装备
    #[serde(default)]
    pub is_component: bool,
    /// 是否为成型装备
    #[serde(default)]
    pub is_completed: bool,
}

/// 羁绊数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TraitData {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub checkId: String,
    /// 0 = 种族, 1 = 职业
    #[serde(default, rename = "type")]
    pub trait_type: i32,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub maxLevel: String,
    #[serde(default)]
    pub num: String,
    #[serde(default)]
    pub numList: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub realDesc: String,
    #[serde(default)]
    pub picture: String,
    #[serde(default)]
    pub values: String,
    /// 激活阈值列表（numList 解析后的数值）
    #[serde(default)]
    pub thresholds: Vec<i32>,
}

/// 强化符文数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct HexData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub is_legend: i32,
    #[serde(default)]
    pub hero_enhancement_type: String,
    #[serde(default)]
    pub fetterId: String,
}

/// 种族/职业 ID→名称 映射条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceJobEntry {
    pub id: Option<String>,
    pub name: String,
}

/// 解锁任务数据（mode16 特有）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionData {
    pub id: String,
    pub hero_id: String,
    #[serde(default)]
    pub task_tips: String,
    #[serde(default)]
    pub desc: String,
}

/// 神明数据容器（mode17 特有）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodData {
    #[serde(default, rename = "godId", deserialize_with = "deserialize_flexible_id")]
    pub god_id: String,
    #[serde(default, rename = "godName")]
    pub god_name: String,
    #[serde(default)]
    pub stages: Vec<GodStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodStage {
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub num: String,
    #[serde(default)]
    pub wishes: Vec<GodWishEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodWishEntry {
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub icon: String,
    /// 所属神明 ID
    #[serde(default)]
    pub god_id: String,
    /// 所属神明名称
    #[serde(default)]
    pub god_name: String,
    /// 阶段编号
    #[serde(default)]
    pub stage: i32,
}

// ============================================================
// 阵容领域模型（lineup_detail_total.json / 本地缓存）
// ============================================================

/// 阵容列表容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupListContainer {
    #[serde(rename = "lineup_list", default)]
    pub lineup_list: Vec<LineupRawItem>,
}

/// 阵容原始条目（来自 lineup_detail_total.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupRawItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub pid: String,
    #[serde(default)]
    pub quality: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub lineupauthor_data: serde_json::Value,
    /// detail 是 JSON 字符串，需二次解析
    #[serde(default)]
    pub detail: String,
    /// 保留所有未知字段
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// 阵容卡片（解析后的展示模型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupCardData {
    pub id: String,
    pub name: String,
    pub author: String,
    pub author_avatar: String,
    pub quality: String,
    pub traits: Vec<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub top4_rate: f64,
    pub detail: LineupDetailData,
}

/// 阵容详情（detail 字符串二次解析后的完整内容）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LineupDetailData {
    #[serde(default)]
    pub final_heroes: Vec<LineupPieceData>,
    #[serde(default)]
    pub early_heroes: Vec<LineupPieceData>,
    #[serde(default)]
    pub mid_heroes: Vec<LineupPieceData>,
    #[serde(default)]
    pub recommended_hex_ids: Vec<String>,
    #[serde(default)]
    pub replacement_hex_ids: Vec<String>,
    #[serde(default)]
    pub equipment_order_ids: Vec<String>,
    #[serde(default)]
    pub level_3_hero_ids: Vec<String>,
    #[serde(default)]
    pub hero_replacements: Vec<HeroReplacementData>,
    #[serde(default)]
    pub unlock_tasks: Vec<UnlockTaskData>,
    #[serde(default)]
    pub god_rewards: Vec<GodRewardData>,
    #[serde(default)]
    pub official_traits: Vec<TraitContactData>,
    #[serde(default)]
    pub early_traits: Vec<TraitContactData>,
    #[serde(default)]
    pub mid_traits: Vec<TraitContactData>,
    #[serde(default)]
    pub chosen_contact: Option<TraitContactData>,
    #[serde(default)]
    pub messenger_contact: Option<TraitContactData>,
    #[serde(default)]
    pub chosen_backups: Vec<ChosenBackupData>,
    // 文本字段
    #[serde(default)]
    pub line_feature: String,
    #[serde(default)]
    pub early_info: String,
    #[serde(default)]
    pub d_time: String,
    #[serde(default)]
    pub location_info: String,
    #[serde(default)]
    pub location_info2: String,
    #[serde(default)]
    pub enemy_info: String,
    #[serde(default)]
    pub hex_info: String,
    #[serde(default)]
    pub equipment_info: String,
    #[serde(default)]
    pub god_reward_info: String,
    #[serde(default)]
    pub task_info: String,
    #[serde(default)]
    pub chosen_info: String,
    #[serde(default)]
    pub early_round: String,
    #[serde(default)]
    pub mid_round: String,
    #[serde(default)]
    pub staff_info: String,
    #[serde(default)]
    pub goop_info: String,
    #[serde(default)]
    pub trait_party_info: String,
    #[serde(default)]
    pub legend_galaxy_info: String,
}

/// 阵容棋子站位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupPieceData {
    #[serde(default)]
    pub id_in_lineup: i32,
    #[serde(default)]
    pub chess_type: String,
    #[serde(default)]
    pub hero_id: String,
    #[serde(default)]
    pub equipment_ids: Vec<String>,
    #[serde(default)]
    pub is_carry_hero: bool,
    #[serde(default)]
    pub row: i32,
    #[serde(default)]
    pub col: i32,
    /// "row,col" 格式的站位 key
    #[serde(default)]
    pub location_key: String,
}

/// 羁绊计数（官方 contact 字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitContactData {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "type")]
    pub contact_type: String,
    #[serde(default)]
    pub count: i32,
    #[serde(default)]
    pub color: i32,
    #[serde(default)]
    pub level: i32,
}

/// 英雄替换关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroReplacementData {
    #[serde(default)]
    pub hero_id: String,
    #[serde(default)]
    pub replacement_hero_ids: Vec<String>,
}

/// 解锁任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockTaskData {
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub chess_id: String,
    #[serde(default)]
    pub hero_id: String,
}

/// 星神奖励
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodRewardData {
    #[serde(default)]
    pub stage: i32,
    #[serde(default)]
    pub god_id: String,
    #[serde(default)]
    pub wish_ids: Vec<String>,
}

/// 天选备选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChosenBackupData {
    #[serde(default)]
    pub hero_id: String,
    #[serde(default)]
    pub trait_id: String,
    #[serde(default, rename = "type")]
    pub backup_type: String,
}

// ============================================================
// 模式配置
// ============================================================

/// 模式能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeCapability {
    pub standard_lineup: bool,
    pub god_rewards: bool,
    pub unlock_tasks: bool,
    pub chosen_mechanics: bool,
    pub transition_contacts: bool,
}

/// 模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeProfileData {
    pub id: String,
    pub name: String,
    pub season: String,
    pub lineup_version_path: String,
    pub channel: String,
    pub capabilities: Vec<String>,
    pub cache_file_name: String,
}

// ============================================================
// 规则上下文输出类型
// ============================================================

/// 规则英雄快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleHeroSnapshot {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub picture: String,
    pub position: String,
    pub is_carry: bool,
    pub equipment_ids: Vec<String>,
    pub equipment_names: Vec<String>,
}

/// 规则装备快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEquipmentSnapshot {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub equip_type: String,
    pub picture: String,
}

/// 规则强化符文快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleHexSnapshot {
    pub id: String,
    pub name: String,
    pub level: i32,
    pub desc: String,
    pub icon: String,
}

/// 规则羁绊快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTraitSnapshot {
    pub id: String,
    pub trait_id: String,
    #[serde(rename = "type")]
    pub trait_type: String,
    pub name: String,
    pub count: i32,
    pub color: i32,
    pub level: i32,
    pub picture: String,
}

/// LineupRulesContext 输出（对齐目标文档第七章 JSON Schema）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesContextOutput {
    pub mode: ModeContextInfo,
    pub lineup: LineupContextInfo,
    #[serde(rename = "finalHeroes")]
    pub final_heroes: Vec<RuleHeroSnapshot>,
    pub traits: Vec<RuleTraitSnapshot>,
    pub augments: AugmentsContext,
    pub equipment: EquipmentContext,
    #[serde(rename = "modeSpecific")]
    pub mode_specific: ModeSpecificContext,
    #[serde(rename = "strategyTexts")]
    pub strategy_texts: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeContextInfo {
    pub id: String,
    pub name: String,
    pub season: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupContextInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub quality: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AugmentsContext {
    pub recommended: Vec<HexRef>,
    pub replacement: Vec<HexRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexRef {
    pub id: String,
    pub name: String,
    pub level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentContext {
    pub order: Vec<EquipRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModeSpecificContext {
    #[serde(rename = "godRewards", default)]
    pub god_rewards: Vec<serde_json::Value>,
    #[serde(rename = "unlockTasks", default)]
    pub unlock_tasks: Vec<serde_json::Value>,
    #[serde(default)]
    pub chosen: Option<serde_json::Value>,
}

/// 版本校验报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub mode: String,
    pub total_lineups: usize,
    pub total_hero_ids: usize,
    pub total_equip_ids: usize,
    pub missing_hero_ids: Vec<String>,
    pub missing_equip_ids: Vec<String>,
    pub parseable_lineups: usize,
    pub unparseable_lineups: Vec<String>,
}
