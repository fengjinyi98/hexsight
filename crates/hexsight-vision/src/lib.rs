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

use hexsight_core::{Hero, HexResult, RecognizedFrame, Rect, RegionConfig};

/// 识别单帧画面
/// pixels: BGRA 格式像素数据
/// regions: 固定分辨率 ROI 坐标配置
pub fn recognize(
    pixels: &[u8],
    regions: &RegionConfig,
    width: u32,
    height: u32,
) -> HexResult<RecognizedFrame> {
    let mut frame = RecognizedFrame::default();

    frame.gold = digit::parse_digits(pixels, &regions.gold_rect)?;
    frame.hp = bar::scan_hp_bar(pixels, &regions.hp_bar)?;
    frame.level = digit::parse_digits(pixels, &regions.level_rect)?;
    frame.exp = bar::scan_exp_bar(pixels, &regions.exp_bar)?;
    frame.own_heroes = recognize_board_heroes(pixels, regions, width, height)?;

    Ok(frame)
}

fn recognize_board_heroes(
    pixels: &[u8],
    regions: &RegionConfig,
    width: u32,
    height: u32,
) -> HexResult<Vec<Hero>> {
    let mut heroes = Vec::new();
    for (index, rect) in regions.board_grid.iter().enumerate() {
        let scaled = scale_rect(*rect, regions.resolution, width, height);
        let Some(roi) = crop_bgra(pixels, width, height, scaled) else {
            continue;
        };
        if let Some(name) = icon::match_hero_with_size(&roi, scaled.w, scaled.h)? {
            heroes.push(Hero {
                name,
                cost: 0,
                star: 1,
                position: ((index / 7) as u32, (index % 7) as u32),
                items: Vec::new(),
            });
        }
    }
    Ok(heroes)
}

fn scale_rect(rect: Rect, resolution: (u32, u32), width: u32, height: u32) -> Rect {
    let base_width = resolution.0.max(1) as f32;
    let base_height = resolution.1.max(1) as f32;
    Rect {
        x: ((rect.x as f32 / base_width) * width as f32).round() as u32,
        y: ((rect.y as f32 / base_height) * height as f32).round() as u32,
        w: ((rect.w as f32 / base_width) * width as f32)
            .round()
            .max(1.0) as u32,
        h: ((rect.h as f32 / base_height) * height as f32)
            .round()
            .max(1.0) as u32,
    }
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
    let source_len = width as usize * height as usize * 4;
    if pixels.len() < source_len {
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
    Some(roi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn recognize_populates_board_heroes_from_templates() {
        let _guard = icon::template_test_lock().lock().unwrap();
        let dir = std::env::temp_dir().join(format!(
            "hexsight_board_hero_templates_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("heroes")).unwrap();
        ImageBuffer::from_pixel(8, 8, Rgba([220_u8, 40_u8, 70_u8, 255_u8]))
            .save(dir.join("heroes/测试英雄.png"))
            .unwrap();
        icon::init_templates(dir.to_str().unwrap()).unwrap();

        let regions = RegionConfig {
            resolution: (16, 16),
            gold_rect: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            hp_bar: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            level_rect: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            exp_bar: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            round_rect: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            equip_slots: Vec::new(),
            board_grid: vec![Rect {
                x: 4,
                y: 4,
                w: 8,
                h: 8,
            }],
            shop_slots: Vec::new(),
            hextech_rects: Vec::new(),
            opponent_rects: Vec::new(),
            streak_rect: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
        };
        let mut pixels = vec![0_u8; 16 * 16 * 4];
        for y in 4..12 {
            for x in 4..12 {
                let offset = ((y * 16 + x) * 4) as usize;
                pixels[offset..offset + 4].copy_from_slice(&[70, 40, 220, 255]);
            }
        }

        let frame = recognize(&pixels, &regions, 16, 16).unwrap();

        assert_eq!(frame.own_heroes.len(), 1);
        assert_eq!(frame.own_heroes[0].name, "测试英雄");
        assert_eq!(frame.own_heroes[0].position, (0, 0));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
