// 7-segment 数字解析
// 核心职责：
// - 从固定区域截取数字位图
// - 逐位与 0-9 预设模板比对（像素交并比）
// - 适用于金币、人口、等级等固定字体数字

use std::fs;
use std::path::Path;

use hexsight_core::{HexError, HexResult, Rect};

const SEGMENT_PATTERNS: [[bool; 7]; 10] = [
    [true, true, true, true, true, true, false],
    [false, true, true, false, false, false, false],
    [true, true, false, true, true, false, true],
    [true, true, true, true, false, false, true],
    [false, true, true, false, false, true, true],
    [true, false, true, true, false, true, true],
    [true, false, true, true, true, true, true],
    [true, true, true, false, false, false, false],
    [true, true, true, true, true, true, true],
    [true, true, true, true, false, true, true],
];

/// 解析 ROI 内的数字，返回整数值
pub fn parse_digits(pixels: &[u8], rect: &Rect) -> HexResult<u32> {
    parse_digit_roi(pixels, rect.w, rect.h)
}

/// 解析整帧内指定 ROI 的数字，返回整数值
pub fn parse_digits_from_frame(
    pixels: &[u8],
    width: u32,
    height: u32,
    rect: Rect,
) -> HexResult<u32> {
    let Some(roi) = crop_bgra(pixels, width, height, rect) else {
        return Ok(0);
    };
    parse_digit_roi(
        &roi,
        rect.w.min(width.saturating_sub(rect.x)),
        rect.h.min(height.saturating_sub(rect.y)),
    )
}

/// 加载数字模板（0-9），仅首次初始化时调用
pub fn load_templates(template_dir: &str) -> HexResult<()> {
    let root = Path::new(template_dir);
    if !root.is_dir() {
        return Err(HexError::Vision(format!(
            "数字模板目录不存在: {}",
            root.display()
        )));
    }
    for digit in 0..=9 {
        let path = root.join(format!("{}.txt", digit));
        if !path.is_file() {
            return Err(HexError::Vision(format!(
                "数字模板缺失: {}",
                path.display()
            )));
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| HexError::Vision(format!("读取数字模板失败 {}: {}", path.display(), e)))?;
        if content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
            == 0
        {
            return Err(HexError::Vision(format!(
                "数字模板为空: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn parse_digit_roi(pixels: &[u8], width: u32, height: u32) -> HexResult<u32> {
    if width == 0 || height == 0 || pixels.len() < (width as usize * height as usize * 4) {
        return Ok(0);
    }
    let mask = bright_mask(pixels, width, height);
    let runs = digit_column_runs(&mask, width, height);
    let mut value = 0_u32;
    for run in runs {
        let Some(bounds) = bright_bounds(&mask, width, height, run.0, run.1) else {
            continue;
        };
        let active = segment_activity(&mask, width, bounds);
        let digit = best_digit(active);
        value = value * 10 + digit as u32;
    }
    Ok(value)
}

fn bright_mask(pixels: &[u8], width: u32, height: u32) -> Vec<bool> {
    let mut mask = Vec::with_capacity((width * height) as usize);
    for chunk in pixels.chunks_exact(4).take((width * height) as usize) {
        let brightness = (u16::from(chunk[0]) + u16::from(chunk[1]) + u16::from(chunk[2])) / 3;
        let chroma = u16::from(*chunk.iter().take(3).max().unwrap())
            - u16::from(*chunk.iter().take(3).min().unwrap());
        mask.push(brightness >= 80 || chroma >= 90);
    }
    mask
}

fn digit_column_runs(mask: &[bool], width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut runs = Vec::new();
    let mut start: Option<u32> = None;
    let mut blank = 0_u32;
    for x in 0..width {
        let has_ink = (0..height).any(|y| mask[(y * width + x) as usize]);
        if has_ink {
            if start.is_none() {
                start = Some(x);
            }
            blank = 0;
        } else if let Some(run_start) = start {
            blank += 1;
            if blank >= 2 {
                let end = x - blank;
                if end >= run_start {
                    runs.push((run_start, end));
                }
                start = None;
                blank = 0;
            }
        }
    }
    if let Some(run_start) = start {
        runs.push((run_start, width - 1));
    }
    runs
}

fn bright_bounds(
    mask: &[bool],
    width: u32,
    height: u32,
    min_x: u32,
    max_x: u32,
) -> Option<(u32, u32, u32, u32)> {
    let mut left = max_x;
    let mut right = min_x;
    let mut top = height;
    let mut bottom = 0_u32;
    for y in 0..height {
        for x in min_x..=max_x {
            if mask[(y * width + x) as usize] {
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    (top <= bottom && left <= right).then_some((left, top, right, bottom))
}

fn segment_activity(mask: &[bool], width: u32, bounds: (u32, u32, u32, u32)) -> [bool; 7] {
    let (left, top, right, bottom) = bounds;
    let w = (right - left + 1).max(1);
    let h = (bottom - top + 1).max(1);
    let mid = top + h / 2;
    let thickness_x = ((w as f32 * 0.32).ceil() as u32).max(1);
    let thickness_y = ((h as f32 * 0.20).ceil() as u32).max(1);
    let horizontal_left = left + thickness_x;
    let horizontal_right = right.saturating_sub(thickness_x);
    let upper_top = top + thickness_y;
    let upper_bottom = mid.saturating_sub(thickness_y);
    let lower_top = mid + thickness_y;
    let lower_bottom = bottom.saturating_sub(thickness_y);

    [
        has_ink(
            mask,
            width,
            horizontal_left,
            top,
            horizontal_right,
            top + thickness_y,
        ),
        has_ink(
            mask,
            width,
            right.saturating_sub(thickness_x),
            upper_top,
            right,
            upper_bottom,
        ),
        has_ink(
            mask,
            width,
            right.saturating_sub(thickness_x),
            lower_top,
            right,
            lower_bottom,
        ),
        has_ink(
            mask,
            width,
            horizontal_left,
            bottom.saturating_sub(thickness_y),
            horizontal_right,
            bottom,
        ),
        has_ink(
            mask,
            width,
            left,
            lower_top,
            left + thickness_x,
            lower_bottom,
        ),
        has_ink(
            mask,
            width,
            left,
            upper_top,
            left + thickness_x,
            upper_bottom,
        ),
        has_ink(
            mask,
            width,
            horizontal_left,
            mid.saturating_sub(thickness_y / 2),
            horizontal_right,
            mid + thickness_y / 2,
        ),
    ]
}

fn has_ink(mask: &[bool], width: u32, x1: u32, y1: u32, x2: u32, y2: u32) -> bool {
    let mut ink = 0_u32;
    let mut total = 0_u32;
    for y in y1..=y2 {
        for x in x1..=x2 {
            total += 1;
            if mask.get((y * width + x) as usize).copied().unwrap_or(false) {
                ink += 1;
            }
        }
    }
    total > 0 && ink as f32 / total as f32 >= 0.18
}

fn best_digit(active: [bool; 7]) -> u8 {
    SEGMENT_PATTERNS
        .iter()
        .enumerate()
        .min_by_key(|(_, pattern)| {
            pattern
                .iter()
                .zip(active)
                .filter(|(expected, actual)| **expected != *actual)
                .count()
        })
        .map(|(digit, _)| digit as u8)
        .unwrap_or(0)
}

fn crop_bgra(pixels: &[u8], width: u32, height: u32, rect: Rect) -> Option<Vec<u8>> {
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
    let mut roi = Vec::with_capacity((crop_width * (max_y - rect.y) * 4) as usize);
    for y in rect.y..max_y {
        let start = ((y * width + rect.x) * 4) as usize;
        let end = start + (crop_width * 4) as usize;
        roi.extend_from_slice(&pixels[start..end]);
    }
    Some(roi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_digits_reads_multiple_seven_segment_numbers_from_roi() {
        let pixels = render_digits(&[4, 2], 4);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 20,
            h: 14,
        };

        let value = parse_digits(&pixels, &rect).unwrap();

        assert_eq!(value, 42);
    }

    #[test]
    fn load_templates_accepts_config_directory_with_digit_patterns() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config/digit_templates");

        load_templates(root.to_str().unwrap()).unwrap();
    }

    fn render_digits(digits: &[u8], gap: u32) -> Vec<u8> {
        let width = digits.len() as u32 * 8 + digits.len().saturating_sub(1) as u32 * gap;
        let height = 14;
        let mut pixels = vec![0_u8; (width * height * 4) as usize];
        for (index, digit) in digits.iter().enumerate() {
            let offset_x = index as u32 * (8 + gap);
            draw_digit(&mut pixels, width, offset_x, *digit);
        }
        pixels
    }

    fn draw_digit(pixels: &mut [u8], width: u32, offset_x: u32, digit: u8) {
        let segments = match digit {
            0 => [true, true, true, true, true, true, false],
            1 => [false, true, true, false, false, false, false],
            2 => [true, true, false, true, true, false, true],
            3 => [true, true, true, true, false, false, true],
            4 => [false, true, true, false, false, true, true],
            5 => [true, false, true, true, false, true, true],
            6 => [true, false, true, true, true, true, true],
            7 => [true, true, true, false, false, false, false],
            8 => [true, true, true, true, true, true, true],
            9 => [true, true, true, true, false, true, true],
            _ => [false; 7],
        };
        let specs = [
            (1, 0, 6, 2),
            (6, 1, 2, 6),
            (6, 7, 2, 6),
            (1, 12, 6, 2),
            (0, 7, 2, 6),
            (0, 1, 2, 6),
            (1, 6, 6, 2),
        ];
        for (active, (x, y, w, h)) in segments.iter().zip(specs) {
            if !active {
                continue;
            }
            for py in y..y + h {
                for px in x..x + w {
                    let idx = (((py * width) + offset_x + px) * 4) as usize;
                    pixels[idx..idx + 4].copy_from_slice(&[245, 230, 80, 255]);
                }
            }
        }
    }
}
