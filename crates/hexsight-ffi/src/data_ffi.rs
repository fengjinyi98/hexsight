// 数据提供 FFI 接口
// 核心职责：
// - 暴露 C ABI 兼容的数据 API 给 Swift 调用
// - 六个无状态函数，每次调用独立加载数据
// - 返回 JSON 字符串，调用方通过 hexsight_free_string 释放

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

use hexsight_engine::{GameDataIndex, LineupAdapter, LineupLoader, RemoteLineupSource, RulesContextBuilder};

// ============================================================
// FFI 辅助函数
// ============================================================

/// &str → *mut c_char
fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("{}").unwrap())
        .into_raw()
}

/// *const c_char → String
unsafe fn from_c_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() { return None; }
    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_owned())
}

/// 错误 JSON 响应
fn error_json(msg: &str) -> *mut c_char {
    let json = serde_json::json!({ "error": msg });
    to_c_string(&json.to_string())
}


// ============================================================
// FFI 函数
// ============================================================

/// 获取支持的模式、赛季、玩法能力列表（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_supported_modes_json(
    _config_root: *const c_char,
) -> *mut c_char {
    let modes = RulesContextBuilder::supported_modes();
    match serde_json::to_string(&modes) {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&format!("序列化失败: {}", e)),
    }
}

/// 获取某模式阵容列表和详情摘要（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_lineups_json(
    config_root: *const c_char,
    mode: *const c_char,
) -> *mut c_char {
    let (config_root, mode) = unsafe {
        match (from_c_str(config_root), from_c_str(mode)) {
            (Some(c), Some(m)) => (c, m),
            _ => return error_json("参数为空"),
        }
    };

    let raw_items = match LineupLoader::load_cached_lineups(Path::new(&config_root), &mode, "S18") {
        Ok(items) => items,
        Err(e) => return error_json(&format!("加载阵容失败: {}", e)),
    };

    let cards = LineupAdapter::cards_from_raw_list(&raw_items, &mode);
    match serde_json::to_string(&cards) {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&format!("序列化失败: {}", e)),
    }
}

/// 获取阵容详情展示数据（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_lineup_detail_json(
    config_root: *const c_char,
    mode: *const c_char,
    lineup_id: *const c_char,
) -> *mut c_char {
    let (config_root, mode, lineup_id) = unsafe {
        match (from_c_str(config_root), from_c_str(mode), from_c_str(lineup_id)) {
            (Some(c), Some(m), Some(l)) => (c, m, l),
            _ => return error_json("参数为空"),
        }
    };

    let raw_items = match LineupLoader::load_cached_lineups(Path::new(&config_root), &mode, "S18") {
        Ok(items) => items,
        Err(e) => return error_json(&format!("加载阵容失败: {}", e)),
    };

    let cards = LineupAdapter::cards_from_raw_list(&raw_items, &mode);
    let card = match cards.iter().find(|c| c.id == lineup_id || c.name.contains(&lineup_id)) {
        Some(c) => c,
        None => return error_json("阵容未找到"),
    };

    match serde_json::to_string(&card.detail) {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&format!("序列化失败: {}", e)),
    }
}

/// 获取阵容规则/LLM上下文（JSON）
#[no_mangle]
pub extern "C" fn hexsight_get_lineup_rules_context_json(
    config_root: *const c_char,
    mode: *const c_char,
    lineup_id: *const c_char,
) -> *mut c_char {
    let (config_root, mode, lineup_id) = unsafe {
        match (from_c_str(config_root), from_c_str(mode), from_c_str(lineup_id)) {
            (Some(c), Some(m), Some(l)) => (c, m, l),
            _ => return error_json("参数为空"),
        }
    };

    let index = match GameDataIndex::load(Path::new(&config_root), &mode) {
        Ok(idx) => idx,
        Err(e) => return error_json(&format!("加载游戏数据失败: {}", e)),
    };

    let raw_items = match LineupLoader::load_cached_lineups(Path::new(&config_root), &mode, "S18") {
        Ok(items) => items,
        Err(e) => return error_json(&format!("加载阵容失败: {}", e)),
    };

    let cards = LineupAdapter::cards_from_raw_list(&raw_items, &mode);
    let card = match cards.iter().find(|c| c.id == lineup_id || c.name.contains(&lineup_id)) {
        Some(c) => c,
        None => return error_json("阵容未找到"),
    };

    let context = RulesContextBuilder::build(card, &mode, Some(&index));
    match serde_json::to_string(&context) {
        Ok(json) => to_c_string(&json),
        Err(e) => error_json(&format!("序列化失败: {}", e)),
    }
}

/// 刷新远端阵容缓存
/// Rust 负责 CDN URL 拼装、HTTP 请求、校验和缓存写入
#[no_mangle]
pub extern "C" fn hexsight_refresh_lineups_json(
    config_root: *const c_char,
    mode: *const c_char,
) -> *mut c_char {
    let (config_root, mode) = unsafe {
        match (from_c_str(config_root), from_c_str(mode)) {
            (Some(c), Some(m)) => (c, m),
            _ => return error_json("参数为空"),
        }
    };

    match RemoteLineupSource::refresh(Path::new(&config_root), &mode) {
        Ok(count) => {
            let json = serde_json::json!({ "status": "ok", "mode": mode, "count": count });
            to_c_string(&json.to_string())
        }
        Err(e) => error_json(&format!("刷新失败: {}", e)),
    }
}

/// 校验静态数据和阵容数据是否对齐
#[no_mangle]
pub extern "C" fn hexsight_validate_data_snapshot_json(
    config_root: *const c_char,
    mode: *const c_char,
) -> *mut c_char {
    let (config_root, mode) = unsafe {
        match (from_c_str(config_root), from_c_str(mode)) {
            (Some(c), Some(m)) => (c, m),
            _ => return error_json("参数为空"),
        }
    };

    let index = match GameDataIndex::load(Path::new(&config_root), &mode) {
        Ok(idx) => idx,
        Err(e) => return error_json(&format!("加载游戏数据失败: {}", e)),
    };

    let raw_items = match LineupLoader::load_cached_lineups(Path::new(&config_root), &mode, "S18") {
        Ok(items) => items,
        Err(e) => return error_json(&format!("加载阵容失败: {}", e)),
    };

    let cards = LineupAdapter::cards_from_raw_list(&raw_items, &mode);
    let mut missing_hero_ids = Vec::new();
    let mut missing_equip_ids = Vec::new();
    let mut total_hero_ids = 0usize;
    let mut total_equip_ids = 0usize;
    let parseable_lineups = cards.len();

    // 收集所有 hero_id 和 equip_id
    for card in &cards {
        for piece in &card.detail.final_heroes {
            total_hero_ids += 1;
            if index.hero(&piece.hero_id).is_none() {
                missing_hero_ids.push(piece.hero_id.clone());
            }
            for eid in &piece.equipment_ids {
                total_equip_ids += 1;
                if index.equipment(eid).is_none() && eid != "0" {
                    missing_equip_ids.push(eid.clone());
                }
            }
        }
    }

    let report = serde_json::json!({
        "mode": mode,
        "total_lineups": cards.len(),
        "total_hero_ids": total_hero_ids,
        "total_equip_ids": total_equip_ids,
        "missing_hero_ids": missing_hero_ids,
        "missing_equip_ids": missing_equip_ids,
        "parseable_lineups": parseable_lineups,
        "unparseable_lineups": [],
    });

    to_c_string(&report.to_string())
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::ffi::CString;

    // 引用同 crate 中 lib.rs 定义的 hexsight_free_string
    extern "C" {
        fn hexsight_free_string(ptr: *mut c_char);
    }

    fn config_root_cstr() -> CString {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .join("config");
        CString::new(path.to_str().unwrap()).unwrap()
    }

    fn mode17_cstr() -> CString {
        CString::new("17").unwrap()
    }

    #[test]
    fn test_get_supported_modes() {
        let root = config_root_cstr();
        let ptr = hexsight_get_supported_modes_json(root.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("星神"));
        assert!(json.contains("英雄联盟传奇"));
        assert!(json.contains("天选福星"));
        unsafe { hexsight_free_string(ptr); }
    }

    #[test]
    fn test_get_lineups_json() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let ptr = hexsight_get_lineups_json(root.as_ptr(), mode.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("final_heroes"));
        unsafe { hexsight_free_string(ptr); }
    }

    #[test]
    fn test_get_lineup_rules_context() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let id = CString::new("神谕龙王").unwrap();
        let ptr = hexsight_get_lineup_rules_context_json(root.as_ptr(), mode.as_ptr(), id.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("finalHeroes"));
        assert!(json.contains("\"traits\""));
        unsafe { hexsight_free_string(ptr); }
    }

    #[test]
    fn test_validate_snapshot() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let ptr = hexsight_validate_data_snapshot_json(root.as_ptr(), mode.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("total_lineups"));
        unsafe { hexsight_free_string(ptr); }
    }

    #[test]
    fn test_lineup_not_found() {
        let root = config_root_cstr();
        let mode = mode17_cstr();
        let id = CString::new("nonexistent_lineup_12345").unwrap();
        let ptr = hexsight_get_lineup_detail_json(root.as_ptr(), mode.as_ptr(), id.as_ptr());
        assert!(!ptr.is_null());
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        assert!(json.contains("error"));
        unsafe { hexsight_free_string(ptr); }
    }
}
