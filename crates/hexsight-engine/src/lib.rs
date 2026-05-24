// hexsight-engine 规则引擎 + 阵容库
// 核心职责：
// - 数据加载、索引与规则上下文生成
// - 阵容规则档案构建（LineupProfileBuilder）
// - 装备/阵容/过渡/赌狗/海克斯/经济/风险评分器
// - 开局路线分类、决策规划、过渡匹配、伤害预测
// - 规则包加载、版本覆写、Schema校验
// - 规则引擎输出结构化 RuleOutput JSON

pub mod augment_economy_planner;
pub mod augment_effect_interpreter;
pub mod board_power_fight;
pub mod champion_combat_profile_builder;
pub mod champion_item_fit;
pub mod damage_patch_loader;
pub mod decision;
pub mod decision_planner;
pub mod game_data_index;
pub mod game_data_loader;
pub mod item_fit_scorer;
pub mod knowledge_builders;
pub mod lineup;
pub mod lineup_adapter;
pub mod lineup_fit_scorer;
pub mod lineup_loader;
pub mod lineup_profile_builder;
pub mod llm_context;
pub mod mode_special_calibrator;
pub mod opening_route_classifier;
pub mod remote_lineup_source;
pub mod reroll_eligibility_scorer;
pub mod rule_pack_loader;
pub mod rules;
pub mod rules_context;
pub mod transition_risk_scorer;
pub mod version_validator;

pub use augment_economy_planner::{AugmentFitScorer, EconomyPlanner};
pub use augment_effect_interpreter::AugmentEffectInterpreter;
pub use board_power_fight::{BoardPower, BoardPowerScorer, FightOutcomeEstimator};
pub use champion_combat_profile_builder::ChampionCombatProfileBuilder;
pub use champion_item_fit::{
    ChampionItemFitScorer, ChampionItemOverrideLoader, ItemReplacementGroupLoader,
    ItemSynthesisIndex,
};
pub use damage_patch_loader::{DamageProfile, PatchOverrideLoader};
pub use decision::DecisionEngine;
pub use decision_planner::DecisionPlanner;
pub use game_data_index::GameDataIndex;
pub use game_data_loader::GameDataLoader;
pub use item_fit_scorer::ItemFitScorer;
pub use knowledge_builders::{
    ChampionCapabilityBuilder, ItemValueBuilder, KnowledgeKeywordLoader,
    AugmentEffectProfileBuilder, TraitEffectProfileBuilder,
    KnowledgeBase, KnowledgeBaseBuilder, KnowledgeCoverageReport,
    ManualOverrideLoader,
};
pub use lineup::LineupDB;
pub use lineup_adapter::LineupAdapter;
pub use lineup_fit_scorer::{LineupFitScorer, LineupItemFitContext, TransitionStrengthScorer};
pub use lineup_loader::LineupLoader;
pub use lineup_profile_builder::LineupProfileBuilder;
pub use llm_context::LlmContextBuilder;
pub use mode_special_calibrator::{ModeSpecialPlanner, DamagePredictionCalibrator};
pub use opening_route_classifier::OpeningRouteClassifier;
pub use remote_lineup_source::RemoteLineupSource;
pub use reroll_eligibility_scorer::RerollEligibilityScorer;
pub use rule_pack_loader::RulePackLoader;
pub use rules::RulesEngine;
pub use rules_context::RulesContextBuilder;
pub use version_validator::VersionValidator;
