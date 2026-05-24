import CoreGraphics
import Foundation

/// OCRFrameRecognizer OCR 帧识别器
/// 核心职责：
/// - 将截图帧映射为 OCR ROI 并执行文本识别
/// - 复用归一化与摘要整理逻辑，输出业务可消费结果
final class OCRFrameRecognizer: @unchecked Sendable {
    private let mapper: OCRRegionMapper
    private let vision: OCRTextRecognizing
    private let normalizer: OCRNormalizer
    private let summarizer: OCRFrameSummarizer

    init(
        mapper: OCRRegionMapper = OCRRegionMapper(),
        vision: OCRTextRecognizing,
        normalizer: OCRNormalizer,
        summarizer: OCRFrameSummarizer = OCRFrameSummarizer()
    ) {
        self.mapper = mapper
        self.vision = vision
        self.normalizer = normalizer
        self.summarizer = summarizer
    }

    func recognizeSummary(cgImage: CGImage, profile: OCRRegionProfile = .appBaoMac) throws -> OCRFrameSummary {
        let regions = mapper.regions(cgImage: cgImage, profile: profile)
        let raw = try vision.recognize(cgImage: cgImage, regions: regions)
        let results = regions.map { region in
            let candidates = raw[region.id] ?? []
            return OCRRegionResult(
                region: region,
                candidates: candidates,
                normalizedText: normalizer.bestNormalizedText(from: candidates, kind: region.kind)
            )
        }

        return summarizer.summarize(results)
    }
}
