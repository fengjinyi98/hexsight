import Foundation

/// ModeCapability 模式能力声明
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
/// - 声明模式 ID、名称、赛季与能力
/// - CDN URL / 缓存文件 / 远端拉取 已下沉 Rust `RemoteLineupSource`
struct ModeProfile: Identifiable, Hashable {
    let id: String
    let name: String
    let season: String
    let capabilities: ModeCapability

    static let supported: [ModeProfile] = [
        ModeProfile(id: "17", name: "星神", season: "S18", capabilities: [.standardLineup, .godRewards, .transitionContacts]),
        ModeProfile(id: "16", name: "英雄联盟传奇", season: "S18", capabilities: [.standardLineup, .unlockTasks, .transitionContacts]),
        ModeProfile(id: "4", name: "天选福星", season: "S18", capabilities: [.standardLineup, .chosenMechanics, .transitionContacts]),
    ]

    static func profile(for id: String) -> ModeProfile? {
        supported.first { $0.id == id }
    }

    static func profileOrFallback(for id: String) -> ModeProfile {
        profile(for: id) ?? ModeProfile(id: id, name: "未知模式", season: "S18", capabilities: [])
    }
}
