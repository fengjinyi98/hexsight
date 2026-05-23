import SwiftUI

/// 主窗口内容视图
struct MainView: View {
    @StateObject private var appState = AppState.shared
    @StateObject private var dataService = GameDataService.shared
    @State private var selectedNav: NavItem = .decision

    enum NavItem: String, CaseIterable {
        case lineup = "阵容"
        case decision = "决策"
        case hero = "英雄"
        case equip = "装备"
        case trait = "羁绊"
        case hex = "海克斯"

        var icon: String {
            switch self {
            case .lineup: "square.grid.2x2.fill"
            case .decision: "bolt.fill"
            case .hero: "person.2.fill"
            case .equip: "shield.fill"
            case .trait: "link"
            case .hex: "sparkles"
            }
        }
    }

    var body: some View {
        GeometryReader { geometry in
            let currentW = geometry.size.width
            let scale = currentW / 1250.0

            HStack(spacing: 0) {
                navSidebar
                Divider().background(Color.white.opacity(0.15))
                contentArea
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            .frame(width: 1250, height: 820)
            .glassEffect(in: .rect(cornerRadius: 14))
            .scaleEffect(scale, anchor: .topLeading)
        }
        .frame(minWidth: 1000, idealWidth: 1250, minHeight: 656, idealHeight: 820)
    }

    // MARK: - 导航侧栏

    private var navSidebar: some View {
        VStack(spacing: 4) {
            VStack(spacing: 2) {
                ForEach(dataService.availableModes, id: \.id) { mode in
                    Button(mode.name) { dataService.switchMode(mode.id) }
                        .buttonStyle(.glass)
                        .font(.system(size: 10, weight: dataService.selectedMode == mode.id ? .bold : .regular))
                        .opacity(dataService.selectedMode == mode.id ? 1.0 : 0.5)
                        .frame(width: 103, height: 22)
                }
            }
            .padding(.horizontal, 6).padding(.vertical, 8)

            Divider().background(Color.white.opacity(0.15))

            ForEach(NavItem.allCases, id: \.self) { item in
                Button { selectedNav = item } label: {
                    HStack(spacing: 8) {
                        Image(systemName: item.icon)
                            .font(.system(size: 13))
                            .frame(width: 18, alignment: .center)
                        Text(item.rawValue)
                            .font(.system(size: 11, weight: selectedNav == item ? .semibold : .regular))
                        Spacer(minLength: 0)
                    }
                    .padding(.horizontal, 10)
                    .frame(width: 103, height: 34)
                    .foregroundStyle(selectedNav == item ? .primary : .secondary)
                    .background(selectedNav == item ? Color.white.opacity(0.12) : Color.clear)
                    .clipShape(RoundedRectangle(cornerRadius: 6))
                }
                .buttonStyle(.plain)
            }

            Spacer()
            Text("HexSight V1.0").font(.system(size: 8)).foregroundStyle(.tertiary).padding(.bottom, 8)
        }
        .frame(width: 115).padding(.top, 8)
    }

    // MARK: - 内容区

    @ViewBuilder
    private var contentArea: some View {
        switch selectedNav {
        case .decision:
            DecisionPanel(appState: appState)
        case .hero:
            HeroPanel()
        case .equip:
            EquipPanel()
        case .trait:
            TraitPanel()
        case .hex:
            HexPanel()
        case .lineup:
            LineupPanel()
        }
    }
}
