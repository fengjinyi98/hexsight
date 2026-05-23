import Foundation

/// 英雄/棋子数据模型
struct HeroModel: Identifiable {
    let id: String
    let name: String
    let price: String
    let picture: String
    let skillName: String
    let skillDesc: String
    let skillIcon: String
    let skillBriefValue: String
    let skillValueDesc: String
    let species: String
    let heroClass: String
    let initHP: String
    let initAttackDamage: String
    let attackSpeed: String
    let armor: String
    let magicResist: String
    let attackRange: String
    let initMP: String
    let maxMP: String
    let criticalStrikeChance: String

    var cost: Int { Int(price) ?? 0 }
    var hp: Int { Int(initHP) ?? 0 }
    var ad: Int { Int(initAttackDamage) ?? 0 }
    var starLevel: Int {
        guard let first = id.first, let lv = Int(String(first)) else { return 1 }
        return lv
    }
    var sortKey: (Int, Int, Int, String) { (cost, hp, ad, id) }

    /// hero_id 去星级的 base key（用于合并同名+同费英雄）
    var baseKey: String { "\(name)_\(price)" }

    init?(dict: [String: Any]) {
        guard let id = dict["id"] as? String ?? (dict["id"] as? Int).map(String.init),
              let name = dict["name"] as? String else { return nil }
        self.id = id
        self.name = name
        self.price = (dict["price"] as? String) ?? (dict["price"] as? Int).map(String.init) ?? "0"
        self.picture = dict["picture"] as? String ?? ""
        self.skillName = dict["skillName"] as? String ?? ""
        self.skillDesc = dict["skillDesc"] as? String ?? ""
        self.skillIcon = dict["skillIcon"] as? String ?? ""
        self.skillBriefValue = dict["skillBriefValue"] as? String ?? ""
        self.skillValueDesc = dict["skillValueDesc"] as? String ?? ""
        self.species = stringValue(dict["species"])
        self.heroClass = stringValue(dict["class"])
        self.initHP = stringValue(dict["initHP"])
        self.initAttackDamage = stringValue(dict["initAttackDamage"])
        self.attackSpeed = stringValue(dict["attackSpeed"])
        self.armor = stringValue(dict["armor"])
        self.magicResist = stringValue(dict["magicResist"])
        self.attackRange = stringValue(dict["attackRange"])
        self.initMP = stringValue(dict["initMP"])
        self.maxMP = stringValue(dict["maxMP"])
        self.criticalStrikeChance = stringValue(dict["criticalStrikeChance"])
    }
}

/// HeroCatalog 英雄图鉴整理器
/// 核心职责：
/// - 过滤训练假人等无效条目
/// - 合并同名英雄的多星级记录
/// - 提供 1/2/3/4 星属性列表
enum HeroCatalog {

    /// 图鉴展示用英雄列表（去重）
    static func displayHeroes(from heroes: [HeroModel]) -> [HeroModel] {
        let filtered = heroes.filter { $0.cost > 0 && !$0.name.contains("假人") }
        let grouped = Dictionary(grouping: filtered) { $0.baseKey }
        return grouped.values.compactMap { $0.min { $0.sortKey < $1.sortKey } }
            .sorted { ($0.cost, $0.name) < ($1.cost, $1.name) }
    }

    /// 获取某英雄的所有星级数据
    static func starVariants(for hero: HeroModel, in heroes: [HeroModel]) -> [HeroModel] {
        heroes
            .filter { $0.baseKey == hero.baseKey }
            .sorted { $0.starLevel < $1.starLevel }
    }
}

/// 装备数据模型
struct EquipmentModel: Identifiable {
    let id: String
    let name: String
    let type: String
    let picture: String
    let basicDesc: String
    let desc: String
    let synthesis1: String
    let synthesis2: String
    let icon: String

    var isComponent: Bool { type == "基础装备" }
    var isCompleted: Bool { type == "成型装备" }

    init?(dict: [String: Any]) {
        guard let id = dict["id"] as? String ?? (dict["id"] as? Int).map(String.init),
              let name = dict["name"] as? String else { return nil }
        self.id = id
        self.name = name
        self.type = dict["type"] as? String ?? ""
        self.picture = dict["picture"] as? String ?? ""
        self.basicDesc = dict["basicDesc"] as? String ?? ""
        self.desc = dict["desc"] as? String ?? ""
        self.synthesis1 = stringValue(dict["synthesis1"])
        self.synthesis2 = stringValue(dict["synthesis2"])
        self.icon = stringValue(dict["icon"])
    }
}

/// 羁绊数据模型
struct TraitModel: Identifiable {
    let id: String
    let name: String
    let level: Int
    let num: String
    let numList: String
    let desc: String
    let realDesc: String
    let picture: String
    let color: String
    let traitType: Int
    let checkId: String
    let values: String

    var thresholds: [Int] {
        numList.split(separator: "|").compactMap { Int($0) }
    }

    init?(dict: [String: Any]) {
        guard let id = dict["id"] as? String ?? (dict["id"] as? Int).map(String.init),
              let name = dict["name"] as? String else { return nil }
        self.id = id
        self.name = name
        self.level = dict["level"] as? Int ?? 1
        self.num = stringValue(dict["num"])
        self.numList = dict["numList"] as? String ?? ""
        self.desc = (dict["desc"] as? String) ?? (dict["prefix"] as? String) ?? ""
        self.realDesc = dict["realDesc"] as? String ?? ""
        self.picture = dict["picture"] as? String ?? ""
        self.color = stringValue(dict["color"])
        self.traitType = dict["type"] as? Int ?? 0
        self.checkId = stringValue(dict["checkId"])
        self.values = dict["values"] as? String ?? ""
    }
}

/// 强化符文数据模型
struct HexModel: Identifiable {
    let id: String
    let name: String
    let level: Int
    let desc: String
    let icon: String
    let isLegend: Bool
    let heroEnhancementType: String
    let fetterId: String

    init?(dict: [String: Any]) {
        guard let id = dict["id"] as? String ?? (dict["id"] as? Int).map(String.init),
              let name = dict["name"] as? String else { return nil }
        self.id = id
        self.name = name
        self.level = (dict["level"] as? Int) ?? Int(dict["level"] as? String ?? "1") ?? 1
        self.desc = dict["desc"] as? String ?? ""
        self.icon = dict["icon"] as? String ?? ""
        self.isLegend = (dict["is_legend"] as? Int) == 1
        self.heroEnhancementType = stringValue(dict["hero_enhancement_type"])
        self.fetterId = dict["fetterId"] as? String ?? ""
    }
}

/// 类型转换辅助
private func stringValue(_ val: Any?) -> String {
    if let s = val as? String { return s }
    if let i = val as? Int { return String(i) }
    if let d = val as? Double { return String(d) }
    return ""
}
