import Foundation

/// LineupCatalog 阵容数据源策略
/// 核心职责：
/// - 统一声明各模式的阵容缓存文件策略
/// - 统一声明各模式是否支持远端阵容拉取
/// - 为阵容页提供稳定的空态文案
enum LineupCatalog {
    static let currentSeason = "S18"
    static let supportedModes = Set(["17", "16", "4"])
    static let channel = "11"

    static func cacheFileName(for mode: String) -> String? {
        ModeProfile.profile(for: mode)?.cacheFileName
    }

    static func supportsRemoteFetch(for mode: String) -> Bool {
        ModeProfile.profile(for: mode) != nil
    }

    static func remoteURL(for mode: String) -> URL? {
        ModeProfile.profile(for: mode)?.remoteURL
    }

    static func emptyState(for mode: String) -> (title: String, hint: String) {
        if supportedModes.contains(mode) {
            return ("阵容数据未加载", "运行 python3 scripts/jcc_api.py fetch-lineups")
        }
        return ("当前模式暂无阵容数据", "当前模式未接入阵容数据源")
    }

    static func categoryName(for lineTag: String) -> String? {
        switch lineTag {
        case "1": "新手推荐"
        case "2": "高手进阶"
        case "3": "趣味娱乐"
        default: nil
        }
    }
}
