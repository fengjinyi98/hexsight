import Foundation
import Combine

/// 全局应用状态
/// 核心职责：
/// - 持有对局决策数据
/// - 通过 @Published 驱动 SwiftUI 悬浮窗刷新
@MainActor
final class AppState: ObservableObject {
    static let shared = AppState()

    // MARK: - 对局基础数据

    @Published var gold: Int = 0
    @Published var hp: Int = 100
    @Published var level: Int = 1
    @Published var exp: Int = 0
    @Published var round: String = ""

    // MARK: - 决策数据

    @Published var lineupName: String = "等待识别..."
    @Published var suggestions: [String] = []
    @Published var rivalCount: Int = 0
    @Published var riskLevel: String = "低"
    @Published var equipRoute: [String] = []
    @Published var transition: [String] = []
    @Published var llmAdvice: String? = nil

    // MARK: - 运行状态

    @Published var engineReady: Bool = false
    @Published var captureActive: Bool = false
    @Published var permissionGranted: Bool = false

    /// 从 Rust FFI 返回的 JSON 更新状态
    func updateFromJSON(_ jsonStr: String) {
        guard let data = jsonStr.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return }

        lineupName = json["lineup_name"] as? String ?? lineupName
        suggestions = json["suggestions"] as? [String] ?? suggestions
        rivalCount = json["rival_count"] as? Int ?? rivalCount
        riskLevel = json["risk_level"] as? String ?? riskLevel
        equipRoute = json["equip_route"] as? [String] ?? equipRoute
        transition = json["transition"] as? [String] ?? transition
        llmAdvice = json["llm_advice"] as? String
    }

    /// 重置对局状态
    func resetGame() {
        gold = 0
        hp = 100
        level = 1
        exp = 0
        round = ""
        lineupName = "等待识别..."
        suggestions = []
        rivalCount = 0
        riskLevel = "低"
        equipRoute = []
        transition = []
        llmAdvice = nil
        RustBridge.shared.reset()
    }
}
