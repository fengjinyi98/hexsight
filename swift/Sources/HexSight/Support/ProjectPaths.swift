import Foundation

/// 项目资源路径解析器
/// 核心职责：
/// - 统一解析开发模式下的项目根目录
/// - 提供配置、阵容与游戏数据目录路径
/// - 避免各模块重复拼接路径
enum ProjectPaths {
    static func repoRoot(sourceFilePath: String = #filePath) -> URL {
        URL(fileURLWithPath: sourceFilePath)
            .deletingLastPathComponent()  // Support/
            .deletingLastPathComponent()  // HexSight/
            .deletingLastPathComponent()  // Sources/
            .deletingLastPathComponent()  // swift/
            .deletingLastPathComponent()  // hexsight/
    }

    static func configFile(named name: String, sourceFilePath: String = #filePath) -> URL {
        repoRoot(sourceFilePath: sourceFilePath)
            .appendingPathComponent("config/\(name)")
    }

    static func configDirectory(sourceFilePath: String = #filePath) -> URL {
        repoRoot(sourceFilePath: sourceFilePath)
            .appendingPathComponent("config", isDirectory: true)
    }

    static func lineupDirectory(sourceFilePath: String = #filePath) -> URL {
        configDirectory(sourceFilePath: sourceFilePath)
            .appendingPathComponent("lineups", isDirectory: true)
    }

    static func gameDataDirectory(mode: String, sourceFilePath: String = #filePath) -> URL {
        configDirectory(sourceFilePath: sourceFilePath)
            .appendingPathComponent("game_data", isDirectory: true)
            .appendingPathComponent("mode\(mode)", isDirectory: true)
    }
}
