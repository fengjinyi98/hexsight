import Foundation

/// OCRNormalizer OCR 文本标准化器
/// 核心职责：
/// - 清理 OCR 原始文本中的噪声字符
/// - 将英雄、羁绊、海克斯文本归一到游戏数据标准名称
final class OCRNormalizer {
    private let heroes: [String]
    private let traits: [String]
    private let augments: [String]
    private let aliases: [String: String]

    init(heroes: [String], traits: [String], augments: [String], aliases: [String: String] = [:]) {
        self.heroes = heroes
        self.traits = traits
        self.augments = augments
        self.aliases = [
            "露露": "璐璐",
            "法帅": "法师",
            "古格莧": "克格莫",
            "塔甲克": "塔里克",
            "送甲古": "塔里克",
            "后出元": "塔里克",
            "布降": "布隆",
            "巾匯": "布隆",
            "阡分妮": "萨勒芬妮",
            "叉北": "安妮",
            "妮": "妮蔻",
        ].merging(aliases) { current, _ in current }
    }

    /// 标准化英雄名称
    func normalizeHero(_ text: String) -> String {
        normalize(text, dictionary: heroes, keepsUnknown: true)
    }

    /// 标准化羁绊名称
    func normalizeTrait(_ text: String) -> String {
        normalize(text, dictionary: traits, keepsUnknown: false)
    }

    /// 标准化海克斯名称
    func normalizeAugment(_ text: String) -> String {
        normalize(text, dictionary: augments, keepsUnknown: false)
    }

    /// 标准化任意 OCR 区域文本
    func normalize(_ text: String, kind: OCRRegionKind) -> String {
        switch kind {
        case .shopHeroName:
            normalizeHero(text)
        case .shopTraitName, .activeTraitName:
            normalizeTrait(text)
        case .round:
            cleanRound(text)
        case .opponentHP:
            cleanDigits(text)
        case .activeTraitCount:
            cleanTraitCount(text)
        case .opponentName:
            cleanName(text)
        }
    }

    /// 从多个 OCR 候选中选择最可信的标准化文本
    func bestNormalizedText(from candidates: [OCRTextCandidate], kind: OCRRegionKind) -> String? {
        candidates
            .map { candidate in
                (text: normalize(candidate.text, kind: kind), confidence: candidate.confidence)
            }
            .filter { !$0.text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
            .sorted { lhs, rhs in
                if lhs.confidence == rhs.confidence {
                    return lhs.text.count > rhs.text.count
                }
                return lhs.confidence > rhs.confidence
            }
            .first?
            .text
    }

    private func normalize(_ text: String, dictionary: [String], keepsUnknown: Bool) -> String {
        let cleaned = cleanName(text)
        if let alias = aliases[cleaned] {
            return alias
        }
        if dictionary.contains(cleaned) {
            return cleaned
        }
        if let alias = aliases.first(where: { cleaned.contains($0.key) }) {
            return alias.value
        }
        if let contained = dictionary.first(where: { cleaned.contains($0) || $0.contains(cleaned) }) {
            return contained
        }
        return nearest(cleaned, in: dictionary) ?? (keepsUnknown ? cleaned : "")
    }

    private func cleanName(_ text: String) -> String {
        text
            .replacingOccurrences(of: " ", with: "")
            .replacingOccurrences(of: "\n", with: "")
            .replacingOccurrences(of: "\t", with: "")
            .replacingOccurrences(of: #"[\?\!！,，。:：;；\[\]\(\)（）/\\\d]+"#, with: "", options: .regularExpression)
            .trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func cleanRound(_ text: String) -> String {
        let compact = text.replacingOccurrences(of: " ", with: "")
        if let range = compact.range(of: #"\d+-\d+"#, options: .regularExpression) {
            return String(compact[range])
        }
        return compact
            .replacingOccurrences(of: #"[^0-9\-]"#, with: "", options: .regularExpression)
            .trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func cleanDigits(_ text: String) -> String {
        text.replacingOccurrences(of: #"[^0-9]"#, with: "", options: .regularExpression)
    }

    private func cleanTraitCount(_ text: String) -> String {
        text
            .replacingOccurrences(of: " ", with: "")
            .replacingOccurrences(of: #"[^0-9/]"#, with: "", options: .regularExpression)
    }

    private func nearest(_ value: String, in dictionary: [String]) -> String? {
        guard !value.isEmpty else { return nil }
        let scored = dictionary
            .filter { !$0.isEmpty }
            .map { candidate in
                (candidate, levenshtein(value, candidate))
            }
            .sorted { lhs, rhs in
                if lhs.1 == rhs.1 {
                    return lhs.0.count < rhs.0.count
                }
                return lhs.1 < rhs.1
            }
        guard let best = scored.first else { return nil }
        let maxAllowed = value.count <= 3 ? 1 : 2
        return best.1 <= maxAllowed ? best.0 : nil
    }

    private func levenshtein(_ lhs: String, _ rhs: String) -> Int {
        let a = Array(lhs)
        let b = Array(rhs)
        if a.isEmpty { return b.count }
        if b.isEmpty { return a.count }

        var previous = Array(0...b.count)
        var current = Array(repeating: 0, count: b.count + 1)

        for i in 1...a.count {
            current[0] = i
            for j in 1...b.count {
                let substitution = previous[j - 1] + (a[i - 1] == b[j - 1] ? 0 : 1)
                let insertion = current[j - 1] + 1
                let deletion = previous[j] + 1
                current[j] = min(substitution, insertion, deletion)
            }
            previous = current
        }

        return previous[b.count]
    }
}
