import SwiftUI

/// 海克斯图鉴面板（对齐官网布局）
/// 顶部：左侧模式卡片，右侧控制项（黄牌、等级菜单、搜索框）
/// 内容：1:1 深紫色正方形磁贴网格 + 详情弹窗
struct HexPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selectedLevel: Int = 1
    @State private var searchQuery: String = ""
    @State private var hoveredHex: HexModel?

    private var displayHexes: [HexModel] {
        let list = data.hexes.filter { $0.level == selectedLevel }
        if searchQuery.isEmpty {
            return list.sorted { $0.name < $1.name }
        } else {
            return list.filter {
                $0.name.localizedCaseInsensitiveContains(searchQuery) ||
                $0.desc.localizedCaseInsensitiveContains(searchQuery)
            }
            .sorted { $0.name < $1.name }
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // 顶栏控制组
            topHeaderView
            
            Divider().background(Color.white.opacity(0.1))

            // 名称网格 + 详情弹窗
            ZStack(alignment: .topTrailing) {
                if displayHexes.isEmpty {
                    emptyView
                } else {
                    ScrollView {
                        LazyVGrid(
                            columns: [GridItem(.adaptive(minimum: 90), spacing: 8)],
                            spacing: 8
                        ) {
                            ForEach(displayHexes) { hex in
                                Button {
                                    hoveredHex = hoveredHex?.id == hex.id ? nil : hex
                                } label: {
                                    VStack(spacing: 8) {
                                        RemoteIcon(url: hex.icon, size: 44, cornerRadius: 4)
                                            .overlay(
                                                RoundedRectangle(cornerRadius: 4)
                                                    .stroke(Color.white.opacity(0.15), lineWidth: 1)
                                            )
                                        
                                        Text(hex.name)
                                            .font(Theme.Font.caption.weight(.medium))
                                            .foregroundStyle(hoveredHex?.id == hex.id ? Theme.Color.textPrimary : Theme.Color.textSecondary)
                                            .lineLimit(2)
                                            .multilineTextAlignment(.center)
                                            .padding(.horizontal, 4)
                                    }
                                    .frame(maxWidth: .infinity)
                                    .aspectRatio(1.0, contentMode: .fit)
                                    .padding(.vertical, 8)
                                    .background(
                                        hoveredHex?.id == hex.id
                                        ? Color(red: 0.28, green: 0.18, blue: 0.52)
                                        : Color(red: 0.18, green: 0.12, blue: 0.36)
                                    )
                                    .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                                    .overlay(
                                        RoundedRectangle(cornerRadius: Theme.CornerRadius.card)
                                            .stroke(hoveredHex?.id == hex.id ? Theme.Color.gold.opacity(0.8) : Color.white.opacity(0.06), lineWidth: 1)
                                    )
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        .padding(.horizontal, 14)
                        .padding(.top, 10)
                        .padding(.bottom, 32)
                    }
                }

                // 详情弹窗
                if let hex = hoveredHex {
                    detailPopover(hex)
                        .transition(.opacity.combined(with: .scale(scale: 0.95)))
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    private var topHeaderView: some View {
        HStack(alignment: .center, spacing: Theme.Spacing.medium) {
            // 左侧：游戏模式快捷切换（药丸按钮组）
            HStack(spacing: 8) {
                ForEach(data.availableModes, id: \.id) { mode in
                    Button {
                        data.switchMode(mode.id)
                        hoveredHex = nil
                    } label: {
                        Text(mode.name)
                            .font(Theme.Font.caption.weight(.bold))
                            .foregroundStyle(data.selectedMode == mode.id ? .white : Theme.Color.textSecondary)
                            .padding(.horizontal, 16)
                            .padding(.vertical, 8)
                            .background(
                                ZStack {
                                    if data.selectedMode == mode.id {
                                        if mode.id == "17" {
                                            LinearGradient(colors: [Color.gray.opacity(0.8), Color.black.opacity(0.6)], startPoint: .topLeading, endPoint: .bottomTrailing)
                                        } else if mode.id == "4" {
                                            LinearGradient(colors: [Color.purple.opacity(0.8), Color(red: 0.19, green: 0.12, blue: 0.37)], startPoint: .topLeading, endPoint: .bottomTrailing)
                                        } else {
                                            LinearGradient(colors: [Color(red: 0.2, green: 0.25, blue: 0.35), Color.black.opacity(0.7)], startPoint: .topLeading, endPoint: .bottomTrailing)
                                        }
                                    } else {
                                        Color.white.opacity(0.04)
                                    }
                                }
                            )
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                            .overlay(
                                RoundedRectangle(cornerRadius: 6)
                                    .stroke(data.selectedMode == mode.id ? (mode.id == "4" ? Color.purple : Color.white.opacity(0.4)) : Color.white.opacity(0.08), lineWidth: 1)
                            )
                            .shadow(color: data.selectedMode == mode.id && mode.id == "4" ? Color.purple.opacity(0.4) : Color.clear, radius: 4)
                    }
                    .buttonStyle(.plain)
                }
            }
            
            Spacer()
            
            // 右侧：控制组
            HStack(spacing: 12) {
                // 黄色高亮标签
                Text("强化符文")
                    .font(Theme.Font.caption.weight(.bold))
                    .foregroundStyle(.black)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 6)
                    .background(Color.yellow.opacity(0.95))
                    .clipShape(RoundedRectangle(cornerRadius: 4))
                
                // 下拉等级菜单
                Menu {
                    ForEach([1, 2, 3], id: \.self) { lv in
                        Button("\(getLevelText(lv))强化符文") {
                            selectedLevel = lv
                            hoveredHex = nil
                        }
                    }
                } label: {
                    HStack(spacing: 6) {
                        Text("\(getLevelText(selectedLevel))强化符文")
                            .font(Theme.Font.caption)
                            .foregroundStyle(Theme.Color.textPrimary)
                        Image(systemName: "chevron.down")
                            .font(.system(size: 8, weight: .bold))
                            .foregroundStyle(Theme.Color.textSecondary)
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(Color.white.opacity(0.06))
                    .clipShape(RoundedRectangle(cornerRadius: 4))
                    .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.white.opacity(0.12), lineWidth: 1))
                }
                .menuStyle(.borderlessButton)
                
                // 搜索框
                HStack(spacing: 6) {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundStyle(Theme.Color.textTertiary)
                    
                    TextField("搜索", text: $searchQuery)
                        .textFieldStyle(.plain)
                        .font(Theme.Font.caption)
                        .foregroundStyle(Theme.Color.textPrimary)
                        .frame(width: 120)
                }
                .padding(.horizontal, 8)
                .padding(.vertical, 5)
                .background(Color.white.opacity(0.06))
                .clipShape(RoundedRectangle(cornerRadius: 4))
                .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.white.opacity(0.12), lineWidth: 1))
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
    }

    private func detailPopover(_ hex: HexModel) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                RemoteIcon(url: hex.icon, size: 36, cornerRadius: 6)
                    .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.white.opacity(0.15), lineWidth: 1))

                VStack(alignment: .leading, spacing: 2) {
                    Text(hex.name).font(Theme.Font.body.weight(.bold)).foregroundStyle(Theme.Color.textPrimary)
                    Text("\(getLevelText(hex.level))强化符文").font(Theme.Font.caption).foregroundStyle(Theme.Color.textTertiary)
                    if hex.isLegend {
                        Text("英雄强化")
                            .font(Theme.Font.micro)
                            .foregroundStyle(Theme.Color.gold)
                            .padding(.horizontal, 4)
                            .padding(.vertical, 1)
                            .background(Theme.Color.gold.opacity(0.1))
                            .clipShape(RoundedRectangle(cornerRadius: 2))
                    }
                }

                Spacer()

                Button { hoveredHex = nil } label: {
                    Image(systemName: "xmark.circle.fill").font(.system(size: 16)).foregroundStyle(Theme.Color.textTertiary)
                }
                .buttonStyle(.plain)
            }

            Text(hex.desc)
                .font(Theme.Font.body)
                .foregroundStyle(Theme.Color.textSecondary)
                .lineSpacing(4)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(14)
        .frame(width: 290)
        .background(
            LinearGradient(
                colors: [
                    Color(red: 0.16, green: 0.10, blue: 0.32).opacity(0.95),
                    Color(red: 0.08, green: 0.05, blue: 0.18).opacity(0.98)
                ],
                startPoint: .top,
                endPoint: .bottom
            )
        )
        .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
        .overlay(
            RoundedRectangle(cornerRadius: Theme.CornerRadius.card)
                .stroke(Theme.Color.gold.opacity(0.38), lineWidth: 1.2)
        )
        .shadow(color: Color.black.opacity(0.4), radius: 10, x: 0, y: 4)
        .padding(14)
    }

    private func getLevelText(_ lv: Int) -> String {
        switch lv {
        case 1: return "一级"
        case 2: return "二级"
        case 3: return "三级"
        default: return ""
        }
    }

    private var emptyView: some View {
        VStack(spacing: 8) {
            Image(systemName: "sparkles").font(.system(size: 32)).foregroundStyle(.tertiary)
            Text("没有找到符合条件的强化符文").font(Theme.Font.body).foregroundStyle(.tertiary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct RemoteIcon: View {
    let url: String
    let width: CGFloat
    let height: CGFloat
    let cornerRadius: CGFloat

    init(url: String, size: CGFloat, cornerRadius: CGFloat) {
        self.url = url
        self.width = size
        self.height = size
        self.cornerRadius = cornerRadius
    }

    init(url: String, width: CGFloat, height: CGFloat, cornerRadius: CGFloat) {
        self.url = url
        self.width = width
        self.height = height
        self.cornerRadius = cornerRadius
    }

    var body: some View {
        AsyncImage(url: URL(string: url)) { phase in
            switch phase {
            case .success(let image):
                image.resizable().aspectRatio(contentMode: .fill)
            case .failure:
                Image(systemName: "photo")
                    .font(.system(size: min(width, height) * 0.38))
                    .foregroundStyle(.secondary)
                    .frame(width: width, height: height)
                    .background(Color.white.opacity(0.05))
            case .empty:
                Color.white.opacity(0.05)
            @unknown default:
                Color.white.opacity(0.05)
            }
        }
        .frame(width: width, height: height)
        .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
    }
}
