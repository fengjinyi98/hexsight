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

    func testSampleImageDiscoveryKeepsUnderscoreNamedSourceScreenshots() throws {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }

        for name in [
            "20260525_5-3_shop_001.png",
            "20260525_5-3_shop_001_opponent_roi.png",
            "20260525_5-3_shop_001_roi_sheet.jpg",
            "ocr_eval_output.json",
        ] {
            FileManager.default.createFile(
                atPath: directory.appendingPathComponent(name).path,
                contents: Data(),
                attributes: nil
            )
        }

        let images = try OCREvaluator.sampleImageFiles(in: directory)

        XCTAssertEqual(images.map(\.lastPathComponent), ["20260525_5-3_shop_001.png"])
    }

    func testOCRSamplesContainMinimalRegressionSet() throws {
        let samples = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("docs/ocr_samples", isDirectory: true)

        let images = try OCREvaluator.sampleImageFiles(in: samples)

        XCTAssertEqual(images.count, 30)
    }

    func testOCREvalOutputContainsShopHeroRecognitionBaseline() throws {
        let output = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("docs/ocr_samples/ocr_eval_output.json")
        let data = try Data(contentsOf: output)
        let frames = try JSONDecoder().decode([OCRFrameResult].self, from: data)
        let heroes = Set(frames.flatMap { frame in
            frame.summary.shop.compactMap(\.heroName)
        })

        XCTAssertEqual(frames.count, 30)
        XCTAssertTrue(heroes.isSuperset(of: ["璐璐", "克格莫", "布隆", "塔里克", "萨勒芬妮", "安妮"]))
    }

    func testFieldAccuracyReportGroupsAnnotatedFields() throws {
        let frames = [
            OCRFrameResult(
                imagePath: "/tmp/sample.png",
                imageSize: CGSize(width: 1920, height: 1080),
                gameContentRect: CGRect(x: 0, y: 0, width: 1920, height: 1080),
                summary: OCRFrameSummary(
                    round: "2-1",
                    shop: [OCRShopSlot(index: 0, heroName: "璐璐", traits: ["法师"])],
                    opponents: [OCROpponentRow(index: 0, name: "屋屋z", hp: 76)],
                    activeTraits: [OCRActiveTraitRow(index: 0, name: "法师", count: "2/4/6")],
                    augments: [OCRAugmentOption(index: 0, name: "潘朵拉的装备")]
                ),
                regions: []
            )
        ]
        let annotations = [
            "/tmp/sample.png": OCRFrameAnnotation(
                round: "2-1",
                shop: ["璐璐"],
                traits: ["法师"],
                opponents: ["屋屋z 76"],
                augments: ["潘朵拉的装备"]
            )
        ]

        let report = OCREvaluationReport.build(frames: frames, annotations: annotations)

        XCTAssertEqual(report.frames.count, 1)
        XCTAssertEqual(report.fieldAccuracy.map(\.field), ["round", "shop", "trait", "opponent", "augment"])
        XCTAssertTrue(report.fieldAccuracy.allSatisfy { $0.accuracy == 1.0 })
    }

    func testAnnotationLoadingReadsBatchAndPerImageFiles() throws {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }

        let batch = [
            "sample_a.png": OCRFrameAnnotation(round: "2-1", shop: ["璐璐"])
        ]
        let batchData = try JSONEncoder().encode(batch)
        try batchData.write(to: directory.appendingPathComponent("ocr_annotations.json"))

        let perImage = OCRFrameAnnotation(traits: ["法师"], augments: ["潘朵拉的装备"])
        let perImageData = try JSONEncoder().encode(perImage)
        try perImageData.write(to: directory.appendingPathComponent("sample_b.json"))

        let annotations = try OCREvaluator.loadAnnotations(in: directory)

        XCTAssertEqual(annotations["sample_a.png"]?.round, "2-1")
        XCTAssertEqual(annotations["sample_b.png"]?.traits, ["法师"])
        XCTAssertEqual(annotations["sample_b.png"]?.augments, ["潘朵拉的装备"])
    }
}
