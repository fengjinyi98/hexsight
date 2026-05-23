import Foundation

/// LineupCard 阵容卡片数据模型
/// 核心职责：
/// - 解析官方阵容 JSON 的列表字段
/// - 暴露列表与详情页需要的展示数据
/// - 保留原始数据用于详情扩展
struct LineupCard: Identifiable {
    let id: String
    let name: String
    let author: String
    let authorAvatar: String
    let quality: String
    let traits: [String]
    let category: String?
    let tags: [String]
    let top4Rate: Double
    let rawData: [String: Any]?
    let detail: LineupDetailData

    var heroPreview: [LineupPiece] { Array(detail.finalHeroes.prefix(8)) }
    var augmentIDs: [String] { Array(detail.recommendedHexIDs.prefix(3)) }

    init(
        id: String,
        name: String,
        author: String,
        authorAvatar: String,
        quality: String,
        traits: [String],
        category: String?,
        tags: [String],
        top4Rate: Double,
        rawData: [String: Any]?,
        detail: LineupDetailData
    ) {
        self.id = id
        self.name = name
        self.author = author
        self.authorAvatar = authorAvatar
        self.quality = quality
        self.traits = traits
        self.category = category
        self.tags = tags
        self.top4Rate = top4Rate
        self.rawData = rawData
        self.detail = detail
    }

    init?(dict: [String: Any], rawData: [String: Any]? = nil) {
        guard let card = LineupSourceAdapter.card(from: rawData ?? dict) else { return nil }
        self = card
    }

    static func extractBracketTraits(from name: String) -> [String] {
        guard let start = name.firstIndex(of: "【"), let end = name.firstIndex(of: "】"), start < end else { return [] }
        let body = String(name[name.index(after: start)..<end])
        return body.split(whereSeparator: { $0 == " " || $0 == "/" || $0 == "、" }).map(String.init)
    }
}

/// LineupDetailData 阵容详情解析结果
/// 核心职责：
/// - 解析最终阵容、过渡阵容、装备、符文与运营文本
/// - 为列表预览与详情页棋盘提供稳定数据
/// - 隔离官方 JSON 字段差异
struct LineupDetailData {
    let raw: [String: Any]
    let finalHeroes: [LineupPiece]
    let earlyHeroes: [LineupPiece]
    let midHeroes: [LineupPiece]
    let recommendedHexIDs: [String]
    let replacementHexIDs: [String]
    let equipmentOrderIDs: [String]
    let level3HeroIDs: [String]
    let heroReplacements: [LineupHeroReplacement]
    let unlockTasks: [LineupUnlockTask]
    let godRewards: [LineupGodReward]
    let officialTraits: [LineupTraitContact]
    let earlyTraits: [LineupTraitContact]
    let midTraits: [LineupTraitContact]
    let chosenContact: LineupTraitContact?
    let messengerContact: LineupTraitContact?
    let chosenBackups: [LineupChosenBackup]
    let lineFeature: String
    let earlyInfo: String
    let dTime: String
    let locationInfo: String
    let enemyInfo: String
    let hexInfo: String
    let equipmentInfo: String
    let godRewardInfo: String
    let taskInfo: String
    let chosenInfo: String
    let locationInfo2: String
    let earlyRound: String
    let midRound: String
    let staffInfo: String
    let goopInfo: String
    let traitPartyInfo: String
    let legendGalaxyInfo: String

    init(
        raw: [String: Any],
        finalHeroes: [LineupPiece],
        earlyHeroes: [LineupPiece],
        midHeroes: [LineupPiece],
        recommendedHexIDs: [String],
        replacementHexIDs: [String],
        equipmentOrderIDs: [String],
        level3HeroIDs: [String],
        heroReplacements: [LineupHeroReplacement],
        unlockTasks: [LineupUnlockTask],
        godRewards: [LineupGodReward],
        officialTraits: [LineupTraitContact],
        earlyTraits: [LineupTraitContact],
        midTraits: [LineupTraitContact],
        chosenContact: LineupTraitContact?,
        messengerContact: LineupTraitContact?,
        chosenBackups: [LineupChosenBackup],
        lineFeature: String,
        earlyInfo: String,
        dTime: String,
        locationInfo: String,
        enemyInfo: String,
        hexInfo: String,
        equipmentInfo: String,
        godRewardInfo: String,
        taskInfo: String,
        chosenInfo: String,
        locationInfo2: String,
        earlyRound: String,
        midRound: String,
        staffInfo: String,
        goopInfo: String,
        traitPartyInfo: String,
        legendGalaxyInfo: String
    ) {
        self.raw = raw
        self.finalHeroes = finalHeroes
        self.earlyHeroes = earlyHeroes
        self.midHeroes = midHeroes
        self.recommendedHexIDs = recommendedHexIDs
        self.replacementHexIDs = replacementHexIDs
        self.equipmentOrderIDs = equipmentOrderIDs
        self.level3HeroIDs = level3HeroIDs
        self.heroReplacements = heroReplacements
        self.unlockTasks = unlockTasks
        self.godRewards = godRewards
        self.officialTraits = officialTraits
        self.earlyTraits = earlyTraits
        self.midTraits = midTraits
        self.chosenContact = chosenContact
        self.messengerContact = messengerContact
        self.chosenBackups = chosenBackups
        self.lineFeature = lineFeature
        self.earlyInfo = earlyInfo
        self.dTime = dTime
        self.locationInfo = locationInfo
        self.enemyInfo = enemyInfo
        self.hexInfo = hexInfo
        self.equipmentInfo = equipmentInfo
        self.godRewardInfo = godRewardInfo
        self.taskInfo = taskInfo
        self.chosenInfo = chosenInfo
        self.locationInfo2 = locationInfo2
        self.earlyRound = earlyRound
        self.midRound = midRound
        self.staffInfo = staffInfo
        self.goopInfo = goopInfo
        self.traitPartyInfo = traitPartyInfo
        self.legendGalaxyInfo = legendGalaxyInfo
    }

    init(payload: [String: Any]) {
        self = LineupSourceAdapter.detail(from: payload)
    }
}


/// LineupUnlockTask 英雄解锁任务模型
/// 核心职责：
/// - 表示官方 task_list 中的任务 ID
/// - 从 task_id 推导任务关联英雄 ID
/// - 为详情页和规则引擎保留解锁条件入口
struct LineupUnlockTask: Identifiable, Equatable {
    var id: String { taskID }
    let taskID: String
    let chessID: String
    let heroID: String

    init?(dict: [String: Any]) {
        let taskID = lineupString(dict["task_id"])
        guard !taskID.isEmpty else { return nil }
        self.taskID = taskID
        self.chessID = lineupString(dict["chess_id"])
        self.heroID = String(taskID.dropLast(2))
    }
}

/// LineupGodReward 星神奖励模型
/// 核心职责：
/// - 表示 mode17 的阶段神明奖励选择
/// - 保留 god_id、stage 与 wish_id 列表
/// - 支撑星神模式规则提取
struct LineupGodReward: Identifiable, Equatable {
    var id: String { "\(stage)-\(godID)-\(wishIDs.joined(separator: "-"))" }
    let stage: Int
    let godID: String
    let wishIDs: [String]

    init?(dict: [String: Any]) {
        let godID = lineupString(dict["god_id"])
        guard !godID.isEmpty else { return nil }
        self.stage = Int(lineupString(dict["stage_num"])) ?? 0
        self.godID = godID
        if let wishes = dict["wishes"] as? [Any] {
            self.wishIDs = wishes.map(lineupString).filter { !$0.isEmpty }
        } else {
            self.wishIDs = splitIDs(dict["wishes"])
        }
    }
}

/// LineupTraitContact 官方羁绊计数模型
/// 核心职责：
/// - 表示官方 contact 字段中的羁绊 ID 与数量
/// - 区分种族、职业和特殊羁绊类型
/// - 保留颜色等级用于还原官网徽章样式
struct LineupTraitContact: Identifiable, Equatable, Hashable {
    let id: String
    let type: String
    let count: Int
    let color: Int
    let level: Int

    init?(dict: [String: Any]) {
        let id = lineupString(dict["id"])
        let type = nonEmptyString(dict["type"]) ?? ""
        guard !id.isEmpty || !type.isEmpty else { return nil }
        self.id = id
        self.type = type
        self.count = Int(lineupString(dict["num"])) ?? 0
        self.color = Int(lineupString(dict["color"])) ?? 0
        self.level = Int(lineupString(dict["level"])) ?? 0
    }

    static func synthetic(id: String, type: String) -> LineupTraitContact {
        LineupTraitContact(id: id, type: type, count: 0, color: 1, level: 1)
    }

    private init(id: String, type: String, count: Int, color: Int, level: Int) {
        self.id = id
        self.type = type
        self.count = count
        self.color = color
        self.level = level
    }
}

/// LineupChosenBackup 天选备选模型
/// 核心职责：
/// - 表示 mode4 天选备选英雄与对应羁绊
/// - 保留官方英雄 key 与羁绊类型
/// - 支撑天选福星模式规则提取
struct LineupChosenBackup: Identifiable, Equatable, Hashable {
    var id: String { "\(heroID)-\(traitID)-\(type)" }
    let heroID: String
    let traitID: String
    let type: String

    init?(dict: [String: Any]) {
        let heroID = nonEmptyString(dict["hero_$key_id"]) ?? ""
        let traitID = nonEmptyString(dict["id"]) ?? ""
        guard !heroID.isEmpty || !traitID.isEmpty else { return nil }
        self.heroID = heroID
        self.traitID = traitID
        self.type = nonEmptyString(dict["type"]) ?? ""
    }
}

/// LineupPiece 阵容棋子站位模型
/// 核心职责：
/// - 表示棋子 ID、站位、装备与主 C 标记
/// - 提供棋盘定位和列表预览所需字段
struct LineupPiece: Identifiable, Equatable {
    let idInLineup: Int
    let chessType: String
    let heroID: String
    let equipmentIDs: [String]
    let isCarryHero: Bool
    let row: Int
    let col: Int

    var id: String { "\(idInLineup)-\(heroID)-\(row)-\(col)" }
    var locationKey: String { "\(row),\(col)" }

    init?(dict: [String: Any]) {
        let heroID = lineupString(dict["hero_id"])
        guard !heroID.isEmpty else { return nil }
        let location = lineupString(dict["location"])
        let parts = location.split(separator: ",").compactMap { Int($0.trimmingCharacters(in: .whitespaces)) }
        guard parts.count == 2 else { return nil }
        self.idInLineup = Int(lineupString(dict["idInLineup"])) ?? 0
        self.chessType = nonEmptyString(dict["chess_type"]) ?? "hero"
        self.heroID = heroID
        self.equipmentIDs = splitIDs(dict["equipment_id"])
        self.isCarryHero = dict["is_carry_hero"] as? Bool ?? false
        self.row = parts[0]
        self.col = parts[1]
    }
}

/// LineupHeroReplacement 阵容替换棋子模型
/// 核心职责：
/// - 表示主棋子与可替换棋子的映射关系
/// - 支撑详情页展示变阵提示
struct LineupHeroReplacement: Equatable {
    let heroID: String
    let replacementHeroIDs: [String]

    init?(dict: [String: Any]) {
        let heroID = lineupString(dict["hero_id"])
        guard !heroID.isEmpty else { return nil }
        self.heroID = heroID
        self.replacementHeroIDs = splitIDs(dict["replace_heros"])
    }
}

func lineupString(_ val: Any?) -> String {
    if let s = val as? String { return s.trimmingCharacters(in: .whitespacesAndNewlines) }
    if let i = val as? Int { return String(i) }
    if let d = val as? Double { return d.truncatingRemainder(dividingBy: 1) == 0 ? String(Int(d)) : String(d) }
    return ""
}

func nonEmptyString(_ val: Any?) -> String? {
    let value = lineupString(val)
    return value.isEmpty ? nil : value
}

func splitIDs(_ val: Any?) -> [String] {
    lineupString(val)
        .split(separator: ",")
        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
        .filter { !$0.isEmpty && $0 != "0" }
}
