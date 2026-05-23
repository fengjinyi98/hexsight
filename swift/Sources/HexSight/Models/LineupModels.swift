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

    /// 从 Rust FFI 返回的 JSON dict 初始化（snake_case key）
    init?(rustDict: [String: Any]) {
        let id = lineupString(rustDict["id"])
        let name = lineupString(rustDict["name"])
        guard !id.isEmpty, !name.isEmpty else { return nil }

        let detailPayload = rustDict["detail"] as? [String: Any] ?? [:]
        let parsedDetail = LineupDetailData(rustPayload: detailPayload)

        self.init(
            id: id,
            name: name,
            author: lineupString(rustDict["author"]),
            authorAvatar: lineupString(rustDict["author_avatar"]),
            quality: lineupString(rustDict["quality"]),
            traits: (rustDict["traits"] as? [String]) ?? [],
            category: rustDict["category"] as? String,
            tags: (rustDict["tags"] as? [String]) ?? [],
            top4Rate: rustDict["top4_rate"] as? Double ?? 0,
            rawData: nil,
            detail: parsedDetail
        )
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

    /// 从 Rust FFI 返回的 JSON dict 初始化（snake_case key）
    init(rustPayload: [String: Any]) {
        let parsePieces: (Any?) -> [LineupPiece] = { val in
            (val as? [[String: Any]] ?? []).compactMap { LineupPiece(dict: $0) }
        }
        let parseContacts: (Any?) -> [LineupTraitContact] = { val in
            (val as? [[String: Any]] ?? []).compactMap { LineupTraitContact(dict: $0) }
        }

        self.init(
            raw: rustPayload,
            finalHeroes: parsePieces(rustPayload["final_heroes"]),
            earlyHeroes: parsePieces(rustPayload["early_heroes"]),
            midHeroes: parsePieces(rustPayload["mid_heroes"]),
            recommendedHexIDs: rustPayload["recommended_hex_ids"] as? [String] ?? [],
            replacementHexIDs: rustPayload["replacement_hex_ids"] as? [String] ?? [],
            equipmentOrderIDs: rustPayload["equipment_order_ids"] as? [String] ?? [],
            level3HeroIDs: rustPayload["level_3_hero_ids"] as? [String] ?? [],
            heroReplacements: (rustPayload["hero_replacements"] as? [[String: Any]] ?? []).compactMap(LineupHeroReplacement.init),
            unlockTasks: (rustPayload["unlock_tasks"] as? [[String: Any]] ?? []).compactMap(LineupUnlockTask.init),
            godRewards: (rustPayload["god_rewards"] as? [[String: Any]] ?? []).compactMap(LineupGodReward.init),
            officialTraits: parseContacts(rustPayload["official_traits"]),
            earlyTraits: parseContacts(rustPayload["early_traits"]),
            midTraits: parseContacts(rustPayload["mid_traits"]),
            chosenContact: LineupTraitContact(dict: rustPayload["chosen_contact"] as? [String: Any] ?? [:]),
            messengerContact: LineupTraitContact(dict: rustPayload["messenger_contact"] as? [String: Any] ?? [:]),
            chosenBackups: (rustPayload["chosen_backups"] as? [[String: Any]] ?? []).compactMap(LineupChosenBackup.init),
            lineFeature: lineupString(rustPayload["line_feature"]),
            earlyInfo: lineupString(rustPayload["early_info"]),
            dTime: lineupString(rustPayload["d_time"]),
            locationInfo: lineupString(rustPayload["location_info"]),
            enemyInfo: lineupString(rustPayload["enemy_info"]),
            hexInfo: lineupString(rustPayload["hex_info"]),
            equipmentInfo: lineupString(rustPayload["equipment_info"]),
            godRewardInfo: lineupString(rustPayload["god_reward_info"]),
            taskInfo: lineupString(rustPayload["task_info"]),
            chosenInfo: lineupString(rustPayload["chosen_info"]),
            locationInfo2: lineupString(rustPayload["location_info2"]),
            earlyRound: lineupString(rustPayload["early_round"]),
            midRound: lineupString(rustPayload["mid_round"]),
            staffInfo: lineupString(rustPayload["staff_info"]),
            goopInfo: lineupString(rustPayload["goop_info"]),
            traitPartyInfo: lineupString(rustPayload["trait_party_info"]),
            legendGalaxyInfo: lineupString(rustPayload["legend_galaxy_info"])
        )
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
        // hero_id：Rust 直接提供，官方需从 task_id 推导
        if let rustHeroID = nonEmptyString(dict["hero_id"]) {
            self.heroID = rustHeroID
        } else {
            self.heroID = String(taskID.dropLast(2))
        }
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
        // stage：官方 "stage_num"，Rust 为 "stage"
        self.stage = Int(lineupString(dict, primary: "stage_num", fallback: "stage")) ?? 0
        self.godID = godID
        // wishes：官方数组 "wishes"，Rust 为 "wish_ids"
        if let wishes = dict["wishes"] as? [Any] {
            self.wishIDs = wishes.map(lineupString).filter { !$0.isEmpty }
        } else if let wishIDs = dict["wish_ids"] as? [String] {
            self.wishIDs = wishIDs.filter { !$0.isEmpty }
        } else if let wishIDs = dict["wish_ids"] as? [Any] {
            self.wishIDs = wishIDs.map(lineupString).filter { !$0.isEmpty }
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
        let type = nonEmptyString(dict, primary: "type", fallback: "contact_type") ?? ""
        guard !id.isEmpty || !type.isEmpty else { return nil }
        self.id = id
        self.type = type
        self.count = Int(lineupString(dict, primary: "num", fallback: "count")) ?? 0
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
        // hero：官方 "hero_$key_id"，Rust 为 "hero_id"
        let heroID = nonEmptyString(dict, primary: "hero_$key_id", fallback: "hero_id") ?? ""
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
        // 位置：官方字段 "location"，Rust snake_case 为 "location_key"
        let location = lineupString(dict, primary: "location", fallback: "location_key")
        let parts = location.split(separator: ",").compactMap { Int($0.trimmingCharacters(in: .whitespaces)) }
        guard parts.count == 2 else { return nil }
        // idInLineup：官方 camelCase，Rust 为 id_in_lineup
        self.idInLineup = Int(lineupString(dict, primary: "idInLineup", fallback: "id_in_lineup")) ?? 0
        self.chessType = nonEmptyString(dict["chess_type"]) ?? "hero"
        self.heroID = heroID
        // 装备：官方 "equipment_id"（逗号分隔字符串），Rust "equipment_ids"（数组）
        if let equipArr = dict["equipment_ids"] as? [String] {
            self.equipmentIDs = equipArr.filter { !$0.isEmpty && $0 != "0" }
        } else if let equipArr = dict["equipment_ids"] as? [Any] {
            self.equipmentIDs = equipArr.map(lineupString).filter { !$0.isEmpty && $0 != "0" }
        } else {
            self.equipmentIDs = splitIDs(dict["equipment_id"])
        }
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

/// 从 dict 取值，优先用 primary 键，为空时回退到 fallback 键
func lineupString(_ dict: [String: Any], primary: String, fallback: String) -> String {
    let val = lineupString(dict[primary])
    if !val.isEmpty { return val }
    return lineupString(dict[fallback])
}

func nonEmptyString(_ val: Any?) -> String? {
    let value = lineupString(val)
    return value.isEmpty ? nil : value
}

/// 从 dict 取值，优先 primary，为空回退 fallback
func nonEmptyString(_ dict: [String: Any], primary: String, fallback: String) -> String? {
    if let val = nonEmptyString(dict[primary]) { return val }
    return nonEmptyString(dict[fallback])
}

func splitIDs(_ val: Any?) -> [String] {
    lineupString(val)
        .split(separator: ",")
        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
        .filter { !$0.isEmpty && $0 != "0" }
}
