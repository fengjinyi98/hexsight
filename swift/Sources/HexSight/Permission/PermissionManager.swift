import AppKit
import AVFoundation
import ScreenCaptureKit

/// 权限管理器
/// 核心职责：
/// - 屏幕录制权限请求与状态检测
/// - 仅申请屏幕录制权限，无其他高危权限
final class PermissionManager: @unchecked Sendable {
    static let shared = PermissionManager()

    private init() {}

    /// 检查屏幕录制权限是否已授权
    var hasScreenRecordingPermission: Bool {
        if #available(macOS 14.0, *) {
            // macOS 14+ 使用 SCShareableContent 检测
            return !NSScreen.screens.isEmpty
        }
        // macOS 13 兼容
        return CGPreflightScreenCaptureAccess()
    }

    /// 请求屏幕录制权限
    /// completion: true = 用户同意, false = 拒绝或已授权
    func requestScreenRecordingPermission(completion: @escaping @Sendable (Bool) -> Void) {
        // 已授权则直接返回
        if hasScreenRecordingPermission {
            completion(true)
            return
        }

        // 触发系统权限弹窗
        if #available(macOS 14.0, *) {
            // macOS 14+: 通过 SCShareableContent 触发权限请求
            Task {
                do {
                    let _ = try await SCShareableContent.current
                    await MainActor.run { completion(true) }
                } catch {
                    await MainActor.run {
                        // 权限被拒绝，引导用户前往系统设置
                        self.openScreenRecordingSettings()
                        completion(false)
                    }
                }
            }
        } else {
            // macOS 13: 使用 CGRequestScreenCaptureAccess
            CGRequestScreenCaptureAccess()
            // 轮询检测权限状态
            DispatchQueue.global().asyncAfter(deadline: .now() + 0.5) { [weak self] in
                if self?.hasScreenRecordingPermission == true {
                    DispatchQueue.main.async { completion(true) }
                } else {
                    DispatchQueue.main.async {
                        self?.openScreenRecordingSettings()
                        completion(false)
                    }
                }
            }
        }
    }

    /// 打开系统设置 → 隐私 → 屏幕录制
    private func openScreenRecordingSettings() {
        let prefURL = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")!
        NSWorkspace.shared.open(prefURL)
    }
}
