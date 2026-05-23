import XCTest
@testable import HexSight

/// ProjectPathsTests 路径回归测试
/// 核心职责：
/// - 校验开发模式资源路径落在仓库根目录
/// - 防止再次回退到错误的相对路径推导
final class ProjectPathsTests: XCTestCase {
    func testGameDataDirectoryPointsToExistingModeFolder() {
        let packageRoot = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()  // HexSightTests/
            .deletingLastPathComponent()  // Tests/
            .deletingLastPathComponent()  // swift/
        let sourceFilePath = packageRoot
            .appendingPathComponent("Sources/HexSight/Support/ProjectPaths.swift")
            .path

        let gameDataDir = ProjectPaths.gameDataDirectory(mode: "17", sourceFilePath: sourceFilePath)

        XCTAssertTrue(FileManager.default.fileExists(atPath: gameDataDir.path), gameDataDir.path)
        XCTAssertTrue(gameDataDir.path.hasSuffix("/Desktop/hexsight/config/game_data/mode17"), gameDataDir.path)
    }

    func testRegionsConfigPointsToExistingFile() {
        let packageRoot = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        let sourceFilePath = packageRoot
            .appendingPathComponent("Sources/HexSight/Support/ProjectPaths.swift")
            .path

        let regionsURL = ProjectPaths.configFile(named: "regions.json", sourceFilePath: sourceFilePath)

        XCTAssertTrue(FileManager.default.fileExists(atPath: regionsURL.path), regionsURL.path)
        XCTAssertTrue(regionsURL.path.hasSuffix("/Desktop/hexsight/config/regions.json"), regionsURL.path)
    }
}
