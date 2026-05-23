// OCR 文字识别（兜底方案）
// 核心职责：
// - 海克斯名称等动态文本识别
// - 仅在模板匹配无法覆盖时调用
// - V1.0 为 stub，后续可接入轻量 OCR 引擎

use hexsight_core::HexResult;

/// 识别 ROI 内的文本
pub fn recognize_text(_pixels: &[u8], _width: u32, _height: u32) -> HexResult<Option<String>> {
    // TODO: 接入轻量 OCR 引擎（如 Tesseract 或 macOS Vision 框架）
    Ok(None)
}

/// 多行文本识别
pub fn recognize_multiline(_pixels: &[u8], _width: u32, _height: u32) -> HexResult<Vec<String>> {
    Ok(vec![])
}
