import Foundation

/// LineupSourceAdapter 官方阵容数据适配器
/// 核心职责：
/// - 将官方 lineup_detail_total JSON 转换为稳定领域模型
/// - 集中处理 detail 二次 JSON 解析与字段兼容优先级
/// - 隔离追版本时的官方字段变化
struct LineupSourceAdapter {
    static func cards(fromTopLevelData data: Data, mode: String) -> [LineupCard] {
        guard let payload = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return [] }
        return cards(fromTopLevelPayload: payload, mode: mode)
    }

    static func cards(fromTopLevelPayload payload: [String: Any], mode: String) -> [LineupCard] {
        let list = payload["lineup_list"] as? [[String: Any]] ?? []
        return cards(fromList: list, mode: mode)
    }

    static func cards(fromList list: [[String: Any]], mode: String) -> [LineupCard] {
        list.compactMap { card(from: $0, mode: mode) }
            .sorted { lhs, rhs in
                if lhs.category == rhs.category { return lhs.name < rhs.name }
                return (lhs.category ?? "") < (rhs.category ?? "")
            }
    }

    static func card(from dict: [String: Any], mode: String? = nil) -> LineupCard? {
        let detailPayload = parseDetailPayload(dict["detail"])
        let parsedDetail = detail(from: detailPayload)
        let resolvedName = nonEmptyString(detailPayload["line_name"])
            ?? nonEmptyString(dict["name"])
            ?? "未知阵容"
        let rid = lineupString(dict["id"])
        let qid = lineupString(dict["queue_id"])
        let id = [rid, qid, resolvedName].first(where: { !$0.isEmpty }) ?? UUID().uuidString

        let authorData = dict["lineupauthor_data"] as? [String: Any]
        let littleLegend = detailPayload["author_littlelegend"] as? [String: Any]
        let author = nonEmptyString(authorData?["name"])
            ?? nonEmptyString(littleLegend?["desc"])
            ?? nonEmptyString(littleLegend?["item_name"])
            ?? nonEmptyString(dict["author"])
            ?? "未知作者"
        let authorAvatar = nonEmptyString(authorData?["imgUrl"])
            ?? nonEmptyString(littleLegend?["imagePath"])
            ?? nonEmptyString(littleLegend?["icon"])
            ?? ""

        let contacts = detailPayload["contact"] as? [[String: Any]] ?? []
        let contactTraits = contacts.compactMap { nonEmptyString($0["name"]) }
        let traits = contactTraits.isEmpty ? LineupCard.extractBracketTraits(from: resolvedName) : contactTraits
        let lineTag = nonEmptyString(detailPayload["line_tag"]) ?? lineupString(detailPayload["line_tag"])
        let category = LineupCatalog.categoryName(for: lineTag)
        let tags = parseTags(detailPayload: detailPayload, fallbackCategory: category)
        let top4Rate = parseTop4Rate(detailPayload: detailPayload)

        return LineupCard(
            id: id,
            name: resolvedName,
            author: author,
            authorAvatar: authorAvatar,
            quality: nonEmptyString(dict["quality"]) ?? "A",
            traits: traits,
            category: category,
            tags: tags,
            top4Rate: top4Rate,
            rawData: dict,
            detail: parsedDetail
        )
    }

    static func detail(from payload: [String: Any]) -> LineupDetailData {
        let hexBuff = payload["hexbuff"] as? [String: Any] ?? [:]
        return LineupDetailData(
            raw: payload,
            finalHeroes: parsePieces(payload["hero_location"]),
            earlyHeroes: parsePieces(payload["y21_early_heros"]),
            midHeroes: parsePieces(payload["y21_metaphase_heros"]),
            recommendedHexIDs: splitIDs(hexBuff["recomm"]),
            replacementHexIDs: splitIDs(hexBuff["replace"]),
            equipmentOrderIDs: splitIDs(payload["equipment_order"]),
            level3HeroIDs: splitIDs(payload["level_3_heros"]),
            heroReplacements: parseHeroReplacements(payload["hero_replace"]),
            unlockTasks: parseUnlockTasks(payload["task_list"]),
            godRewards: parseGodRewards(payload["god_list"]),
            officialTraits: parseTraitContacts(payload["contact"]),
            earlyTraits: parseTraitContacts(payload["y21_early_heros_contact"]),
            midTraits: parseTraitContacts(payload["y21_metaphase_heros_contact"]),
            chosenContact: LineupTraitContact(dict: payload["chosen_contact"] as? [String: Any] ?? [:]),
            messengerContact: LineupTraitContact(dict: payload["messengerContact"] as? [String: Any] ?? [:]),
            chosenBackups: parseChosenBackups(payload["chosen_backup"]),
            lineFeature: nonEmptyString(payload["line_feature"]) ?? "",
            earlyInfo: nonEmptyString(payload["early_info"]) ?? "",
            dTime: nonEmptyString(payload["d_time"]) ?? "",
            locationInfo: nonEmptyString(payload["location_info"]) ?? "",
            enemyInfo: nonEmptyString(payload["enemy_info"]) ?? "",
            hexInfo: nonEmptyString(payload["hex_info"]) ?? "",
            equipmentInfo: nonEmptyString(payload["equipment_info"]) ?? "",
            godRewardInfo: nonEmptyString(payload["godreward_info"]) ?? "",
            taskInfo: nonEmptyString(payload["task_info"]) ?? "",
            chosenInfo: nonEmptyString(payload["chosen_info"]) ?? "",
            locationInfo2: nonEmptyString(payload["location_info_2"]) ?? "",
            earlyRound: nonEmptyString(payload["early_round"]) ?? "",
            midRound: nonEmptyString(payload["metaphase_round"]) ?? "",
            staffInfo: nonEmptyString(payload["staff_info"]) ?? "",
            goopInfo: nonEmptyString(payload["goop_info"]) ?? "",
            traitPartyInfo: nonEmptyString(payload["traitparty_info"]) ?? "",
            legendGalaxyInfo: nonEmptyString(payload["legendgalaxyinfo"]) ?? ""
        )
    }

    private static func parseDetailPayload(_ value: Any?) -> [String: Any] {
        if let dict = value as? [String: Any] { return dict }
        guard let raw = value as? String,
              let data = raw.data(using: .utf8),
              let payload = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return [:] }
        return payload
    }

    private static func parseTags(detailPayload: [String: Any], fallbackCategory: String?) -> [String] {
        let smarTag = detailPayload["smar_lineup_tag"] as? [String: Any]
        let allTag = smarTag?["all"] as? [String: Any]
        if let tags = allTag?["tag"] as? [String] { return tags }
        if let tag = nonEmptyString(allTag?["tag"]) { return [tag] }
        if let fallbackCategory { return [fallbackCategory] }
        return []
    }

    private static func parseTop4Rate(detailPayload: [String: Any]) -> Double {
        let smarTag = detailPayload["smar_lineup_tag"] as? [String: Any]
        let allTag = smarTag?["all"] as? [String: Any]
        let rate = allTag?["rate"] as? [String: Any]
        return (rate?["top4_rate"] as? Double ?? 0) * 100
    }

    private static func parsePieces(_ value: Any?) -> [LineupPiece] {
        (value as? [[String: Any]] ?? []).compactMap(LineupPiece.init(dict:))
    }

    private static func parseTraitContacts(_ value: Any?) -> [LineupTraitContact] {
        (value as? [[String: Any]] ?? []).compactMap(LineupTraitContact.init(dict:))
    }

    private static func parseHeroReplacements(_ value: Any?) -> [LineupHeroReplacement] {
        (value as? [[String: Any]] ?? []).compactMap(LineupHeroReplacement.init(dict:))
    }

    private static func parseUnlockTasks(_ value: Any?) -> [LineupUnlockTask] {
        (value as? [[String: Any]] ?? []).compactMap(LineupUnlockTask.init(dict:))
    }

    private static func parseGodRewards(_ value: Any?) -> [LineupGodReward] {
        (value as? [[String: Any]] ?? []).compactMap(LineupGodReward.init(dict:))
    }

    private static func parseChosenBackups(_ value: Any?) -> [LineupChosenBackup] {
        (value as? [[String: Any]] ?? []).compactMap(LineupChosenBackup.init(dict:))
    }
}
