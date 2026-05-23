import XCTest
@testable import HexSight

/// GameDataServiceTests 阵容棋子映射测试
/// 核心职责：
/// - 校验阵容 hero_id 可回查当前模式英雄名称
/// - 校验阵容 hero_id 可回查当前模式头像链接
@MainActor
final class GameDataServiceTests: XCTestCase {
    func testHeroLookupForMode17LineupPiece() {
        let service = GameDataService.shared

        let hero = service.hero(for: "14384", mode: "17")

        XCTAssertEqual(hero?.name, "超级机甲")
        XCTAssertEqual(service.heroName(for: "14384", mode: "17"), "超级机甲")
        XCTAssertTrue(service.heroPicture(for: "14384", mode: "17").contains("s17_head_galio"))
    }

    func testHeroLookupForMode16LineupPiece() {
        let service = GameDataService.shared

        let hero = service.hero(for: "13321", mode: "16")

        XCTAssertEqual(hero?.name, "薇恩")
        XCTAssertTrue(service.heroPicture(for: "13321", mode: "16").contains("s16_head_vayne"))
    }
}

extension GameDataServiceTests {
    func testMode16UnlockMissionLookup() {
        let service = GameDataService.shared

        let mission = service.mission(for: "1242001", mode: "16")

        XCTAssertEqual(mission?.heroID, "12420")
        XCTAssertTrue(mission?.desc.contains("约德尔人") == true)
    }

    func testMode17GodWishLookup() {
        let service = GameDataService.shared

        let wish = service.godWish(for: "1704022", mode: "17")

        XCTAssertEqual(wish?.id, "1704022")
        XCTAssertFalse(wish?.name.isEmpty ?? true)
        XCTAssertFalse(wish?.icon.isEmpty ?? true)
    }

    func testLineupTraitSummariesCanBeComputedWhenOfficialContactsAreMissing() {
        let service = GameDataService.shared
        let pieces = [
            LineupPiece(dict: ["hero_id": "13330", "location": "1,2"])!,
            LineupPiece(dict: ["hero_id": "14350", "location": "1,3"])!,
            LineupPiece(dict: ["hero_id": "13331", "location": "1,4"])!,
            LineupPiece(dict: ["hero_id": "12420", "location": "1,5"])!,
            LineupPiece(dict: ["hero_id": "14336", "location": "1,6"])!,
            LineupPiece(dict: ["hero_id": "11416", "location": "1,7"])!,
            LineupPiece(dict: ["hero_id": "15271", "location": "4,1"])!,
            LineupPiece(dict: ["hero_id": "12407", "location": "4,4"])!,
            LineupPiece(dict: ["hero_id": "11405", "location": "4,7"])!,
        ]

        let summaries = service.traitSummaries(for: pieces, mode: "16")

        XCTAssertTrue(summaries.contains { $0.name == "约德尔人" && $0.count == 8 })
        XCTAssertTrue(summaries.contains { $0.name == "护卫" && $0.count == 2 })
    }
}
