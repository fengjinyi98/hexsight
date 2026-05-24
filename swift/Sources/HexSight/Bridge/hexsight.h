// HexSight Rust FFI 头文件
// 由 cbindgen 自动生成，或手动维护
// Swift 通过 module map 引用

#ifndef hexsight_h
#define hexsight_h

#include <stdint.h>

/// 初始化引擎
/// config_json: JSON 格式的 ROI 区域配置
/// lineup_dir: 阵容配置 JSON 目录路径
/// 返回：引擎上下文指针，失败返回 NULL
void* hexsight_init(const char* config_json, const char* lineup_dir);

/// 处理单帧画面
/// ctx: 引擎上下文
/// pixels: BGRA 格式像素数据 (w * h * 4 字节)
/// w: 画面宽度
/// h: 画面高度
/// 返回：JSON 格式 Decision 字符串，调用方需用 hexsight_free_string 释放
/// 无状态更新时返回空字符串 ""
char* hexsight_process_frame(void* ctx, const uint8_t* pixels, uint32_t w, uint32_t h);

/// 重置对局记忆
void hexsight_reset(void* ctx);

/// 销毁引擎，释放所有资源
void hexsight_destroy(void* ctx);

/// 释放 Rust 分配的字符串
void hexsight_free_string(char* ptr);

/// 获取引擎版本号
char* hexsight_version(void);

// ---- 数据提供 API ----

/// 获取支持的模式列表（JSON 数组）
/// config_root: config/ 目录路径
/// 返回：JSON 格式模式列表，调用方需用 hexsight_free_string 释放
char* hexsight_get_supported_modes_json(const char* config_root);

/// 获取某模式阵容列表和详情摘要（JSON 数组）
/// config_root: config/ 目录路径
/// mode: 模式 ID（"17"/"16"/"4"）
/// 返回：JSON 格式阵容列表，调用方需用 hexsight_free_string 释放
char* hexsight_get_lineups_json(const char* config_root, const char* mode);

/// 获取阵容详情展示数据（JSON 对象）
/// config_root: config/ 目录路径
/// mode: 模式 ID
/// lineup_id: 阵容 ID 或名称关键词
/// 返回：JSON 格式详情，调用方需用 hexsight_free_string 释放
char* hexsight_get_lineup_detail_json(const char* config_root, const char* mode, const char* lineup_id);

/// 获取阵容规则/LLM 上下文（JSON 对象）
/// config_root: config/ 目录路径
/// mode: 模式 ID
/// lineup_id: 阵容 ID 或名称关键词
/// 返回：JSON 格式规则上下文，调用方需用 hexsight_free_string 释放
char* hexsight_get_lineup_rules_context_json(const char* config_root, const char* mode, const char* lineup_id);

/// 获取 Rust 知识决策 RuleOutput（JSON 对象）
/// config_root: config/ 目录路径
/// mode: 模式 ID
/// lineup_id: 阵容 ID 或名称关键词
/// 返回：JSON 格式 RuleOutput，调用方需用 hexsight_free_string 释放
char* hexsight_get_knowledge_rule_output_json(const char* config_root, const char* mode, const char* lineup_id);

/// 获取带当前局面上下文的 Rust 知识决策 RuleOutput（JSON 对象）
/// config_root: config/ 目录路径
/// mode: 模式 ID
/// context_json: 当前局面上下文 JSON
/// 返回：JSON 格式 RuleOutput，调用方需用 hexsight_free_string 释放
char* hexsight_get_knowledge_rule_output_with_context_json(const char* config_root, const char* mode, const char* context_json);

/// 刷新远端阵容缓存（Rust 负责 CDN URL 拼装和 HTTP 请求）
/// config_root: config/ 目录路径
/// mode: 模式 ID
/// 返回：JSON 状态，调用方需用 hexsight_free_string 释放
char* hexsight_refresh_lineups_json(const char* config_root, const char* mode);

/// 校验静态数据和阵容数据是否对齐
/// config_root: config/ 目录路径
/// mode: 模式 ID
/// 返回：JSON 格式校验报告，调用方需用 hexsight_free_string 释放
char* hexsight_validate_data_snapshot_json(const char* config_root, const char* mode);

#endif /* hexsight_h */
