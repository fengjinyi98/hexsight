// 7-segment 数字解析
// 核心职责：
// - 从固定区域截取数字位图
// - 逐位与 0-9 预设模板比对（像素交并比）
// - 适用于金币、人口、等级等固定字体数字

use hexsight_core::{HexResult, Rect};

/// 解析 ROI 内的数字，返回整数值
/// 当前为 stub，后续实现模板匹配逻辑
pub fn parse_digits(_pixels: &[u8], _rect: &Rect) -> HexResult<u32> {
    // TODO: 实现 7-segment / 像素模板匹配数字解析
    Ok(0)
}

/// 加载数字模板（0-9），仅首次初始化时调用
pub fn load_templates(_template_dir: &str) -> HexResult<()> {
    // TODO: 从 config/digit_templates/ 加载 0-9 的像素模板
    Ok(())
}
