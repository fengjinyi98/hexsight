// hexsight-vision 画面识别引擎
// 核心职责：
// - 7-segment 数字解析（金币、人口、等级）
// - 图标模板匹配（装备、英雄头像、羁绊标识）
// - 条状数据扫描（血量条、经验条）
// - OCR 文字识别兜底（海克斯名称等动态文本）

pub mod bar;
pub mod digit;
pub mod icon;
pub mod ocr;

use hexsight_core::{HexResult, RecognizedFrame, RegionConfig};

/// 识别单帧画面
/// pixels: BGRA 格式像素数据
/// regions: 固定分辨率 ROI 坐标配置
pub fn recognize(
    pixels: &[u8],
    regions: &RegionConfig,
    _width: u32,
    _height: u32,
) -> HexResult<RecognizedFrame> {
    let mut frame = RecognizedFrame::default();

    frame.gold = digit::parse_digits(pixels, &regions.gold_rect)?;
    frame.hp = bar::scan_hp_bar(pixels, &regions.hp_bar)?;
    frame.level = digit::parse_digits(pixels, &regions.level_rect)?;
    frame.exp = bar::scan_exp_bar(pixels, &regions.exp_bar)?;

    // TODO: 图标匹配、OCR、棋盘识别等后续实现

    Ok(frame)
}
