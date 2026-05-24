import Foundation

/// 项目资源路径解析器
/// 核心职责：
/// - 统一解析开发模式下的项目根目录
/// - 提供配置、阵容与游戏数据目录路径
/// - 避免各模块重复拼接路径
enum ProjectPaths {
    private static let anchorFilePath = #filePath
    private static let configRootEnvironmentKey = "HEXSIGHT_CONFIG_ROOT"

    static func repoRoot() -> URL {
        repoRoot(sourceFilePath: anchorFilePath)
    }

    static func repoRoot(sourceFilePath: String) -> URL {
        URL(fileURLWithPath: sourceFilePath)
            .deletingLastPathComponent()  // Support/
            .deletingLastPathComponent()  // HexSight/
            .deletingLastPathComponent()  // Sources/
            .deletingLastPathComponent()  // swift/
            .deletingLastPathComponent()  // hexsight/
    }

    static func configFile(named name: String) -> URL {
        configFile(named: name, sourceFilePath: anchorFilePath)
    }

    static func configFile(named name: String, sourceFilePath: String) -> URL {
        configDirectory(sourceFilePath: sourceFilePath)
            .appendingPathComponent(name)
    }

    static func configDirectory() -> URL {
        configDirectory(sourceFilePath: anchorFilePath)
    }

    static func configDirectory(sourceFilePath: String) -> URL {
        if let overridePath = ProcessInfo.processInfo.environment[configRootEnvironmentKey],
           !overridePath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return URL(fileURLWithPath: overridePath, isDirectory: true).standardizedFileURL
        }

        return repoRoot(sourceFilePath: sourceFilePath)
            .appendingPathComponent("config", isDirectory: true)
    }

    static func lineupDirectory() -> URL {
        lineupDirectory(sourceFilePath: anchorFilePath)
    }

    static func lineupDirectory(sourceFilePath: String) -> URL {
        configDirectory(sourceFilePath: sourceFilePath)
            .appendingPathComponent("lineups", isDirectory: true)
    }

    static func gameDataDirectory(mode: String) -> URL {
        gameDataDirectory(mode: mode, sourceFilePath: anchorFilePath)
    }

    static func gameDataDirectory(mode: String, sourceFilePath: String) -> URL {
        configDirectory(sourceFilePath: sourceFilePath)
            .appendingPathComponent("game_data", isDirectory: true)
            .appendingPathComponent("mode\(mode)", isDirectory: true)
    }
}
