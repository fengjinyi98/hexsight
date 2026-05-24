import AppKit
import Foundation

/// OCRCliRunner OCR 命令行评测入口
/// 核心职责：
/// - 在传入 --ocr-eval 时运行离线 OCR 样本评测
/// - 保持常规 App 启动流程独立
enum OCRCliRunner {
    static func runIfNeeded(arguments: [String]) -> Bool {
        guard let index = arguments.firstIndex(of: "--ocr-eval") else { return false }
        let samplePath = arguments.dropFirst(index + 1).first ?? "docs/ocr_samples"
        let sampleURL = URL(fileURLWithPath: samplePath, relativeTo: URL(fileURLWithPath: FileManager.default.currentDirectoryPath))
            .standardizedFileURL

        do {
            let dictionary = OCRDataDictionary.load()
            let customWords = Array(Set(dictionary.heroes + dictionary.traits + dictionary.augments)).sorted()
            let evaluator = OCREvaluator(
                vision: OCRVisionService(customWords: customWords),
                normalizer: dictionary.normalizer()
            )
            let results = try evaluator.evaluateDirectory(sampleURL)
            let annotations = try OCREvaluator.loadAnnotations(in: sampleURL)
            let report = OCREvaluationReport.build(frames: results, annotations: annotations)
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.prettyPrinted, .sortedKeys, .withoutEscapingSlashes]
            let data = try encoder.encode(report)
            print(String(decoding: data, as: UTF8.self))
            writeStderr("[HexSight] OCR 样本评测完成: \(results.count) 张截图")
            return true
        } catch {
            writeStderr("[HexSight] OCR 样本评测失败: \(error.localizedDescription)")
            Foundation.exit(1)
        }
    }

    private static func writeStderr(_ message: String) {
        FileHandle.standardError.write(Data((message + "\n").utf8))
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
if !OCRCliRunner.runIfNeeded(arguments: CommandLine.arguments) {
    app.delegate = delegate
    app.setActivationPolicy(.accessory)
    app.run()
}
