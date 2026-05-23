import AppKit
import SwiftUI

/// AppDelegate 应用入口
/// 核心职责：
/// - 初始化 Rust 引擎
/// - 创建主窗口
/// - 请求屏幕录制权限
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var mainWindow: MainWindow?
    private var captureEngine: ScreenCapture?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // 初始化 Rust 引擎
        RustBridge.shared.initialize()

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
        captureEngine?.onFrameCaptured = { [weak self] pixels, width, height in
            self?.processFrame(pixels: pixels, width: width, height: height)
        }
        captureEngine?.start()
    }

    private func processFrame(pixels: UnsafePointer<UInt8>, width: Int, height: Int) {
        guard let jsonStr = RustBridge.shared.processFrame(
            pixels: pixels, width: UInt32(width), height: UInt32(height)
        ), !jsonStr.isEmpty else { return }

        DispatchQueue.main.async {
            AppState.shared.updateFromJSON(jsonStr)
        }
    }
}
