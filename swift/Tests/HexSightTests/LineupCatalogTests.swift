import XCTest
@testable import HexSight

/// LineupCatalogTests 阵容数据源策略测试
/// 核心职责：
/// - 校验各模式的缓存文件策略
/// - 校验仅主模式支持远端阵容拉取
final class LineupCatalogTests: XCTestCase {
    func testSupportedModesUseSeasonCacheFile() {
        XCTAssertEqual(LineupCatalog.cacheFileName(for: "17"), "mode17_S18.json")
        XCTAssertEqual(LineupCatalog.cacheFileName(for: "16"), "mode16_S18.json")
        XCTAssertEqual(LineupCatalog.cacheFileName(for: "4"), "mode4_S18.json")
        XCTAssertTrue(LineupCatalog.supportsRemoteFetch(for: "17"))
        XCTAssertTrue(LineupCatalog.supportsRemoteFetch(for: "16"))
        XCTAssertTrue(LineupCatalog.supportsRemoteFetch(for: "4"))
    }

    func testUnsupportedModeHasNoCatalog() {
        XCTAssertNil(LineupCatalog.cacheFileName(for: "999"))
        XCTAssertFalse(LineupCatalog.supportsRemoteFetch(for: "999"))
    }

    func testSupportedModesHaveRemoteURLs() {
        XCTAssertEqual(
            LineupCatalog.remoteURL(for: "17")?.absoluteString,
            "https://game.gtimg.cn/images/lol/act/jkzlkauto/json/lineupJson/m18/11/17/lineup_detail_total.json"
        )
        XCTAssertEqual(
            LineupCatalog.remoteURL(for: "16")?.absoluteString,
            "https://game.gtimg.cn/images/lol/act/jkzlkauto/json/lineupJson/m17/11/16/lineup_detail_total.json"
        )
        XCTAssertEqual(
            LineupCatalog.remoteURL(for: "4")?.absoluteString,
            "https://game.gtimg.cn/images/lol/act/jkzlkauto/json/lineupJson/m17/11/4/lineup_detail_total.json"
        )
    }
}
