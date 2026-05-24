// 条状数据扫描
// 核心职责：
// - 血量条：从左侧向右侧扫描填充色像素，计算百分比
// - 经验条：同理，判定经验值 (0-100)

use hexsight_core::{HexResult, Rect};

/// 扫描血量条，返回当前血量值
/// 算法：从左到右扫描条状区域，找填充色边界
pub fn scan_hp_bar(pixels: &[u8], rect: &Rect) -> HexResult<u32> {
    Ok(scan_bar_percentage(pixels, rect.w, rect.h))
}

/// 扫描经验条，返回经验百分比 (0-100)
pub fn scan_exp_bar(pixels: &[u8], rect: &Rect) -> HexResult<u32> {
    Ok(scan_bar_percentage(pixels, rect.w, rect.h))
}

/// 扫描整帧内的血量条，返回当前血量值
pub fn scan_hp_bar_from_frame(
    pixels: &[u8],
    width: u32,
    height: u32,
    rect: Rect,
) -> HexResult<u32> {
    Ok(crop_bgra(pixels, width, height, rect)
        .map(|(roi, w, h)| scan_bar_percentage(&roi, w, h))
        .unwrap_or(0))
}

/// 扫描整帧内的经验条，返回经验百分比 (0-100)
pub fn scan_exp_bar_from_frame(
    pixels: &[u8],
    width: u32,
    height: u32,
    rect: Rect,
) -> HexResult<u32> {
    Ok(crop_bgra(pixels, width, height, rect)
        .map(|(roi, w, h)| scan_bar_percentage(&roi, w, h))
        .unwrap_or(0))
}

fn scan_bar_percentage(pixels: &[u8], width: u32, height: u32) -> u32 {
    if width == 0 || height == 0 || pixels.len() < (width as usize * height as usize * 4) {
        return 0;
    }

    let filled_columns = (0..width)
        .take_while(|x| column_fill_ratio(pixels, width, height, *x) >= 0.45)
        .count() as u32;

    ((filled_columns as f32 / width as f32) * 100.0).round() as u32
}

fn column_fill_ratio(pixels: &[u8], width: u32, height: u32, x: u32) -> f32 {
    let mut filled = 0_u32;
    for y in 0..height {
        let offset = ((y * width + x) * 4) as usize;
        if is_bar_fill(&pixels[offset..offset + 4]) {
            filled += 1;
        }
    }
    filled as f32 / height.max(1) as f32
}

fn is_bar_fill(pixel: &[u8]) -> bool {
    let brightness = (u16::from(pixel[0]) + u16::from(pixel[1]) + u16::from(pixel[2])) / 3;
    let max_channel = pixel[0].max(pixel[1]).max(pixel[2]);
    let min_channel = pixel[0].min(pixel[1]).min(pixel[2]);
    let chroma = max_channel - min_channel;
    brightness >= 45 && chroma >= 35
}

fn crop_bgra(pixels: &[u8], width: u32, height: u32, rect: Rect) -> Option<(Vec<u8>, u32, u32)> {
    if width == 0 || height == 0 || rect.w == 0 || rect.h == 0 {
        return None;
    }
    let max_x = rect.x.checked_add(rect.w)?.min(width);
    let max_y = rect.y.checked_add(rect.h)?.min(height);
    if rect.x >= max_x || rect.y >= max_y {
        return None;
    }
    if pixels.len() < width as usize * height as usize * 4 {
        return None;
    }
    let crop_width = max_x - rect.x;
    let crop_height = max_y - rect.y;
    let mut roi = Vec::with_capacity((crop_width * crop_height * 4) as usize);
    for y in rect.y..max_y {
        let start = ((y * width + rect.x) * 4) as usize;
        let end = start + (crop_width * 4) as usize;
        roi.extend_from_slice(&pixels[start..end]);
    }
    Some((roi, crop_width, crop_height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_hp_bar_returns_green_fill_percentage() {
        let pixels = render_bar(100, 4, 73, [30, 210, 70, 255]);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 100,
            h: 4,
        };

        let hp = scan_hp_bar(&pixels, &rect).unwrap();

        assert_eq!(hp, 73);
    }

    #[test]
    fn scan_exp_bar_returns_blue_fill_percentage() {
        let pixels = render_bar(100, 4, 42, [220, 120, 35, 255]);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 100,
            h: 4,
        };

        let exp = scan_exp_bar(&pixels, &rect).unwrap();

        assert_eq!(exp, 42);
    }

    fn render_bar(width: u32, height: u32, filled_columns: u32, color: [u8; 4]) -> Vec<u8> {
        let mut pixels = vec![0_u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let offset = ((y * width + x) * 4) as usize;
                let pixel = if x < filled_columns {
                    color
                } else {
                    [12, 14, 18, 255]
                };
                pixels[offset..offset + 4].copy_from_slice(&pixel);
            }
        }
        pixels
    }
}
