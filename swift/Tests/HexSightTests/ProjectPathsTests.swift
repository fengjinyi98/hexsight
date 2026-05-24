import XCTest
@testable import HexSight

/// ProjectPathsTests 路径回归测试
/// 核心职责：
/// - 校验开发模式资源路径落在仓库根目录
/// - 防止再次回退到错误的相对路径推导
final class ProjectPathsTests: XCTestCase {
    func testGameDataDirectoryPointsToExistingModeFolder() {
        let fixture = makeConfigFixture()

        withEnvironment("HEXSIGHT_CONFIG_ROOT", value: fixture.config.path) {
            let gameDataDir = ProjectPaths.gameDataDirectory(mode: "17", sourceFilePath: fixture.sourceFilePath)

            XCTAssertTrue(FileManager.default.fileExists(atPath: gameDataDir.path), gameDataDir.path)
            XCTAssertEqual(gameDataDir.path, fixture.config.appendingPathComponent("game_data/mode17", isDirectory: true).path)
        }
    }

    func testRegionsConfigPointsToExistingFile() {
        let fixture = makeConfigFixture()

        withEnvironment("HEXSIGHT_CONFIG_ROOT", value: fixture.config.path) {
            let regionsURL = ProjectPaths.configFile(named: "regions.json", sourceFilePath: fixture.sourceFilePath)

            XCTAssertTrue(FileManager.default.fileExists(atPath: regionsURL.path), regionsURL.path)
            XCTAssertEqual(regionsURL.path, fixture.config.appendingPathComponent("regions.json").path)
        }
    }

    func testLineupDirectoryPointsToExistingFolderUsingDefaultAnchor() {
        let fixture = makeConfigFixture()

        withEnvironment("HEXSIGHT_CONFIG_ROOT", value: fixture.config.path) {
            let lineupDir = ProjectPaths.lineupDirectory()

            XCTAssertTrue(FileManager.default.fileExists(atPath: lineupDir.path), lineupDir.path)
            XCTAssertEqual(lineupDir.path, fixture.config.appendingPathComponent("lineups", isDirectory: true).path)
        }
    }

    private func makeConfigFixture() -> (config: URL, sourceFilePath: String) {
        let root = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        let config = root.appendingPathComponent("config", isDirectory: true)
        let sourceFilePath = root
            .appendingPathComponent("swift/Sources/HexSight/Support/ProjectPaths.swift")
            .path

        try? FileManager.default.createDirectory(
            at: config.appendingPathComponent("game_data/mode17", isDirectory: true),
            withIntermediateDirectories: true
        )
        try? FileManager.default.createDirectory(
            at: config.appendingPathComponent("lineups", isDirectory: true),
            withIntermediateDirectories: true
        )
        FileManager.default.createFile(
            atPath: config.appendingPathComponent("regions.json").path,
            contents: Data("{}".utf8),
            attributes: nil
        )
        addTeardownBlock {
            try? FileManager.default.removeItem(at: root)
        }
        return (config.standardizedFileURL, sourceFilePath)
    }

    private func withEnvironment(_ key: String, value: String, run: () -> Void) {
        let previous = getenv(key).map { String(cString: $0) }
        setenv(key, value, 1)
        run()
        if let previous {
            setenv(key, previous, 1)
        } else {
            unsetenv(key)
        }
    }
}
