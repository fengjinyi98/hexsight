import Foundation

/// ModeCapability 模式能力声明
/// 核心职责：
/// - 描述不同玩法模式具备的阵容详情能力
/// - 为 UI、规则上下文和适配层提供统一能力判断
/// - 降低新增模式时的条件分支扩散
struct ModeCapability: OptionSet, Hashable {
    let rawValue: Int

    static let standardLineup = ModeCapability(rawValue: 1 << 0)
    static let godRewards = ModeCapability(rawValue: 1 << 1)
    static let unlockTasks = ModeCapability(rawValue: 1 << 2)
    static let chosenMechanics = ModeCapability(rawValue: 1 << 3)
    static let transitionContacts = ModeCapability(rawValue: 1 << 4)
}

/// ModeProfile 游戏模式配置
/// 核心职责：
/// - 统一声明模式 ID、名称、赛季与阵容 CDN 路径
/// - 统一声明模式专属能力
/// - 为追版本提供单点配置入口
struct ModeProfile: Identifiable, Hashable {
    let id: String
    let name: String
    let season: String
    let lineupVersionPath: String
    let channel: String
    let capabilities: ModeCapability

    var cacheFileName: String { "mode\(id)_\(season).json" }

    var remoteURL: URL? {
        URL(string: "https://game.gtimg.cn/images/lol/act/jkzlkauto/json/lineupJson/\(lineupVersionPath)/\(channel)/\(id)/lineup_detail_total.json")
    }

    static let supported: [ModeProfile] = [
        ModeProfile(
            id: "17",
            name: "星神",
            season: "S18",
            lineupVersionPath: "m18",
            channel: "11",
            capabilities: [.standardLineup, .godRewards, .transitionContacts]
        ),
        ModeProfile(
            id: "16",
            name: "英雄联盟传奇",
            season: "S18",
            lineupVersionPath: "m17",
            channel: "11",
            capabilities: [.standardLineup, .unlockTasks, .transitionContacts]
        ),
        ModeProfile(
            id: "4",
            name: "天选福星",
            season: "S18",
            lineupVersionPath: "m17",
            channel: "11",
            capabilities: [.standardLineup, .chosenMechanics, .transitionContacts]
        ),
    ]

    static func profile(for id: String) -> ModeProfile? {
        supported.first { $0.id == id }
    }

    static func profileOrFallback(for id: String) -> ModeProfile {
        profile(for: id) ?? ModeProfile(
            id: id,
            name: "未知模式",
            season: LineupCatalog.currentSeason,
            lineupVersionPath: "",
            channel: LineupCatalog.channel,
            capabilities: []
        )
    }
}
