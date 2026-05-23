import AppKit
import ScreenCaptureKit
import AVFoundation

/// 屏幕采集引擎
/// 核心职责：
/// - ScreenCaptureKit 封装
/// - 锁定金铲铲游戏窗口
/// - 输出 BGRA 像素缓冲区
/// - 动态帧率控制
final class ScreenCapture: NSObject, @unchecked Sendable {
    /// 帧回调：BGRA 像素数据指针、宽度、高度
    var onFrameCaptured: ((UnsafePointer<UInt8>, Int, Int) -> Void)?

    private var stream: SCStream?
    private var streamOutput: CaptureOutput?
    private var queue = DispatchQueue(label: "com.hexsight.capture", qos: .userInteractive)

    func start() {
        Task {
            await requestStream()
        }
    }

    func stop() {
        stream?.stopCapture()
        stream = nil
    }

    /// 请求屏幕内容并创建采集流
    private func requestStream() async {
        do {
            let content = try await SCShareableContent.current

            // 查找金铲铲窗口
            guard let gameWindow = content.windows.first(where: { window in
                window.title?.contains("金铲铲") == true
                || window.owningApplication?.bundleIdentifier.contains("com.netease") == true
            }) else {
                // 未找到游戏窗口，持续等待
                DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) { [weak self] in
                    Task { await self?.requestStream() }
                }
                return
            }

            let filter = SCContentFilter(desktopIndependentWindow: gameWindow)
            let config = SCStreamConfiguration()
            config.pixelFormat = kCVPixelFormatType_32BGRA
            config.width = Int(gameWindow.frame.width)
            config.height = Int(gameWindow.frame.height)
            config.queueDepth = 3

            let output = CaptureOutput()
            output.onFrame = { [weak self] sampleBuffer in
                self?.handleSampleBuffer(sampleBuffer)
            }
            self.streamOutput = output

            let stream = SCStream(filter: filter, configuration: config, delegate: nil)
            try stream.addStreamOutput(output, type: SCStreamOutputType.screen, sampleHandlerQueue: queue)
            try await stream.startCapture()
            self.stream = stream

        } catch {
            print("[HexSight] 采集流启动失败: \(error.localizedDescription)")
            DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) { [weak self] in
                Task { await self?.requestStream() }
            }
        }
    }

    /// 处理采集到的帧
    private func handleSampleBuffer(_ sampleBuffer: CMSampleBuffer) {
        guard let imageBuffer = sampleBuffer.imageBuffer else { return }

        CVPixelBufferLockBaseAddress(imageBuffer, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(imageBuffer, .readOnly) }

        guard let baseAddress = CVPixelBufferGetBaseAddress(imageBuffer) else { return }

        let width = CVPixelBufferGetWidth(imageBuffer)
        let height = CVPixelBufferGetHeight(imageBuffer)

        onFrameCaptured?(baseAddress.assumingMemoryBound(to: UInt8.self), width, height)
    }
}

/// SCStreamOutput 代理
private final class CaptureOutput: NSObject, SCStreamOutput {
    var onFrame: ((CMSampleBuffer) -> Void)?

    func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer,
                of type: SCStreamOutputType) {
        onFrame?(sampleBuffer)
    }
}
