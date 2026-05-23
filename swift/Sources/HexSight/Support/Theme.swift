import SwiftUI

/// Theme 现代设计系统 Token
/// 核心职责：
/// - 定义软件统一色彩、间距、圆角与字体规范
/// - 提供基础组件扩展，保证全局设计高度一致与系统化
public struct Theme {
    public struct Spacing {
        public static let small: CGFloat = 8
        public static let medium: CGFloat = 16
        public static let large: CGFloat = 24
        public static let extraLarge: CGFloat = 32
    }

    public struct CornerRadius {
        public static let card: CGFloat = 12
        public static let chip: CGFloat = 6
        public static let button: CGFloat = 8
        public static let panel: CGFloat = 14
    }

    public struct Color {
        // 主底色渐变
        public static let backgroundStart = SwiftUI.Color(red: 0.08, green: 0.07, blue: 0.15)
        public static let backgroundEnd = SwiftUI.Color(red: 0.12, green: 0.10, blue: 0.20)
        
        // 玻璃面板底色
        public static let panelBackground = SwiftUI.Color.white.opacity(0.04)
        public static let cardBackground = SwiftUI.Color.white.opacity(0.02)
        public static let cardHover = SwiftUI.Color.white.opacity(0.06)
        
        // 文字颜色
        public static let textPrimary = SwiftUI.Color.white.opacity(0.95)
        public static let textSecondary = SwiftUI.Color.white.opacity(0.65)
        public static let textTertiary = SwiftUI.Color.white.opacity(0.35)
        
        // 核心强调色
        public static let gold = SwiftUI.Color(red: 0.95, green: 0.77, blue: 0.35)
        public static let accent = SwiftUI.Color.purple
        public static let active = SwiftUI.Color.purple.opacity(0.4)
    }

    public struct Font {
        // 大标题 (24px, 粗体)
        public static let title1 = SwiftUI.Font.system(size: 24, weight: .bold)
        // 区域/卡片标题 (18px, 粗体)
        public static let title2 = SwiftUI.Font.system(size: 18, weight: .bold)
        // 列表大文本 (15px, 半粗)
        public static let title3 = SwiftUI.Font.system(size: 15, weight: .semibold)
        // 主正文字体 (14px，大方、醒目)
        public static let body = SwiftUI.Font.system(size: 14, weight: .regular)
        // 辅助性小字 (11px)
        public static let caption = SwiftUI.Font.system(size: 11, weight: .regular)
        // 微型字体 (9px)
        public static let micro = SwiftUI.Font.system(size: 9, weight: .regular)
    }
}

extension View {
    /// 统一全局主渐变暗色背景
    public func themeBackground() -> some View {
        self.background(
            LinearGradient(
                colors: [Theme.Color.backgroundStart, Theme.Color.backgroundEnd],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
    }
}
