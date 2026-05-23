// hexsight-memory 全局对局记忆系统
// 核心职责：
// - 单局全局状态常驻内存
// - 增量更新（仅变化数据刷新）
// - 多帧降噪校验（连续N帧一致才更新）
// - 快照保存与异常恢复
// - 新对局/退出大厅自动重置

pub mod snapshot;
pub mod state;

pub use state::GameMemory;
