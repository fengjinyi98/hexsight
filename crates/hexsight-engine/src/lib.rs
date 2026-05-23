// hexsight-engine 规则引擎 + 阵容库
// 核心职责：
// - 加载版本阵容 JSON 配置
// - 固定规则决策（开局选阵、过渡运营、同行博弈、装备补救、海克斯匹配）
// - 输出最终 Decision

pub mod decision;
pub mod lineup;
pub mod rules;

pub use decision::DecisionEngine;
pub use lineup::LineupDB;
pub use rules::RulesEngine;
