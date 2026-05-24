import XCTest
@testable import HexSight

/// OCREvaluatorTests OCR 评测输入测试
/// 核心职责：
/// - 校验离线评测只读取原始截图样本
/// - 避免调试裁剪图和评测产物污染样本统计
final class OCREvaluatorTests: XCTestCase {
    func testSampleImageDiscoverySkipsDebugArtifacts() throws {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }

        for name in [
            "截屏2026-05-24 10.43.23.png",
            "截屏2026-05-24 10.43.23_bottom.jpg",
            "截屏2026-05-24 10.43.23_roi_sheet.jpg",
            "ocr_eval_output.json",
        ] {
            FileManager.default.createFile(
                atPath: directory.appendingPathComponent(name).path,
                contents: Data(),
                attributes: nil
            )
        }

        let images = try OCREvaluator.sampleImageFiles(in: directory)

        XCTAssertEqual(images.map(\.lastPathComponent), ["截屏2026-05-24 10.43.23.png"])
    }
}
