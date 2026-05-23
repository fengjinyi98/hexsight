import Foundation

/// RuleHeroSnapshot 规则英雄快照
/// 核心职责：
/// - 将阵容棋子 ID 解析为可读英雄信息
/// - 保留站位、主 C 标记和装备名称
/// - 为规则引擎和 LLM 提供直观英雄输入
struct RuleHeroSnapshot: Identifiable, Equatable {
    let id: String
    let name: String
    let cost: Int
    let picture: String
    let position: String
    let isCarry: Bool
    let equipmentIDs: [String]
    let equipmentNames: [String]
}

/// RuleEquipmentSnapshot 规则装备快照
/// 核心职责：
/// - 将装备 ID 解析为名称、类型和图标
/// - 支撑装备优先级与主 C 装备规则
struct RuleEquipmentSnapshot: Identifiable, Equatable {
    let id: String
    let name: String
    let type: String
    let picture: String
}

/// RuleHexSnapshot 规则强化符文快照
/// 核心职责：
/// - 将强化符文 ID 解析为名称、等级和描述
/// - 支撑强化符文优先级和语义理解
struct RuleHexSnapshot: Identifiable, Equatable {
    let id: String
    let name: String
    let level: Int
    let desc: String
    let icon: String
}

/// RuleTraitSnapshot 规则羁绊快照
/// 核心职责：
/// - 表示阵容激活羁绊的可读名称与计数
/// - 兼容官方羁绊计数和本地棋子反推
struct RuleTraitSnapshot: Identifiable, Equatable {
    let id: String
    let traitID: String
    let type: String
    let name: String
    let count: Int
    let color: Int
    let level: Int
    let picture: String
}

/// LineupRulesContext 阵容规则上下文
/// 核心职责：
/// - 将阵容领域模型整理为规则引擎和 LLM 可消费的稳定输入
/// - 暴露英雄、装备、羁绊、强化符文和模式玩法字段
/// - 避免后续规则直接读取 UI 展示状态
struct LineupRulesContext {
    let mode: ModeProfile
    let lineupID: String
    let name: String
    let author: String
    let quality: String
    let tags: [String]
    let finalHeroIDs: [String]
    let earlyHeroIDs: [String]
    let midHeroIDs: [String]
    let carryHeroIDs: [String]
    let finalHeroEquipment: [String: [String]]
    let boardPositions: [String: String]
    let resolvedFinalHeroes: [RuleHeroSnapshot]
    let resolvedEquipmentOrder: [RuleEquipmentSnapshot]
    let resolvedRecommendedHexes: [RuleHexSnapshot]
    let resolvedReplacementHexes: [RuleHexSnapshot]
    let resolvedTraits: [RuleTraitSnapshot]
    let recommendedHexIDs: [String]
    let replacementHexIDs: [String]
    let equipmentOrderIDs: [String]
    let traitContacts: [LineupTraitContact]
    let earlyTraitContacts: [LineupTraitContact]
    let midTraitContacts: [LineupTraitContact]
    let unlockTasks: [LineupUnlockTask]
    let godRewards: [LineupGodReward]
    let chosenContact: LineupTraitContact?
    let messengerContact: LineupTraitContact?
    let chosenBackups: [LineupChosenBackup]
    let modeSpecificTexts: [String: String]
    let strategyTexts: [String: String]

    init(card: LineupCard, mode: String) {
        self.init(card: card, mode: mode, gameData: nil)
    }

    init(card: LineupCard, mode: String, gameData: GameDataIndex?) {
        let detail = card.detail
        self.mode = ModeProfile.profileOrFallback(for: mode)
        self.lineupID = card.id
        self.name = card.name
        self.author = card.author
        self.quality = card.quality
        self.tags = card.tags
        self.finalHeroIDs = detail.finalHeroes.map(\.heroID)
        self.earlyHeroIDs = detail.earlyHeroes.map(\.heroID)
        self.midHeroIDs = detail.midHeroes.map(\.heroID)
        self.carryHeroIDs = detail.finalHeroes.filter(\.isCarryHero).map(\.heroID)
        self.finalHeroEquipment = Dictionary(detail.finalHeroes.map { ($0.heroID, $0.equipmentIDs) }) { current, _ in current }
        self.boardPositions = Dictionary(detail.finalHeroes.map { ($0.heroID, $0.locationKey) }) { current, _ in current }
        self.resolvedFinalHeroes = LineupRulesContext.resolveHeroes(detail.finalHeroes, gameData: gameData)
        self.resolvedEquipmentOrder = LineupRulesContext.resolveEquipment(detail.equipmentOrderIDs, gameData: gameData)
        self.resolvedRecommendedHexes = LineupRulesContext.resolveHexes(detail.recommendedHexIDs, gameData: gameData)
        self.resolvedReplacementHexes = LineupRulesContext.resolveHexes(detail.replacementHexIDs, gameData: gameData)
        self.resolvedTraits = gameData?.traitSummaries(for: detail.finalHeroes, officialContacts: detail.officialTraits) ?? []
        self.recommendedHexIDs = detail.recommendedHexIDs
        self.replacementHexIDs = detail.replacementHexIDs
        self.equipmentOrderIDs = detail.equipmentOrderIDs
        self.traitContacts = detail.officialTraits
        self.earlyTraitContacts = detail.earlyTraits
        self.midTraitContacts = detail.midTraits
        self.unlockTasks = detail.unlockTasks
        self.godRewards = detail.godRewards
        self.chosenContact = detail.chosenContact
        self.messengerContact = detail.messengerContact
        self.chosenBackups = detail.chosenBackups
        self.modeSpecificTexts = LineupRulesContext.compactTextMap([
            "godRewardInfo": detail.godRewardInfo,
            "taskInfo": detail.taskInfo,
            "chosenInfo": detail.chosenInfo,
            "staffInfo": detail.staffInfo,
            "goopInfo": detail.goopInfo,
            "traitPartyInfo": detail.traitPartyInfo,
            "legendGalaxyInfo": detail.legendGalaxyInfo,
        ])
        self.strategyTexts = LineupRulesContext.compactTextMap([
            "lineFeature": detail.lineFeature,
            "earlyInfo": detail.earlyInfo,
            "dTime": detail.dTime,
            "locationInfo": detail.locationInfo,
            "locationInfo2": detail.locationInfo2,
            "enemyInfo": detail.enemyInfo,
            "hexInfo": detail.hexInfo,
            "equipmentInfo": detail.equipmentInfo,
            "earlyRound": detail.earlyRound,
            "midRound": detail.midRound,
        ])
    }

    private static func compactTextMap(_ source: [String: String]) -> [String: String] {
        source.filter { !$0.value.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    }

    private static func resolveHeroes(_ pieces: [LineupPiece], gameData: GameDataIndex?) -> [RuleHeroSnapshot] {
        guard let gameData else { return [] }
        return pieces.map { piece in
            let hero = gameData.hero(id: piece.heroID)
            let equipmentNames = piece.equipmentIDs.map { gameData.equipment(id: $0)?.name ?? $0 }
            return RuleHeroSnapshot(
                id: piece.heroID,
                name: hero?.name ?? piece.heroID,
                cost: hero?.cost ?? 0,
                picture: hero?.picture ?? "",
                position: piece.locationKey,
                isCarry: piece.isCarryHero,
                equipmentIDs: piece.equipmentIDs,
                equipmentNames: equipmentNames
            )
        }
    }

    private static func resolveEquipment(_ ids: [String], gameData: GameDataIndex?) -> [RuleEquipmentSnapshot] {
        guard let gameData else { return [] }
        return ids.map { id in
            let equipment = gameData.equipment(id: id)
            return RuleEquipmentSnapshot(
                id: id,
                name: equipment?.name ?? id,
                type: equipment?.type ?? "",
                picture: equipment?.picture ?? ""
            )
        }
    }

    private static func resolveHexes(_ ids: [String], gameData: GameDataIndex?) -> [RuleHexSnapshot] {
        guard let gameData else { return [] }
        return ids.map { id in
            let hex = gameData.hex(id: id)
            return RuleHexSnapshot(
                id: id,
                name: hex?.name ?? id,
                level: hex?.level ?? 0,
                desc: hex?.desc ?? "",
                icon: hex?.icon ?? ""
            )
        }
    }
}
