import AppKit
import ScreenCaptureKit
import AVFoundation

/// CapturedFrame 采集帧快照
/// 核心职责：
/// - 保存连续 BGRA 像素数据
/// - 隔离 CVPixelBuffer 行跨度与生命周期差异
struct CapturedFrame {
    let pixels: Data
    let width: Int
    let height: Int

    var bytesPerRow: Int {
        width * 4
    }
}

/// 屏幕采集引擎
/// 核心职责：
/// - ScreenCaptureKit 封装
/// - 锁定金铲铲游戏窗口
/// - 输出 BGRA 像素缓冲区
/// - 动态帧率控制
final class ScreenCapture: NSObject, @unchecked Sendable {
    /// 帧回调：连续 BGRA 像素快照
    var onFrameCaptured: ((CapturedFrame) -> Void)?

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
        let sourceBytesPerRow = CVPixelBufferGetBytesPerRow(imageBuffer)

        guard let frame = makeContiguousFrame(
            baseAddress: baseAddress,
            width: width,
            height: height,
            sourceBytesPerRow: sourceBytesPerRow
        ) else { return }

        onFrameCaptured?(frame)
    }

    private func makeContiguousFrame(
        baseAddress: UnsafeMutableRawPointer,
        width: Int,
        height: Int,
        sourceBytesPerRow: Int
    ) -> CapturedFrame? {
        guard width > 0, height > 0 else { return nil }
        let targetBytesPerRow = width * 4

        if sourceBytesPerRow == targetBytesPerRow {
            return CapturedFrame(
                pixels: Data(bytes: baseAddress, count: targetBytesPerRow * height),
                width: width,
                height: height
            )
        }

        var data = Data(count: targetBytesPerRow * height)
        data.withUnsafeMutableBytes { destination in
            guard let rawDestinationBase = destination.baseAddress else { return }
            let sourceBase = baseAddress.assumingMemoryBound(to: UInt8.self)
            let destinationBase = rawDestinationBase.assumingMemoryBound(to: UInt8.self)

            for row in 0..<height {
                destinationBase
                    .advanced(by: row * targetBytesPerRow)
                    .update(from: sourceBase.advanced(by: row * sourceBytesPerRow), count: targetBytesPerRow)
            }
        }

        return CapturedFrame(pixels: data, width: width, height: height)
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
