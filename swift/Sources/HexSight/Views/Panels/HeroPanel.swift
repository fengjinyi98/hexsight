import SwiftUI

/// 英雄图鉴面板
/// 顶部：左侧版本切换卡片，右侧三下拉选项（费用、特质、职业）与搜索框
/// 内容：扑克牌比例紧凑立绘卡片网格
/// 点击：弹出全屏三栏式详情仪表盘
struct HeroPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selectedHero: HeroModel?
    @State private var searchQuery: String = ""
    @State private var selectedCost: Int? = nil
    @State private var selectedRace: String? = nil
    @State private var selectedJob: String? = nil

    private var displayHeroes: [HeroModel] {
        let rawList = HeroCatalog.displayHeroes(from: data.heroes)
        return rawList.filter { hero in
            // 搜索过滤
            if !searchQuery.isEmpty {
                let matchesName = hero.name.localizedCaseInsensitiveContains(searchQuery)
                let matchesSkill = hero.skillName.localizedCaseInsensitiveContains(searchQuery) || hero.skillDesc.localizedCaseInsensitiveContains(searchQuery)
                if !matchesName && !matchesSkill { return false }
            }
            // 费用过滤
            if let cost = selectedCost {
                if hero.cost != cost { return false }
            }
            // 特质种族过滤
            if let race = selectedRace {
                let races = hero.species.split(separator: "|").map(String.init)
                if !races.contains(race) { return false }
            }
            // 职业过滤
            if let job = selectedJob {
                let jobs = hero.heroClass.split(separator: "|").map(String.init)
                if !jobs.contains(job) { return false }
            }
            return true
        }
        .sorted { $0.cost == $1.cost ? $0.name < $1.name : $0.cost < $1.cost }
    }

    var body: some View {
        ZStack {
            VStack(spacing: 0) {
                // 顶部筛选控制栏
                topHeaderView
                
                Divider().background(Color.white.opacity(0.1))

                // 英雄卡片磁贴网格
                ZStack {
                    if displayHeroes.isEmpty {
                        emptyView
                    } else {
                        ScrollView {
                            LazyVGrid(
                                columns: [GridItem(.adaptive(minimum: 110, maximum: 110), spacing: 8)],
                                spacing: 8
                            ) {
                                ForEach(displayHeroes) { hero in
                                    Button {
                                        selectedHero = hero
                                    } label: {
                                        ZStack(alignment: .topLeading) {
                                            // 英雄半身立绘背景
                                            AsyncImage(url: URL(string: hero.picture)) { phase in
                                                switch phase {
                                                case .success(let image):
                                                    image.resizable()
                                                        .aspectRatio(contentMode: .fill)
                                                        .frame(width: 110, height: 176)
                                                        .clipped()
                                                case .failure, .empty:
                                                    Color.white.opacity(0.04)
                                                        .overlay(
                                                            Image(systemName: "person.fill")
                                                                .font(.system(size: 28))
                                                                .foregroundStyle(.secondary)
                                                        )
                                                @unknown default:
                                                    Color.white.opacity(0.04)
                                                }
                                            }
                                            .frame(width: 110, height: 176)
                                            .clipped()
                                            
                                            // 费用金币角标 (Top-Left)
                                            HStack(spacing: 2) {
                                                Image(systemName: "dollarsign.circle.fill")
                                                    .font(.system(size: 8.5))
                                                Text("\(hero.cost)")
                                                    .font(.system(size: 9.5, weight: .bold))
                                            }
                                            .foregroundStyle(.white)
                                            .padding(.horizontal, 6)
                                            .padding(.vertical, 3)
                                            .background(Color.black.opacity(0.65))
                                            .clipShape(Capsule())
                                            .overlay(Capsule().stroke(Color.white.opacity(0.2), lineWidth: 0.5))
                                            .padding(6)
                                            
                                            // 底部名字与阴影 (Bottom)
                                            VStack {
                                                Spacer()
                                                Text(hero.name)
                                                    .font(Theme.Font.body.weight(.bold))
                                                    .foregroundStyle(.white)
                                                    .lineLimit(1)
                                                    .frame(maxWidth: .infinity)
                                                    .padding(.top, 20)
                                                    .padding(.bottom, 8)
                                                    .background(
                                                        LinearGradient(
                                                            colors: [.clear, .black.opacity(0.8)],
                                                            startPoint: .top,
                                                            endPoint: .bottom
                                                        )
                                                    )
                                            }
                                        }
                                    }
                                    .buttonStyle(.plain)
                                    .frame(width: 110, height: 176)
                                    .clipShape(RoundedRectangle(cornerRadius: 8))
                                    .overlay(
                                        RoundedRectangle(cornerRadius: 8)
                                            .stroke(Color.white.opacity(0.08), lineWidth: 1)
                                    )
                                    .shadow(color: Color.black.opacity(0.2), radius: 4, y: 2)
                                }
                            }
                            .padding(.horizontal, 14)
                            .padding(.top, 10)
                            .padding(.bottom, 32)
                        }
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }

            // 详情大仪表盘浮层
            if let hero = selectedHero {
                heroDetailOverlay(hero)
                    .transition(.opacity.combined(with: .scale(scale: 0.95)))
            }
        }
        .animation(.easeInOut(duration: 0.2), value: selectedHero != nil)
    }

    // MARK: - 顶栏

    private var topHeaderView: some View {
        HStack(alignment: .center, spacing: Theme.Spacing.medium) {
            // 左侧：版本药丸切换组
            HStack(spacing: 8) {
                ForEach(data.availableModes, id: \.id) { mode in
                    Button {
                        data.switchMode(mode.id)
                        selectedHero = nil
                        selectedCost = nil
                        selectedRace = nil
                        selectedJob = nil
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
            
            // 右侧：下拉菜单组 + 搜索
            HStack(spacing: 8) {
                // 费用
                Menu {
                    Button("全部费用") { selectedCost = nil }
                    ForEach([1, 2, 3, 4, 5], id: \.self) { cost in
                        Button("\(cost) 费") { selectedCost = cost }
                    }
                } label: {
                    dropdownLabel(text: selectedCost == nil ? "费用" : "\(selectedCost!) 费")
                }
                .menuStyle(.borderlessButton)
                
                // 特质
                Menu {
                    Button("全部特质") { selectedRace = nil }
                    ForEach(data.raceNames.keys.sorted(by: { data.raceNames[$0]! < data.raceNames[$1]! }), id: \.self) { rid in
                        if let name = data.raceNames[rid] {
                            Button(name) { selectedRace = rid }
                        }
                    }
                } label: {
                    dropdownLabel(text: selectedRace == nil ? "特质" : data.raceName(for: selectedRace!))
                }
                .menuStyle(.borderlessButton)

                // 职业
                Menu {
                    Button("全部职业") { selectedJob = nil }
                    ForEach(data.jobNames.keys.sorted(by: { data.jobNames[$0]! < data.jobNames[$1]! }), id: \.self) { jid in
                        if let name = data.jobNames[jid] {
                            Button(name) { selectedJob = jid }
                        }
                    }
                } label: {
                    dropdownLabel(text: selectedJob == nil ? "职业" : data.jobName(for: selectedJob!))
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
                        .frame(width: 100)
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

    private func dropdownLabel(text: String) -> some View {
        HStack(spacing: 4) {
            Text(text)
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

    // MARK: - 详情仪表盘浮层

    private func heroDetailOverlay(_ hero: HeroModel) -> some View {
        ZStack(alignment: .bottom) {
            HStack(alignment: .top, spacing: 20) {
                // 左侧栏：技能、属性、协同
                leftColumn(hero)
                    .frame(maxWidth: .infinity)
                
                // 中间栏：立绘海报与名字
                centerColumn(hero)
                    .frame(width: 320)
                
                // 右侧栏：羁绊卡片
                rightColumn(hero)
                    .frame(maxWidth: .infinity)
            }
            .padding(.horizontal, 24)
            .padding(.top, 24)
            .padding(.bottom, 72) // 避开底部关闭按钮

            // 底部关闭 X 按钮
            Button {
                selectedHero = nil
            } label: {
                Image(systemName: "xmark")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundStyle(Theme.Color.textSecondary)
                    .frame(width: 44, height: 44)
                    .background(Color.white.opacity(0.08))
                    .clipShape(Circle())
                    .overlay(Circle().stroke(Color.white.opacity(0.12), lineWidth: 1))
            }
            .buttonStyle(.plain)
            .padding(.bottom, 16)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(red: 0.05, green: 0.05, blue: 0.12).opacity(0.85))
        .background(.ultraThinMaterial)
    }

    private func leftColumn(_ hero: HeroModel) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                // 1. 技能说明
                VStack(alignment: .leading, spacing: 10) {
                    Text("英雄技能 : \(hero.skillName)")
                        .font(Theme.Font.title3)
                        .foregroundStyle(Theme.Color.gold)
                    
                    HStack(alignment: .top, spacing: 12) {
                        RemoteIcon(url: hero.skillIcon, size: 48, cornerRadius: 6)
                            .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.white.opacity(0.12), lineWidth: 1))
                        
                        VStack(alignment: .leading, spacing: 6) {
                            Text(hero.skillDesc)
                                .font(Theme.Font.body)
                                .foregroundStyle(Theme.Color.textSecondary)
                                .lineSpacing(4)
                                .fixedSize(horizontal: false, vertical: true)
                            
                            if !hero.skillValueDesc.isEmpty {
                                Text(hero.skillValueDesc)
                                    .font(.system(size: 11, design: .monospaced))
                                    .foregroundStyle(Theme.Color.gold.opacity(0.95))
                                    .lineSpacing(2)
                                    .fixedSize(horizontal: false, vertical: true)
                            }
                        }
                    }
                }
                .padding(14)
                .background(Theme.Color.cardBackground)
                .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.06), lineWidth: 1))

                // 2. 详细属性星级表
                VStack(alignment: .leading, spacing: 10) {
                    Text("属性")
                        .font(Theme.Font.title3)
                        .foregroundStyle(Theme.Color.textPrimary)
                    
                    let variants = HeroCatalog.starVariants(for: hero, in: data.heroes)
                    let statDefs: [(String, KeyPath<HeroModel, String>)] = [
                        ("生命", \.initHP), ("物攻", \.initAttackDamage),
                        ("攻速", \.attackSpeed), ("护甲", \.armor),
                        ("魔抗", \.magicResist), ("射程", \.attackRange),
                        ("法力", \.initMP), ("暴击率", \.criticalStrikeChance),
                    ]
                    
                    VStack(spacing: 2) {
                        ForEach(statDefs, id: \.0) { label, kp in
                            HStack(spacing: 0) {
                                Text(label)
                                    .font(Theme.Font.caption)
                                    .foregroundStyle(Theme.Color.textSecondary)
                                    .frame(width: 60, alignment: .leading)
                                Spacer()
                                ForEach(variants, id: \.id) { v in
                                    Text(v[keyPath: kp])
                                        .font(.system(size: 11, design: .monospaced))
                                        .foregroundStyle(Theme.Color.textPrimary)
                                        .frame(width: 52, alignment: .trailing)
                                }
                            }
                            .padding(.horizontal, 10)
                            .padding(.vertical, 6)
                            .background(Color.white.opacity(0.02))
                        }
                    }
                    .clipShape(RoundedRectangle(cornerRadius: 6))
                    .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.white.opacity(0.06), lineWidth: 1))
                }
                .padding(14)
                .background(Theme.Color.cardBackground)
                .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.06), lineWidth: 1))

                // 3. 协同英雄
                VStack(alignment: .leading, spacing: 12) {
                    Text("协同英雄")
                        .font(Theme.Font.title3)
                        .foregroundStyle(Theme.Color.textPrimary)
                    
                    let speciesList = hero.species.split(separator: "|").map(String.init)
                    let classList = hero.heroClass.split(separator: "|").map(String.init)
                    let allTraitIds = speciesList + classList
                    
                    ForEach(allTraitIds, id: \.self) { traitId in
                        let isRace = speciesList.contains(traitId)
                        let name = isRace ? data.raceName(for: traitId) : data.jobName(for: traitId)
                        let related = data.heroesForTrait(traitId: traitId, isRace: isRace)
                            .filter { $0.baseKey != hero.baseKey }
                        
                        if !related.isEmpty {
                            VStack(alignment: .leading, spacing: 8) {
                                HStack(spacing: 8) {
                                    if let tr = data.traits.first(where: { $0.checkId == traitId }) {
                                        RemoteIcon(url: tr.picture, size: 20, cornerRadius: 3)
                                    }
                                    Text(name)
                                        .font(Theme.Font.caption.weight(.semibold))
                                        .foregroundStyle(Theme.Color.gold)
                                    Text(isRace ? "种族" : "职业")
                                        .font(Theme.Font.micro)
                                        .foregroundStyle(Theme.Color.textTertiary)
                                }
                                
                                ScrollView(.horizontal, showsIndicators: false) {
                                    HStack(spacing: 6) {
                                        ForEach(related) { h in
                                            VStack(spacing: 3) {
                                                RemoteIcon(url: h.picture, size: 26, cornerRadius: 13)
                                                    .overlay(Circle().stroke(Color.white.opacity(0.12), lineWidth: 1))
                                                Text(h.name)
                                                    .font(Theme.Font.micro)
                                                    .foregroundStyle(Theme.Color.textSecondary)
                                                    .lineLimit(1)
                                            }
                                            .frame(width: 36)
                                        }
                                    }
                                }
                            }
                            .padding(8)
                            .background(Color.white.opacity(0.03))
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                        }
                    }
                }
                .padding(14)
                .background(Theme.Color.cardBackground)
                .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.06), lineWidth: 1))
            }
            .padding(.trailing, 2)
        }
    }

    private func centerColumn(_ hero: HeroModel) -> some View {
        VStack(spacing: 0) {
            Spacer()
            
            VStack(spacing: 0) {
                ZStack(alignment: .bottom) {
                    // 大立绘海报
                    RemoteIcon(url: hero.picture, width: 280, height: 280, cornerRadius: 12)
                        .overlay(RoundedRectangle(cornerRadius: 12).stroke(Theme.Color.gold.opacity(0.6), lineWidth: 1.5))
                        .shadow(color: Color.black.opacity(0.4), radius: 8, y: 4)
                    
                    // 悬浮小圆形头像
                    RemoteIcon(url: hero.picture, size: 54, cornerRadius: 27)
                        .overlay(Circle().stroke(Theme.Color.gold, lineWidth: 2))
                        .shadow(radius: 4)
                        .offset(y: 27)
                }
                
                Text(hero.name)
                    .font(Theme.Font.title1.weight(.black))
                    .foregroundStyle(Theme.Color.textPrimary)
                    .padding(.top, 40)
            }
            
            Spacer()
        }
    }

    private func rightColumn(_ hero: HeroModel) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                Text("羁绊效果")
                    .font(Theme.Font.title3)
                    .foregroundStyle(Theme.Color.textPrimary)
                    .padding(.leading, 4)
                
                let speciesList = hero.species.split(separator: "|").map(String.init)
                let classList = hero.heroClass.split(separator: "|").map(String.init)
                let allTraitIds = speciesList + classList
                
                ForEach(allTraitIds, id: \.self) { traitId in
                    let isRace = speciesList.contains(traitId)
                    let levels = data.traitLevels(for: traitId)
                    
                    if let trait = data.traits.first(where: { $0.checkId == traitId }) {
                        VStack(alignment: .leading, spacing: 10) {
                            // 标题栏
                            HStack {
                                RemoteIcon(url: trait.picture, size: 24, cornerRadius: 4)
                                Text(trait.name)
                                    .font(Theme.Font.body.weight(.bold))
                                    .foregroundStyle(Theme.Color.textPrimary)
                                Spacer()
                                Text(isRace ? "种族" : "职业")
                                    .font(Theme.Font.caption.weight(.semibold))
                                    .foregroundStyle(Theme.Color.gold)
                            }
                            
                            Divider().background(Color.white.opacity(0.08))
                            
                            // 羁绊描述
                            Text(trait.desc)
                                .font(Theme.Font.caption)
                                .foregroundStyle(Theme.Color.textSecondary)
                                .lineSpacing(3)
                                .fixedSize(horizontal: false, vertical: true)
                            
                            // 人数分级详情
                            if !levels.isEmpty {
                                VStack(alignment: .leading, spacing: 6) {
                                    ForEach(levels) { lv in
                                        HStack(alignment: .top, spacing: 8) {
                                            Text("\(lv.num)人")
                                                .font(.system(size: 11, weight: .bold, design: .monospaced))
                                                .foregroundStyle(Theme.Color.gold)
                                                .frame(width: 36, alignment: .leading)
                                            
                                            Text(lv.realDesc)
                                                .font(Theme.Font.caption)
                                                .foregroundStyle(Theme.Color.textSecondary)
                                                .fixedSize(horizontal: false, vertical: true)
                                        }
                                        .padding(6)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                        .background(Color.white.opacity(0.02))
                                        .clipShape(RoundedRectangle(cornerRadius: 4))
                                    }
                                }
                            }
                        }
                        .padding(14)
                        .background(Theme.Color.cardBackground)
                        .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                        .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.06), lineWidth: 1))
                    }
                }
            }
            .padding(.trailing, 2)
        }
    }

    private var emptyView: some View {
        VStack(spacing: 8) {
            Image(systemName: "person.fill").font(.system(size: 32)).foregroundStyle(.tertiary)
            Text("没有找到符合条件的英雄").font(Theme.Font.body).foregroundStyle(.tertiary)
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
