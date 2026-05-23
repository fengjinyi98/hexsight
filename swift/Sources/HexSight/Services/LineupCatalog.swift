import Foundation

/// LineupCatalog 阵容数据源策略
/// 核心职责：
/// - 声明支持的模式集合
/// - 为阵容页提供空态文案
/// CDN URL / 缓存文件 / 远端拉取已下沉 Rust `RemoteLineupSource`
enum LineupCatalog {
    static let supportedModes = Set(["17", "16", "4"])

    static func supportsRemoteFetch(for mode: String) -> Bool {
        ModeProfile.profile(for: mode) != nil
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
