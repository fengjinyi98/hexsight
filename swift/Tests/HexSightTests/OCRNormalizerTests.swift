import XCTest
@testable import HexSight

/// OCRNormalizerTests OCR 文本标准化测试
/// 核心职责：
/// - 校验 OCR 原文能归一到游戏数据标准名称
/// - 校验常见误识别字符不会污染展示层实体
final class OCRNormalizerTests: XCTestCase {
    func testNormalizesShopHeroNamesAgainstDictionary() {
        let normalizer = OCRNormalizer(
            heroes: ["璐璐", "克格莫", "慎", "艾尼维亚", "嘉文四世"],
            traits: [],
            augments: []
        )

        XCTAssertEqual(normalizer.normalizeHero("露露"), "璐璐")
        XCTAssertEqual(normalizer.normalizeHero("艾尼维亚 1"), "艾尼维亚")
        XCTAssertEqual(normalizer.normalizeHero("嘉文四世5"), "嘉文四世")
    }

    func testNormalizesTraitsAgainstDictionary() {
        let normalizer = OCRNormalizer(
            heroes: [],
            traits: ["虚空", "法师", "狙神", "弗雷尔卓德", "艾欧尼亚"],
            augments: []
        )

        XCTAssertEqual(normalizer.normalizeTrait("弗雷尔卓德 3/5/7"), "弗雷尔卓德")
        XCTAssertEqual(normalizer.normalizeTrait("法帅"), "法师")
        XCTAssertEqual(normalizer.normalizeTrait("狙 神"), "狙神")
        XCTAssertEqual(normalizer.normalizeTrait("欧尼亚"), "艾欧尼亚")
        XCTAssertEqual(normalizer.normalizeTrait("F"), "")
        XCTAssertEqual(normalizer.normalizeTrait("至中"), "")
    }

    func testUnknownTextReturnsCleanedText() {
        let normalizer = OCRNormalizer(heroes: ["璐璐"], traits: ["法师"], augments: [])

        XCTAssertEqual(normalizer.normalizeHero(" ??? 黑暗之女 5 "), "黑暗之女")
    }

    func testKeepsRoundAndNumericRegionText() {
        let normalizer = OCRNormalizer(heroes: [], traits: [], augments: [])

        XCTAssertEqual(normalizer.normalize("1-3", kind: .round), "1-3")
        XCTAssertEqual(normalizer.normalize("100", kind: .opponentHP), "100")
        XCTAssertEqual(normalizer.normalize("2 / 4 / 6", kind: .activeTraitCount), "2/4/6")
    }

    func testShopHeroDictionaryUsesKnownOcrAliases() {
        let normalizer = OCRNormalizer(
            heroes: ["璐璐", "克格莫", "塔里克", "妮蔻", "嘉文四世"],
            traits: [],
            augments: []
        )

        XCTAssertEqual(normalizer.normalizeHero("古格莧"), "克格莫")
        XCTAssertEqual(normalizer.normalizeHero("塔甲克"), "塔里克")
        XCTAssertEqual(normalizer.normalizeHero("送甲古"), "塔里克")
        XCTAssertEqual(normalizer.normalizeHero("后出元"), "塔里克")
        XCTAssertEqual(normalizer.normalizeHero("布降"), "布隆")
        XCTAssertEqual(normalizer.normalizeHero("巾匯"), "布隆")
        XCTAssertEqual(normalizer.normalizeHero("阡分妮"), "萨勒芬妮")
        XCTAssertEqual(normalizer.normalizeHero("叉北"), "安妮")
        XCTAssertEqual(normalizer.normalizeHero("妮"), "妮蔻")
        XCTAssertEqual(normalizer.normalizeHero("安"), "安")
    }

    func testChoosesUsableNormalizedTextFromMultipleCandidates() {
        let normalizer = OCRNormalizer(
            heroes: [],
            traits: ["虚空", "法师"],
            augments: []
        )
        let candidates = [
            OCRTextCandidate(text: "So", confidence: 0.3),
            OCRTextCandidate(text: "虚空", confidence: 0.5),
            OCRTextCandidate(text: "k 法师", confidence: 0.3),
        ]

        XCTAssertEqual(normalizer.bestNormalizedText(from: candidates, kind: .shopTraitName), "虚空")
    }
}
