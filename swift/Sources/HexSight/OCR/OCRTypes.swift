import CoreGraphics
import Foundation

/// OCRRegionKind OCR 区域类型
/// 核心职责：
/// - 标识截图中需要文字识别的业务区域
/// - 为后续文本归一化选择对应字典
enum OCRRegionKind: String, Codable, Hashable {
    case round
    case shopHeroName
    case shopTraitName
    case opponentName
    case opponentHP
    case activeTraitName
    case activeTraitCount
    case augmentName
}

/// OCRRegionProfile OCR 区域配置档案
/// 核心职责：
/// - 区分不同客户端和窗口布局
/// - 为 ROI 映射选择标准坐标模板
enum OCRRegionProfile: String, Codable {
    case appBaoMac
}

/// OCRRegion OCR 识别区域
/// 核心职责：
/// - 保存截图像素坐标中的裁剪区域
/// - 携带区域类型和稳定标识，便于离线评测与实时识别复用
struct OCRRegion: Codable, Hashable {
    let id: String
    let kind: OCRRegionKind
    let rect: CGRect
}

/// OCRTextCandidate OCR 文本候选
/// 核心职责：
/// - 保存 Vision 返回的候选文本
/// - 保留置信度供多帧稳定和候选排序使用
struct OCRTextCandidate: Codable, Hashable {
    let text: String
    let confidence: Float
}

/// OCRRegionResult OCR 区域识别结果
/// 核心职责：
/// - 聚合单个 ROI 的原始候选和归一化文本
/// - 为离线评测输出稳定 JSON 结构
struct OCRRegionResult: Codable, Hashable {
    let region: OCRRegion
    let candidates: [OCRTextCandidate]
    let normalizedText: String?
}

/// OCRFrameResult OCR 单帧识别结果
/// 核心职责：
/// - 保存单张截图的 OCR 全量输出
/// - 作为离线评测和实时链路接入的中间结构
struct OCRFrameResult: Codable {
    let imagePath: String
    let imageSize: CGSize
    let gameContentRect: CGRect
    let summary: OCRFrameSummary
    let regions: [OCRRegionResult]
}

/// OCRFrameSummary OCR 业务摘要
/// 核心职责：
/// - 将 ROI 级识别结果整理成业务可消费字段
/// - 降低实时识别链路和规则层对 OCR 细节的依赖
struct OCRFrameSummary: Codable, Equatable {
    let round: String?
    let shop: [OCRShopSlot]
    let opponents: [OCROpponentRow]
    let activeTraits: [OCRActiveTraitRow]
    let augments: [OCRAugmentOption]

    init(
        round: String?,
        shop: [OCRShopSlot],
        opponents: [OCROpponentRow],
        activeTraits: [OCRActiveTraitRow],
        augments: [OCRAugmentOption] = []
    ) {
        self.round = round
        self.shop = shop
        self.opponents = opponents
        self.activeTraits = activeTraits
        self.augments = augments
    }

    private enum CodingKeys: String, CodingKey {
        case round
        case shop
        case opponents
        case activeTraits
        case augments
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        round = try container.decodeIfPresent(String.self, forKey: .round)
        shop = try container.decodeIfPresent([OCRShopSlot].self, forKey: .shop) ?? []
        opponents = try container.decodeIfPresent([OCROpponentRow].self, forKey: .opponents) ?? []
        activeTraits = try container.decodeIfPresent([OCRActiveTraitRow].self, forKey: .activeTraits) ?? []
        augments = try container.decodeIfPresent([OCRAugmentOption].self, forKey: .augments) ?? []
    }
}

/// OCRShopSlot 商店卡槽识别结果
/// 核心职责：
/// - 表示单个商店卡槽的英雄名与羁绊文本
/// - 支撑 D 牌、卡池和阵容转向判断
struct OCRShopSlot: Codable, Equatable {
    let index: Int
    let heroName: String?
    let traits: [String]
}

/// OCROpponentRow 对手栏识别结果
/// 核心职责：
/// - 表示右侧玩家列表中的名称和血量
/// - 支撑同行、血量压力和决赛圈判断
struct OCROpponentRow: Codable, Equatable {
    let index: Int
    let name: String?
    let hp: Int?
}

/// OCRActiveTraitRow 当前羁绊识别结果
/// 核心职责：
/// - 表示左侧当前激活/候选羁绊名称与计数
/// - 支撑当前阵容状态兜底识别
struct OCRActiveTraitRow: Codable, Equatable {
    let index: Int
    let name: String?
    let count: String?
}

/// OCRAugmentOption 海克斯选项识别结果
/// 核心职责：
/// - 表示海克斯选择界面中的单个选项名称
/// - 支撑海克斯推荐与 OCR 字段级评测
struct OCRAugmentOption: Codable, Equatable {
    let index: Int
    let name: String?
}

/// OCRFrameAnnotation OCR 样本标注
/// 核心职责：
/// - 保存单张样本截图的人工期望字段
/// - 为字段级准确率统计提供对照答案
struct OCRFrameAnnotation: Codable, Equatable {
    let round: String?
    let shop: [String]
    let traits: [String]
    let opponents: [String]
    let augments: [String]

    private enum CodingKeys: String, CodingKey {
        case round
        case shop
        case traits
        case opponents
        case augments
    }

    init(
        round: String? = nil,
        shop: [String] = [],
        traits: [String] = [],
        opponents: [String] = [],
        augments: [String] = []
    ) {
        self.round = round
        self.shop = shop
        self.traits = traits
        self.opponents = opponents
        self.augments = augments
    }

    /// init(from:) 标注容错解码入口
    /// 核心职责：
    /// - 支持单张样本只标注部分彩段
    /// - 将缺失数组字段归一为空数组
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        round = try container.decodeIfPresent(String.self, forKey: .round)
        shop = try container.decodeIfPresent([String].self, forKey: .shop) ?? []
        traits = try container.decodeIfPresent([String].self, forKey: .traits) ?? []
        opponents = try container.decodeIfPresent([String].self, forKey: .opponents) ?? []
        augments = try container.decodeIfPresent([String].self, forKey: .augments) ?? []
    }
}

/// OCRFieldAccuracy OCR 字段级统计
/// 核心职责：
/// - 汇总单类字段的样本数量、命中数量与准确率
/// - 区分人工标注评测和无标注观测统计
struct OCRFieldAccuracy: Codable, Equatable {
    let field: String
    let total: Int
    let matched: Int
    let missing: Int
    let extra: Int
    let accuracy: Double
    let annotated: Bool
}

/// OCREvaluationReport OCR 评测报告
/// 核心职责：
/// - 聚合逐帧 OCR 输出和字段级统计
/// - 作为 `--ocr-eval` 的稳定 JSON 输出结构
struct OCREvaluationReport: Codable {
    let frames: [OCRFrameResult]
    let fieldAccuracy: [OCRFieldAccuracy]

    static func build(
        frames: [OCRFrameResult],
        annotations: [String: OCRFrameAnnotation] = [:]
    ) -> OCREvaluationReport {
        let fields = ["round", "shop", "trait", "opponent", "augment"]
        let stats = fields.map { field in
            accuracy(for: field, frames: frames, annotations: annotations)
        }
        return OCREvaluationReport(frames: frames, fieldAccuracy: stats)
    }

    private static func accuracy(
        for field: String,
        frames: [OCRFrameResult],
        annotations: [String: OCRFrameAnnotation]
    ) -> OCRFieldAccuracy {
        var expectedTotal = 0
        var matched = 0
        var missing = 0
        var extra = 0
        var observedTotal = 0
        var hasAnnotation = false

        for frame in frames {
            let observed = observedValues(field: field, summary: frame.summary)
            observedTotal += observed.count
            guard let annotation = annotation(for: frame, in: annotations) else {
                continue
            }
            hasAnnotation = true
            let expected = expectedValues(field: field, annotation: annotation)
            expectedTotal += expected.count
            let observedSet = Set(observed.map(normalizedMetricValue))
            let expectedSet = Set(expected.map(normalizedMetricValue))
            matched += expectedSet.intersection(observedSet).count
            missing += expectedSet.subtracting(observedSet).count
            extra += observedSet.subtracting(expectedSet).count
        }

        if hasAnnotation {
            let denominator = max(expectedTotal, 1)
            return OCRFieldAccuracy(
                field: field,
                total: expectedTotal,
                matched: matched,
                missing: missing,
                extra: extra,
                accuracy: Double(matched) / Double(denominator),
                annotated: true
            )
        }

        return OCRFieldAccuracy(
            field: field,
            total: observedTotal,
            matched: observedTotal,
            missing: 0,
            extra: 0,
            accuracy: observedTotal > 0 ? 1.0 : 0.0,
            annotated: false
        )
    }

    private static func annotation(
        for frame: OCRFrameResult,
        in annotations: [String: OCRFrameAnnotation]
    ) -> OCRFrameAnnotation? {
        annotations[frame.imagePath] ?? annotations[(frame.imagePath as NSString).lastPathComponent]
    }

    private static func observedValues(field: String, summary: OCRFrameSummary) -> [String] {
        switch field {
        case "round":
            return summary.round.map { [$0] } ?? []
        case "shop":
            return summary.shop.compactMap(\.heroName)
        case "trait":
            return summary.activeTraits.compactMap(\.name)
        case "opponent":
            return summary.opponents.map { row in
                [row.name, row.hp.map(String.init)].compactMap { $0 }.joined(separator: " ")
            }
            .filter { !$0.isEmpty }
        case "augment":
            return summary.augments.compactMap(\.name)
        default:
            return []
        }
    }

    private static func expectedValues(field: String, annotation: OCRFrameAnnotation) -> [String] {
        switch field {
        case "round":
            return annotation.round.map { [$0] } ?? []
        case "shop":
            return annotation.shop
        case "trait":
            return annotation.traits
        case "opponent":
            return annotation.opponents
        case "augment":
            return annotation.augments
        default:
            return []
        }
    }

    private static func normalizedMetricValue(_ value: String) -> String {
        value
            .replacingOccurrences(of: " ", with: "")
            .replacingOccurrences(of: "\n", with: "")
            .trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
