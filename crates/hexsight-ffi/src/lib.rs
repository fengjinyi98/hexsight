// hexsight-ffi C FFI 桥接层
// 核心职责：
// - 导出 C ABI 兼容函数给 Swift 调用
// - 管理引擎生命周期（init / process / reset / destroy）
// - 数据序列化：Swift → 像素指针 → Rust → JSON 字符串 → Swift
// - 数据提供：阵容列表、详情、规则上下文 JSON API
//
// 导出函数：
//   游戏识别管线：
//     hexsight_init(config_json) → *mut EngineContext
//     hexsight_process_frame(ctx, pixels, w, h) → *mut c_char (JSON Decision)
//     hexsight_reset(ctx)
//     hexsight_destroy(ctx)
//     hexsight_free_string(ptr)
//   数据提供：
//     hexsight_get_supported_modes_json(config_root)
//     hexsight_get_lineups_json(config_root, mode)
//     hexsight_get_lineup_detail_json(config_root, mode, lineup_id)
//     hexsight_get_lineup_rules_context_json(config_root, mode, lineup_id)
//     hexsight_refresh_lineups_json(config_root, mode, raw_json)
//     hexsight_validate_data_snapshot_json(config_root, mode)

#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod data_ffi;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

use hexsight_core::RegionConfig;
use hexsight_engine::DecisionEngine;
use hexsight_memory::GameMemory;

/// 引擎上下文 —— 持有所有子模块实例
pub struct EngineContext {
    memory: GameMemory,
    engine: DecisionEngine,
    regions: RegionConfig,
}

/// 初始化引擎
/// config_json: JSON 格式配置（包含 region 坐标、阵容目录等）
/// 返回：引擎上下文指针，失败时返回 null
#[no_mangle]
pub extern "C" fn hexsight_init(
    config_json: *const c_char,
    lineup_dir: *const c_char,
) -> *mut EngineContext {
    if config_json.is_null() {
        return std::ptr::null_mut();
    }

    let _config_str = unsafe {
        match CStr::from_ptr(config_json).to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let lineup_dir_str = unsafe {
        match CStr::from_ptr(lineup_dir).to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    // 解析 region 配置
    let regions: RegionConfig = match serde_json::from_str(&_config_str) {
        Ok(r) => r,
        Err(_) => return std::ptr::null_mut(),
    };

    let mut engine = DecisionEngine::new();
    // 尝试加载阵容库，失败不阻塞初始化
    let _ = engine.init(&lineup_dir_str);
    if let Some(template_dir) = vision_template_dir_from_lineups(&lineup_dir_str) {
        let _ = hexsight_vision::icon::init_templates(&template_dir.to_string_lossy());
    }

    let ctx = Box::new(EngineContext {
        memory: GameMemory::new(),
        engine,
        regions,
    });

    Box::into_raw(ctx)
}

/// 处理单帧画面
/// ctx: 引擎上下文
/// pixels: BGRA 格式像素数据指针
/// w, h: 画面宽高
/// 返回：JSON 格式的 Decision 字符串，调用方需通过 hexsight_free_string 释放
#[no_mangle]
pub extern "C" fn hexsight_process_frame(
    ctx: *mut EngineContext,
    pixels: *const u8,
    w: u32,
    h: u32,
) -> *mut c_char {
    if ctx.is_null() || pixels.is_null() {
        return std::ptr::null_mut();
    }

    let ctx = unsafe { &mut *ctx };

    // 1. 画面识别
    let pixel_slice = unsafe { std::slice::from_raw_parts(pixels, (w * h * 4) as usize) };
    let frame = match hexsight_vision::recognize(pixel_slice, &ctx.regions, w, h) {
        Ok(f) => f,
        Err(_) => return std::ptr::null_mut(),
    };

    // 2. 更新记忆
    let state = match ctx.memory.feed(frame) {
        Ok(Some(s)) => s,
        Ok(None) => {
            // 降噪门控未触发，返回空
            return to_c_string("");
        }
        Err(_) => return std::ptr::null_mut(),
    };

    // 3. 决策
    let decision = ctx.engine.decide(state);

    // 4. 序列化为 JSON
    to_c_string(&serde_json::to_string(&decision).unwrap_or_default())
}

/// 重置对局记忆
#[no_mangle]
pub extern "C" fn hexsight_reset(ctx: *mut EngineContext) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };
    ctx.memory.reset();
}

/// 销毁引擎，释放所有资源
#[no_mangle]
pub extern "C" fn hexsight_destroy(ctx: *mut EngineContext) {
    if ctx.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ctx);
    }
}

/// 释放由 Rust 分配的字符串
/// Swift 调用方获取 JSON 后必须调用此函数释放内存
#[no_mangle]
pub extern "C" fn hexsight_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

/// 获取引擎版本号
#[no_mangle]
pub extern "C" fn hexsight_version() -> *mut c_char {
    to_c_string(env!("CARGO_PKG_VERSION"))
}

// 辅助函数：&str → C 字符串
fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

fn vision_template_dir_from_lineups(lineup_dir: &str) -> Option<PathBuf> {
    let config_root = Path::new(lineup_dir).parent()?;
    let template_dir = config_root.join("vision_templates");
    template_dir.is_dir().then_some(template_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_vision_template_dir_from_lineup_dir() {
        let root =
            std::env::temp_dir().join(format!("hexsight_ffi_templates_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("config/lineups")).unwrap();
        std::fs::create_dir_all(root.join("config/vision_templates/heroes")).unwrap();

        let found = vision_template_dir_from_lineups(root.join("config/lineups").to_str().unwrap());

        assert_eq!(found, Some(root.join("config/vision_templates")));
        let _ = std::fs::remove_dir_all(&root);
    }
}
