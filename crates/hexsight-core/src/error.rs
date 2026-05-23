use std::fmt;

/// 统一错误类型
#[derive(Debug)]
pub enum HexError {
    /// 识别错误
    Vision(String),
    /// 记忆系统错误
    Memory(String),
    /// 决策引擎错误
    Engine(String),
    /// LLM 推理错误
    Llm(String),
    /// 配置加载错误
    Config(String),
    /// FFI 边界错误
    Ffi(String),
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HexError::Vision(msg) => write!(f, "识别错误: {}", msg),
            HexError::Memory(msg) => write!(f, "记忆错误: {}", msg),
            HexError::Engine(msg) => write!(f, "引擎错误: {}", msg),
            HexError::Llm(msg) => write!(f, "LLM错误: {}", msg),
            HexError::Config(msg) => write!(f, "配置错误: {}", msg),
            HexError::Ffi(msg) => write!(f, "FFI错误: {}", msg),
        }
    }
}

impl std::error::Error for HexError {}

pub type HexResult<T> = Result<T, HexError>;
