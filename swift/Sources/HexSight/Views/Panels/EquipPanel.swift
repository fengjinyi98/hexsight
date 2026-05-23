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
            .padding(.bottom, 32)
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
                .font(Theme.Font.title3)
                .foregroundStyle(Theme.Color.textSecondary)
                .padding(.horizontal, 8).padding(.top, 12)
        }
    }

    private func equipDetail(_ equip: EquipmentModel) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: Theme.Spacing.medium) {
                HStack(spacing: 12) {
                    AsyncImage(url: URL(string: equip.picture)) { img in
                        img.resizable().aspectRatio(contentMode: .fit)
                    } placeholder: { Color.gray.opacity(0.2) }
                    .frame(width: 72, height: 72).clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                    .background(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).fill(Theme.Color.panelBackground))

                    VStack(alignment: .leading, spacing: 4) {
                        Text(equip.name).font(Theme.Font.title1).foregroundStyle(Theme.Color.textPrimary)
                        Text(equip.type).font(Theme.Font.body).foregroundStyle(Theme.Color.textSecondary)
                    }
                    Spacer()
                }

                if !equip.basicDesc.isEmpty {
                    Text(equip.basicDesc).font(Theme.Font.body.weight(.semibold)).foregroundStyle(Theme.Color.gold)
                }
                if !equip.desc.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    Text(equip.desc).font(Theme.Font.body).foregroundStyle(Theme.Color.textSecondary).lineSpacing(5)
                }
                if !equip.synthesis1.isEmpty, equip.synthesis1 != "0" {
                    Divider().background(Color.white.opacity(0.1))
                    Text("合成配方").font(Theme.Font.title2).foregroundStyle(Theme.Color.textPrimary)
                    HStack(spacing: 14) {
                        synthIcon(equip.synthesis1)
                        Image(systemName: "plus")
                            .font(.system(size: 16, weight: .bold))
                            .foregroundStyle(Theme.Color.textSecondary)
                        synthIcon(equip.synthesis2)
                        Image(systemName: "equal")
                            .font(.system(size: 16, weight: .bold))
                            .foregroundStyle(Theme.Color.textSecondary)
                        synthIcon(equip.id, isResult: true)
                    }
                    .padding(Theme.Spacing.medium)
                    .background(Theme.Color.cardBackground)
                    .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                    .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.06), lineWidth: 1))
                    .padding(.top, 4)
                }
                if equip.isComponent {
                    let buildFrom = data.equipment.filter { $0.synthesis1 == equip.id || $0.synthesis2 == equip.id }
                    if !buildFrom.isEmpty {
                        Divider().background(Color.white.opacity(0.1))
                        Text("可合成的装备 (\(buildFrom.count))").font(Theme.Font.title2).foregroundStyle(Theme.Color.textPrimary)
                        VStack(spacing: 8) {
                            ForEach(buildFrom.sorted(by: { $0.name < $1.name })) { eq in
                                let otherId = eq.synthesis1 == equip.id ? eq.synthesis2 : eq.synthesis1
                                HStack(spacing: 12) {
                                    AsyncImage(url: URL(string: eq.picture)) { img in
                                        img.resizable().aspectRatio(contentMode: .fit)
                                    } placeholder: { Color.gray.opacity(0.2) }
                                    .frame(width: 42, height: 42)
                                    .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.chip))
                                    
                                    VStack(alignment: .leading, spacing: 3) {
                                        Text(eq.name).font(Theme.Font.body.bold()).foregroundStyle(Theme.Color.textPrimary)
                                        Text(eq.basicDesc).font(Theme.Font.caption).foregroundStyle(Theme.Color.textSecondary).lineLimit(1)
                                    }
                                    
                                    Spacer()
                                    
                                    HStack(spacing: 6) {
                                        Text("配方:").font(Theme.Font.caption).foregroundStyle(Theme.Color.textTertiary)
                                        AsyncImage(url: URL(string: equip.picture)) { img in
                                            img.resizable().aspectRatio(contentMode: .fit)
                                        } placeholder: { Color.clear }
                                        .frame(width: 24, height: 24)
                                        .clipShape(RoundedRectangle(cornerRadius: 2))
                                        
                                        Image(systemName: "plus").font(Theme.Font.caption).foregroundStyle(Theme.Color.textSecondary)
                                        
                                        if let otherEq = data.getEquip(otherId) {
                                            AsyncImage(url: URL(string: otherEq.picture)) { img in
                                                img.resizable().aspectRatio(contentMode: .fit)
                                            } placeholder: { Color.clear }
                                            .frame(width: 24, height: 24)
                                            .clipShape(RoundedRectangle(cornerRadius: 2))
                                            .help(otherEq.name)
                                        }
                                    }
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 4)
                                    .background(Theme.Color.panelBackground)
                                    .clipShape(RoundedRectangle(cornerRadius: 5))
                                }
                                .padding(10)
                                .background(Theme.Color.cardBackground)
                                .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                                .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.04), lineWidth: 1))
                            }
                        }
                    }
                }
            }
            .padding(.horizontal, Theme.Spacing.medium)
            .padding(.top, Theme.Spacing.medium)
            .padding(.bottom, 32)
        }
    }

    private func synthIcon(_ id: String, isResult: Bool = false) -> some View {
        let eq = data.getEquip(id)
        return VStack(spacing: 2) {
            AsyncImage(url: URL(string: eq?.picture ?? "")) { img in
                img.resizable().aspectRatio(contentMode: .fit)
            } placeholder: { Color.gray.opacity(0.2) }
            .frame(width: 48, height: 48).clipShape(RoundedRectangle(cornerRadius: 4))
            .overlay(isResult ? RoundedRectangle(cornerRadius: 4).strokeBorder(.tint, lineWidth: 1.5) : nil)
            Text(eq?.name ?? id)
                .font(Theme.Font.caption)
                .foregroundColor(isResult ? .accentColor : Theme.Color.textSecondary)
        }
    }

    private func emptyHint(_ icon: String, _ text: String) -> some View {
        VStack {
            Image(systemName: icon).font(.system(size: 36)).foregroundStyle(.tertiary)
            Text(text).font(Theme.Font.body).foregroundStyle(.tertiary)
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
            .frame(width: 36, height: 36).clipShape(RoundedRectangle(cornerRadius: 4))
            VStack(alignment: .leading, spacing: 2) {
                Text(equip.name).font(Theme.Font.body.weight(.medium)).foregroundStyle(Theme.Color.textPrimary)
                Text(equip.basicDesc).font(Theme.Font.caption).foregroundStyle(Theme.Color.textTertiary).lineLimit(1)
            }
        }
        .padding(.horizontal, 10).padding(.vertical, 8)
        .background(isSelected ? Theme.Color.cardHover : Color.clear)
        .contentShape(Rectangle())
    }
}
