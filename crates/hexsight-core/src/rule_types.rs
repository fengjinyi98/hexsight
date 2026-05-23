// 规则引擎核心类型
// 核心职责：
// - 定义阵容规则档案、评分权重、规则输出 Schema
// - 定义玩法标签、阶段路线、经济/装备/海克斯动作类型
// - 所有类型支持 serde 序列化，供 FFI 输出和规则包加载

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 玩法标签（对齐文档第五章）
// ============================================================

/// 玩法标签
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaystyleTag {
    /// 常规运营
    #[serde(rename = "standard")]
    Standard,
    /// 1 费赌狗
    #[serde(rename = "reroll_1_cost")]
    Reroll1Cost,
    /// 2 费赌狗
    #[serde(rename = "reroll_2_cost")]
    Reroll2Cost,
    /// 3 费赌狗
    #[serde(rename = "reroll_3_cost")]
    Reroll3Cost,
    /// 速 8
    #[serde(rename = "fast_8")]
    Fast8,
    /// 速 9
    #[serde(rename = "fast_9")]
    Fast9,
    /// 节奏稳血
    #[serde(rename = "tempo")]
    Tempo,
    /// 后期上限
    #[serde(rename = "late_cap")]
    LateCap,
    /// 海克斯开关
    #[serde(rename = "augment_enabled")]
    AugmentEnabled,
    /// 依赖专属强化
    #[serde(rename = "requires_augment")]
    RequiresAugment,
    /// 装备严格
    #[serde(rename = "item_strict")]
    ItemStrict,
    /// 装备弹性
    #[serde(rename = "item_flexible")]
    ItemFlexible,
    /// 经济开局适合
    #[serde(rename = "econ_open")]
    EconOpen,
    /// 连败收菜
    #[serde(rename = "loss_streak")]
    LossStreak,
    /// 模式特殊
    #[serde(rename = "mode_special")]
    ModeSpecial,
}

// ============================================================
// 阵容规则档案（对齐文档第四章）
// ============================================================

/// 阵容规则档案 —— 从官网阵容数据生成的稳定规则输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupProfile {
    /// 官方阵容 ID
    pub lineup_id: String,
    /// 阵容名称
    pub name: String,
    /// 版本基础强度 (S/A/B → 100/80/60)
    pub base_tier: i32,
    /// 玩法标签
    pub playstyle_tags: Vec<PlaystyleTag>,
    /// 成型英雄 ID 列表
    pub final_hero_ids: Vec<String>,
    /// 主 C 英雄 ID
    pub carry_hero_ids: Vec<String>,
    /// 主坦英雄 ID
    pub tank_hero_ids: Vec<String>,
    /// 核心装备 ID（主 C 装）
    pub core_equipment_ids: Vec<String>,
    /// 主坦装备 ID
    pub tank_equipment_ids: Vec<String>,
    /// 装备拿取优先级
    pub equipment_order_ids: Vec<String>,
    /// 推荐海克斯 ID
    pub recommended_hex_ids: Vec<String>,
    /// 备选海克斯 ID
    pub replacement_hex_ids: Vec<String>,
    /// 前期过渡英雄 ID
    pub early_hero_ids: Vec<String>,
    /// 中期过渡英雄 ID
    pub mid_hero_ids: Vec<String>,
    /// 羁绊目标 (trait_check_id → count)
    pub trait_targets: HashMap<String, i32>,
    /// 策略文本
    pub strategy_texts: HashMap<String, String>,
    /// 模式特殊数据
    pub mode_specific: serde_json::Value,
    /// 核心英雄费用（用于判断赌狗费用）
    pub carry_costs: Vec<i32>,
    /// 阵容分类标签
    pub category: Option<String>,
}

// ============================================================
// 开局路线（对齐文档第七章）
// ============================================================

/// 开局路线分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpeningRoute {
    /// 连胜路线
    #[serde(rename = "win_streak")]
    WinStreak,
    /// 精致连败
    #[serde(rename = "loss_streak")]
    LossStreak,
    /// 混合过渡
    #[serde(rename = "mixed")]
    Mixed,
}

/// 开局路线分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningRouteResult {
    pub route: OpeningRoute,
    pub confidence: f64,
    pub reasons: Vec<String>,
    /// 二星英雄数
    pub two_star_count: i32,
    /// 前排质量分 (0-100)
    pub frontline_quality: i32,
    /// 装备能合成战力装
    pub can_build_combat_item: bool,
    /// 推荐动作
    pub recommended_actions: Vec<String>,
}

// ============================================================
// 阵容评分（对齐文档第六章）
// ============================================================

/// 阵容评分结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupScore {
    pub lineup_id: String,
    pub name: String,
    /// 总分 (0-100)
    pub total_score: i32,
    /// 版本基础分
    pub base_score: i32,
    /// 装备匹配分
    pub item_fit_score: i32,
    /// 棋子命中分
    pub champion_hit_score: i32,
    /// 海克斯适配分
    pub augment_fit_score: i32,
    /// 羁绊成型分
    pub trait_fit_score: i32,
    /// 阶段适配分
    pub stage_fit_score: i32,
    /// 经济适配分
    pub economy_fit_score: i32,
    /// 血量安全分
    pub health_safety_score: i32,
    /// 玩法开关分
    pub playstyle_switch_score: i32,
    /// 同行风险扣分
    pub rival_penalty: i32,
    /// 成型难度扣分
    pub difficulty_penalty: i32,
    /// 推荐理由
    pub reasons: Vec<String>,
    /// 风险提示
    pub risks: Vec<String>,
    /// 是否需要专属海克斯
    pub requires_augment: bool,
}

// ============================================================
// 装备评分
// ============================================================

/// 装备匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemFitResult {
    /// 当前装备最适合的阵容 ID 列表
    pub best_fit_lineup_ids: Vec<String>,
    /// 装备方向 (AD/AP/坦/混合)
    pub direction: String,
    /// 核心散件数
    pub core_component_count: i32,
    /// 可合成核心装数量
    pub can_build_core_count: i32,
    /// 装备通用性 (0-100)
    pub flexibility: i32,
    /// 推荐动作
    pub recommended_action: ItemAction,
    /// 解释
    pub reasons: Vec<String>,
}

/// 装备动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemAction {
    /// 立即合成战力装
    #[serde(rename = "build_combat_now")]
    BuildCombatNow,
    /// 等待核心装
    #[serde(rename = "wait_core")]
    WaitCore,
    /// 合通用装
    #[serde(rename = "build_generic")]
    BuildGeneric,
    /// 保留散件弹性
    #[serde(rename = "hold_components")]
    HoldComponents,
    /// 装备严重不匹配，考虑转向
    #[serde(rename = "pivot_for_items")]
    PivotForItems,
}

// ============================================================
// 过渡战力评分（对齐文档 7.2）
// ============================================================

/// 过渡战力评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionStrength {
    /// 总战力分 (0-100)
    pub total_score: i32,
    /// 二星数量分
    pub two_star_score: i32,
    /// 前排质量分
    pub frontline_score: i32,
    /// 输出质量分
    pub backline_score: i32,
    /// 羁绊激活分
    pub trait_score: i32,
    /// 装备战力分
    pub item_score: i32,
    /// 海克斯战力分
    pub augment_score: i32,
    /// 在全场中的排名估计
    pub lobby_rank_estimate: Option<i32>,
    /// 当前战力是否足够
    pub is_strong_enough: bool,
    /// 需要改进的方面
    pub weaknesses: Vec<String>,
}

// ============================================================
// 赌狗资格评分（对齐文档第十章）
// ============================================================

/// 赌狗资格评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerollEligibility {
    /// 可赌程度 (0-100)
    pub score: i32,
    /// 候选阵容
    pub lineup_id: String,
    /// 核心费用
    pub core_cost: i32,
    /// 当前核心牌数
    pub current_core_count: i32,
    /// 目标三星数
    pub target_count: i32,
    /// 推荐卡等级
    pub recommended_level: i32,
    /// 推荐 D 牌节奏
    pub tempo: String,
    /// 停手条件
    pub stop_conditions: Vec<String>,
    /// 放弃条件
    pub abandon_conditions: Vec<String>,
    /// 同行数量
    pub rival_count: i32,
    /// 是否推荐赌
    pub is_recommended: bool,
}

// ============================================================
// 规则输出（对齐文档第十五章）
// ============================================================

/// 规则引擎最终输出
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleOutput {
    /// 开局策略
    pub strategy: String,
    /// 阵容推荐列表（Top 3）
    pub lineup_recommendations: Vec<LineupRecommendation>,
    /// 经济动作
    pub economy_action: EconomyDecision,
    /// 装备动作
    pub item_action: ItemDecision,
    /// 海克斯动作
    pub augment_action: AugmentDecision,
    /// 过渡动作
    pub transition_action: TransitionDecision,
    /// 转向条件
    pub pivot_conditions: Vec<String>,
    /// 战斗预测（可选）
    pub fight_outcome: Option<FightOutcome>,
    /// 生成时间
    pub generated_at: String,
    /// 规则引擎版本
    pub engine_version: String,
}

/// 阵容推荐条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineupRecommendation {
    pub lineup_id: String,
    pub name: String,
    pub score: i32,
    pub reason: Vec<String>,
    pub risk: Vec<String>,
}

/// 经济决策
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EconomyDecision {
    pub action: String,
    pub reason: String,
}

/// 装备决策
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDecision {
    pub action: String,
    pub reason: String,
}

/// 海克斯决策
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AugmentDecision {
    pub recommended: String,
    pub lock_lineup: bool,
    pub follow_up: String,
}

/// 过渡决策
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionDecision {
    pub route: String,
    pub target_damage_per_round: String,
    pub stop_loss_stage: String,
}

// ============================================================
// 战斗预测（对齐文档第十六章）
// ============================================================

/// 战斗结果预测
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FightOutcome {
    /// 胜率 (0.0-1.0)
    #[serde(rename = "winProbability")]
    pub win_probability: f64,
    /// 预计掉血
    #[serde(rename = "expectedDamageTaken")]
    pub expected_damage_taken: f64,
    /// 掉血区间 [min, max]
    #[serde(rename = "damageRange")]
    pub damage_range: [i32; 2],
    /// 预计对方剩余棋子
    #[serde(rename = "expectedEnemySurvivors")]
    pub expected_enemy_survivors: f64,
    /// 风险等级
    #[serde(rename = "riskLevel")]
    pub risk_level: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 影响胜负的关键因素
    pub reason: Vec<String>,
    /// 推荐动作
    #[serde(rename = "recommendedAction")]
    pub recommended_action: String,
}

// ============================================================
// 规则包（对齐文档 17.2）
// ============================================================

/// 规则包 —— 版本化评分参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePack {
    pub version: String,
    pub modes: Vec<String>,
    /// 阶段基础伤害表
    #[serde(rename = "damageProfile")]
    pub damage_profile: HashMap<String, i32>,
    /// 评分权重
    pub weights: ScoreWeights,
    /// 阈值
    pub thresholds: RuleThresholds,
}

/// 评分权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    #[serde(rename = "lineupFit")]
    pub lineup_fit: LineupFitWeights,
    #[serde(rename = "transition")]
    pub transition: TransitionWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupFitWeights {
    #[serde(rename = "itemFit")]
    pub item_fit: f64,
    #[serde(rename = "championHit")]
    pub champion_hit: f64,
    #[serde(rename = "augmentFit")]
    pub augment_fit: f64,
    #[serde(rename = "traitFit")]
    pub trait_fit: f64,
    #[serde(rename = "stageFit")]
    pub stage_fit: f64,
    #[serde(rename = "economyFit")]
    pub economy_fit: f64,
    #[serde(rename = "healthSafety")]
    pub health_safety: f64,
    #[serde(rename = "playstyleSwitch")]
    pub playstyle_switch: f64,
    #[serde(rename = "baseScore")]
    pub base_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionWeights {
    #[serde(rename = "twoStar")]
    pub two_star: f64,
    #[serde(rename = "frontline")]
    pub frontline: f64,
    #[serde(rename = "backline")]
    pub backline: f64,
    #[serde(rename = "trait")]
    pub transition_trait: f64,
    #[serde(rename = "item")]
    pub transition_item: f64,
    #[serde(rename = "augment")]
    pub transition_augment: f64,
}

/// 规则阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleThresholds {
    /// 锁阵容分数阈值
    #[serde(rename = "lockLineupScore")]
    pub lock_lineup_score: i32,
    /// 转向风险阈值
    #[serde(rename = "pivotRiskScore")]
    pub pivot_risk_score: i32,
    /// 必须止血血量
    #[serde(rename = "mustStabilizeHealth")]
    pub must_stabilize_health: i32,
    /// 大入预警血量
    #[serde(rename = "largeDamageWarning")]
    pub large_damage_warning: i32,
}

impl Default for RulePack {
    fn default() -> Self {
        Self {
            version: "S18.1".to_string(),
            modes: vec!["17".to_string(), "16".to_string(), "4".to_string()],
            damage_profile: HashMap::from([
                ("2".to_string(), 0),
                ("3".to_string(), 2),
                ("4".to_string(), 4),
                ("5".to_string(), 6),
                ("6".to_string(), 8),
            ]),
            weights: ScoreWeights {
                lineup_fit: LineupFitWeights {
                    item_fit: 0.22,
                    champion_hit: 0.18,
                    augment_fit: 0.12,
                    trait_fit: 0.08,
                    stage_fit: 0.05,
                    economy_fit: 0.05,
                    health_safety: 0.05,
                    playstyle_switch: 0.05,
                    base_score: 0.20,
                },
                transition: TransitionWeights {
                    two_star: 0.30,
                    frontline: 0.25,
                    backline: 0.20,
                    transition_trait: 0.10,
                    transition_item: 0.10,
                    transition_augment: 0.05,
                },
            },
            thresholds: RuleThresholds {
                lock_lineup_score: 75,
                pivot_risk_score: 70,
                must_stabilize_health: 35,
                large_damage_warning: 8,
            },
        }
    }
}
