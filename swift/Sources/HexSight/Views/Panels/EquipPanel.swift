import SwiftUI

struct EquipPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selected: EquipmentModel?

    var body: some View {
        HSplitView {
            equipList.frame(minWidth: 180, idealWidth: 220)
            if let equip = selected { equipDetail(equip) }
            else { emptyHint("shield.fill", "选择装备查看详情") }
        }
    }

    private var equipList: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 0) {
                let completed = data.equipment.filter { $0.isCompleted }
                let components = data.equipment.filter { $0.isComponent }
                let others = data.equipment.filter { !$0.isCompleted && !$0.isComponent }

                sectionView("成型装备", completed)
                sectionView("基础装备", components)
                if !others.isEmpty { sectionView("其他装备", others) }
            }
            .padding(.bottom, 12)
        }
    }

    private func sectionView(_ title: String, _ items: [EquipmentModel]) -> some View {
        Section {
            ForEach(items.sorted(by: { $0.name < $1.name })) { eq in
                EquipListRow(equip: eq, isSelected: selected?.id == eq.id)
                    .onTapGesture { selected = eq }
            }
        } header: {
            Text("\(title) (\(items.count))")
                .font(.system(size: 11, weight: .bold))
                .foregroundStyle(.secondary)
                .padding(.horizontal, 8).padding(.top, 8)
        }
    }

    private func equipDetail(_ equip: EquipmentModel) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                HStack(spacing: 12) {
                    AsyncImage(url: URL(string: equip.picture)) { img in
                        img.resizable().aspectRatio(contentMode: .fit)
                    } placeholder: { Color.gray.opacity(0.2) }
                    .frame(width: 56, height: 56).clipShape(RoundedRectangle(cornerRadius: 8))
                    .background(RoundedRectangle(cornerRadius: 8).fill(Color.white.opacity(0.05)))

                    VStack(alignment: .leading, spacing: 3) {
                        Text(equip.name).font(.title3).foregroundStyle(.primary)
                        Text(equip.type).font(.system(size: 11)).foregroundStyle(.secondary)
                    }
                    Spacer()
                }

                if !equip.basicDesc.isEmpty {
                    Text(equip.basicDesc).font(.system(size: 13, weight: .medium)).foregroundStyle(.tint)
                }
                if !equip.desc.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    Text(equip.desc).font(.system(size: 12)).foregroundStyle(.secondary).lineSpacing(4)
                }
                if !equip.synthesis1.isEmpty, equip.synthesis1 != "0" {
                    Divider().background(Color.white.opacity(0.1))
                    Text("合成配方").font(.headline).foregroundStyle(.primary)
                    HStack(spacing: 12) {
                        synthIcon(equip.synthesis1)
                        Image(systemName: "plus")
                            .font(.system(size: 12, weight: .bold))
                            .foregroundStyle(.secondary)
                        synthIcon(equip.synthesis2)
                        Image(systemName: "equal")
                            .font(.system(size: 12, weight: .bold))
                            .foregroundStyle(.secondary)
                        synthIcon(equip.id, isResult: true)
                    }
                    .padding(12)
                    .background(Color.white.opacity(0.025))
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.white.opacity(0.06), lineWidth: 1))
                    .padding(.top, 4)
                }
                if equip.isComponent {
                    let buildFrom = data.equipment.filter { $0.synthesis1 == equip.id || $0.synthesis2 == equip.id }
                    if !buildFrom.isEmpty {
                        Divider().background(Color.white.opacity(0.1))
                        Text("可合成的装备 (\(buildFrom.count))").font(.headline).foregroundStyle(.primary)
                        VStack(spacing: 8) {
                            ForEach(buildFrom.sorted(by: { $0.name < $1.name })) { eq in
                                let otherId = eq.synthesis1 == equip.id ? eq.synthesis2 : eq.synthesis1
                                HStack(spacing: 12) {
                                    AsyncImage(url: URL(string: eq.picture)) { img in
                                        img.resizable().aspectRatio(contentMode: .fit)
                                    } placeholder: { Color.gray.opacity(0.2) }
                                    .frame(width: 32, height: 32)
                                    .clipShape(RoundedRectangle(cornerRadius: 4))
                                    
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(eq.name).font(.system(size: 11, weight: .bold)).foregroundStyle(.primary)
                                        Text(eq.basicDesc).font(.system(size: 9)).foregroundStyle(.secondary).lineLimit(1)
                                    }
                                    
                                    Spacer()
                                    
                                    HStack(spacing: 6) {
                                        Text("配方:").font(.system(size: 9)).foregroundStyle(.tertiary)
                                        AsyncImage(url: URL(string: equip.picture)) { img in
                                            img.resizable().aspectRatio(contentMode: .fit)
                                        } placeholder: { Color.clear }
                                        .frame(width: 18, height: 18)
                                        .clipShape(RoundedRectangle(cornerRadius: 2))
                                        
                                        Image(systemName: "plus").font(.system(size: 8)).foregroundStyle(.secondary)
                                        
                                        if let otherEq = data.getEquip(otherId) {
                                            AsyncImage(url: URL(string: otherEq.picture)) { img in
                                                img.resizable().aspectRatio(contentMode: .fit)
                                            } placeholder: { Color.clear }
                                            .frame(width: 18, height: 18)
                                            .clipShape(RoundedRectangle(cornerRadius: 2))
                                            .help(otherEq.name)
                                        }
                                    }
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 4)
                                    .background(Color.white.opacity(0.02))
                                    .clipShape(RoundedRectangle(cornerRadius: 5))
                                }
                                .padding(8)
                                .background(Color.white.opacity(0.02))
                                .clipShape(RoundedRectangle(cornerRadius: 6))
                                .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.white.opacity(0.04), lineWidth: 1))
                            }
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private func synthIcon(_ id: String, isResult: Bool = false) -> some View {
        let eq = data.getEquip(id)
        return VStack(spacing: 2) {
            AsyncImage(url: URL(string: eq?.picture ?? "")) { img in
                img.resizable().aspectRatio(contentMode: .fit)
            } placeholder: { Color.gray.opacity(0.2) }
            .frame(width: 36, height: 36).clipShape(RoundedRectangle(cornerRadius: 4))
            .overlay(isResult ? RoundedRectangle(cornerRadius: 4).strokeBorder(.tint, lineWidth: 1.5) : nil)
            Text(eq?.name ?? id).font(.system(size: 9)).foregroundColor(isResult ? .accentColor : .secondary)
        }
    }

    private func emptyHint(_ icon: String, _ text: String) -> some View {
        VStack {
            Image(systemName: icon).font(.system(size: 36)).foregroundStyle(.tertiary)
            Text(text).font(.system(size: 13)).foregroundStyle(.tertiary)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct EquipListRow: View {
    let equip: EquipmentModel; let isSelected: Bool
    var body: some View {
        HStack(spacing: 8) {
            AsyncImage(url: URL(string: equip.picture)) { img in
                img.resizable().aspectRatio(contentMode: .fit)
            } placeholder: { Color.gray.opacity(0.2) }
            .frame(width: 28, height: 28).clipShape(RoundedRectangle(cornerRadius: 3))
            VStack(alignment: .leading, spacing: 1) {
                Text(equip.name).font(.system(size: 11, weight: .medium)).foregroundStyle(.primary)
                Text(equip.basicDesc).font(.system(size: 9)).foregroundStyle(.tertiary).lineLimit(1)
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 5)
        .background(isSelected ? Color.white.opacity(0.1) : Color.clear)
        .contentShape(Rectangle())
    }
}
