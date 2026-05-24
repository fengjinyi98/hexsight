import XCTest
@testable import HexSight

/// AppStateTests 全局状态回归测试
/// 核心职责：
/// - 验证旧画面识别 JSON 与 Rust RuleOutput 的状态更新边界
/// - 防止普通识别结果被误解析为知识决策摘要
@MainActor
final class AppStateTests: XCTestCase {
    func testLegacyFrameJSONDoesNotCreateKnowledgeDecisionSummary() {
        let state = AppState.shared
        state.resetGame()

        state.updateFromJSON("""
        {
          "lineup_name": "实况阵容",
          "suggestions": ["升8", "保利息"],
          "risk_level": "低",
          "equip_route": ["无尽", "轻语"],
          "transition": ["卢锡安"],
          "llm_advice": "继续稳血"
        }
        """)

        XCTAssertNil(state.knowledgeDecision)
        XCTAssertEqual(state.lineupName, "实况阵容")
        XCTAssertEqual(state.suggestions, ["升8", "保利息"])
    }

    func testKnowledgeDecisionLoadSkipsWhenLineupIdIsEmpty() async {
        let state = AppState.shared
        state.resetGame()

        await state.loadKnowledgeDecision(mode: "17", lineupId: "")

        XCTAssertNil(state.knowledgeDecision)
        XCTAssertEqual(state.lineupName, "等待识别...")
    }

    func testKnowledgeDecisionLoadUsesExplicitLineupId() async {
        let state = AppState.shared
        state.resetGame()

        await state.loadKnowledgeDecision(mode: "17", lineupId: "神谕龙王")

        XCTAssertNotNil(state.knowledgeDecision)
        XCTAssertFalse(state.lineupName.isEmpty)
        XCTAssertFalse(state.suggestions.isEmpty)
    }
}
