import AppKit
import CoreGraphics
import Foundation
import Vision

/// OCRTextRecognizing OCR 文本识别协议
/// 核心职责：
/// - 抽象 Vision 文本识别入口
/// - 支撑运行时识别和单元测试替身复用同一条归一化链路
protocol OCRTextRecognizing {
    func recognize(imageURL: URL, regions: [OCRRegion]) throws -> [String: [OCRTextCandidate]]
    func recognize(cgImage: CGImage, regions: [OCRRegion]) throws -> [String: [OCRTextCandidate]]
}

/// OCRVisionService Apple Vision OCR 服务
/// 核心职责：
/// - 对截图 ROI 执行本地系统 OCR
/// - 返回带置信度的文本候选，供字典归一化和多帧稳定使用
final class OCRVisionService: OCRTextRecognizing {
    private let recognitionLanguages: [String]
    private let customWords: [String]

    init(recognitionLanguages: [String] = ["zh-Hans", "en-US"], customWords: [String] = []) {
        self.recognitionLanguages = recognitionLanguages
        self.customWords = customWords
    }

    func recognize(imageURL: URL, regions: [OCRRegion]) throws -> [String: [OCRTextCandidate]] {
        guard let image = NSImage(contentsOf: imageURL),
              let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil)
        else {
            throw OCRError.imageLoadFailed(imageURL.path)
        }

        var output: [String: [OCRTextCandidate]] = [:]
        for region in regions {
            output[region.id] = try recognize(cgImage: cgImage, rect: region.rect)
        }
        return output
    }

    func recognize(cgImage: CGImage, regions: [OCRRegion]) throws -> [String: [OCRTextCandidate]] {
        var output: [String: [OCRTextCandidate]] = [:]
        for region in regions {
            output[region.id] = try recognize(cgImage: cgImage, rect: region.rect)
        }
        return output
    }

    func recognize(cgImage: CGImage, rect: CGRect) throws -> [OCRTextCandidate] {
        let cropped = try crop(cgImage: cgImage, rect: rect)
        let request = VNRecognizeTextRequest()
        request.recognitionLevel = .accurate
        request.usesLanguageCorrection = true
        request.recognitionLanguages = recognitionLanguages
        request.customWords = customWords

        let handler = VNImageRequestHandler(cgImage: cropped, options: [:])
        try handler.perform([request])

        let observations = request.results ?? []
        return observations.flatMap { observation in
            observation.topCandidates(3).map { candidate in
                OCRTextCandidate(text: candidate.string, confidence: candidate.confidence)
            }
        }
    }

    private func crop(cgImage: CGImage, rect: CGRect) throws -> CGImage {
        let bounds = CGRect(x: 0, y: 0, width: cgImage.width, height: cgImage.height)
        let clamped = rect.integral.intersection(bounds)
        guard !clamped.isNull,
              clamped.width > 1,
              clamped.height > 1,
              let cropped = cgImage.cropping(to: clamped)
        else {
            throw OCRError.invalidRegion(rect)
        }
        return cropped
    }
}

/// OCRError OCR 错误类型
/// 核心职责：
/// - 表达离线评测和 Vision 识别中的可诊断失败
/// - 为 CLI 输出提供明确错误信息
enum OCRError: LocalizedError {
    case imageLoadFailed(String)
    case invalidRegion(CGRect)
    case noImagesFound(String)

    var errorDescription: String? {
        switch self {
        case .imageLoadFailed(let path):
            return "图片加载失败: \(path)"
        case .invalidRegion(let rect):
            return "OCR 区域无效: \(rect)"
        case .noImagesFound(let path):
            return "目录中没有 PNG/JPG 样本: \(path)"
        }
    }
}
