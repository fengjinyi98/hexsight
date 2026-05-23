import XCTest
@testable import HexSight

/// LineupAdapterSnapshotTests 阵容适配层快照测试
/// 核心职责：
/// - 锁定官方阵容 JSON 到稳定领域模型的转换边界
/// - 覆盖不同模式的关键玩法字段
/// - 防止 UI 直接承担字段兼容逻辑
final class LineupAdapterSnapshotTests: XCTestCase {
    func testAdapterParsesMode17LineupAndRulesContext() throws {
        let payload = makeTopLevelPayload(detail: """
        {
          "line_name":"【神谕龙王】3牧羊人3霸天机甲3神谕",
          "line_tag":"2",
          "hero_location":[
            {"idInLineup":15,"chess_type":"hero","equipment_id":"2028,2034,2007","hero_id":"14384","is_carry_hero":false,"location":"1,4"},
            {"idInLineup":20,"chess_type":"hero","equipment_id":"2038,2016,2046","hero_id":"14382","is_carry_hero":true,"location":"4,1"}
          ],
          "contact":[{"color":3,"id":"409","level":2,"type":"race","num":3}],
          "hexbuff":{"recomm":"1895,30621,20586","replace":"3128,20723"},
          "equipment_order":"1003,1009,1007,1002,1004",
          "god_list":[{"stage_num":2,"god_id":4,"wishes":[1704022,1704020]}],
          "godreward_info":"优先凯尔补装备"
        }
        """)

        let cards = LineupSourceAdapter.cards(fromTopLevelPayload: payload, mode: "17")
        let card = try XCTUnwrap(cards.first)
        let context = LineupRulesContext(card: card, mode: "17")

        XCTAssertEqual(card.name, "【神谕龙王】3牧羊人3霸天机甲3神谕")
        XCTAssertEqual(card.author, "掌盟阵容推荐小助手")
        XCTAssertEqual(card.category, "高手进阶")
        XCTAssertEqual(context.mode.id, "17")
        XCTAssertTrue(context.mode.capabilities.contains(.godRewards))
        XCTAssertEqual(context.finalHeroIDs, ["14384", "14382"])
        XCTAssertEqual(context.carryHeroIDs, ["14382"])
        XCTAssertEqual(context.finalHeroEquipment["14382"], ["2038", "2016", "2046"])
        XCTAssertEqual(context.boardPositions["14384"], "1,4")
        XCTAssertEqual(context.recommendedHexIDs, ["1895", "30621", "20586"])
        XCTAssertEqual(context.equipmentOrderIDs, ["1003", "1009", "1007", "1002", "1004"])
        XCTAssertEqual(context.traitContacts.first?.id, "409")
        XCTAssertEqual(context.godRewards.first?.wishIDs, ["1704022", "1704020"])
    }

    func testAdapterParsesMode16UnlockTasks() throws {
        let payload = makeTopLevelPayload(detail: """
        {
          "line_name":"【约德尔人吉格斯】8约德尔人2护卫2主宰2法师",
          "task_list":[
            {"task_id":1242001,"chess_id":"2420"},
            {"task_id":1527101,"chess_id":"5271"}
          ],
          "task_info":"优先完成吉格斯任务"
        }
        """)

        let card = try XCTUnwrap(LineupSourceAdapter.cards(fromTopLevelPayload: payload, mode: "16").first)
        let context = LineupRulesContext(card: card, mode: "16")

        XCTAssertTrue(context.mode.capabilities.contains(.unlockTasks))
        XCTAssertEqual(context.unlockTasks.map(\.taskID), ["1242001", "1527101"])
        XCTAssertEqual(context.unlockTasks.map(\.heroID), ["12420", "15271"])
        XCTAssertEqual(context.modeSpecificTexts["taskInfo"], "优先完成吉格斯任务")
    }

    func testAdapterParsesMode4ChosenMechanics() throws {
        let payload = makeTopLevelPayload(detail: """
        {
          "line_name":"【新年第一把-武财神德莱文】3战神3三国猛将",
          "chosen_contact":{"id":"91","type":"job"},
          "messengerContact":{"id":"102","type":"race"},
          "chosen_backup":[{"hero_$key_id":"15091","id":"90","type":"race"}],
          "contact":[
            {"color":2,"id":"94","level":1,"type":"job","num":3},
            {"color":1,"id":"90","level":1,"type":"race","num":3}
          ],
          "chosen_info":"优先找战神天选",
          "legendgalaxyinfo":"适配经济奇遇"
        }
        """)

        let card = try XCTUnwrap(LineupSourceAdapter.cards(fromTopLevelPayload: payload, mode: "4").first)
        let context = LineupRulesContext(card: card, mode: "4")

        XCTAssertTrue(context.mode.capabilities.contains(.chosenMechanics))
        XCTAssertEqual(context.chosenContact?.id, "91")
        XCTAssertEqual(context.messengerContact?.id, "102")
        XCTAssertEqual(context.chosenBackups.first?.heroID, "15091")
        XCTAssertEqual(context.traitContacts.map(\.id), ["94", "90"])
        XCTAssertEqual(context.modeSpecificTexts["chosenInfo"], "优先找战神天选")
        XCTAssertEqual(context.modeSpecificTexts["legendGalaxyInfo"], "适配经济奇遇")
    }

    private func makeTopLevelPayload(detail: String) -> [String: Any] {
        [
            "lineup_list": [
                [
                    "id": "4514",
                    "quality": "S",
                    "lineupauthor_data": [
                        "name": "掌盟阵容推荐小助手",
                        "imgUrl": "https://game.gtimg.cn/images/lol/act/jkzlk/author/eb.png",
                    ],
                    "detail": detail,
                ]
            ]
        ]
    }
}
