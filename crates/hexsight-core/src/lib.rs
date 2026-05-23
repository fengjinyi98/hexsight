// hexsight-core 共享类型定义
// 核心职责：
// - 定义对局中所有数据结构（GameState, Decision, RecognizedFrame 等）
// - 统一错误类型
// - 所有类型支持 JSON 序列化，用于 FFI 跨语言传递

pub mod error;
pub mod types;

pub use error::{HexError, HexResult};
pub use types::*;
