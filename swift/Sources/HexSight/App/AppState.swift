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
    @Published var knowledgeDecision: KnowledgeDecisionSummary? = nil

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

        if let summary = KnowledgeDecisionSummary(ruleOutput: json) {
            applyKnowledgeDecision(summary)
        }
    }

    /// 加载 Rust 知识决策并更新展示状态
    func loadKnowledgeDecision(mode: String, lineupId: String) async {
        guard let output = LineupRepository.shared.loadKnowledgeRuleOutput(mode: mode, lineupId: lineupId),
              let summary = KnowledgeDecisionSummary(ruleOutput: output)
        else { return }

        applyKnowledgeDecision(summary)
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
        knowledgeDecision = nil
        RustBridge.shared.reset()
    }

    /// 应用 Rust 知识决策展示摘要
    private func applyKnowledgeDecision(_ summary: KnowledgeDecisionSummary) {
        knowledgeDecision = summary
        lineupName = summary.lineupName
        suggestions = summary.suggestions
        riskLevel = summary.riskLevel
        equipRoute = summary.equipmentActions
        transition = summary.transitionActions
        llmAdvice = summary.advice
    }
}
