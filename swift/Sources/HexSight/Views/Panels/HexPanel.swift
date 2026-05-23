import SwiftUI

struct HexPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selected: HexModel?

    var body: some View {
        HSplitView {
            hexList.frame(minWidth: 180, idealWidth: 220)
            if let hex = selected { hexDetail(hex) }
            else { emptyHint("sparkles", "选择海克斯查看详情") }
        }
    }

    private var hexList: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 0) {
                let grouped = Dictionary(grouping: data.hexes) { $0.level }
                ForEach([1, 2, 3], id: \.self) { lv in
                    if let list = grouped[lv] {
                        let sorted = list.sorted { $0.name < $1.name }
                        Section {
                            ForEach(sorted) { hex in
                                let sel = selected?.id == hex.id
                                HexListRow(hex: hex, isSelected: sel)
                                    .onTapGesture { selected = hex }
                            }
                        } header: {
                            Text("\(lv) 级海克斯 (\(sorted.count))")
                                .font(.system(size: 11, weight: .bold))
                                .foregroundStyle(.secondary)
                                .padding(.horizontal, 8).padding(.top, 8)
                        }
                    }
                }
            }
            .padding(.bottom, 12)
        }
    }

    private func hexDetail(_ hex: HexModel) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                HStack(spacing: 12) {
                    AsyncImage(url: URL(string: hex.icon)) { img in
                        img.resizable().aspectRatio(contentMode: .fit)
                    } placeholder: { Color.gray.opacity(0.2) }
                    .frame(width: 48, height: 48).clipShape(RoundedRectangle(cornerRadius: 8))
                    .background(RoundedRectangle(cornerRadius: 8).fill(Color.white.opacity(0.05)))
                    VStack(alignment: .leading, spacing: 3) {
                        Text(hex.name).font(.title3).foregroundStyle(.primary)
                        HStack(spacing: 6) {
                            tierBadge("\(hex.level) 级")
                            if hex.isLegend { tierBadge("英雄强化") }
                        }
                    }
                    Spacer()
                }
                if !hex.desc.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    Text(hex.desc).font(.system(size: 12)).foregroundStyle(.secondary).lineSpacing(5)
                }
            }
            .padding(16)
        }
    }

    private func tierBadge(_ text: String) -> some View {
        Text(text).font(.system(size: 10)).foregroundStyle(.secondary)
            .padding(.horizontal, 6).padding(.vertical, 2)
            .background(Color.white.opacity(0.06)).clipShape(Capsule())
    }

    private func emptyHint(_ icon: String, _ text: String) -> some View {
        VStack {
            Image(systemName: icon).font(.system(size: 36)).foregroundStyle(.tertiary)
            Text(text).font(.system(size: 13)).foregroundStyle(.tertiary)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct HexListRow: View {
    let hex: HexModel; let isSelected: Bool
    var body: some View {
        HStack(spacing: 8) {
            AsyncImage(url: URL(string: hex.icon)) { img in
                img.resizable().aspectRatio(contentMode: .fit)
            } placeholder: { Color.gray.opacity(0.2) }
            .frame(width: 26, height: 26).clipShape(RoundedRectangle(cornerRadius: 3))
            VStack(alignment: .leading, spacing: 1) {
                Text(hex.name).font(.system(size: 11, weight: .medium)).foregroundStyle(.primary)
                Text(hex.desc).font(.system(size: 9)).foregroundStyle(.tertiary).lineLimit(1)
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 5)
        .background(isSelected ? Color.white.opacity(0.1) : Color.clear)
        .contentShape(Rectangle())
    }
}
