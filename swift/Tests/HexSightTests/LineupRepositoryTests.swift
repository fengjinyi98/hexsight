import XCTest
@testable import HexSight

/// LineupRepositoryTests Rust FFI 数据链路回归测试
/// 核心职责：
/// - 验证 Rust FFI 返回的阵容数据能正确解码为 Swift 展示模型
/// - 验证三种模式的英雄、装备、羁绊、模式特有字段不缺
@MainActor
final class LineupRepositoryTests: XCTestCase {

    // MARK: - 模式 17（星神）

    func testMode17LineupsLoadViaRustFFI() async throws {
        let repo = LineupRepository.shared
        await repo.loadLineups(mode: "17")
        XCTAssertFalse(repo.lineups.isEmpty, "mode17 阵容为空")

        let first = repo.lineups[0]
        XCTAssertFalse(first.name.isEmpty)
        XCTAssertFalse(first.id.isEmpty)
        XCTAssertFalse(first.detail.finalHeroes.isEmpty, "无最终英雄")
        XCTAssertFalse(first.heroPreview.isEmpty, "heroPreview 为空")
        XCTAssertFalse(first.augmentIDs.isEmpty, "augmentIDs 为空")
    }

    func testMode17GodRewardsParseViaRustFFI() async throws {
        let repo = LineupRepository.shared
        let ctx = repo.loadRulesContext(mode: "17", lineupId: "")
        let modeSpecific = try XCTUnwrap(ctx?["modeSpecific"] as? [String: Any])
        XCTAssertNotNil(modeSpecific["godRewards"])
    }

    // MARK: - 模式 16（英雄联盟传奇）

    func testMode16LineupsLoadViaRustFFI() async throws {
        let repo = LineupRepository.shared
        await repo.loadLineups(mode: "16")
        XCTAssertFalse(repo.lineups.isEmpty, "mode16 阵容为空")

        let first = repo.lineups[0]
        XCTAssertFalse(first.detail.finalHeroes.isEmpty, "无最终英雄")
        XCTAssertFalse(first.detail.equipmentOrderIDs.isEmpty, "equipmentOrderIDs 为空")
    }

    // MARK: - 模式 4（天选福星）

    func testMode4LineupsLoadViaRustFFI() async throws {
        let repo = LineupRepository.shared
        await repo.loadLineups(mode: "4")
        XCTAssertFalse(repo.lineups.isEmpty, "mode4 阵容为空")

        let first = repo.lineups[0]
        XCTAssertFalse(first.detail.finalHeroes.isEmpty, "无最终英雄")
    }

    // MARK: - 通用

    func testAllSupportedModesHaveLineups() async throws {
        for mode in ["17", "16", "4"] {
            let repo = LineupRepository.shared
            await repo.loadLineups(mode: mode)
            XCTAssertFalse(repo.lineups.isEmpty, "mode\(mode) 阵容为空")
        }
    }

    func testSupportedModesListFromRust() async {
        let repo = LineupRepository.shared
        let modes = repo.supportedModes()
        XCTAssertEqual(modes.count, 3)
        let names = modes.compactMap { $0["name"] as? String }
        XCTAssertTrue(names.contains("星神"))
        XCTAssertTrue(names.contains("英雄联盟传奇"))
        XCTAssertTrue(names.contains("天选福星"))
    }

    func testDataValidationReport() async {
        let repo = LineupRepository.shared
        let report = repo.validateSnapshot(mode: "17")
        XCTAssertNotNil(report)
        XCTAssertNotNil(report?["total_lineups"])
        XCTAssertNotNil(report?["missing_hero_ids"])
    }
}
