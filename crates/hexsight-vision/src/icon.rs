// 图标模板匹配
// 核心职责：
// - 装备图标识别（感知哈希 + 汉明距离）
// - 英雄头像识别（归一化相关系数匹配 NCC）
// - 羁绊标识识别
// - 模板库加载与缓存管理

use std::fs;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Mutex;
use std::sync::{OnceLock, RwLock};

use hexsight_core::{HexError, HexResult};
use image::{DynamicImage, ImageBuffer, Rgba};

const FEATURE_SIZE: u32 = 16;
const HERO_MATCH_THRESHOLD: f32 = 0.94;

#[derive(Debug, Clone)]
struct IconTemplate {
    name: String,
    feature: Vec<f32>,
}

#[derive(Debug, Default)]
struct TemplateLibrary {
    heroes: Vec<IconTemplate>,
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

    let library = TEMPLATE_LIBRARY.get_or_init(|| RwLock::new(TemplateLibrary::default()));
    let mut guard = library
        .write()
        .map_err(|_| HexError::Vision("图标模板库写锁失败".into()))?;
    guard.heroes = heroes;
    Ok(())
}

/// 匹配装备图标，返回装备名称
/// roi_pixels: 截取区域的像素数据
pub fn match_equipment(_roi_pixels: &[u8]) -> HexResult<Option<String>> {
    // TODO: 感知哈希比对，返回匹配的装备名
    Ok(None)
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
    let library = TEMPLATE_LIBRARY.get_or_init(|| RwLock::new(TemplateLibrary::default()));
    let guard = library
        .read()
        .map_err(|_| HexError::Vision("图标模板库读锁失败".into()))?;

    let best = guard
        .heroes
        .iter()
        .map(|template| (template, feature_similarity(&feature, &template.feature)))
        .max_by(|lhs, rhs| lhs.1.total_cmp(&rhs.1));

    match best {
        Some((template, score)) if score >= HERO_MATCH_THRESHOLD => Ok(Some(template.name.clone())),
        _ => Ok(None),
    }
}

/// 匹配羁绊标识
pub fn match_trait(_roi_pixels: &[u8]) -> HexResult<Option<String>> {
    // TODO: 羁绊图标匹配
    Ok(None)
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
        if !is_supported_image(&path) {
            continue;
        }
        let Some(name) = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        let image = image::open(&path)
            .map_err(|e| HexError::Vision(format!("加载模板图片失败 {}: {}", path.display(), e)))?;
        templates.push(IconTemplate {
            name,
            feature: image_feature(&image),
        });
    }

    templates.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    Ok(templates)
}

fn is_supported_image(path: &PathBuf) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp")
    )
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
    if pixels.len() % 4 != 0 {
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
}
