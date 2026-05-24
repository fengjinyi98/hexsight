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

use std::time::Instant;

use hexsight_core::{Equipment, GamePhase, Hero, HexResult, RecognizedFrame, Rect, RegionConfig};

/// 单帧识别性能报告
/// 核心职责：
/// - 保存识别结果
/// - 暴露单帧耗时用于性能基准
#[derive(Debug, Clone)]
pub struct RecognitionReport {
    pub frame: RecognizedFrame,
    pub elapsed_ms: f64,
}

/// 识别单帧画面
/// pixels: BGRA 格式像素数据
/// regions: 固定分辨率 ROI 坐标配置
pub fn recognize(
    pixels: &[u8],
    regions: &RegionConfig,
    width: u32,
    height: u32,
) -> HexResult<RecognizedFrame> {
    Ok(RecognizedFrame {
        gold: digit::parse_digits_from_frame(
            pixels,
            width,
            height,
            scale_rect(regions.gold_rect, regions.resolution, width, height),
        )?,
        hp: bar::scan_hp_bar_from_frame(
            pixels,
            width,
            height,
            scale_rect(regions.hp_bar, regions.resolution, width, height),
        )?,
        level: digit::parse_digits_from_frame(
            pixels,
            width,
            height,
            scale_rect(regions.level_rect, regions.resolution, width, height),
        )?,
        exp: bar::scan_exp_bar_from_frame(
            pixels,
            width,
            height,
            scale_rect(regions.exp_bar, regions.resolution, width, height),
        )?,
        own_heroes: recognize_board_heroes(pixels, regions, width, height)?,
        bench_heroes: recognize_bench_heroes(pixels, regions, width, height)?,
        opponent_heroes: recognize_opponent_heroes(pixels, regions, width, height)?,
        own_equipment: recognize_equipment(pixels, regions, width, height)?,
        carousel_equipment: recognize_carousel_equipment(pixels, regions, width, height)?,
        phase: classify_phase(pixels, regions, width, height),
        ..Default::default()
    })
}

/// 识别单帧画面并返回耗时
pub fn recognize_measured(
    pixels: &[u8],
    regions: &RegionConfig,
    width: u32,
    height: u32,
) -> HexResult<RecognitionReport> {
    let started = Instant::now();
    let frame = recognize(pixels, regions, width, height)?;
    Ok(RecognitionReport {
        frame,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
    })
}

/// 根据关键 ROI 像素活动推断当前场景
pub fn classify_phase(pixels: &[u8], regions: &RegionConfig, width: u32, height: u32) -> GamePhase {
    let hextech_activity = regions
        .hextech_rects
        .iter()
        .map(|rect| scale_rect(*rect, regions.resolution, width, height))
        .map(|rect| rect_activity(pixels, width, height, rect))
        .fold(0.0_f32, f32::max);
    if hextech_activity >= 0.30 {
        return GamePhase::HextechSelect;
    }

    let shop_activity = regions
        .shop_slots
        .iter()
        .map(|rect| scale_rect(*rect, regions.resolution, width, height))
        .map(|rect| rect_activity(pixels, width, height, rect))
        .fold(0.0_f32, f32::max);
    if shop_activity >= 0.30 {
        return GamePhase::Shop;
    }

    let board_activity = regions
        .board_grid
        .iter()
        .map(|rect| scale_rect(*rect, regions.resolution, width, height))
        .map(|rect| rect_activity(pixels, width, height, rect))
        .fold(0.0_f32, f32::max);
    if board_activity >= 0.20 {
        return GamePhase::Hud;
    }

    GamePhase::Unknown
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
                cost: icon::hero_cost(&name).unwrap_or(0),
                star: detect_star_level(&roi, scaled.w, scaled.h),
                name,
                position: ((index / 7) as u32, (index % 7) as u32),
                items: recognize_hero_items(&roi, scaled.w, scaled.h)?,
            });
        }
    }
    Ok(heroes)
}

fn recognize_opponent_heroes(
    pixels: &[u8],
    regions: &RegionConfig,
    width: u32,
    height: u32,
) -> HexResult<Vec<Hero>> {
    let mut heroes = Vec::new();
    for (index, rect) in regions.opponent_board_grid.iter().enumerate() {
        let scaled = scale_rect(*rect, regions.resolution, width, height);
        let Some(roi) = crop_bgra(pixels, width, height, scaled) else {
            continue;
        };
        if let Some(name) = icon::match_hero_with_size(&roi, scaled.w, scaled.h)? {
            heroes.push(Hero {
                cost: icon::hero_cost(&name).unwrap_or(0),
                star: detect_star_level(&roi, scaled.w, scaled.h),
                name,
                position: ((index / 7) as u32, (index % 7) as u32),
                items: recognize_hero_items(&roi, scaled.w, scaled.h)?,
            });
        }
    }
    Ok(heroes)
}

fn recognize_bench_heroes(
    pixels: &[u8],
    regions: &RegionConfig,
    width: u32,
    height: u32,
) -> HexResult<Vec<Hero>> {
    let mut heroes = Vec::new();
    for (index, rect) in regions.bench_slots.iter().enumerate() {
        let scaled = scale_rect(*rect, regions.resolution, width, height);
        let Some(roi) = crop_bgra(pixels, width, height, scaled) else {
            continue;
        };
        if let Some(name) = icon::match_hero_with_size(&roi, scaled.w, scaled.h)? {
            heroes.push(Hero {
                cost: icon::hero_cost(&name).unwrap_or(0),
                star: detect_star_level(&roi, scaled.w, scaled.h),
                name,
                position: (0, index as u32),
                items: Vec::new(),
            });
        }
    }
    Ok(heroes)
}

fn recognize_equipment(
    pixels: &[u8],
    regions: &RegionConfig,
    width: u32,
    height: u32,
) -> HexResult<Vec<Equipment>> {
    let mut equipment = Vec::new();
    for rect in &regions.equip_slots {
        let scaled = scale_rect(*rect, regions.resolution, width, height);
        let Some(roi) = crop_bgra(pixels, width, height, scaled) else {
            continue;
        };
        if let Some(name) = icon::match_equipment_with_size(&roi, scaled.w, scaled.h)? {
            equipment.push(Equipment {
                equip_type: icon::equipment_type(&name),
                completed: icon::equipment_completed(&name),
                name,
            });
        }
    }
    Ok(equipment)
}

fn recognize_carousel_equipment(
    pixels: &[u8],
    regions: &RegionConfig,
    width: u32,
    height: u32,
) -> HexResult<Vec<Equipment>> {
    let mut equipment = Vec::new();
    for rect in &regions.carousel_slots {
        let scaled = scale_rect(*rect, regions.resolution, width, height);
        let Some(roi) = crop_bgra(pixels, width, height, scaled) else {
            continue;
        };
        if let Some(name) = icon::match_equipment_with_size(&roi, scaled.w, scaled.h)? {
            equipment.push(Equipment {
                equip_type: icon::equipment_type(&name),
                completed: icon::equipment_completed(&name),
                name,
            });
        }
    }
    Ok(equipment)
}

fn recognize_hero_items(roi: &[u8], width: u32, height: u32) -> HexResult<Vec<Equipment>> {
    if width < 12 || height < 12 {
        return Ok(Vec::new());
    }

    let slot = (width / 3).max(8).min(height / 2);
    let top = height.saturating_sub(slot);
    let starts = [0_u32, slot, slot * 2];
    let mut items = Vec::new();
    for start in starts {
        if start + slot > width {
            continue;
        }
        let rect = Rect {
            x: start,
            y: top,
            w: slot,
            h: slot,
        };
        let Some(item_roi) = crop_bgra(roi, width, height, rect) else {
            continue;
        };
        if let Some(name) = icon::match_equipment_with_size(&item_roi, slot, slot)? {
            items.push(Equipment {
                equip_type: icon::equipment_type(&name),
                completed: icon::equipment_completed(&name),
                name,
            });
        }
    }
    Ok(items)
}

fn detect_star_level(roi: &[u8], width: u32, height: u32) -> u32 {
    if width == 0 || height == 0 || roi.len() < (width as usize * height as usize * 4) {
        return 1;
    }
    let scan_top = height.saturating_mul(3) / 4;
    let mut runs = 0_u32;
    let mut in_run = false;
    for x in 0..width {
        let has_star = (scan_top..height).any(|y| {
            let offset = ((y * width + x) * 4) as usize;
            let b = roi[offset];
            let g = roi[offset + 1];
            let r = roi[offset + 2];
            r >= 180 && g >= 140 && b <= 90
        });
        if has_star && !in_run {
            runs += 1;
            in_run = true;
        } else if !has_star {
            in_run = false;
        }
    }
    runs.clamp(1, 3)
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

fn rect_activity(pixels: &[u8], width: u32, height: u32, rect: Rect) -> f32 {
    if width == 0
        || height == 0
        || rect.w == 0
        || rect.h == 0
        || pixels.len() < width as usize * height as usize * 4
    {
        return 0.0;
    }
    let max_x = rect.x.saturating_add(rect.w).min(width);
    let max_y = rect.y.saturating_add(rect.h).min(height);
    if rect.x >= max_x || rect.y >= max_y {
        return 0.0;
    }

    let mut active = 0_u32;
    let mut total = 0_u32;
    for y in rect.y..max_y {
        for x in rect.x..max_x {
            let offset = ((y * width + x) * 4) as usize;
            let b = pixels[offset];
            let g = pixels[offset + 1];
            let r = pixels[offset + 2];
            let brightness = (u16::from(r) + u16::from(g) + u16::from(b)) / 3;
            let chroma = r.max(g).max(b) - r.min(g).min(b);
            total += 1;
            if brightness >= 45 && chroma >= 35 {
                active += 1;
            }
        }
    }
    active as f32 / total.max(1) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn classify_phase_routes_shop_and_hextech_fingerprints() {
        let regions = RegionConfig {
            resolution: (32, 32),
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
                x: 0,
                y: 0,
                w: 4,
                h: 4,
            }],
            shop_slots: vec![
                Rect {
                    x: 0,
                    y: 20,
                    w: 4,
                    h: 4,
                },
                Rect {
                    x: 5,
                    y: 20,
                    w: 4,
                    h: 4,
                },
                Rect {
                    x: 10,
                    y: 20,
                    w: 4,
                    h: 4,
                },
            ],
            bench_slots: Vec::new(),
            opponent_board_grid: Vec::new(),
            carousel_slots: Vec::new(),
            hextech_rects: vec![
                Rect {
                    x: 8,
                    y: 8,
                    w: 4,
                    h: 4,
                },
                Rect {
                    x: 14,
                    y: 8,
                    w: 4,
                    h: 4,
                },
            ],
            opponent_rects: Vec::new(),
            streak_rect: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
        };
        let mut shop_pixels = vec![0_u8; 32 * 32 * 4];
        for rect in &regions.shop_slots {
            fill_rect(&mut shop_pixels, 32, *rect, [90, 180, 230, 255]);
        }
        let mut hextech_pixels = vec![0_u8; 32 * 32 * 4];
        for rect in &regions.hextech_rects {
            fill_rect(&mut hextech_pixels, 32, *rect, [180, 90, 230, 255]);
        }

        assert_eq!(
            classify_phase(&shop_pixels, &regions, 32, 32),
            GamePhase::Shop
        );
        assert_eq!(
            classify_phase(&hextech_pixels, &regions, 32, 32),
            GamePhase::HextechSelect
        );
    }

    #[test]
    fn recognize_measured_returns_frame_and_elapsed_time() {
        let regions = RegionConfig {
            resolution: (4, 4),
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
            board_grid: Vec::new(),
            shop_slots: Vec::new(),
            bench_slots: Vec::new(),
            opponent_board_grid: Vec::new(),
            carousel_slots: Vec::new(),
            hextech_rects: Vec::new(),
            opponent_rects: Vec::new(),
            streak_rect: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
        };
        let pixels = vec![0_u8; 4 * 4 * 4];

        let report = recognize_measured(&pixels, &regions, 4, 4).unwrap();

        assert_eq!(report.frame.phase, GamePhase::Unknown);
        assert!(report.elapsed_ms >= 0.0);
    }

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
            bench_slots: Vec::new(),
            opponent_board_grid: Vec::new(),
            carousel_slots: Vec::new(),
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

    #[test]
    fn recognize_populates_equipment_bench_cost_and_star_fields() {
        let _guard = icon::template_test_lock().lock().unwrap();
        let dir =
            std::env::temp_dir().join(format!("hexsight_full_templates_{}", std::process::id()));
        let template_root = dir.join("config/vision_templates");
        let game_data_root = dir.join("config/game_data/mode-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(template_root.join("heroes")).unwrap();
        std::fs::create_dir_all(template_root.join("equipment")).unwrap();
        std::fs::create_dir_all(&game_data_root).unwrap();
        ImageBuffer::from_pixel(8, 8, Rgba([220_u8, 40_u8, 70_u8, 255_u8]))
            .save(template_root.join("heroes/测试英雄.png"))
            .unwrap();
        std::fs::write(
            template_root.join("equipment/暴风大剑.txt"),
            "RRRRRRRR\nRR....RR\nRR....RR\nRR....RR\nRR....RR\nRR....RR\nRR....RR\nRRRRRRRR\n",
        )
        .unwrap();
        std::fs::write(
            game_data_root.join("chess.json"),
            r#"{"data":{"1":{"name":"测试英雄","price":"3"}}}"#,
        )
        .unwrap();
        icon::init_templates(template_root.to_str().unwrap()).unwrap();

        let regions = RegionConfig {
            resolution: (24, 24),
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
            equip_slots: vec![Rect {
                x: 16,
                y: 0,
                w: 8,
                h: 8,
            }],
            board_grid: vec![Rect {
                x: 0,
                y: 0,
                w: 8,
                h: 8,
            }],
            shop_slots: Vec::new(),
            bench_slots: vec![Rect {
                x: 0,
                y: 12,
                w: 8,
                h: 8,
            }],
            opponent_board_grid: Vec::new(),
            carousel_slots: Vec::new(),
            hextech_rects: Vec::new(),
            opponent_rects: Vec::new(),
            streak_rect: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
        };
        let mut pixels = vec![0_u8; 24 * 24 * 4];
        fill_rect(
            &mut pixels,
            24,
            Rect {
                x: 0,
                y: 0,
                w: 8,
                h: 8,
            },
            [70, 40, 220, 255],
        );
        draw_star_runs(&mut pixels, 24, 0, 0, &[1, 4]);
        fill_rect(
            &mut pixels,
            24,
            Rect {
                x: 0,
                y: 12,
                w: 8,
                h: 8,
            },
            [70, 40, 220, 255],
        );
        draw_equipment_icon(&mut pixels, 24, 16, 0);

        let frame = recognize(&pixels, &regions, 24, 24).unwrap();

        assert_eq!(frame.own_heroes.len(), 1);
        assert_eq!(frame.own_heroes[0].name, "测试英雄");
        assert_eq!(frame.own_heroes[0].cost, 3);
        assert_eq!(frame.own_heroes[0].star, 2);
        assert_eq!(frame.bench_heroes.len(), 1);
        assert_eq!(frame.bench_heroes[0].name, "测试英雄");
        assert_eq!(frame.own_equipment.len(), 1);
        assert_eq!(frame.own_equipment[0].name, "暴风大剑");
        assert!(!frame.own_equipment[0].completed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn recognize_populates_opponent_carousel_and_hero_item_outputs() {
        let _guard = icon::template_test_lock().lock().unwrap();
        let dir =
            std::env::temp_dir().join(format!("hexsight_p2_templates_{}", std::process::id()));
        let template_root = dir.join("config/vision_templates");
        let game_data_root = dir.join("config/game_data/mode-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(template_root.join("heroes")).unwrap();
        std::fs::create_dir_all(template_root.join("equipment")).unwrap();
        std::fs::create_dir_all(&game_data_root).unwrap();
        ImageBuffer::from_pixel(16, 16, Rgba([220_u8, 40_u8, 70_u8, 255_u8]))
            .save(template_root.join("heroes/测试英雄.png"))
            .unwrap();
        std::fs::write(
            template_root.join("equipment/暴风大剑.txt"),
            "RRRRRRRR\nRR....RR\nRR....RR\nRR....RR\nRR....RR\nRR....RR\nRR....RR\nRRRRRRRR\n",
        )
        .unwrap();
        std::fs::write(
            game_data_root.join("chess.json"),
            r#"{"data":{"1":{"name":"测试英雄","price":"3"}}}"#,
        )
        .unwrap();
        icon::init_templates(template_root.to_str().unwrap()).unwrap();

        let regions = RegionConfig {
            resolution: (48, 48),
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
                x: 0,
                y: 0,
                w: 16,
                h: 16,
            }],
            shop_slots: Vec::new(),
            bench_slots: Vec::new(),
            opponent_board_grid: vec![Rect {
                x: 20,
                y: 0,
                w: 16,
                h: 16,
            }],
            carousel_slots: vec![Rect {
                x: 0,
                y: 32,
                w: 8,
                h: 8,
            }],
            hextech_rects: Vec::new(),
            opponent_rects: Vec::new(),
            streak_rect: Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
        };
        let mut pixels = vec![0_u8; 48 * 48 * 4];
        fill_rect(
            &mut pixels,
            48,
            Rect {
                x: 0,
                y: 0,
                w: 16,
                h: 16,
            },
            [70, 40, 220, 255],
        );
        fill_rect(
            &mut pixels,
            48,
            Rect {
                x: 0,
                y: 7,
                w: 8,
                h: 8,
            },
            [0, 0, 0, 255],
        );
        draw_equipment_icon(&mut pixels, 48, 0, 8);
        fill_rect(
            &mut pixels,
            48,
            Rect {
                x: 20,
                y: 0,
                w: 16,
                h: 16,
            },
            [70, 40, 220, 255],
        );
        draw_equipment_icon(&mut pixels, 48, 0, 32);

        let hero_roi = crop_bgra(
            &pixels,
            48,
            48,
            Rect {
                x: 0,
                y: 0,
                w: 16,
                h: 16,
            },
        )
        .unwrap();
        let item_roi = crop_bgra(
            &hero_roi,
            16,
            16,
            Rect {
                x: 0,
                y: 8,
                w: 8,
                h: 8,
            },
        )
        .unwrap();
        assert_eq!(
            icon::match_equipment_with_size(&item_roi, 8, 8)
                .unwrap()
                .as_deref(),
            Some("暴风大剑")
        );

        let frame = recognize(&pixels, &regions, 48, 48).unwrap();

        assert_eq!(frame.own_heroes.len(), 1);
        assert_eq!(frame.own_heroes[0].items.len(), 1);
        assert_eq!(frame.own_heroes[0].items[0].name, "暴风大剑");
        assert_eq!(frame.opponent_heroes.len(), 1);
        assert_eq!(frame.opponent_heroes[0].name, "测试英雄");
        assert_eq!(frame.carousel_equipment.len(), 1);
        assert_eq!(frame.carousel_equipment[0].name, "暴风大剑");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn fill_rect(pixels: &mut [u8], width: u32, rect: Rect, bgra: [u8; 4]) {
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                let offset = ((y * width + x) * 4) as usize;
                pixels[offset..offset + 4].copy_from_slice(&bgra);
            }
        }
    }

    fn draw_star_runs(pixels: &mut [u8], width: u32, x: u32, y: u32, starts: &[u32]) {
        for start in starts {
            for px in x + start..x + start + 2 {
                let offset = (((y + 6) * width + px) * 4) as usize;
                pixels[offset..offset + 4].copy_from_slice(&[40, 190, 240, 255]);
            }
        }
    }

    fn draw_equipment_icon(pixels: &mut [u8], width: u32, x: u32, y: u32) {
        let rows = [
            "RRRRRRRR", "RR....RR", "RR....RR", "RR....RR", "RR....RR", "RR....RR", "RR....RR",
            "RRRRRRRR",
        ];
        for (row_index, row) in rows.iter().enumerate() {
            for (col_index, ch) in row.chars().enumerate() {
                let color = if ch == 'R' {
                    [55, 45, 220, 255]
                } else {
                    [0, 0, 0, 255]
                };
                let offset = (((y + row_index as u32) * width + x + col_index as u32) * 4) as usize;
                pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }
}
