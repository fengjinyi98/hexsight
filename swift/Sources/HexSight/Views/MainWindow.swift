import AppKit
import SwiftUI

/// 主窗口 — 单一 NSPanel 承载所有 UI
/// 核心职责：
/// - 左侧导航：决策 / 英雄 / 装备 / 羁绊 / 海克斯
/// - 右侧内容面板
/// - Liquid Glass 玻璃质感
final class MainWindow: NSPanel {
    init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 1250, height: 798),
            styleMask: [.titled, .closable, .resizable, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )

        self.title = "HexSight"
        self.level = .normal
        self.isFloatingPanel = false
        self.isMovableByWindowBackground = false
        self.collectionBehavior = [.canJoinAllSpaces, .stationary]
        self.contentMinSize = NSSize(width: 1000, height: 638)
        self.contentAspectRatio = NSSize(width: 1250, height: 798)

        self.isOpaque = false
        self.backgroundColor = .clear
        self.hasShadow = true

        let hostingView = NSHostingView(rootView: MainView())
        hostingView.autoresizingMask = [.width, .height]
        self.contentView = hostingView
    }
}
