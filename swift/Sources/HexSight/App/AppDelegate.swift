import AppKit
import CoreGraphics
import SwiftUI

/// AppDelegate 应用入口
/// 核心职责：
/// - 初始化 Rust 引擎
/// - 创建主窗口
/// - 请求屏幕录制权限
/// - 以低频 OCR 兜底识别动态文本区域
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var mainWindow: MainWindow?
    private var captureEngine: ScreenCapture?
    private let ocrQueue = DispatchQueue(label: "com.hexsight.ocr", qos: .utility)
    private var ocrRecognizer: OCRFrameRecognizer?
    private var lastOCRTime: Date = .distantPast
    private var isOCRInFlight = false
    private let ocrInterval: TimeInterval = 1.25

    func applicationDidFinishLaunching(_ notification: Notification) {
        // 初始化 Rust 引擎
        RustBridge.shared.initialize()
        prepareOCRRecognizer()

        // 请求屏幕录制权限
        PermissionManager.shared.requestScreenRecordingPermission { [weak self] granted in
            guard let self else { return }
            if granted {
                Task { @MainActor in
                    self.startCapture()
                }
            }
        }

        // 单一主窗口
        mainWindow = MainWindow()
        mainWindow?.center()
        mainWindow?.makeKeyAndOrderFront(nil)
    }

    func applicationWillTerminate(_ notification: Notification) {
        captureEngine?.stop()
        RustBridge.shared.destroy()
    }

    private func startCapture() {
        captureEngine = ScreenCapture()
        captureEngine?.onFrameCaptured = { [weak self] frame in
            self?.processFrame(frame)
        }
        captureEngine?.start()
    }

    private func processFrame(_ frame: CapturedFrame) {
        frame.pixels.withUnsafeBytes { buffer in
            guard let pixels = buffer.baseAddress?.assumingMemoryBound(to: UInt8.self) else { return }

            if let jsonStr = RustBridge.shared.processFrame(
                pixels: pixels,
                width: UInt32(frame.width),
                height: UInt32(frame.height)
            ), !jsonStr.isEmpty {
                DispatchQueue.main.async {
                    AppState.shared.updateFromJSON(jsonStr)
                }
            }
        }

        scheduleOCRIfNeeded(frame)
    }

    private func prepareOCRRecognizer() {
        let dictionary = OCRDataDictionary.load()
        let customWords = Array(Set(dictionary.heroes + dictionary.traits + dictionary.augments)).sorted()
        ocrRecognizer = OCRFrameRecognizer(
            vision: OCRVisionService(customWords: customWords),
            normalizer: dictionary.normalizer()
        )
    }

    private func scheduleOCRIfNeeded(_ frame: CapturedFrame) {
        guard let ocrRecognizer, !isOCRInFlight else { return }
        let now = Date()
        guard now.timeIntervalSince(lastOCRTime) >= ocrInterval else { return }
        guard let cgImage = makeCGImage(frame) else { return }

        isOCRInFlight = true
        lastOCRTime = now
        ocrQueue.async { [weak self, ocrRecognizer, cgImage] in
            do {
                let summary = try ocrRecognizer.recognizeSummary(cgImage: cgImage)
                DispatchQueue.main.async {
                    AppState.shared.applyOCRSummary(summary)
                    self?.isOCRInFlight = false
                }
            } catch {
                DispatchQueue.main.async {
                    print("[HexSight] OCR 识别失败: \(error.localizedDescription)")
                    self?.isOCRInFlight = false
                }
            }
        }
    }

    private func makeCGImage(_ frame: CapturedFrame) -> CGImage? {
        guard frame.width > 0, frame.height > 0 else { return nil }
        guard let provider = CGDataProvider(data: frame.pixels as CFData) else { return nil }
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedFirst.rawValue)
            .union(.byteOrder32Little)

        return CGImage(
            width: frame.width,
            height: frame.height,
            bitsPerComponent: 8,
            bitsPerPixel: 32,
            bytesPerRow: frame.bytesPerRow,
            space: colorSpace,
            bitmapInfo: bitmapInfo,
            provider: provider,
            decode: nil,
            shouldInterpolate: false,
            intent: .defaultIntent
        )
    }
}
