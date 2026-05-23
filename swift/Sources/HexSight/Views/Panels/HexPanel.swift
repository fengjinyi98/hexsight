import SwiftUI

/// 海克斯图鉴面板（对齐官网布局）
/// 顶部：1/2/3 级子 tab
/// 内容：名称网格 + 点击查看详情弹窗
struct HexPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selectedLevel: Int = 1
    @State private var hoveredHex: HexModel?

    private var displayHexes: [HexModel] {
        data.hexes.filter { $0.level == selectedLevel }
            .sorted { $0.name < $1.name }
    }

    var body: some View {
        VStack(spacing: 0) {
            // 等级子 tab
            HStack(spacing: 0) {
                ForEach([1, 2, 3], id: \.self) { lv in
                    Button {
                        selectedLevel = lv
                        hoveredHex = nil
                    } label: {
                        VStack(spacing: 2) {
                            Text("\(lv) 级强化符文")
                                .font(.system(size: 12, weight: selectedLevel == lv ? .bold : .regular))
                            Text(getLevelLabel(lv))
                                .font(.system(size: 9))
                                .foregroundStyle(.tertiary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 8)
                        .foregroundStyle(selectedLevel == lv ? .primary : .secondary)
                        .background(selectedLevel == lv ? Color.white.opacity(0.08) : Color.clear)
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 10).padding(.top, 8)

            Divider().background(Color.white.opacity(0.1))

            // 名称网格 + 详情弹窗
            ZStack(alignment: .topTrailing) {
                ScrollView {
                    LazyVGrid(
                        columns: [GridItem(.adaptive(minimum: 110, maximum: 140), spacing: 4)],
                        spacing: 4
                    ) {
                        ForEach(displayHexes) { hex in
                            Button {
                                hoveredHex = hoveredHex?.id == hex.id ? nil : hex
                            } label: {
                                HStack(spacing: 4) {
                                    AsyncImage(url: URL(string: hex.icon)) { img in
                                        img.resizable().aspectRatio(contentMode: .fit)
                                    } placeholder: { Color.clear }
                                    .frame(width: 20, height: 20)
                                    .clipShape(RoundedRectangle(cornerRadius: 3))

                                    Text(hex.name)
                                        .font(.system(size: 10))
                                        .foregroundStyle(.primary)
                                        .lineLimit(1)
                                    Spacer()
                                }
                                .padding(.horizontal, 8)
                                .padding(.vertical, 6)
                                .background(
                                    hoveredHex?.id == hex.id
                                    ? Color.white.opacity(0.1)
                                    : Color.white.opacity(0.03)
                                )
                                .clipShape(RoundedRectangle(cornerRadius: 6))
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(10)
                }

                // 详情弹窗
                if let hex = hoveredHex {
                    detailPopover(hex)
                        .transition(.opacity.combined(with: .scale(scale: 0.95)))
                }
            }
        }
    }

    private func detailPopover(_ hex: HexModel) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                AsyncImage(url: URL(string: hex.icon)) { img in
                    img.resizable().aspectRatio(contentMode: .fit)
                } placeholder: { Color.gray.opacity(0.2) }
                .frame(width: 36, height: 36)
                .clipShape(RoundedRectangle(cornerRadius: 6))

                VStack(alignment: .leading, spacing: 2) {
                    Text(hex.name).font(.system(size: 13, weight: .semibold)).foregroundStyle(.primary)
                    Text("\(hex.level) 级强化符文").font(.system(size: 10)).foregroundStyle(.tertiary)
                    if hex.isLegend { Text("英雄强化").font(.system(size: 9)).foregroundStyle(.tint) }
                }

                Spacer()

                Button { hoveredHex = nil } label: {
                    Image(systemName: "xmark.circle.fill").font(.system(size: 16)).foregroundStyle(.tertiary)
                }
                .buttonStyle(.plain)
            }

            Text(hex.desc)
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
                .lineSpacing(4)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(14)
        .frame(width: 280)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .shadow(radius: 8)
        .padding(12)
    }

    private func getLevelLabel(_ lv: Int) -> String {
        switch lv {
        case 1: return "白银"
        case 2: return "黄金"
        case 3: return "棱彩"
        default: return ""
        }
    }
}
