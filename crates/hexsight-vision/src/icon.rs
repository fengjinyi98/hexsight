// 图标模板匹配
// 核心职责：
// - 装备图标识别（感知哈希 + 汉明距离）
// - 英雄头像识别（归一化相关系数匹配 NCC）
// - 羁绊标识识别
// - 模板库加载与缓存管理

use std::collections::HashMap;
use std::fs;
use std::path::Path;
#[cfg(test)]
use std::sync::Mutex;
use std::sync::{OnceLock, RwLock};

use hexsight_core::{EquipmentType, HexError, HexResult};
use image::{DynamicImage, ImageBuffer, Rgba};
use serde_json::Value;

const FEATURE_SIZE: u32 = 16;
const HERO_MATCH_THRESHOLD: f32 = 0.88;
const ICON_MATCH_THRESHOLD: f32 = 0.92;

#[derive(Debug, Clone)]
struct IconTemplate {
    name: String,
    feature: Vec<f32>,
}

#[derive(Debug, Default)]
struct TemplateLibrary {
    heroes: Vec<IconTemplate>,
    equipment: Vec<IconTemplate>,
    traits: Vec<IconTemplate>,
    hero_costs: HashMap<String, u32>,
}

static TEMPLATE_LIBRARY: OnceLock<RwLock<TemplateLibrary>> = OnceLock::new();

#[cfg(test)]
pub(crate) fn template_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// 初始化图标模板库
/// template_dir: 模板图片目录路径
pub fn init_templates(template_dir: &str) -> HexResult<()> {
    let root = Path::new(template_dir);
    let hero_dir = if root.join("heroes").is_dir() {
        root.join("heroes")
    } else {
        root.to_path_buf()
    };
    let heroes = load_image_templates(&hero_dir)?;
    let equipment = load_image_templates(&root.join("equipment"))?;
    let traits = load_image_templates(&root.join("traits"))?;
    let hero_costs = load_hero_costs(root)?;

    let library = TEMPLATE_LIBRARY.get_or_init(|| RwLock::new(TemplateLibrary::default()));
    let mut guard = library
        .write()
        .map_err(|_| HexError::Vision("图标模板库写锁失败".into()))?;
    guard.heroes = heroes;
    guard.equipment = equipment;
    guard.traits = traits;
    guard.hero_costs = hero_costs;
    Ok(())
}

/// 匹配装备图标，返回装备名称
/// roi_pixels: 截取区域的像素数据
pub fn match_equipment(roi_pixels: &[u8]) -> HexResult<Option<String>> {
    let Some(side) = inferred_square_side(roi_pixels) else {
        return Ok(None);
    };
    match_equipment_with_size(roi_pixels, side, side)
}

/// 匹配指定尺寸装备图标，返回装备名称
pub fn match_equipment_with_size(
    roi_pixels: &[u8],
    width: u32,
    height: u32,
) -> HexResult<Option<String>> {
    match_template_group(roi_pixels, width, height, TemplateGroup::Equipment)
}

/// 匹配英雄头像，返回英雄名称
pub fn match_hero(roi_pixels: &[u8]) -> HexResult<Option<String>> {
    let Some(side) = inferred_square_side(roi_pixels) else {
        return Ok(None);
    };
    match_hero_with_size(roi_pixels, side, side)
}

/// 匹配指定尺寸英雄头像，返回英雄名称
pub fn match_hero_with_size(
    roi_pixels: &[u8],
    width: u32,
    height: u32,
) -> HexResult<Option<String>> {
    let Some(feature) = bgra_feature(roi_pixels, width, height) else {
        return Ok(None);
    };
    let cropped_feature = (height >= 8)
        .then(|| bgra_feature_crop(roi_pixels, width, height, 0, 0, width, height * 2 / 3))
        .flatten();
    let library = TEMPLATE_LIBRARY.get_or_init(|| RwLock::new(TemplateLibrary::default()));
    let guard = library
        .read()
        .map_err(|_| HexError::Vision("图标模板库读锁失败".into()))?;

    let best = guard
        .heroes
        .iter()
        .map(|template| {
            let full_score = feature_similarity(&feature, &template.feature);
            let cropped_score = cropped_feature
                .as_ref()
                .map(|candidate| feature_similarity(candidate, &template.feature))
                .unwrap_or(0.0);
            (template, full_score.max(cropped_score))
        })
        .max_by(|lhs, rhs| lhs.1.total_cmp(&rhs.1));

    match best {
        Some((template, score)) if score >= HERO_MATCH_THRESHOLD => Ok(Some(template.name.clone())),
        _ => Ok(None),
    }
}

/// 匹配羁绊标识
pub fn match_trait(_roi_pixels: &[u8]) -> HexResult<Option<String>> {
    let Some(side) = inferred_square_side(_roi_pixels) else {
        return Ok(None);
    };
    match_trait_with_size(_roi_pixels, side, side)
}

/// 匹配指定尺寸羁绊标识
pub fn match_trait_with_size(
    roi_pixels: &[u8],
    width: u32,
    height: u32,
) -> HexResult<Option<String>> {
    match_template_group(roi_pixels, width, height, TemplateGroup::Trait)
}

/// 查询英雄费用
pub fn hero_cost(name: &str) -> Option<u32> {
    let library = TEMPLATE_LIBRARY.get_or_init(|| RwLock::new(TemplateLibrary::default()));
    let guard = library.read().ok()?;
    guard.hero_costs.get(name).copied()
}

/// 推断装备类型
pub fn equipment_type(name: &str) -> EquipmentType {
    if name.contains("纹章") || name.contains("转职") {
        EquipmentType::Emblem
    } else if [
        "狂徒",
        "反甲",
        "龙牙",
        "石像鬼",
        "救赎",
        "日炎",
        "坚定",
        "冠冕",
    ]
    .iter()
    .any(|keyword| name.contains(keyword))
    {
        EquipmentType::Defensive
    } else if [
        "蓝霸符",
        "青龙刀",
        "纳什",
        "电刀",
        "圣杯",
        "基克",
        "灵风",
        "传送门",
    ]
    .iter()
    .any(|keyword| name.contains(keyword))
    {
        EquipmentType::Utility
    } else {
        EquipmentType::Offensive
    }
}

/// 推断装备是否为成装
pub fn equipment_completed(name: &str) -> bool {
    ![
        "暴风大剑",
        "反曲之弓",
        "无用大棒",
        "女神之泪",
        "巨人腰带",
        "锁子甲",
        "负极斗篷",
        "拳套",
        "金铲铲",
    ]
    .iter()
    .any(|keyword| name.contains(keyword))
}

#[derive(Clone, Copy)]
enum TemplateGroup {
    Equipment,
    Trait,
}

fn match_template_group(
    roi_pixels: &[u8],
    width: u32,
    height: u32,
    group: TemplateGroup,
) -> HexResult<Option<String>> {
    let Some(feature) = bgra_feature(roi_pixels, width, height) else {
        return Ok(None);
    };
    let library = TEMPLATE_LIBRARY.get_or_init(|| RwLock::new(TemplateLibrary::default()));
    let guard = library
        .read()
        .map_err(|_| HexError::Vision("图标模板库读锁失败".into()))?;
    let templates = match group {
        TemplateGroup::Equipment => &guard.equipment,
        TemplateGroup::Trait => &guard.traits,
    };
    let best = templates
        .iter()
        .map(|template| (template, feature_similarity(&feature, &template.feature)))
        .max_by(|lhs, rhs| lhs.1.total_cmp(&rhs.1));

    match best {
        Some((template, score)) if score >= ICON_MATCH_THRESHOLD => Ok(Some(template.name.clone())),
        _ => Ok(None),
    }
}

fn load_image_templates(dir: &Path) -> HexResult<Vec<IconTemplate>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();
    for entry in
        fs::read_dir(dir).map_err(|e| HexError::Vision(format!("读取模板目录失败: {}", e)))?
    {
        let path = entry
            .map_err(|e| HexError::Vision(format!("读取模板文件失败: {}", e)))?
            .path();
        if !is_supported_template(&path) {
            continue;
        }
        let Some(name) = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        let image = load_template_image(&path)?;
        templates.push(IconTemplate {
            name,
            feature: image_feature(&image),
        });
    }

    templates.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    Ok(templates)
}

fn load_hero_costs(template_root: &Path) -> HexResult<HashMap<String, u32>> {
    let Some(config_root) = template_root.parent() else {
        return Ok(HashMap::new());
    };
    let game_data_root = config_root.join("game_data");
    if !game_data_root.is_dir() {
        return Ok(HashMap::new());
    }

    let mut costs = HashMap::new();
    for entry in fs::read_dir(&game_data_root)
        .map_err(|e| HexError::Vision(format!("读取游戏数据目录失败: {}", e)))?
    {
        let path = entry
            .map_err(|e| HexError::Vision(format!("读取游戏数据项失败: {}", e)))?
            .path()
            .join("chess.json");
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| HexError::Vision(format!("读取英雄数据失败 {}: {}", path.display(), e)))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| HexError::Vision(format!("解析英雄数据失败 {}: {}", path.display(), e)))?;
        let Some(map) = value
            .get("data")
            .and_then(Value::as_object)
            .or_else(|| value.as_object())
        else {
            continue;
        };
        for hero in map.values() {
            let Some(name) = hero.get("name").and_then(Value::as_str) else {
                continue;
            };
            let cost = hero
                .get("price")
                .and_then(Value::as_str)
                .and_then(|value| value.parse::<u32>().ok())
                .or_else(|| {
                    hero.get("price")
                        .and_then(Value::as_u64)
                        .map(|value| value as u32)
                })
                .unwrap_or(0);
            if cost > 0 {
                costs.entry(name.to_string()).or_insert(cost);
            }
        }
    }
    Ok(costs)
}

fn is_supported_template(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "txt")
    )
}

fn load_template_image(path: &Path) -> HexResult<DynamicImage> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
    {
        return load_text_bitmap(path);
    }
    image::open(path)
        .map_err(|e| HexError::Vision(format!("加载模板图片失败 {}: {}", path.display(), e)))
}

fn load_text_bitmap(path: &Path) -> HexResult<DynamicImage> {
    let content = fs::read_to_string(path)
        .map_err(|e| HexError::Vision(format!("读取文本模板失败 {}: {}", path.display(), e)))?;
    let rows: Vec<&str> = content
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect();
    let height = rows.len() as u32;
    let width = rows
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u32;
    if width == 0 || height == 0 {
        return Err(HexError::Vision(format!(
            "文本模板为空: {}",
            path.display()
        )));
    }
    let mut image = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 255]));
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let color = match ch {
                '#' => Rgba([240, 240, 240, 255]),
                'R' | 'r' => Rgba([220, 45, 55, 255]),
                'G' | 'g' => Rgba([45, 210, 80, 255]),
                'B' | 'b' => Rgba([65, 125, 230, 255]),
                'Y' | 'y' => Rgba([240, 205, 60, 255]),
                'P' | 'p' => Rgba([190, 90, 230, 255]),
                _ => Rgba([0, 0, 0, 255]),
            };
            image.put_pixel(x as u32, y as u32, color);
        }
    }
    Ok(DynamicImage::ImageRgba8(image))
}

fn image_feature(image: &DynamicImage) -> Vec<f32> {
    let resized = image.resize_exact(
        FEATURE_SIZE,
        FEATURE_SIZE,
        image::imageops::FilterType::Triangle,
    );
    let rgba = resized.to_rgba8();
    rgba.pixels()
        .flat_map(|pixel| {
            [
                f32::from(pixel[0]) / 255.0,
                f32::from(pixel[1]) / 255.0,
                f32::from(pixel[2]) / 255.0,
            ]
        })
        .collect()
}

fn bgra_feature(pixels: &[u8], width: u32, height: u32) -> Option<Vec<f32>> {
    if width == 0 || height == 0 || pixels.len() < (width as usize * height as usize * 4) {
        return None;
    }
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for chunk in pixels.chunks_exact(4).take((width * height) as usize) {
        rgba.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
    }
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba)?;
    Some(image_feature(&DynamicImage::ImageRgba8(image)))
}

fn bgra_feature_crop(
    pixels: &[u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    crop_width: u32,
    crop_height: u32,
) -> Option<Vec<f32>> {
    if width == 0
        || height == 0
        || crop_width == 0
        || crop_height == 0
        || x.checked_add(crop_width)? > width
        || y.checked_add(crop_height)? > height
        || pixels.len() < (width as usize * height as usize * 4)
    {
        return None;
    }

    let mut rgba = Vec::with_capacity((crop_width * crop_height * 4) as usize);
    for row in y..y + crop_height {
        for col in x..x + crop_width {
            let offset = ((row * width + col) * 4) as usize;
            rgba.extend_from_slice(&[
                pixels[offset + 2],
                pixels[offset + 1],
                pixels[offset],
                pixels[offset + 3],
            ]);
        }
    }
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(crop_width, crop_height, rgba)?;
    Some(image_feature(&DynamicImage::ImageRgba8(image)))
}

fn feature_similarity(lhs: &[f32], rhs: &[f32]) -> f32 {
    if lhs.len() != rhs.len() || lhs.is_empty() {
        return 0.0;
    }
    let mse = lhs
        .iter()
        .zip(rhs.iter())
        .map(|(a, b)| {
            let delta = a - b;
            delta * delta
        })
        .sum::<f32>()
        / lhs.len() as f32;
    (1.0 - mse.sqrt()).clamp(0.0, 1.0)
}

fn inferred_square_side(pixels: &[u8]) -> Option<u32> {
    if !pixels.len().is_multiple_of(4) {
        return None;
    }
    let pixel_count = pixels.len() / 4;
    let side = (pixel_count as f64).sqrt() as usize;
    if side * side == pixel_count {
        Some(side as u32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn match_hero_returns_loaded_template_name() {
        let _guard = template_test_lock().lock().unwrap();
        let dir =
            std::env::temp_dir().join(format!("hexsight_hero_templates_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("heroes")).unwrap();
        let template_path = dir.join("heroes/测试英雄.png");
        let image = ImageBuffer::from_pixel(8, 8, Rgba([18_u8, 90_u8, 160_u8, 255_u8]));
        image.save(&template_path).unwrap();

        init_templates(dir.to_str().unwrap()).unwrap();
        let mut bgra = Vec::new();
        for _ in 0..64 {
            bgra.extend_from_slice(&[160, 90, 18, 255]);
        }

        let matched = match_hero(&bgra).unwrap();

        assert_eq!(matched.as_deref(), Some("测试英雄"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn real_mode17_hero_template_is_matchable() {
        let _guard = template_test_lock().lock().unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config/vision_templates");
        init_templates(root.to_str().unwrap()).unwrap();

        let image = image::open(root.join("heroes/丽桑卓.png"))
            .expect("真实 mode17 英雄模板应已落盘")
            .to_rgba8();
        let mut bgra = Vec::with_capacity((image.width() * image.height() * 4) as usize);
        for pixel in image.pixels() {
            bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }

        let matched = match_hero_with_size(&bgra, image.width(), image.height()).unwrap();

        assert_eq!(matched.as_deref(), Some("丽桑卓"));
    }

    #[test]
    fn match_equipment_and_trait_return_loaded_template_names() {
        let _guard = template_test_lock().lock().unwrap();
        let dir =
            std::env::temp_dir().join(format!("hexsight_icon_templates_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("equipment")).unwrap();
        std::fs::create_dir_all(dir.join("traits")).unwrap();
        std::fs::write(
            dir.join("equipment/暴风大剑.txt"),
            "RRRRRRRR\nRR....RR\nRRRRRRRR\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("traits/裁决使.txt"),
            "YYYYYYYY\nYY....YY\nYYYYYYYY\n",
        )
        .unwrap();

        init_templates(dir.to_str().unwrap()).unwrap();

        let equipment = bgra_from_rows(&["RRRRRRRR", "RR....RR", "RRRRRRRR"]);
        let matched_equipment = match_equipment_with_size(&equipment, 8, 3).unwrap();
        let trait_icon = bgra_from_rows(&["YYYYYYYY", "YY....YY", "YYYYYYYY"]);
        let matched_trait = match_trait_with_size(&trait_icon, 8, 3).unwrap();

        assert_eq!(matched_equipment.as_deref(), Some("暴风大剑"));
        assert_eq!(matched_trait.as_deref(), Some("裁决使"));
        assert_eq!(equipment_type("魔法师纹章"), EquipmentType::Emblem);
        assert!(!equipment_completed("暴风大剑"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn real_config_initialization_loads_hero_costs() {
        let _guard = template_test_lock().lock().unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config/vision_templates");

        init_templates(root.to_str().unwrap()).unwrap();

        assert_eq!(hero_cost("伊泽瑞尔"), Some(1));
    }

    fn bgra_from_rows(rows: &[&str]) -> Vec<u8> {
        let mut pixels = Vec::new();
        for row in rows {
            for ch in row.chars() {
                let rgba = match ch {
                    'R' => [220, 45, 55, 255],
                    'Y' => [240, 205, 60, 255],
                    _ => [0, 0, 0, 255],
                };
                pixels.extend_from_slice(&[rgba[2], rgba[1], rgba[0], rgba[3]]);
            }
        }
        pixels
    }
}
