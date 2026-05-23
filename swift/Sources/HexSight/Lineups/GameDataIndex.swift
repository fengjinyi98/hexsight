import Foundation

/// GameDataIndex 游戏数据纯索引
/// 核心职责：
/// - 从 config/game_data 加载英雄、装备、羁绊、强化符文静态数据
/// - 为规则上下文提供 ID 到可读信息的解析能力
/// - 复用本地快照完成追版本回归校验
struct GameDataIndex {
    let mode: String
    let heroesByID: [String: HeroModel]
    let equipmentByID: [String: EquipmentModel]
    let hexesByID: [String: HexModel]
    let traits: [TraitModel]

    static func load(mode: String) throws -> GameDataIndex {
        let dir = ProjectPaths.gameDataDirectory(mode: mode)
        let heroes: [HeroModel] = try loadArray(url: dir.appendingPathComponent("chess.json"))
        let equipment: [EquipmentModel] = try loadArray(url: dir.appendingPathComponent("equip.json"))
        let hexes: [HexModel] = try loadArray(url: dir.appendingPathComponent("hex.json"))
        let traits: [TraitModel] = try loadArray(url: dir.appendingPathComponent("trait.json"))
        return GameDataIndex(
            mode: mode,
            heroesByID: Dictionary(heroes.map { ($0.id, $0) }) { current, _ in current },
            equipmentByID: Dictionary(equipment.map { ($0.id, $0) }) { current, _ in current },
            hexesByID: Dictionary(hexes.map { ($0.id, $0) }) { current, _ in current },
            traits: traits
        )
    }

    func hero(id: String) -> HeroModel? {
        heroesByID[id]
    }

    func equipment(id: String) -> EquipmentModel? {
        equipmentByID[id]
    }

    func hex(id: String) -> HexModel? {
        hexesByID[id]
    }

    func traitSummaries(for pieces: [LineupPiece], officialContacts: [LineupTraitContact]) -> [RuleTraitSnapshot] {
        if !officialContacts.isEmpty {
            return officialContacts.compactMap { contact in
                guard contact.color > 0 || contact.count > 0 else { return nil }
                guard let trait = traits.first(where: { trait in
                    trait.checkId == contact.id && traitMatches(contact.type, trait: trait)
                }) ?? traits.first(where: { $0.checkId == contact.id }) else { return nil }
                return RuleTraitSnapshot(
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
            guard let hero = hero(id: piece.heroID) else { continue }
            for id in splitTraitIDs(hero.species) { counts["race:\(id)", default: 0] += 1 }
            for id in splitTraitIDs(hero.heroClass) { counts["job:\(id)", default: 0] += 1 }
        }

        let summaries = counts.compactMap { key, count -> RuleTraitSnapshot? in
            let parts = key.split(separator: ":").map(String.init)
            guard parts.count == 2 else { return nil }
            let type = parts[0]
            let checkId = parts[1]
            let candidates = traits.filter { $0.checkId == checkId && traitMatches(type, trait: $0) }
            guard let active = candidates
                .filter({ count >= (Int($0.num) ?? Int.max) })
                .max(by: { $0.level < $1.level })
            else { return nil }
            return RuleTraitSnapshot(
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

    private static func loadArray<T>(url: URL) throws -> [T] {
        let data = try Data(contentsOf: url)
        guard let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let container = json["data"] as? [String: Any]
        else { return [] }

        return container.values.compactMap { value in
            guard let dict = value as? [String: Any] else { return nil }
            if T.self == HeroModel.self { return HeroModel(dict: dict) as? T }
            if T.self == EquipmentModel.self { return EquipmentModel(dict: dict) as? T }
            if T.self == TraitModel.self { return TraitModel(dict: dict) as? T }
            if T.self == HexModel.self { return HexModel(dict: dict) as? T }
            return nil
        }
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
