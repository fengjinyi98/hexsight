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
