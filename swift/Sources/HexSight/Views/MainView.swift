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
        HStack(spacing: 0) {
            navSidebar
            Divider().background(Color.white.opacity(0.15))
            contentArea
        }
        .frame(width: 700, height: 580)
        .glassEffect(in: .rect(cornerRadius: 14))
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
                }
            }
            .padding(.horizontal, 6).padding(.vertical, 8)

            Divider().background(Color.white.opacity(0.15))

            ForEach(NavItem.allCases, id: \.self) { item in
                Button { selectedNav = item } label: {
                    VStack(spacing: 3) {
                        Image(systemName: item.icon).font(.system(size: 16))
                        Text(item.rawValue).font(.system(size: 9))
                    }
                    .frame(width: 60, height: 48)
                    .foregroundStyle(selectedNav == item ? .primary : .secondary)
                    .background(selectedNav == item ? Color.white.opacity(0.12) : Color.clear)
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                }
                .buttonStyle(.plain)
            }

            Spacer()
            Text("HexSight V1.0").font(.system(size: 8)).foregroundStyle(.tertiary).padding(.bottom, 8)
        }
        .frame(width: 72).padding(.top, 8)
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
