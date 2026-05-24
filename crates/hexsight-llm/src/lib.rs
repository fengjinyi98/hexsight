// hexsight-llm 端侧 LLM 推理模块 (V1.5)
// 核心职责：
// - 复杂海克斯决策（冷门/多海克斯组合）
// - 动态变阵建议（来牌极差、装备完全不符）
// - 对局博弈分析（对手成型速度预判）
// - 自然语言建议输出
//
// V1.0 为纯 stub，V1.5 接入 MLX + Qwen2.5-3B 量化模型

use hexsight_core::{GameState, HexResult};

/// LLM 推理器接口
pub trait LlmInference {
    /// 根据对局状态生成智能建议
    fn infer(&self, state: &GameState) -> HexResult<String>;

    /// 检查模型是否已加载
    fn is_ready(&self) -> bool;
}

/// 空实现 —— V1.0 不使用 LLM
pub struct NoOpLLM;

impl LlmInference for NoOpLLM {
    fn infer(&self, _state: &GameState) -> HexResult<String> {
        Ok(String::new())
    }

    fn is_ready(&self) -> bool {
        false
    }
}

/// MLX LLM 推理器 —— V1.5 实现
pub struct MlxlLlm {
    /// 模型是否已加载
    loaded: bool,
}

impl MlxlLlm {
    pub fn new() -> Self {
        Self { loaded: false }
    }

    /// 加载量化模型
    pub fn load_model(&mut self, _model_path: &str) -> HexResult<()> {
        // TODO: 加载 MLX 量化模型 (Qwen2.5-3B-int4)
        self.loaded = true;
        Ok(())
    }
}

impl Default for MlxlLlm {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmInference for MlxlLlm {
    fn infer(&self, _state: &GameState) -> HexResult<String> {
        // TODO: 构建 prompt → 模型推理 → 返回自然语言建议
        Ok(String::new())
    }

    fn is_ready(&self) -> bool {
        self.loaded
    }
}
