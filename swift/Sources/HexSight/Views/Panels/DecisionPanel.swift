import SwiftUI

/// 决策面板
struct DecisionPanel: View {
    @ObservedObject var appState: AppState
    @State private var mode: DisplayMode = .compact

    enum DisplayMode: String, CaseIterable {
        case compact = "极简"
        case full = "完整"
        case alert = "预警"
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Circle()
                    .fill(appState.engineReady ? Color.green : Color.orange)
                    .frame(width: 6, height: 6)
                Text("HexSight 决策")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(.primary)
                Spacer()
                Picker("模式", selection: $mode) {
                    ForEach(DisplayMode.allCases, id: \.self) { m in Text(m.rawValue).tag(m) }
                }
                .pickerStyle(.segmented).labelsHidden()
                .frame(width: 100).scaleEffect(0.75)
                Button("重置") { appState.resetGame() }
                    .buttonStyle(.glass).font(.system(size: 9))
            }
            .padding(.horizontal, 12).padding(.vertical, 8)
            Divider().background(Color.white.opacity(0.15))

            ScrollView {
                VStack(alignment: .leading, spacing: 8) {
                    switch mode {
                    case .compact: compactContent
                    case .full: fullContent
                    case .alert: alertContent
                    }
                }
                .padding(12)
            }
        }
    }

    private var compactContent: some View {
        VStack(alignment: .leading, spacing: 8) {
            statusBar
            Text("推荐阵容：\(appState.lineupName)")
                .font(.system(size: 13, weight: .semibold)).foregroundStyle(.primary)
            if !appState.suggestions.isEmpty {
                Text("操作：\(appState.suggestions.first ?? "")")
                    .font(.system(size: 12)).foregroundStyle(.secondary)
            }
            HStack(spacing: 12) {
                Label("同行 \(appState.rivalCount) 家", systemImage: "person.2")
                riskBadge(appState.riskLevel)
            }
            .font(.system(size: 11)).foregroundStyle(.secondary)
        }
    }

    private var fullContent: some View {
        VStack(alignment: .leading, spacing: 8) {
            statusBar
            Text("推荐阵容：\(appState.lineupName)")
                .font(.system(size: 13, weight: .semibold)).foregroundStyle(.primary)
            ForEach(appState.suggestions, id: \.self) { s in
                Label(s, systemImage: "arrow.right").font(.system(size: 11)).foregroundStyle(.secondary)
            }
            if !appState.equipRoute.isEmpty {
                Label("装备：\(appState.equipRoute.joined(separator: " → "))", systemImage: "wrench")
                    .font(.system(size: 11)).foregroundStyle(.tint)
            }
            if !appState.transition.isEmpty {
                Label("过渡：\(appState.transition.joined(separator: ", "))", systemImage: "arrow.triangle.swap")
                    .font(.system(size: 11)).foregroundStyle(.tint)
            }
            if let llm = appState.llmAdvice, !llm.isEmpty {
                Label(llm, systemImage: "brain").font(.system(size: 11)).foregroundStyle(.tint)
            }
        }
    }

    private var alertContent: some View {
        VStack(alignment: .leading, spacing: 8) {
            if appState.rivalCount >= 3 {
                AlertLine("exclamationmark.triangle.fill", "同行 \(appState.rivalCount) 家！建议转型", .red)
            }
            if appState.hp < 30 {
                AlertLine("heart.fill", "血量 \(appState.hp) 速D保命", .red)
            }
            if appState.riskLevel == "高" {
                AlertLine("arrow.triangle.swap", "强制转型建议", .orange)
            }
        }
    }

    private var statusBar: some View {
        HStack(spacing: 16) {
            StatItem(label: "金币", value: "\(appState.gold)")
            StatItem(label: "血量", value: "\(appState.hp)")
            StatItem(label: "人口", value: "\(appState.level)")
            StatItem(label: "回合", value: appState.round.isEmpty ? "-" : appState.round)
        }
    }

    private func riskBadge(_ level: String) -> some View {
        let color: Color = level == "高" ? .red : level == "中" ? .orange : .green
        return Label("风险 \(level)", systemImage: "flag.fill").foregroundColor(color)
    }
}

private struct StatItem: View {
    let label: String; let value: String
    var body: some View {
        VStack(spacing: 1) {
            Text(value).font(.system(size: 14, weight: .bold)).foregroundStyle(.primary)
            Text(label).font(.system(size: 9)).foregroundStyle(.tertiary)
        }
    }
}

private struct AlertLine: View {
    let icon: String; let text: String; let color: Color
    init(_ icon: String, _ text: String, _ color: Color) {
        self.icon = icon; self.text = text; self.color = color
    }
    var body: some View {
        Label(text, systemImage: icon).font(.system(size: 13, weight: .medium)).foregroundColor(color)
    }
}
