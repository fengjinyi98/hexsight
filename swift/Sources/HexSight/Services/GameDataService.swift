import Foundation

/// 游戏数据加载器
@MainActor
final class GameDataService: ObservableObject {
    static let shared = GameDataService()

    @Published var heroes: [HeroModel] = []
    @Published var equipment: [EquipmentModel] = []
    @Published var traits: [TraitModel] = []
    @Published var hexes: [HexModel] = []
    @Published var selectedMode: String = "17"

    /// ID → 名称 映射
    private(set) var raceNames: [String: String] = [:]
    private(set) var jobNames: [String: String] = [:]
    private(set) var equipNames: [String: String] = [:]

    let availableModes: [(id: String, name: String)] = [
        ("17", "星神"),
        ("16", "英雄联盟传奇"),
        ("4", "天选福星"),
    ]

    /// 全模式英雄名 → 头像 URL 缓存
    private var allHeroPictures: [String: String] = [:]
    /// 全模式 hero_id → 英雄数据缓存
    private var allHeroesByMode: [String: [String: HeroModel]] = [:]
    /// 全模式羁绊缓存
    private var allTraitsByMode: [String: [TraitModel]] = [:]
    /// 全模式任务缓存
    private var allMissionsByMode: [String: [String: MissionModel]] = [:]
    /// 全模式星神奖励缓存
    private var allGodWishesByMode: [String: [String: GodWishModel]] = [:]

    private init() {
        // 预加载所有模式的头像缓存
        for modeId in ["17", "16", "4"] {
            let dir = ProjectPaths.gameDataDirectory(mode: modeId).path
            let h: [HeroModel] = loadArray("\(dir)/chess.json", key: "data")
            var heroMap: [String: HeroModel] = [:]
            for hero in h where !hero.name.isEmpty && !hero.picture.isEmpty {
                allHeroPictures[hero.name] = hero.picture
                heroMap[hero.id] = hero
            }
            allHeroesByMode[modeId] = heroMap
            allTraitsByMode[modeId] = loadArray("\(dir)/trait.json", key: "data")
            allMissionsByMode[modeId] = loadMissions("\(dir)/mission.json")
            allGodWishesByMode[modeId] = loadGodWishes("\(dir)/god.json")
        }
        loadMode("17")
    }

    func switchMode(_ modeId: String) {
        guard modeId != selectedMode else { return }
        selectedMode = modeId
        loadMode(modeId)
    }

    func loadMode(_ modeId: String) {
        let dir = ProjectPaths.gameDataDirectory(mode: modeId).path

        raceNames = loadMapping("\(dir)/race.json", key: "data")
        jobNames = loadMapping("\(dir)/job.json", key: "data")
        equipNames = loadMapping("\(dir)/equip.json", key: "data")

        heroes = loadArray("\(dir)/chess.json", key: "data")
        equipment = loadArray("\(dir)/equip.json", key: "data")
        traits = loadArray("\(dir)/trait.json", key: "data")
        hexes = loadArray("\(dir)/hex.json", key: "data")
    }

    // MARK: - 查询方法

    /// 按名称查找英雄头像（跨所有模式）
    func heroPicture(named name: String) -> String {
        allHeroPictures[name] ?? ""
    }

    /// 按 hero_id 查找当前模式英雄
    func hero(for id: String, mode: String) -> HeroModel? {
        allHeroesByMode[mode]?[id]
    }

    /// 按 hero_id 查找当前模式英雄名称
    func heroName(for id: String, mode: String) -> String {
        hero(for: id, mode: mode)?.name ?? ""
    }

    /// 按 hero_id 查找当前模式英雄头像
    func heroPicture(for id: String, mode: String) -> String {
        hero(for: id, mode: mode)?.picture ?? ""
    }

    /// 英雄的种族名称
    func raceName(for speciesId: String) -> String {
        raceNames[speciesId] ?? ""
    }

    /// 英雄的职业名称
    func jobName(for classId: String) -> String {
        jobNames[classId] ?? ""
    }

    /// 装备名称
    func equipName(for equipId: String) -> String {
        equipNames[equipId] ?? equipId
    }

    /// 装备详情
    func getEquip(_ id: String) -> EquipmentModel? {
        equipment.first { $0.id == id }
    }

    /// 解锁任务详情
    func mission(for id: String, mode: String) -> MissionModel? {
        allMissionsByMode[mode]?[id]
    }

    /// 星神奖励详情
    func godWish(for id: String, mode: String) -> GodWishModel? {
        allGodWishesByMode[mode]?[id]
    }

    /// 阵容羁绊总览
    func traitSummaries(
        for pieces: [LineupPiece],
        mode: String,
        officialContacts: [LineupTraitContact] = []
    ) -> [LineupTraitSummary] {
        let traits = allTraitsByMode[mode] ?? []
        if !officialContacts.isEmpty {
            return officialContacts.compactMap { contact in
                guard contact.color > 0 || contact.count > 0 else { return nil }
                guard let trait = traits.first(where: { trait in
                    trait.checkId == contact.id && traitMatches(contact.type, trait: trait)
                }) ?? traits.first(where: { $0.checkId == contact.id }) else { return nil }
                return LineupTraitSummary(
                    id: "\(contact.type)-\(contact.id)",
                    traitID: contact.id,
                    type: contact.type,
                    name: trait.name,
                    count: contact.count,
                    color: contact.color,
                    level: contact.level,
                    picture: trait.picture
                )
            }
        }

        var counts: [String: Int] = [:]
        for piece in pieces where piece.chessType == "hero" {
            guard let hero = hero(for: piece.heroID, mode: mode) else { continue }
            for id in splitTraitIDs(hero.species) { counts["race:\(id)", default: 0] += 1 }
            for id in splitTraitIDs(hero.heroClass) { counts["job:\(id)", default: 0] += 1 }
        }

        let summaries = counts.compactMap { key, count -> LineupTraitSummary? in
            let parts = key.split(separator: ":").map(String.init)
            guard parts.count == 2 else { return nil }
            let type = parts[0]
            let checkId = parts[1]
            let candidates = traits.filter { $0.checkId == checkId && traitMatches(type, trait: $0) }
            guard let active = candidates
                .filter({ count >= (Int($0.num) ?? Int.max) })
                .max(by: { $0.level < $1.level })
            else { return nil }
            return LineupTraitSummary(
                id: key,
                traitID: checkId,
                type: type,
                name: active.name,
                count: count,
                color: Int(active.color) ?? 0,
                level: active.level,
                picture: active.picture
            )
        }

        return summaries.sorted { lhs, rhs in
            if lhs.color == rhs.color {
                if lhs.count == rhs.count { return lhs.name < rhs.name }
                return lhs.count > rhs.count
            }
            return lhs.color > rhs.color
        }
    }

    /// 获取羁绊的所有等级
    func traitLevels(for checkId: String) -> [TraitModel] {
        traits.filter { $0.checkId == checkId }
            .sorted { $0.level < $1.level }
    }

    /// 获取某羁绊包含的英雄
    func heroesForTrait(traitId: String, isRace: Bool) -> [HeroModel] {
        let displayable = HeroCatalog.displayHeroes(from: heroes)
        return displayable.filter { h in
            let ids = (isRace ? h.species : h.heroClass).split(separator: "|").map(String.init)
            return ids.contains(traitId)
        }
    }

    // MARK: - 内部加载

    private func loadMapping(_ path: String, key: String) -> [String: String] {
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let container = json[key] as? [String: Any]
        else { return [:] }

        var map: [String: String] = [:]
        for (_, value) in container {
            guard let dict = value as? [String: Any],
                  let name = dict["name"] as? String else { continue }
            let id = stringValue(dict["id"])
            map[id] = name
        }
        return map
    }

    private func loadMissions(_ path: String) -> [String: MissionModel] {
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let container = json["data"] as? [String: Any]
        else { return [:] }

        var map: [String: MissionModel] = [:]
        for (_, value) in container {
            guard let group = value as? [String: Any] else { continue }
            let heroID = stringValue(group["heroid"])
            let missions = group["mission"] as? [[String: Any]] ?? []
            for missionDict in missions {
                guard let mission = MissionModel(dict: missionDict, heroID: heroID) else { continue }
                map[mission.id] = mission
            }
        }
        return map
    }

    private func loadGodWishes(_ path: String) -> [String: GodWishModel] {
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let container = json["data"] as? [[String: Any]]
        else { return [:] }

        var map: [String: GodWishModel] = [:]
        for god in container {
            let godID = stringValue(god["godId"])
            let godName = stringValue(god["godName"])
            let stages = god["stages"] as? [[String: Any]] ?? []
            for stage in stages {
                let stageNumber = Int(stringValue(stage["num"])) ?? 0
                let wishes = stage["wishes"] as? [[String: Any]] ?? []
                for wishDict in wishes {
                    guard let wish = GodWishModel(
                        dict: wishDict,
                        godID: godID,
                        godName: godName,
                        stage: stageNumber
                    ) else { continue }
                    map[wish.id] = wish
                }
            }
        }
        return map
    }

    private func loadArray<T>(_ path: String, key: String) -> [T] {
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let container = json[key] as? [String: Any]
        else {
            print("[GameData] 加载失败: \(path)")
            return []
        }

        let items: [T] = container.values.compactMap { value in
            guard let dict = value as? [String: Any] else { return nil }
            if T.self == HeroModel.self { return HeroModel(dict: dict) as? T }
            if T.self == EquipmentModel.self { return EquipmentModel(dict: dict) as? T }
            if T.self == TraitModel.self { return TraitModel(dict: dict) as? T }
            if T.self == HexModel.self { return HexModel(dict: dict) as? T }
            return nil
        }
        print("[GameData] \(path.split(separator: "/").last ?? ""): \(items.count) 条")
        return items
    }
}

private func splitTraitIDs(_ raw: String) -> [String] {
    raw.split(separator: "|")
        .map(String.init)
        .filter { !$0.isEmpty && $0 != "0" && $0 != "-1" }
}

private func traitMatches(_ type: String, trait: TraitModel) -> Bool {
    if type == "race" { return trait.traitType == 0 }
    if type == "job" { return trait.traitType == 1 }
    return true
}

private func stringValue(_ val: Any?) -> String {
    if let s = val as? String { return s }
    if let i = val as? Int { return String(i) }
    if let d = val as? Double { return d.truncatingRemainder(dividingBy: 1) == 0 ? String(Int(d)) : String(d) }
    return ""
}
