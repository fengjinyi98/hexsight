import AppKit
import Foundation

/// OCREvaluator OCR 离线评测器
/// 核心职责：
/// - 批量读取真实金铲铲截图样本
/// - 运行 ROI 映射、Vision OCR、字典归一化并输出结构化结果
final class OCREvaluator {
    private let mapper: OCRRegionMapper
    private let vision: OCRVisionService
    private let normalizer: OCRNormalizer

    init(
        mapper: OCRRegionMapper = OCRRegionMapper(),
        vision: OCRVisionService,
        normalizer: OCRNormalizer
    ) {
        self.mapper = mapper
        self.vision = vision
        self.normalizer = normalizer
    }

    func evaluateDirectory(_ directory: URL, profile: OCRRegionProfile = .appBaoMac) throws -> [OCRFrameResult] {
        let images = try Self.sampleImageFiles(in: directory)
        guard !images.isEmpty else {
            throw OCRError.noImagesFound(directory.path)
        }
        return try images.map { try evaluateImage($0, profile: profile) }
    }

    func evaluateImage(_ imageURL: URL, profile: OCRRegionProfile = .appBaoMac) throws -> OCRFrameResult {
        guard let image = NSImage(contentsOf: imageURL) else {
            throw OCRError.imageLoadFailed(imageURL.path)
        }
        guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
            throw OCRError.imageLoadFailed(imageURL.path)
        }

        let imageSize = image.pixelSize
        let regions = mapper.regions(cgImage: cgImage, profile: profile)
        let raw = try vision.recognize(imageURL: imageURL, regions: regions)
        let results = regions.map { region in
            let candidates = raw[region.id] ?? []
            let normalized = normalizer.bestNormalizedText(from: candidates, kind: region.kind)
            return OCRRegionResult(region: region, candidates: candidates, normalizedText: normalized)
        }

        return OCRFrameResult(
            imagePath: imageURL.path,
            imageSize: imageSize,
            gameContentRect: mapper.gameContentRect(cgImage: cgImage, profile: profile),
            summary: OCRFrameSummarizer().summarize(results),
            regions: results
        )
    }

    static func sampleImageFiles(in directory: URL) throws -> [URL] {
        let urls = try FileManager.default.contentsOfDirectory(
            at: directory,
            includingPropertiesForKeys: nil,
            options: [.skipsHiddenFiles]
        )
        return urls
            .filter { $0.pathExtension.lowercased() == "png" }
            .filter { !Self.isDebugArtifact($0.deletingPathExtension().lastPathComponent) }
            .sorted { $0.lastPathComponent < $1.lastPathComponent }
    }

    private static func isDebugArtifact(_ basename: String) -> Bool {
        let debugSuffixes = [
            "_bottom",
            "_opponent_roi",
            "_roi_sheet",
            "_trait_roi",
        ]
        return debugSuffixes.contains { basename.hasSuffix($0) }
    }
}

private extension NSImage {
    var pixelSize: CGSize {
        guard let representation = representations.first else {
            return size
        }
        return CGSize(width: representation.pixelsWide, height: representation.pixelsHigh)
    }
}
