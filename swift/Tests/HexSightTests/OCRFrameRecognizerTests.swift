import CoreGraphics
import XCTest
@testable import HexSight

/// OCRFrameRecognizerTests OCR 运行时识别测试
/// 核心职责：
/// - 校验运行时 CGImage 识别链路复用 ROI、候选选择和摘要整理
/// - 避免真实 Vision 调用影响单测稳定性
final class OCRFrameRecognizerTests: XCTestCase {
    func testRecognizesSummaryFromRuntimeFrame() throws {
        let vision = StubTextRecognizer(candidatesByRegion: [
            "round": [OCRTextCandidate(text: "1-3", confidence: 1.0)],
            "shop_hero_name_0": [OCRTextCandidate(text: "露露", confidence: 0.6)],
            "shop_trait_name_0": [
                OCRTextCandidate(text: "So", confidence: 0.3),
                OCRTextCandidate(text: "虚空", confidence: 0.5),
            ],
            "opponent_hp_0": [OCRTextCandidate(text: "76", confidence: 0.8)],
        ])
        let normalizer = OCRNormalizer(
            heroes: ["璐璐"],
            traits: ["虚空"],
            augments: []
        )
        let recognizer = OCRFrameRecognizer(vision: vision, normalizer: normalizer)

        let summary = try recognizer.recognizeSummary(cgImage: makeImage())

        XCTAssertEqual(summary.round, "1-3")
        XCTAssertEqual(summary.shop, [OCRShopSlot(index: 0, heroName: "璐璐", traits: ["虚空"])])
        XCTAssertEqual(summary.opponents, [OCROpponentRow(index: 0, name: nil, hp: 76)])
    }

    private func makeImage() -> CGImage {
        let width = 3840
        let height = 2414
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let context = CGContext(
            data: nil,
            width: width,
            height: height,
            bitsPerComponent: 8,
            bytesPerRow: width * 4,
            space: colorSpace,
            bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
        )!
        return context.makeImage()!
    }
}

private final class StubTextRecognizer: OCRTextRecognizing {
    private let candidatesByRegion: [String: [OCRTextCandidate]]

    init(candidatesByRegion: [String: [OCRTextCandidate]]) {
        self.candidatesByRegion = candidatesByRegion
    }

    func recognize(imageURL: URL, regions: [OCRRegion]) throws -> [String: [OCRTextCandidate]] {
        candidates(for: regions)
    }

    func recognize(cgImage: CGImage, regions: [OCRRegion]) throws -> [String: [OCRTextCandidate]] {
        candidates(for: regions)
    }

    private func candidates(for regions: [OCRRegion]) -> [String: [OCRTextCandidate]] {
        Dictionary(uniqueKeysWithValues: regions.map { region in
            (region.id, candidatesByRegion[region.id] ?? [])
        })
    }
}
