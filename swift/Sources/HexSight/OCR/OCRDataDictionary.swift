import Foundation

/// OCRDataDictionary OCR 纠错字典
/// 核心职责：
/// - 从游戏数据配置中加载英雄、羁绊、海克斯标准名称
/// - 为 OCRNormalizer 提供跨模式候选词表
struct OCRDataDictionary {
    let heroes: [String]
    let traits: [String]
    let augments: [String]

    static func load(configRoot: URL = ProjectPaths.configDirectory()) -> OCRDataDictionary {
        var heroes = Set<String>()
        var traits = Set<String>()
        var augments = Set<String>()

        let gameDataRoot = configRoot.appendingPathComponent("game_data", isDirectory: true)
        let modeDirs = (try? FileManager.default.contentsOfDirectory(
            at: gameDataRoot,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles]
        )) ?? []

        for modeDir in modeDirs {
            loadNames(from: modeDir.appendingPathComponent("chess.json")).forEach { heroes.insert($0) }
            loadNames(from: modeDir.appendingPathComponent("race.json")).forEach { traits.insert($0) }
            loadNames(from: modeDir.appendingPathComponent("job.json")).forEach { traits.insert($0) }
            loadNames(from: modeDir.appendingPathComponent("trait.json")).forEach { traits.insert($0) }
            loadNames(from: modeDir.appendingPathComponent("hex.json")).forEach { augments.insert($0) }
        }

        return OCRDataDictionary(
            heroes: heroes.filter { isUsefulName($0) }.sorted(),
            traits: traits.filter { isUsefulName($0) }.sorted(),
            augments: augments.filter { isUsefulName($0) }.sorted()
        )
    }

    func normalizer() -> OCRNormalizer {
        OCRNormalizer(heroes: heroes, traits: traits, augments: augments)
    }

    private static func loadNames(from url: URL) -> [String] {
        guard let data = try? Data(contentsOf: url),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let container = json["data"]
        else { return [] }

        if let dict = container as? [String: Any] {
            return dict.values.compactMap { value in
                guard let item = value as? [String: Any] else { return nil }
                return item["name"] as? String
            }
        }

        if let array = container as? [[String: Any]] {
            return array.compactMap { $0["name"] as? String }
        }

        return []
    }

    private static func isUsefulName(_ value: String) -> Bool {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        return !trimmed.isEmpty && trimmed != "0" && !trimmed.contains("假人")
    }
}
