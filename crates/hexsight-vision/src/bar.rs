// 条状数据扫描
// 核心职责：
// - 血量条：从左侧向右侧扫描填充色像素，计算百分比
// - 经验条：同理，判定经验值 (0-100)

use hexsight_core::{HexResult, Rect};

/// 扫描血量条，返回当前血量值
/// 算法：从左到右扫描条状区域，找填充色边界
pub fn scan_hp_bar(_pixels: &[u8], _rect: &Rect) -> HexResult<u32> {
    // TODO: 实现列扫描边界检测
    // 固定坐标像素采样 → 绿色/红色阈值判定 → 计算比例 → 映射血量
    Ok(100)
}

/// 扫描经验条，返回经验百分比 (0-100)
pub fn scan_exp_bar(_pixels: &[u8], _rect: &Rect) -> HexResult<u32> {
    // TODO: 实现经验条扫描
    Ok(0)
}
