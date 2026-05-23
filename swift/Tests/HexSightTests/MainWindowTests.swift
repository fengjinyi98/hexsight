import AppKit
import XCTest
@testable import HexSight

/// MainWindowTests 主窗口配置测试
/// 核心职责：
/// - 验证窗口滚动区域不会触发整窗拖拽
/// - 验证窗口使用普通层级，允许其他窗口覆盖
@MainActor
final class MainWindowTests: XCTestCase {
    func testMainWindowUsesNormalLevelAndDisablesBackgroundDragging() {
        let window = MainWindow()

        XCTAssertEqual(window.level, .normal)
        XCTAssertFalse(window.isFloatingPanel)
        XCTAssertFalse(window.isMovableByWindowBackground)
    }
}
