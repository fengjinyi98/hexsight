import XCTest
@testable import HexSight

/// OCRFrameSummarizerTests OCR 摘要整理测试
/// 核心职责：
/// - 校验 ROI 结果能整理为商店、对手、羁绊业务字段
/// - 校验空文本不会生成无效业务条目
final class OCRFrameSummarizerTests: XCTestCase {
    func testBuildsBusinessSummaryFromRegionResults() {
        let regions = [
            result(id: "round", kind: .round, text: "1-3"),
            result(id: "shop_hero_name_0", kind: .shopHeroName, text: "璐璐"),
            result(id: "shop_trait_name_0", kind: .shopTraitName, text: "法师"),
            result(id: "shop_trait_name_1", kind: .shopTraitName, text: "狙神"),
            result(id: "opponent_name_0", kind: .opponentName, text: "屋屋z"),
            result(id: "opponent_hp_0", kind: .opponentHP, text: "76"),
            result(id: "active_trait_name_0", kind: .activeTraitName, text: "虚空"),
            result(id: "active_trait_count_0", kind: .activeTraitCount, text: "1/2"),
        ]

        let summary = OCRFrameSummarizer().summarize(regions)

        XCTAssertEqual(summary.round, "1-3")
        XCTAssertEqual(summary.shop, [OCRShopSlot(index: 0, heroName: "璐璐", traits: ["法师"])])
        XCTAssertEqual(summary.opponents, [OCROpponentRow(index: 0, name: "屋屋z", hp: 76)])
        XCTAssertEqual(summary.activeTraits, [OCRActiveTraitRow(index: 0, name: "虚空", count: "1/2")])
    }

    func testSkipsBlankSummaryRows() {
        let regions = [
            result(id: "shop_hero_name_0", kind: .shopHeroName, text: nil),
            result(id: "opponent_hp_0", kind: .opponentHP, text: nil),
            result(id: "active_trait_count_0", kind: .activeTraitCount, text: ""),
        ]

        let summary = OCRFrameSummarizer().summarize(regions)

        XCTAssertNil(summary.round)
        XCTAssertTrue(summary.shop.isEmpty)
        XCTAssertTrue(summary.opponents.isEmpty)
        XCTAssertTrue(summary.activeTraits.isEmpty)
    }

    func testRepairsActiveTraitRowsWhenNameAndCountAreOffset() {
        let regions = [
            result(id: "active_trait_name_0", kind: .activeTraitName, text: "时光守护者"),
            result(id: "active_trait_name_1", kind: .activeTraitName, text: "3/5/7"),
            result(id: "active_trait_name_2", kind: .activeTraitName, text: "2/4/6"),
            result(id: "active_trait_count_4", kind: .activeTraitCount, text: "虚空"),
            result(id: "active_trait_count_5", kind: .activeTraitCount, text: "神盾使"),
            result(id: "active_trait_count_6", kind: .activeTraitCount, text: "迅击战士"),
        ]

        let summary = OCRFrameSummarizer().summarize(regions)

        XCTAssertEqual(summary.activeTraits, [
            OCRActiveTraitRow(index: 0, name: "时光守护者", count: "3/5/7"),
            OCRActiveTraitRow(index: 2, name: nil, count: "2/4/6"),
            OCRActiveTraitRow(index: 4, name: "虚空", count: nil),
            OCRActiveTraitRow(index: 5, name: "神盾使", count: nil),
            OCRActiveTraitRow(index: 6, name: "迅击战士", count: nil),
        ])
    }

    private func result(id: String, kind: OCRRegionKind, text: String?) -> OCRRegionResult {
        OCRRegionResult(
            region: OCRRegion(id: id, kind: kind, rect: .zero),
            candidates: [],
            normalizedText: text
        )
    }
}
