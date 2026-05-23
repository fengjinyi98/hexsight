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
