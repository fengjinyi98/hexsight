import SwiftUI

/// 羁绊图鉴面板（对齐官网布局）
/// 顶部：种族/职业 子tab
/// 左侧：图标网格
/// 右侧：详情面板（分级效果 + 协同英雄）
struct TraitPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selected: TraitModel?
    @State private var filterType: FilterType = .race

    enum FilterType: String, CaseIterable {
        case race = "种族"
        case job = "职业"
    }

    /// 去重后的羁绊列表
    private var displayTraits: [TraitModel] {
        let grouped = Dictionary(grouping: data.traits) { $0.checkId }
        let unique = grouped.values.compactMap { $0.min { $0.level < $1.level } }
        return unique
            .filter { $0.traitType == filterType.ordinal }
            .sorted { $0.name < $1.name }
    }

    private var filteredTypes: [Int] {
        [FilterType.race.ordinal, FilterType.job.ordinal]
    }

    var body: some View {
        VStack(spacing: 0) {
            // 子 tab
            HStack(spacing: 0) {
                ForEach(FilterType.allCases, id: \.self) { ft in
                    Button(ft.rawValue) { filterType = ft }
                        .font(.system(size: 11, weight: filterType == ft ? .bold : .regular))
                        .foregroundStyle(filterType == ft ? .primary : .secondary)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 6)
                        .background(filterType == ft ? Color.white.opacity(0.08) : Color.clear)
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                }
            }
            .padding(.horizontal, 10).padding(.top, 8)

            Divider().background(Color.white.opacity(0.1))

            HSplitView {
                // 左侧图标网格
                traitGrid.frame(minWidth: 220, idealWidth: 260)

                // 右侧详情
                if let trait = selected {
                    traitDetail(trait)
                } else {
                    emptyHint
                }
            }
        }
    }

    // MARK: - 网格

    private var traitGrid: some View {
        ScrollView {
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 56), spacing: 4)], spacing: 4) {
                ForEach(displayTraits) { trait in
                    VStack(spacing: 3) {
                        AsyncImage(url: URL(string: trait.picture)) { img in
                            img.resizable().aspectRatio(contentMode: .fit)
                        } placeholder: { Color.gray.opacity(0.15) }
                        .frame(width: 32, height: 32)
                        .clipShape(RoundedRectangle(cornerRadius: 5))
                        .background(
                            RoundedRectangle(cornerRadius: 5)
                                .fill(selected?.checkId == trait.checkId ? Color.white.opacity(0.12) : Color.clear)
                        )
                        Text(trait.name)
                            .font(.system(size: 9))
                            .foregroundStyle(selected?.checkId == trait.checkId ? .primary : .secondary)
                            .lineLimit(1)
                    }
                    .frame(width: 56)
                    .padding(.vertical, 4)
                    .contentShape(Rectangle())
                    .onTapGesture { selected = trait }
                }
            }
            .padding(8)
        }
    }

    // MARK: - 详情

    private func traitDetail(_ trait: TraitModel) -> some View {
        let levels = data.traitLevels(for: trait.checkId)
        let isRace = trait.traitType == 0
        let heroes = data.heroesForTrait(traitId: trait.checkId, isRace: isRace)

        return ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                // 图标 + 名称
                HStack(spacing: 12) {
                    AsyncImage(url: URL(string: trait.picture)) { img in
                        img.resizable().aspectRatio(contentMode: .fit)
                    } placeholder: { Color.gray.opacity(0.2) }
                    .frame(width: 48, height: 48).clipShape(RoundedRectangle(cornerRadius: 8))
                    .background(RoundedRectangle(cornerRadius: 8).fill(Color.white.opacity(0.05)))

                    VStack(alignment: .leading, spacing: 3) {
                        Text(trait.name).font(.title3).foregroundStyle(.primary)
                        Text(filterType.rawValue).font(.system(size: 11)).foregroundStyle(.tertiary)
                    }
                    Spacer()
                }

                // 概述（取第一级的 realDesc 作为摘要）
                if let firstLevel = levels.first, !firstLevel.realDesc.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    Text(firstLevel.realDesc)
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                        .lineSpacing(4)
                        .fixedSize(horizontal: false, vertical: true)
                }

                // 分级效果
                if !levels.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    VStack(alignment: .leading, spacing: 10) {
                        ForEach(levels) { lv in
                            VStack(alignment: .leading, spacing: 4) {
                                HStack(spacing: 6) {
                                    Text("(\(lv.num))")
                                        .font(.system(size: 13, weight: .bold, design: .monospaced))
                                        .foregroundStyle(.tint)
                                    Text("级")
                                        .font(.system(size: 10))
                                        .foregroundStyle(.tertiary)
                                }
                                if !lv.realDesc.isEmpty {
                                    Text(lv.realDesc)
                                        .font(.system(size: 12))
                                        .foregroundStyle(.secondary)
                                        .lineSpacing(3)
                                        .fixedSize(horizontal: false, vertical: true)
                                }
                            }
                            .padding(8)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Color.white.opacity(0.03))
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                        }
                    }
                }

                // 协同英雄
                if !heroes.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    VStack(alignment: .leading, spacing: 6) {
                        Text("协同英雄 (\(heroes.count))").font(.headline).foregroundStyle(.primary)
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 56))], spacing: 4) {
                            ForEach(heroes) { hero in
                                VStack(spacing: 2) {
                                    AsyncImage(url: URL(string: hero.picture)) { img in
                                        img.resizable().aspectRatio(contentMode: .fit)
                                    } placeholder: { Color.gray.opacity(0.2) }
                                    .frame(width: 32, height: 32).clipShape(RoundedRectangle(cornerRadius: 4))
                                    Text(hero.name).font(.system(size: 8)).foregroundStyle(.secondary).lineLimit(1)
                                }.frame(width: 56)
                            }
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private var emptyHint: some View {
        VStack {
            Image(systemName: "link").font(.system(size: 36)).foregroundStyle(.tertiary)
            Text("选择羁绊查看详情").font(.system(size: 13)).foregroundStyle(.tertiary)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private extension TraitPanel.FilterType {
    var ordinal: Int { self == .race ? 0 : 1 }
}
