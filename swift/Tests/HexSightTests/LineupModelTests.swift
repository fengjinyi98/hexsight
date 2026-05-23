import XCTest
@testable import HexSight

/// LineupModelTests 阵容官方数据解析测试
/// 核心职责：
/// - 校验官方 detail 字符串能解析出列表与详情所需字段
/// - 校验棋子站位、装备、符文与作者信息不会丢失
final class LineupModelTests: XCTestCase {
    func testOfficialLineupPayloadParsesPreviewAndDetailFields() throws {
        let payload: [String: Any] = [
            "id": "4514",
            "quality": "S",
            "lineupauthor_data": [
                "name": "掌盟阵容推荐小助手",
                "imgUrl": "https://game.gtimg.cn/images/lol/act/jkzlk/author/eb.png",
            ],
            "detail": """
            {
              "line_name":"【神谕龙王】3牧羊人3霸天机甲3神谕",
              "line_tag":"2",
              "hero_location":[
                {"idInLineup":15,"chess_type":"hero","equipment_id":"2028,2034,2007","hero_id":"14384","is_carry_hero":false,"location":"1,4"},
                {"idInLineup":20,"chess_type":"hero","equipment_id":"2038,2016,2046","hero_id":"14382","is_carry_hero":true,"location":"4,1"}
              ],
              "hexbuff":{"recomm":"1895,30621,20586","replace":"3128,20723"},
              "equipment_order":"1003,1009,1007,1002,1004",
              "location_info":"超级机甲单顶",
              "equipment_info":"龙王三输出",
              "hex_info":"建议1经济1装备1战力",
              "early_info":"重装牧羊人体系过渡"
            }
            """,
        ]

        let card = try XCTUnwrap(LineupCard(dict: payload, rawData: payload))

        XCTAssertEqual(card.name, "【神谕龙王】3牧羊人3霸天机甲3神谕")
        XCTAssertEqual(card.author, "掌盟阵容推荐小助手")
        XCTAssertEqual(card.authorAvatar, "https://game.gtimg.cn/images/lol/act/jkzlk/author/eb.png")
        XCTAssertEqual(card.quality, "S")
        XCTAssertEqual(card.category, "高手进阶")
        XCTAssertEqual(card.traits, ["神谕龙王"])
        XCTAssertEqual(card.detail.finalHeroes.count, 2)
        XCTAssertEqual(card.detail.finalHeroes.first?.heroID, "14384")
        XCTAssertEqual(card.detail.finalHeroes.first?.equipmentIDs, ["2028", "2034", "2007"])
        XCTAssertEqual(card.detail.finalHeroes.last?.isCarryHero, true)
        XCTAssertEqual(card.detail.recommendedHexIDs, ["1895", "30621", "20586"])
        XCTAssertEqual(card.detail.equipmentOrderIDs, ["1003", "1009", "1007", "1002", "1004"])
        XCTAssertEqual(card.detail.locationInfo, "超级机甲单顶")
    }
}

extension LineupModelTests {
    func testMode16DetailParsesUnlockTasks() throws {
        let payload: [String: Any] = [
            "id": "3941",
            "quality": "S",
            "detail": """
            {
              "line_name":"【约德尔人吉格斯】8约德尔人2护卫2主宰2法师",
              "task_list":[
                {"task_id":1242001,"chess_id":"2420"},
                {"task_id":1527101,"chess_id":"5271"}
              ]
            }
            """,
        ]

        let card = try XCTUnwrap(LineupCard(dict: payload, rawData: payload))

        XCTAssertEqual(card.detail.unlockTasks.count, 2)
        XCTAssertEqual(card.detail.unlockTasks.first?.taskID, "1242001")
        XCTAssertEqual(card.detail.unlockTasks.first?.heroID, "12420")
        XCTAssertEqual(card.detail.unlockTasks.last?.heroID, "15271")
    }

    func testMode17DetailParsesGodRewards() throws {
        let payload: [String: Any] = [
            "id": "4514",
            "quality": "S",
            "detail": """
            {
              "line_name":"【神谕龙王】3牧羊人3霸天机甲3神谕",
              "god_list":[
                {"stage_num":2,"god_id":4,"wishes":[1704022,1704020]},
                {"stage_num":3,"god_id":5,"wishes":[1705031,1705041]}
              ],
              "godreward_info":"优先凯尔补装备"
            }
            """,
        ]

        let card = try XCTUnwrap(LineupCard(dict: payload, rawData: payload))

        XCTAssertEqual(card.detail.godRewards.count, 2)
        XCTAssertEqual(card.detail.godRewards.first?.stage, 2)
        XCTAssertEqual(card.detail.godRewards.first?.godID, "4")
        XCTAssertEqual(card.detail.godRewards.first?.wishIDs, ["1704022", "1704020"])
        XCTAssertEqual(card.detail.godRewardInfo, "优先凯尔补装备")
    }

    func testMode4DetailParsesChosenAndOfficialContacts() throws {
        let payload: [String: Any] = [
            "id": "4242",
            "quality": "S",
            "detail": """
            {
              "line_name":"【新年第一把-武财神德莱文】3战神3三国猛将",
              "chosen_contact":{"id":"91","type":"job"},
              "messengerContact":{"id":"102","type":"race"},
              "chosen_backup":[{"hero_$key_id":"15091","id":"90","type":"race"}],
              "contact":[
                {"color":2,"id":"94","level":1,"type":"job","num":3},
                {"color":1,"id":"90","level":1,"type":"race","num":3}
              ],
              "early_round":"2-3",
              "metaphase_round":"4-3"
            }
            """,
        ]

        let card = try XCTUnwrap(LineupCard(dict: payload, rawData: payload))

        XCTAssertEqual(card.detail.officialTraits.count, 2)
        XCTAssertEqual(card.detail.officialTraits.first?.id, "94")
        XCTAssertEqual(card.detail.chosenContact?.id, "91")
        XCTAssertEqual(card.detail.messengerContact?.id, "102")
        XCTAssertEqual(card.detail.chosenBackups.first?.heroID, "15091")
        XCTAssertEqual(card.detail.earlyRound, "2-3")
        XCTAssertEqual(card.detail.midRound, "4-3")
    }
}
