import SwiftUI

struct TraitPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selected: TraitModel?

    private var uniqueTraits: [TraitModel] {
        let grouped = Dictionary(grouping: data.traits) { $0.checkId }
        return grouped.values.compactMap { $0.min { $0.level < $1.level } }
            .sorted { $0.name < $1.name }
    }

    var body: some View {
        HSplitView {
            traitList.frame(minWidth: 170, idealWidth: 200)
            if let trait = selected { traitDetail(trait) }
            else { emptyHint("link", "选择羁绊查看详情") }
        }
    }

    private var traitList: some View {
        let races = uniqueTraits.filter { $0.traitType == 0 }
        let jobs = uniqueTraits.filter { $0.traitType == 1 }
        return ScrollView {
            LazyVStack(alignment: .leading, spacing: 0) {
                sectionView("种族", races)
                sectionView("职业", jobs)
            }.padding(.bottom, 12)
        }
    }

    private func sectionView(_ title: String, _ items: [TraitModel]) -> some View {
        Section {
            ForEach(items) { t in
                TraitListRow(trait: t, isSelected: selected?.checkId == t.checkId)
                    .onTapGesture { selected = t }
            }
        } header: {
            Text("\(title) (\(items.count))")
                .font(.system(size: 11, weight: .bold))
                .foregroundStyle(.secondary)
                .padding(.horizontal, 8).padding(.top, 8)
        }
    }

    private func traitDetail(_ trait: TraitModel) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                HStack(spacing: 12) {
                    AsyncImage(url: URL(string: trait.picture)) { img in
                        img.resizable().aspectRatio(contentMode: .fit)
                    } placeholder: { Color.gray.opacity(0.2) }
                    .frame(width: 48, height: 48).clipShape(RoundedRectangle(cornerRadius: 8))
                    .background(RoundedRectangle(cornerRadius: 8).fill(Color.white.opacity(0.05)))
                    VStack(alignment: .leading, spacing: 3) {
                        Text(trait.name).font(.title3).foregroundStyle(.primary)
                        Text(trait.traitType == 0 ? "种族" : "职业").font(.system(size: 11)).foregroundStyle(.secondary)
                    }
                    Spacer()
                }

                if !trait.desc.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    Text(trait.desc).font(.system(size: 12)).foregroundStyle(.secondary).lineSpacing(4)
                }

                let levels = data.traitLevels(for: trait.checkId)
                if !levels.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    Text("激活阈值").font(.headline).foregroundStyle(.primary)
                    ForEach(levels) { lv in
                        HStack {
                            ForEach(lv.thresholds, id: \.self) { t in
                                let text = String(t)
                                return Text(verbatim: text)
                                    .font(.system(size: 11, weight: .bold, design: .monospaced))
                                    .padding(.horizontal, 6).padding(.vertical, 2)
                                    .background(lv.level == trait.level ? Color.accentColor.opacity(0.2) : Color.white.opacity(0.06))
                                    .clipShape(Capsule())
                            }
                            Text("人").font(.system(size: 10)).foregroundStyle(.tertiary)
                            Spacer()
                        }
                    }
                }

                let isRace = trait.traitType == 0
                let heroes = data.heroesForTrait(traitId: trait.checkId, isRace: isRace)
                if !heroes.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    Text("包含英雄 (\(heroes.count))").font(.headline).foregroundStyle(.primary)
                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 56))], spacing: 4) {
                        ForEach(heroes) { hero in
                            VStack(spacing: 2) {
                                AsyncImage(url: URL(string: hero.picture)) { img in
                                    img.resizable().aspectRatio(contentMode: .fit)
                                } placeholder: { Color.gray.opacity(0.2) }
                                .frame(width: 30, height: 30).clipShape(RoundedRectangle(cornerRadius: 4))
                                Text(hero.name).font(.system(size: 8)).foregroundStyle(.secondary).lineLimit(1)
                            }.frame(width: 56)
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private func emptyHint(_ icon: String, _ text: String) -> some View {
        VStack {
            Image(systemName: icon).font(.system(size: 36)).foregroundStyle(.tertiary)
            Text(text).font(.system(size: 13)).foregroundStyle(.tertiary)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct TraitListRow: View {
    let trait: TraitModel; let isSelected: Bool
    var body: some View {
        HStack(spacing: 8) {
            AsyncImage(url: URL(string: trait.picture)) { img in
                img.resizable().aspectRatio(contentMode: .fit)
            } placeholder: { Color.gray.opacity(0.2) }
            .frame(width: 26, height: 26).clipShape(RoundedRectangle(cornerRadius: 3))
            Text(trait.name).font(.system(size: 11, weight: .medium)).foregroundStyle(.primary)
            Spacer()
            HStack(spacing: 3) {
                ForEach(Array(trait.thresholds.prefix(3)), id: \.self) { t in
                    Text(verbatim: "\(t)")
                        .font(.system(size: 8, design: .monospaced))
                        .padding(.horizontal, 3).padding(.vertical, 1)
                        .background(Color.white.opacity(0.06)).clipShape(Capsule())
                }
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 5)
        .background(isSelected ? Color.white.opacity(0.1) : Color.clear)
        .contentShape(Rectangle())
    }
}
