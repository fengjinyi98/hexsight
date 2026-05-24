import XCTest
@testable import HexSight

/// AppStateOCRTests OCR 状态更新测试
/// 核心职责：
/// - 校验 OCR 摘要能进入主应用状态
/// - 避免 OCR 结果覆盖 Rust 已识别出的有效基础数值
@MainActor
final class AppStateOCRTests: XCTestCase {
    func testAppliesOCRSummaryToRuntimeState() {
        let state = AppState()
        let summary = OCRFrameSummary(
            round: "1-3",
            shop: [OCRShopSlot(index: 0, heroName: "璐璐", traits: ["虚空"])],
            opponents: [OCROpponentRow(index: 0, name: "玩家A", hp: 76)],
            activeTraits: [OCRActiveTraitRow(index: 0, name: "虚空", count: "1")]
        )

        state.applyOCRSummary(summary)

        XCTAssertEqual(state.round, "1-3")
        XCTAssertEqual(state.ocrShop, summary.shop)
        XCTAssertEqual(state.ocrOpponents, summary.opponents)
        XCTAssertEqual(state.ocrActiveTraits, summary.activeTraits)
    }

    func testBlankOCRRoundDoesNotClearExistingRound() {
        let state = AppState()
        state.round = "2-1"

        state.applyOCRSummary(OCRFrameSummary(round: nil, shop: [], opponents: [], activeTraits: []))

        XCTAssertEqual(state.round, "2-1")
    }
}
