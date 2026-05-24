import Foundation

/// OCRFrameSummarizer OCR 帧摘要整理器
/// 核心职责：
/// - 将 ROI 识别结果聚合为商店、对手、羁绊等业务字段
/// - 过滤空文本，避免噪声进入实时识别状态
final class OCRFrameSummarizer {
    func summarize(_ regions: [OCRRegionResult]) -> OCRFrameSummary {
        OCRFrameSummary(
            round: text(for: "round", in: regions),
            shop: summarizeShop(regions),
            opponents: summarizeOpponents(regions),
            activeTraits: summarizeActiveTraits(regions),
            augments: summarizeAugments(regions)
        )
    }

    private func summarizeShop(_ regions: [OCRRegionResult]) -> [OCRShopSlot] {
        (0..<5).compactMap { index in
            let heroName = text(for: "shop_hero_name_\(index)", in: regions)
            let trait = text(for: "shop_trait_name_\(index)", in: regions)
            guard heroName != nil else { return nil }
            return OCRShopSlot(index: index, heroName: heroName, traits: trait.map { [$0] } ?? [])
        }
    }

    private func summarizeOpponents(_ regions: [OCRRegionResult]) -> [OCROpponentRow] {
        (0..<8).compactMap { index in
            let name = text(for: "opponent_name_\(index)", in: regions)
            let hpText = text(for: "opponent_hp_\(index)", in: regions)
            let hp = hpText.flatMap(Int.init)
            guard name != nil || hp != nil else { return nil }
            return OCROpponentRow(index: index, name: name, hp: hp)
        }
    }

    private func summarizeActiveTraits(_ regions: [OCRRegionResult]) -> [OCRActiveTraitRow] {
        var rows: [OCRActiveTraitRow] = []
        var pendingCount: String?

        for index in 0..<8 {
            let nameText = text(for: "active_trait_name_\(index)", in: regions)
            let countText = text(for: "active_trait_count_\(index)", in: regions)

            let nameFromName = nameText.flatMap { isTraitCount($0) ? nil : $0 }
            let countFromName = nameText.flatMap { isTraitCount($0) ? $0 : nil }
            let nameFromCount = countText.flatMap { isTraitCount($0) ? nil : $0 }
            let countFromCount = countText.flatMap { isTraitCount($0) ? $0 : nil }

            if let name = nameFromName ?? nameFromCount {
                rows.append(OCRActiveTraitRow(index: index, name: name, count: pendingCount ?? countFromCount))
                pendingCount = countFromName
            } else if let count = countFromName ?? countFromCount {
                if rows.indices.contains(rows.count - 1), rows[rows.count - 1].count == nil {
                    let previous = rows[rows.count - 1]
                    rows[rows.count - 1] = OCRActiveTraitRow(index: previous.index, name: previous.name, count: count)
                    pendingCount = nil
                } else {
                    rows.append(OCRActiveTraitRow(index: index, name: nil, count: count))
                    pendingCount = nil
                }
            }
        }

        return rows
    }

    private func summarizeAugments(_ regions: [OCRRegionResult]) -> [OCRAugmentOption] {
        (0..<3).compactMap { index in
            let name = text(for: "augment_name_\(index)", in: regions)
            guard name != nil else { return nil }
            return OCRAugmentOption(index: index, name: name)
        }
    }

    private func text(for id: String, in regions: [OCRRegionResult]) -> String? {
        guard let value = regions.first(where: { $0.region.id == id })?.normalizedText?
            .trimmingCharacters(in: .whitespacesAndNewlines),
              !value.isEmpty
        else { return nil }
        return value
    }

    private func isTraitCount(_ value: String) -> Bool {
        value.range(of: #"^\d+(?:/\d+)*$"#, options: .regularExpression) != nil
    }
}
