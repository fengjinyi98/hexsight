import Foundation

/// Rust FFI 桥接层
/// 核心职责：
/// - 封装 Rust extern "C" 函数调用
/// - 管理引擎生命周期（init / process_frame / reset / destroy）
/// - JSON 字符串 ↔ Swift 字典转换
final class RustBridge: @unchecked Sendable {
    static let shared = RustBridge()

    /// Rust 引擎上下文指针
    private var ctx: UnsafeMutableRawPointer?

    private init() {}

    // MARK: - 生命周期

    /// 初始化 Rust 引擎
    /// 传入 ROI 坐标配置和阵容目录路径
    /// 优先从 Bundle 加载文件，开发模式下回退到项目目录
    func initialize() {
        let configJson: String
        let lineupDir: String

        if let configPath = Bundle.main.path(forResource: "regions", ofType: "json"),
           let lineupPath = Bundle.main.path(forResource: "lineups", ofType: nil),
           let data = try? Data(contentsOf: URL(fileURLWithPath: configPath)),
           let json = String(data: data, encoding: .utf8) {
            configJson = json
            lineupDir = lineupPath
        } else {
            // 开发模式：从项目目录加载配置
            let devConfigPath = ProjectPaths.configFile(named: "regions.json")
            let devLineupDir = ProjectPaths.lineupDirectory()

            guard let data = try? Data(contentsOf: devConfigPath),
                  let json = String(data: data, encoding: .utf8) else {
                print("[HexSight] 配置加载失败: \(devConfigPath.path)")
                return
            }
            configJson = json
            lineupDir = devLineupDir.path
            print("[HexSight] 开发模式加载配置: \(devConfigPath.path)")
        }

        ctx = configJson.withCString { configPtr in
            lineupDir.withCString { lineupPtr in
                hexsight_init(configPtr, lineupPtr)
            }
        }

        if ctx != nil {
            print("[HexSight] Rust 引擎初始化成功")
            DispatchQueue.main.async {
                AppState.shared.engineReady = true
            }
        } else {
            print("[HexSight] Rust 引擎初始化失败")
        }
    }

    /// 处理单帧画面
    /// 返回：JSON 格式的 Decision 字符串
    func processFrame(pixels: UnsafePointer<UInt8>, width: UInt32, height: UInt32) -> String? {
        guard let ctx = ctx else { return nil }

        guard let resultPtr = hexsight_process_frame(ctx, pixels, width, height) else {
            return nil
        }

        let result = String(cString: resultPtr)
        hexsight_free_string(resultPtr)
        return result
    }

    /// 重置对局
    func reset() {
        guard let ctx = ctx else { return }
        hexsight_reset(ctx)
    }

    /// 销毁引擎
    func destroy() {
        guard let ctx = ctx else { return }
        hexsight_destroy(ctx)
        self.ctx = nil
    }

    // MARK: - 数据提供 API

    /// 获取支持的模式列表
    func getSupportedModes() -> [[String: Any]] {
        callWithConfigRoot { root in
            guard let ptr = hexsight_get_supported_modes_json(root),
                  let json = String(validatingCString: ptr) else { return [] }
            hexsight_free_string(ptr)
            return (try? JSONSerialization.jsonObject(with: Data(json.utf8)) as? [[String: Any]]) ?? []
        } ?? []
    }

    /// 获取阵容列表
    func getLineups(mode: String) -> [[String: Any]] {
        callWithConfigRoot { root in
            mode.withCString { modePtr in
                guard let ptr = hexsight_get_lineups_json(root, modePtr),
                      let json = String(validatingCString: ptr) else { return [] }
                hexsight_free_string(ptr)
                return (try? JSONSerialization.jsonObject(with: Data(json.utf8)) as? [[String: Any]]) ?? []
            }
        } ?? []
    }

    /// 获取阵容详情
    func getLineupDetail(mode: String, lineupId: String) -> [String: Any]? {
        callWithConfigRoot { root in
            mode.withCString { modePtr in
                lineupId.withCString { idPtr in
                    guard let ptr = hexsight_get_lineup_detail_json(root, modePtr, idPtr),
                          let json = String(validatingCString: ptr) else { return nil }
                    hexsight_free_string(ptr)
                    return (try? JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String: Any])
                }
            }
        } ?? nil
    }

    /// 获取阵容规则/LLM上下文
    func getLineupRulesContext(mode: String, lineupId: String) -> [String: Any]? {
        callWithConfigRoot { root in
            mode.withCString { modePtr in
                lineupId.withCString { idPtr in
                    guard let ptr = hexsight_get_lineup_rules_context_json(root, modePtr, idPtr),
                          let json = String(validatingCString: ptr) else { return nil }
                    hexsight_free_string(ptr)
                    return (try? JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String: Any])
                }
            }
        } ?? nil
    }

    /// 获取 Rust 知识决策 RuleOutput
    func getKnowledgeRuleOutput(mode: String, lineupId: String) -> [String: Any]? {
        callWithConfigRoot { root in
            mode.withCString { modePtr in
                lineupId.withCString { idPtr in
                    guard let ptr = hexsight_get_knowledge_rule_output_json(root, modePtr, idPtr),
                          let json = String(validatingCString: ptr) else { return nil }
                    hexsight_free_string(ptr)
                    return (try? JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String: Any])
                }
            }
        } ?? nil
    }

    /// 刷新远端阵容缓存（Rust 负责 CDN URL 拼装和 HTTP 请求）
    func refreshLineups(mode: String) -> Bool {
        callWithConfigRoot { root in
            mode.withCString { modePtr in
                guard let ptr = hexsight_refresh_lineups_json(root, modePtr),
                      let json = String(validatingCString: ptr) else { return false }
                hexsight_free_string(ptr)
                return json.contains("\"ok\"")
            }
        } ?? false
    }

    /// 校验数据快照
    func validateDataSnapshot(mode: String) -> [String: Any]? {
        callWithConfigRoot { root in
            mode.withCString { modePtr in
                guard let ptr = hexsight_validate_data_snapshot_json(root, modePtr),
                      let json = String(validatingCString: ptr) else { return nil }
                hexsight_free_string(ptr)
                return (try? JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String: Any])
            }
        } ?? nil
    }

    // MARK: - 辅助

    /// 获取 config 目录路径并传入闭包
    private func callWithConfigRoot<T>(_ block: (UnsafePointer<CChar>) -> T) -> T? {
        let configPath = ProjectPaths.configDirectory().path
        return configPath.withCString { rootPtr in
            block(rootPtr)
        }
    }
}

// MARK: - C FFI 函数声明

/// 初始化引擎
/// config_json: JSON 格式 ROI 区域配置
/// lineup_dir: 阵容配置目录路径
/// 返回：引擎上下文指针
@_silgen_name("hexsight_init")
private func hexsight_init(
    _ config_json: UnsafePointer<CChar>,
    _ lineup_dir: UnsafePointer<CChar>
) -> UnsafeMutableRawPointer?

/// 处理单帧画面
/// pixels: BGRA 像素数据指针
/// w, h: 画面宽高
/// 返回：JSON 格式 Decision 字符串，调用方需用 hexsight_free_string 释放
@_silgen_name("hexsight_process_frame")
private func hexsight_process_frame(
    _ ctx: UnsafeMutableRawPointer,
    _ pixels: UnsafePointer<UInt8>,
    _ w: UInt32,
    _ h: UInt32
) -> UnsafeMutablePointer<CChar>?

/// 重置对局
@_silgen_name("hexsight_reset")
private func hexsight_reset(_ ctx: UnsafeMutableRawPointer)

/// 销毁引擎
@_silgen_name("hexsight_destroy")
private func hexsight_destroy(_ ctx: UnsafeMutableRawPointer)

/// 释放 Rust 分配的字符串
@_silgen_name("hexsight_free_string")
private func hexsight_free_string(_ ptr: UnsafeMutablePointer<CChar>)

// MARK: - 数据提供 FFI

/// 获取支持的模式列表（JSON）
@_silgen_name("hexsight_get_supported_modes_json")
private func hexsight_get_supported_modes_json(
    _ config_root: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?

/// 获取阵容列表（JSON）
@_silgen_name("hexsight_get_lineups_json")
private func hexsight_get_lineups_json(
    _ config_root: UnsafePointer<CChar>,
    _ mode: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?

/// 获取阵容详情（JSON）
@_silgen_name("hexsight_get_lineup_detail_json")
private func hexsight_get_lineup_detail_json(
    _ config_root: UnsafePointer<CChar>,
    _ mode: UnsafePointer<CChar>,
    _ lineup_id: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?

/// 获取阵容规则上下文（JSON）
@_silgen_name("hexsight_get_lineup_rules_context_json")
private func hexsight_get_lineup_rules_context_json(
    _ config_root: UnsafePointer<CChar>,
    _ mode: UnsafePointer<CChar>,
    _ lineup_id: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?

/// 获取 Rust 知识决策 RuleOutput（JSON）
@_silgen_name("hexsight_get_knowledge_rule_output_json")
private func hexsight_get_knowledge_rule_output_json(
    _ config_root: UnsafePointer<CChar>,
    _ mode: UnsafePointer<CChar>,
    _ lineup_id: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?

/// 刷新远端阵容缓存
@_silgen_name("hexsight_refresh_lineups_json")
private func hexsight_refresh_lineups_json(
    _ config_root: UnsafePointer<CChar>,
    _ mode: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?

/// 校验数据对齐
@_silgen_name("hexsight_validate_data_snapshot_json")
private func hexsight_validate_data_snapshot_json(
    _ config_root: UnsafePointer<CChar>,
    _ mode: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?
