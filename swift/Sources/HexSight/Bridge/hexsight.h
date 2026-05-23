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

#endif /* hexsight_h */
