import SwiftUI

/// 英雄图鉴面板（对齐官网布局）
/// 左侧：按费用分组的英雄列表
/// 右侧：技能详情 + 多星属性表 + 种族职业详情
struct HeroPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selected: HeroModel?

    private var displayHeroes: [HeroModel] {
        HeroCatalog.displayHeroes(from: data.heroes)
    }

    var body: some View {
        HSplitView {
            heroList.frame(minWidth: 160, idealWidth: 190)
            if let hero = selected {
                heroDetail(hero)
            } else {
                emptyHint
            }
        }
    }

    // MARK: - 列表

    private var heroList: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 0) {
                let grouped = Dictionary(grouping: displayHeroes) { $0.cost }
                ForEach([1, 2, 3, 4, 5], id: \.self) { cost in
                    if let list = grouped[cost]?.sorted(by: { $0.name < $1.name }) {
                        Section {
                            ForEach(list) { hero in
                                HeroListRow(hero: hero, isSelected: selected?.baseKey == hero.baseKey)
                                    .onTapGesture { selected = hero }
                            }
                        } header: {
                            Text("\(cost) 费 (\(list.count))")
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

    // MARK: - 详情

    private func heroDetail(_ hero: HeroModel) -> some View {
        let variants = HeroCatalog.starVariants(for: hero, in: data.heroes)
        let raceName = data.raceName(for: hero.species)
        let jobName = data.jobName(for: hero.heroClass)
        let raceTraits = data.traits.filter { $0.checkId == hero.species }
        let jobTraits = data.traits.filter { $0.checkId == hero.heroClass }

        return ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // 头像 + 名称
                heroHeader(hero, raceName: raceName, jobName: jobName)

                Divider().background(Color.white.opacity(0.1))

                // 技能
                skillSection(hero)

                Divider().background(Color.white.opacity(0.1))

                // 多星属性表
                statsTable(variants)

                // 种族详情
                if let race = raceTraits.first {
                    Divider().background(Color.white.opacity(0.1))
                    traitDetailSection(race, title: raceName, isRace: true, icon: race.picture)
                }

                // 职业详情
                if let job = jobTraits.first {
                    Divider().background(Color.white.opacity(0.1))
                    traitDetailSection(job, title: jobName, isRace: false, icon: job.picture)
                }

                // 协同英雄
                let relatedHeroes = data.heroesForTrait(
                    traitId: hero.species, isRace: true
                ).filter { $0.baseKey != hero.baseKey }
                if !relatedHeroes.isEmpty {
                    Divider().background(Color.white.opacity(0.1))
                    synergySection(relatedHeroes)
                }
            }
            .padding(16)
        }
    }

    // MARK: - 头部

    private func heroHeader(_ hero: HeroModel, raceName: String, jobName: String) -> some View {
        HStack(spacing: 14) {
            AsyncImage(url: URL(string: hero.picture)) { img in
                img.resizable().aspectRatio(contentMode: .fit)
            } placeholder: { Color.gray.opacity(0.2) }
            .frame(width: 80, height: 80)
            .clipShape(RoundedRectangle(cornerRadius: 12))
            .background(RoundedRectangle(cornerRadius: 12).fill(Color.white.opacity(0.05)))

            VStack(alignment: .leading, spacing: 6) {
                HStack(spacing: 8) {
                    Text(hero.name).font(.title2).foregroundStyle(.primary)
                    Text("\(hero.cost)费")
                        .font(.system(size: 12, weight: .bold))
                        .foregroundStyle(.white)
                        .padding(.horizontal, 8).padding(.vertical, 3)
                        .background(Color.accentColor).clipShape(Capsule())
                }
                HStack(spacing: 6) {
                    if !raceName.isEmpty { tagBadge(raceName) }
                    if !jobName.isEmpty { tagBadge(jobName) }
                }
            }
            Spacer()
        }
    }

    // MARK: - 技能

    private func skillSection(_ hero: HeroModel) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 10) {
                AsyncImage(url: URL(string: hero.skillIcon)) { img in
                    img.resizable().aspectRatio(contentMode: .fit)
                } placeholder: { Color.clear }
                .frame(width: 42, height: 42)
                .clipShape(RoundedRectangle(cornerRadius: 6))
                .background(RoundedRectangle(cornerRadius: 6).fill(Color.white.opacity(0.05)))

                VStack(alignment: .leading, spacing: 2) {
                    Text("英雄技能").font(.system(size: 10)).foregroundStyle(.tertiary)
                    Text(hero.skillName).font(.headline).foregroundStyle(.primary)
                }
                Spacer()
            }

            Text(hero.skillDesc)
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
                .lineSpacing(4)
                .fixedSize(horizontal: false, vertical: true)

            if !hero.skillValueDesc.isEmpty {
                Text(hero.skillValueDesc)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(.tint)
                    .lineSpacing(2)
            }
        }
    }

    // MARK: - 多星属性表

    private func statsTable(_ variants: [HeroModel]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("属性").font(.headline).foregroundStyle(.primary)

            let statDefs: [(String, KeyPath<HeroModel, String>)] = [
                ("生命", \.initHP), ("物攻", \.initAttackDamage),
                ("攻速", \.attackSpeed), ("护甲", \.armor),
                ("魔抗", \.magicResist), ("射程", \.attackRange),
                ("法力", \.initMP), ("暴击率", \.criticalStrikeChance),
            ]

            ForEach(statDefs, id: \.0) { label, kp in
                HStack(spacing: 0) {
                    Text(label)
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                        .frame(width: 48, alignment: .leading)
                    Spacer()
                    ForEach(variants, id: \.id) { v in
                        Text(v[keyPath: kp])
                            .font(.system(size: 11, design: .monospaced))
                            .foregroundStyle(.primary)
                            .frame(width: 56, alignment: .trailing)
                    }
                }
                .padding(.horizontal, 8).padding(.vertical, 4)
                .background(Color.white.opacity(0.03))
                .clipShape(RoundedRectangle(cornerRadius: 4))
            }
        }
    }

    // MARK: - 种族/职业详情

    private func traitDetailSection(_ trait: TraitModel, title: String, isRace: Bool, icon: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                AsyncImage(url: URL(string: icon)) { img in
                    img.resizable().aspectRatio(contentMode: .fit)
                } placeholder: { Color.gray.opacity(0.2) }
                .frame(width: 28, height: 28).clipShape(RoundedRectangle(cornerRadius: 4))

                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(.headline).foregroundStyle(.primary)
                    Text(isRace ? "种族" : "职业").font(.system(size: 10)).foregroundStyle(.tertiary)
                }
            }

            if !trait.desc.isEmpty {
                Text(trait.desc)
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .lineSpacing(3)
                    .fixedSize(horizontal: false, vertical: true)
            }

            // 阈值
            let levels = data.traitLevels(for: trait.checkId)
            if !levels.isEmpty {
                HStack(spacing: 6) {
                    ForEach(levels) { lv in
                        VStack(spacing: 2) {
                            Text("\(lv.thresholds.first ?? 0)人")
                                .font(.system(size: 11, weight: .bold, design: .monospaced))
                                .foregroundStyle(.primary)
                            if lv.thresholds.count >= 2 {
                                Text("+\(lv.thresholds[1])%")
                                    .font(.system(size: 9))
                                    .foregroundStyle(.tint)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 6)
                        .background(Color.white.opacity(0.04))
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                    }
                }
            }
        }
    }

    // MARK: - 协同英雄

    private func synergySection(_ heroes: [HeroModel]) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("协同英雄").font(.headline).foregroundStyle(.primary)
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 56))], spacing: 6) {
                ForEach(heroes) { h in
                    VStack(spacing: 3) {
                        AsyncImage(url: URL(string: h.picture)) { img in
                            img.resizable().aspectRatio(contentMode: .fit)
                        } placeholder: { Color.gray.opacity(0.2) }
                        .frame(width: 34, height: 34).clipShape(RoundedRectangle(cornerRadius: 5))
                        Text(h.name).font(.system(size: 9)).foregroundStyle(.secondary).lineLimit(1)
                    }
                    .frame(width: 58)
                }
            }
        }
    }

    // MARK: - Helper

    private var emptyHint: some View {
        VStack {
            Image(systemName: "person.2.fill").font(.system(size: 36)).foregroundStyle(.tertiary)
            Text("选择英雄查看详情").font(.system(size: 13)).foregroundStyle(.tertiary)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func tagBadge(_ text: String) -> some View {
        Text(text).font(.system(size: 10)).foregroundStyle(.secondary)
            .padding(.horizontal, 6).padding(.vertical, 2)
            .background(Color.white.opacity(0.06)).clipShape(Capsule())
    }
}

private struct HeroListRow: View {
    let hero: HeroModel; let isSelected: Bool
    var body: some View {
        HStack(spacing: 8) {
            AsyncImage(url: URL(string: hero.picture)) { img in
                img.resizable().aspectRatio(contentMode: .fit)
            } placeholder: { Color.gray.opacity(0.2) }
            .frame(width: 34, height: 34).clipShape(RoundedRectangle(cornerRadius: 5))
            VStack(alignment: .leading, spacing: 1) {
                Text(hero.name).font(.system(size: 11, weight: .medium)).foregroundStyle(.primary)
                Text("HP:\(hero.hp) AD:\(hero.ad)").font(.system(size: 9)).foregroundStyle(.tertiary)
            }
            Spacer()
            Text("\(hero.cost)费").font(.system(size: 9, weight: .bold)).foregroundStyle(.tint)
        }
        .padding(.horizontal, 8).padding(.vertical, 5)
        .background(isSelected ? Color.white.opacity(0.1) : Color.clear)
        .contentShape(Rectangle())
    }
}
