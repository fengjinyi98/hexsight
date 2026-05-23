import XCTest
@testable import HexSight

/// HeroCatalogTests 英雄图鉴去重测试
/// 核心职责：
/// - 验证图鉴会折叠同名英雄的星级重复项
/// - 验证图鉴会过滤训练假人
final class HeroCatalogTests: XCTestCase {
    func testDisplayHeroesDeduplicatesByNameAndKeepsCanonicalStats() {
        let heroes = [
            HeroModel(dict: [
                "id": "21451",
                "name": "伊泽瑞尔",
                "price": "1",
                "picture": "",
                "skillName": "",
                "skillDesc": "",
                "skillIcon": "",
                "species": "410",
                "class": "318",
                "initHP": "810",
                "initAttackDamage": "60",
                "attackSpeed": "0.75",
                "armor": "20",
                "magicResist": "20",
                "attackRange": "4",
            ])!,
            HeroModel(dict: [
                "id": "11451",
                "name": "伊泽瑞尔",
                "price": "1",
                "picture": "",
                "skillName": "",
                "skillDesc": "",
                "skillIcon": "",
                "species": "410",
                "class": "318",
                "initHP": "450",
                "initAttackDamage": "40",
                "attackSpeed": "0.75",
                "armor": "20",
                "magicResist": "20",
                "attackRange": "4",
            ])!,
            HeroModel(dict: [
                "id": "0",
                "name": "木桩假人",
                "price": "0",
                "picture": "",
                "skillName": "",
                "skillDesc": "",
                "skillIcon": "",
                "species": "-1",
                "class": "-1",
                "initHP": "550",
                "initAttackDamage": "35",
                "attackSpeed": "0.40",
                "armor": "0",
                "magicResist": "0",
                "attackRange": "1",
            ])!,
        ]

        let result = HeroCatalog.displayHeroes(from: heroes)

        XCTAssertEqual(result.count, 1)
        XCTAssertEqual(result.first?.name, "伊泽瑞尔")
        XCTAssertEqual(result.first?.hp, 450)
        XCTAssertEqual(result.first?.ad, 40)
    }
}
