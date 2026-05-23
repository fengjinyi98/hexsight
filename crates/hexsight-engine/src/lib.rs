// hexsight-engine 规则引擎 + 阵容库
// 核心职责：
// - 加载版本阵容 JSON 配置
// - 固定规则决策（开局选阵、过渡运营、同行博弈、装备补救、海克斯匹配）
// - 输出最终 Decision
// - 数据加载、索引与规则上下文生成

pub mod decision;
pub mod game_data_index;
pub mod game_data_loader;
pub mod lineup;
pub mod lineup_adapter;
pub mod lineup_loader;
pub mod llm_context;
pub mod remote_lineup_source;
pub mod rules;
pub mod rules_context;
pub mod version_validator;

pub use decision::DecisionEngine;
pub use game_data_index::GameDataIndex;
pub use game_data_loader::GameDataLoader;
pub use lineup::LineupDB;
pub use lineup_adapter::LineupAdapter;
pub use lineup_loader::LineupLoader;
pub use llm_context::LlmContextBuilder;
pub use remote_lineup_source::RemoteLineupSource;
pub use rules::RulesEngine;
pub use rules_context::RulesContextBuilder;
pub use version_validator::VersionValidator;