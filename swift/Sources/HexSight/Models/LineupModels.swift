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

    init?(dict: [String: Any], rawData: [String: Any]? = nil) {
        let rid = lineupString(dict["id"])
        let qid = lineupString(dict["queue_id"])
        let detailRaw = dict["detail"] as? String ?? "{}"
        let detailPayload = (try? JSONSerialization.jsonObject(with: Data(detailRaw.utf8))) as? [String: Any] ?? [:]
        let parsedDetail = LineupDetailData(payload: detailPayload)
        let resolvedName = nonEmptyString(detailPayload["line_name"])
            ?? nonEmptyString(dict["name"])
            ?? "未知阵容"

        self.name = resolvedName
        self.id = [rid, qid, resolvedName].first(where: { !$0.isEmpty }) ?? UUID().uuidString
        self.rawData = rawData
        self.detail = parsedDetail

        let authorData = dict["lineupauthor_data"] as? [String: Any]
        let littleLegend = detailPayload["author_littlelegend"] as? [String: Any]
        self.author = nonEmptyString(authorData?["name"])
            ?? nonEmptyString(littleLegend?["desc"])
            ?? nonEmptyString(littleLegend?["item_name"])
            ?? nonEmptyString(dict["author"])
            ?? "未知作者"
        self.authorAvatar = nonEmptyString(authorData?["imgUrl"])
            ?? nonEmptyString(littleLegend?["imagePath"])
            ?? nonEmptyString(littleLegend?["icon"])
            ?? ""

        self.quality = nonEmptyString(dict["quality"]) ?? "A"

        let contacts = detailPayload["contact"] as? [[String: Any]] ?? []
        let contactTraits = contacts.compactMap { nonEmptyString($0["name"]) }
        self.traits = contactTraits.isEmpty ? LineupCard.extractBracketTraits(from: resolvedName) : contactTraits

        let lineTag = nonEmptyString(detailPayload["line_tag"]) ?? lineupString(detailPayload["line_tag"])
        self.category = LineupCatalog.categoryName(for: lineTag)

        let smarTag = detailPayload["smar_lineup_tag"] as? [String: Any]
        let allTag = smarTag?["all"] as? [String: Any]
        if let t = allTag?["tag"] as? [String] { self.tags = t }
        else if let t = nonEmptyString(allTag?["tag"]) { self.tags = [t] }
        else if let category { self.tags = [category] }
        else { self.tags = [] }

        let rate = allTag?["rate"] as? [String: Any]
        self.top4Rate = (rate?["top4_rate"] as? Double ?? 0) * 100
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
    let lineFeature: String
    let earlyInfo: String
    let dTime: String
    let locationInfo: String
    let enemyInfo: String
    let hexInfo: String
    let equipmentInfo: String

    init(payload: [String: Any]) {
        self.raw = payload
        self.finalHeroes = LineupDetailData.parsePieces(payload["hero_location"])
        self.earlyHeroes = LineupDetailData.parsePieces(payload["y21_early_heros"])
        self.midHeroes = LineupDetailData.parsePieces(payload["y21_metaphase_heros"])
        let hexBuff = payload["hexbuff"] as? [String: Any] ?? [:]
        self.recommendedHexIDs = splitIDs(hexBuff["recomm"])
        self.replacementHexIDs = splitIDs(hexBuff["replace"])
        self.equipmentOrderIDs = splitIDs(payload["equipment_order"])
        self.level3HeroIDs = splitIDs(payload["level_3_heros"])
        let replacements = payload["hero_replace"] as? [[String: Any]] ?? []
        self.heroReplacements = replacements.compactMap(LineupHeroReplacement.init(dict:))
        self.lineFeature = nonEmptyString(payload["line_feature"]) ?? ""
        self.earlyInfo = nonEmptyString(payload["early_info"]) ?? ""
        self.dTime = nonEmptyString(payload["d_time"]) ?? ""
        self.locationInfo = nonEmptyString(payload["location_info"]) ?? ""
        self.enemyInfo = nonEmptyString(payload["enemy_info"]) ?? ""
        self.hexInfo = nonEmptyString(payload["hex_info"]) ?? ""
        self.equipmentInfo = nonEmptyString(payload["equipment_info"]) ?? ""
    }

    private static func parsePieces(_ value: Any?) -> [LineupPiece] {
        let list = value as? [[String: Any]] ?? []
        return list.compactMap(LineupPiece.init(dict:))
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
