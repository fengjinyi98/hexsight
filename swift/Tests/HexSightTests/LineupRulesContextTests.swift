import XCTest
@testable import HexSight

/// LineupRulesContextTests 真实数据规则上下文测试
/// 核心职责：
/// - 校验阵容缓存能生成可读规则快照
/// - 覆盖英雄、装备、羁绊、强化符文名称解析
/// - 防止规则层退回到 UI 或原始 JSON 查字段
final class LineupRulesContextTests: XCTestCase {
    func testMode17RealCacheBuildsReadableRulesContext() throws {
        let card = try cachedLineup(mode: "17", nameContains: "神谕龙王")
        let index = try GameDataIndex.load(mode: "17")
        let context = LineupRulesContext(card: card, mode: "17", gameData: index)

        XCTAssertEqual(context.name, "【神谕龙王】3牧羊人3霸天机甲3神谕")
        XCTAssertTrue(context.resolvedFinalHeroes.contains { $0.name == "超级机甲" })
        XCTAssertTrue(context.resolvedFinalHeroes.contains { $0.name == "羊咩咩 & 咩咩羊" })
        XCTAssertEqual(context.resolvedFinalHeroes.first(where: { $0.id == "14384" })?.equipmentNames, ["石像鬼石板甲", "狂徒铠甲", "斯特拉克的挑战护手"])
        XCTAssertEqual(context.resolvedEquipmentOrder.map(\.name).prefix(5), ["无用大棒", "拳套", "巨人腰带", "反曲之弓", "女神之泪"])
        XCTAssertEqual(context.resolvedRecommendedHexes.map(\.name), ["四费增援", "四费小组", "炽天使之拥"])
        XCTAssertFalse(context.resolvedTraits.isEmpty)
        XCTAssertTrue(context.resolvedTraits.contains { $0.name == "牧羊人" || $0.name == "霸天机甲" || $0.name == "神谕者" })
    }

    func testEverySupportedModeFirstCacheHasReadableCoreContext() throws {
        for profile in ModeProfile.supported {
            let card = try firstCachedLineup(mode: profile.id)
            let index = try GameDataIndex.load(mode: profile.id)
            let context = LineupRulesContext(card: card, mode: profile.id, gameData: index)

            XCTAssertFalse(context.resolvedFinalHeroes.isEmpty, "mode \(profile.id) 缺少可读英雄")
            XCTAssertFalse(context.resolvedEquipmentOrder.isEmpty, "mode \(profile.id) 缺少可读装备顺序")
            XCTAssertFalse(context.resolvedRecommendedHexes.isEmpty, "mode \(profile.id) 缺少可读强化符文")
            XCTAssertFalse(context.resolvedTraits.isEmpty, "mode \(profile.id) 缺少可读羁绊")
        }
    }

    private func firstCachedLineup(mode: String) throws -> LineupCard {
        try XCTUnwrap(cachedLineups(mode: mode).first)
    }

    private func cachedLineup(mode: String, nameContains keyword: String) throws -> LineupCard {
        try XCTUnwrap(cachedLineups(mode: mode).first { $0.name.contains(keyword) })
    }

    private func cachedLineups(mode: String) throws -> [LineupCard] {
        let fileName = try XCTUnwrap(LineupCatalog.cacheFileName(for: mode))
        let url = ProjectPaths.lineupDirectory().appendingPathComponent(fileName)
        let data = try Data(contentsOf: url)
        return LineupSourceAdapter.cards(fromTopLevelData: data, mode: mode)
    }
}
