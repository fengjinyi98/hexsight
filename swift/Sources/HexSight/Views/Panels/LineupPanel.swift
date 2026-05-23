import SwiftUI

// MARK: - 阵容列表

/// LineupPanel 阵容推荐面板
/// 核心职责：
/// - 加载并筛选官方阵容推荐数据
/// - 按官网信息密度展示品质、符文、棋子与装备预览
/// - 承载阵容详情页切换
struct LineupPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var lineups: [LineupCard] = []
    @State private var selectedTrait: String?
    @State private var selectedCategory: String?
    @State private var detailLineup: LineupCard?
    @State private var isLoading = false
    @State private var searchQuery: String = ""
    @State private var loadToken = 0

    private let categories = ["新手推荐", "高手进阶", "趣味娱乐"]

    var body: some View {
        ZStack {
            lineupList

            if let card = detailLineup {
                LineupDetailView(card: card) {
                    detailLineup = nil
                }
                .id(card.id)
                .transition(.move(edge: .trailing))
            }
        }
        .animation(.easeInOut(duration: 0.25), value: detailLineup != nil)
    }

    // MARK: - 列表

    private var lineupList: some View {
        VStack(spacing: 0) {
            topFilters
            Divider().background(Color.white.opacity(0.1))

            if filteredLineups.isEmpty {
                emptyView
            } else {
                ScrollView {
                    LazyVStack(spacing: 8) {
                        ForEach(filteredLineups) { card in lineupRow(card) }
                    }
                    .padding(.horizontal, 10)
                    .padding(.top, 10)
                    .padding(.bottom, 32)
                }
            }
        }
        .task { await loadLineups() }
        .onChange(of: data.selectedMode) { _, _ in
            Task { await loadLineups() }
        }
    }

    private var topFilters: some View {
        HStack(alignment: .center, spacing: Theme.Spacing.medium) {
            // 左侧：版本大卡片切换组 (仿官网大卡片设计)
            HStack(spacing: 12) {
                ForEach(data.availableModes, id: \.id) { mode in
                    Button {
                        data.switchMode(mode.id)
                        selectedTrait = nil
                        selectedCategory = nil
                        searchQuery = ""
                    } label: {
                        SeasonTabCard(name: mode.name, id: mode.id, isSelected: data.selectedMode == mode.id)
                    }
                    .buttonStyle(.plain)
                }
            }
            
            Spacer()
            
            // 右侧：大类过滤组 + 下拉筛选 + 搜索
            HStack(spacing: 8) {
                // 大类横向选项卡组
                HStack(spacing: 6) {
                    ForEach(["全部"] + categories, id: \.self) { cat in
                        Button {
                            selectedCategory = cat == "全部" ? nil : cat
                        } label: {
                            Text(cat)
                                .font(.system(size: 11, weight: (selectedCategory == cat || (cat == "全部" && selectedCategory == nil)) ? .bold : .medium))
                                .foregroundStyle((selectedCategory == cat || (cat == "全部" && selectedCategory == nil)) ? Color(red: 0.05, green: 0.05, blue: 0.1) : Theme.Color.textSecondary)
                                .padding(.horizontal, 12)
                                .padding(.vertical, 6)
                                .background(
                                    (selectedCategory == cat || (cat == "全部" && selectedCategory == nil))
                                    ? Theme.Color.gold
                                    : Color.white.opacity(0.04)
                                )
                                .clipShape(RoundedRectangle(cornerRadius: 4))
                                .overlay(
                                    RoundedRectangle(cornerRadius: 4)
                                        .stroke((selectedCategory == cat || (cat == "全部" && selectedCategory == nil)) ? Theme.Color.gold : Color.white.opacity(0.08), lineWidth: 1)
                                )
                        }
                        .buttonStyle(.plain)
                    }
                }
                
                // 下拉羁绊筛选 Menu
                Menu {
                    Button("全部羁绊") { selectedTrait = nil }
                    ForEach(traitFilters, id: \.self) { trait in
                        Button(trait) { selectedTrait = trait }
                    }
                } label: {
                    dropdownLabel(text: selectedTrait == nil ? "筛选" : selectedTrait!)
                }
                .menuStyle(.borderlessButton)
                
                // 模糊搜索输入框 (仿官网，搜索放大镜在右侧)
                HStack(spacing: 6) {
                    TextField("搜索", text: $searchQuery)
                        .textFieldStyle(.plain)
                        .font(.system(size: 11))
                        .foregroundStyle(Theme.Color.textPrimary)
                        .frame(width: 110)
                    
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundStyle(Theme.Color.textSecondary)
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(Color.white.opacity(0.05))
                .clipShape(RoundedRectangle(cornerRadius: 4))
                .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.white.opacity(0.12), lineWidth: 1))
            }
        }
        .padding(.horizontal, 10)
        .padding(.top, 14)
        .padding(.bottom, 10)
    }
    
    private func dropdownLabel(text: String) -> some View {
        HStack(spacing: 6) {
            Text(text)
                .font(.system(size: 11, weight: .medium))
                .foregroundStyle(selectedTrait == nil ? Theme.Color.textSecondary : .white)
            Image(systemName: "chevron.down")
                .font(.system(size: 9, weight: .bold))
                .foregroundStyle(Theme.Color.textSecondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(Color.white.opacity(0.05))
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.white.opacity(0.12), lineWidth: 1))
    }

    private func lineupRow(_ card: LineupCard) -> some View {
        Button { detailLineup = card } label: {
            HStack(spacing: Theme.Spacing.medium) {
                lineupIdentity(card)
                    .frame(width: 210, alignment: .leading)

                HStack(spacing: 12) {
                    QualityBadge(quality: card.quality)
                    augmentPreview(ids: card.augmentIDs)
                }
                .frame(width: 190, alignment: .leading)

                heroPreview(card)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Image(systemName: "chevron.right")
                    .font(Theme.Font.caption.weight(.semibold))
                    .foregroundStyle(Theme.Color.textSecondary)
            }
            .padding(.horizontal, Theme.Spacing.medium)
            .padding(.vertical, 12)
            .background(LineupRowBackground())
            .overlay(
                RoundedRectangle(cornerRadius: Theme.CornerRadius.card)
                    .stroke(Color.white.opacity(0.06), lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
        }
        .buttonStyle(.plain)
    }

    private func lineupIdentity(_ card: LineupCard) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(card.name)
                .font(Theme.Font.title3)
                .foregroundStyle(Theme.Color.textPrimary)
                .lineLimit(2)
                .multilineTextAlignment(.leading)

            HStack(spacing: 6) {
                RemoteIcon(url: card.authorAvatar, size: 26, cornerRadius: 13)
                Text(card.author)
                    .font(Theme.Font.caption)
                    .foregroundStyle(Theme.Color.textSecondary)
                    .lineLimit(1)
                Spacer(minLength: 0)
            }

            if let tag = card.tags.first {
                Text(tag)
                    .font(Theme.Font.micro)
                    .foregroundStyle(Theme.Color.textPrimary)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 3)
                    .background(Color.white.opacity(0.06))
                    .overlay(RoundedRectangle(cornerRadius: 2).stroke(Color.white.opacity(0.12), lineWidth: 1))
            }
        }
    }

    private func augmentPreview(ids: [String]) -> some View {
        HStack(spacing: 8) {
            ForEach(ids, id: \.self) { id in
                if let hex = data.hexes.first(where: { $0.id == id }) {
                    RemoteIcon(url: hex.icon, size: 40, cornerRadius: 20)
                        .overlay(Circle().stroke(Theme.Color.gold.opacity(0.75), lineWidth: 1.4))
                        .help(hex.name)
                }
            }
        }
    }

    private func heroPreview(_ card: LineupCard) -> some View {
        HStack(spacing: 8) {
            ForEach(card.heroPreview) { piece in
                let is3Star = card.detail.level3HeroIDs.contains(piece.heroID)
                LineupHeroChip(
                    piece: piece,
                    mode: data.selectedMode,
                    style: .circle,
                    showName: false,
                    is3Star: is3Star
                )
            }
        }
    }

    // MARK: - 数据

    private var traitFilters: [String] {
        Array(Set(lineups.flatMap(\.traits))).sorted()
    }

    private var filteredLineups: [LineupCard] {
        var result = lineups
        if let cat = selectedCategory { result = result.filter { $0.category == cat } }
        if let trait = selectedTrait { result = result.filter { $0.traits.contains(trait) } }
        
        if !searchQuery.isEmpty {
            result = result.filter { card in
                let nameMatch = card.name.localizedCaseInsensitiveContains(searchQuery)
                let authorMatch = card.author.localizedCaseInsensitiveContains(searchQuery)
                let tagMatch = card.tags.contains { $0.localizedCaseInsensitiveContains(searchQuery) }
                let heroMatch = card.heroPreview.contains { piece in
                    if let hero = data.heroes.first(where: { $0.id == piece.heroID }) {
                        return hero.name.localizedCaseInsensitiveContains(searchQuery)
                    }
                    return false
                }
                return nameMatch || authorMatch || tagMatch || heroMatch
            }
        }
        return result
    }

    private func loadLineups() async {
        let mode = data.selectedMode
        loadToken += 1
        let token = loadToken

        isLoading = true
        detailLineup = nil
        selectedTrait = nil
        selectedCategory = nil
        lineups = []

        if let cached = loadLocalCache(mode: mode), shouldApply(mode: mode, token: token) {
            lineups = cached
        }

        guard LineupCatalog.supportsRemoteFetch(for: mode) else {
            if shouldApply(mode: mode, token: token) {
                isLoading = false
            }
            return
        }

        let fetched = await fetchFromAPI(mode: mode)
        if shouldApply(mode: mode, token: token) {
            if !fetched.isEmpty {
                lineups = fetched
            }
            isLoading = false
        }
    }

    private func loadLocalCache(mode: String) -> [LineupCard]? {
        guard let fileName = LineupCatalog.cacheFileName(for: mode) else { return nil }
        let path = ProjectPaths.lineupDirectory().appendingPathComponent(fileName).path
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
              let payload = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let json = payload["lineup_list"] as? [[String: Any]]
        else { return nil }
        return json.compactMap { LineupCard(dict: $0, rawData: $0) }
    }

    private func fetchFromAPI(mode: String) async -> [LineupCard] {
        guard let url = LineupCatalog.remoteURL(for: mode),
              let (data, _) = try? await URLSession.shared.data(from: url),
              let payload = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let details = payload["lineup_list"] as? [[String: Any]]
        else { return [] }

        let fetched = details.compactMap { LineupCard(dict: $0, rawData: $0) }
        persistRemoteCache(data: data, mode: mode)
        return fetched.sorted { lhs, rhs in
            if lhs.category == rhs.category { return lhs.name < rhs.name }
            return (lhs.category ?? "") < (rhs.category ?? "")
        }
    }

    private func shouldApply(mode: String, token: Int) -> Bool {
        data.selectedMode == mode && loadToken == token
    }

    private func persistRemoteCache(data: Data, mode: String) {
        guard let fileName = LineupCatalog.cacheFileName(for: mode) else { return }
        let url = ProjectPaths.lineupDirectory().appendingPathComponent(fileName)
        try? FileManager.default.createDirectory(
            at: ProjectPaths.lineupDirectory(),
            withIntermediateDirectories: true
        )
        try? data.write(to: url, options: .atomic)
    }

    private var emptyView: some View {
        let state = LineupCatalog.emptyState(for: data.selectedMode)
        return VStack(spacing: 8) {
            if isLoading {
                ProgressView()
                Text("正在加载阵容数据...").font(.system(size: 12)).foregroundStyle(.tertiary)
            } else {
                Image(systemName: "square.grid.2x2").font(.system(size: 32)).foregroundStyle(.tertiary)
                Text(state.title).font(.system(size: 12)).foregroundStyle(.tertiary)
                Text(state.hint).font(.system(size: 10, design: .monospaced)).foregroundStyle(.tertiary)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - 阵容详情

/// LineupDetailView 阵容详情页
/// 核心职责：
/// - 展示最终站位、过渡阵容、推荐符文与装备顺序
/// - 展示官方运营、站位、对位和装备说明
/// - 提供与列表一致的棋子头像和装备图标
private struct LineupDetailView: View {
    let card: LineupCard
    let onClose: () -> Void
    @StateObject private var data = GameDataService.shared

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: Theme.Spacing.medium) {
                header
                Divider().background(Color.white.opacity(0.1))

                sectionTitle("阵容站位")
                finalHeroesRow(card)
                    .padding(.bottom, 4)
                
                if !card.traits.isEmpty {
                    traitOverviewRow
                        .padding(.bottom, 8)
                }
                
                ChessboardView(pieces: card.detail.finalHeroes, mode: data.selectedMode)

                detailIconSection(title: "推荐强化", ids: card.detail.recommendedHexIDs) { id in
                    if let hex = data.hexes.first(where: { $0.id == id }) {
                        IconTextItem(icon: hex.icon, title: hex.name, subtitle: "\(hex.level)级")
                    }
                }

                detailIconSection(title: "可替换强化", ids: card.detail.replacementHexIDs) { id in
                    if let hex = data.hexes.first(where: { $0.id == id }) {
                        IconTextItem(icon: hex.icon, title: hex.name, subtitle: "\(hex.level)级")
                    }
                }

                equipmentAnalysisSection

                if !card.detail.earlyHeroes.isEmpty || !card.detail.midHeroes.isEmpty {
                    transitionSection
                }

                textSection("站位说明", card.detail.locationInfo)
                textSection("强化思路", card.detail.hexInfo)
                textSection("前期过渡", card.detail.earlyInfo)
                textSection("搜牌节奏", card.detail.dTime)
                textSection("克制与变阵", card.detail.enemyInfo)
            }
            .padding(Theme.Spacing.medium)
            .padding(.bottom, 40)
        }
        .background(.ultraThinMaterial)
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Button { onClose() } label: {
                    HStack(spacing: 4) { Image(systemName: "chevron.left"); Text("返回") }
                }
                .buttonStyle(.glass)
                .font(Theme.Font.caption)
                Spacer()
                QualityBadge(quality: card.quality)
            }

            HStack(spacing: 10) {
                RemoteIcon(url: card.authorAvatar, size: 34, cornerRadius: 17)
                VStack(alignment: .leading, spacing: 4) {
                    Text(card.name)
                        .font(Theme.Font.title1)
                        .foregroundStyle(Theme.Color.textPrimary)
                        .fixedSize(horizontal: false, vertical: true)
                    HStack(spacing: 8) {
                        if let tag = card.tags.first { Text(tag).badgeStyle() }
                        Text(card.author)
                            .font(Theme.Font.caption)
                            .foregroundStyle(Theme.Color.textSecondary)
                    }
                }
                Spacer()
            }
        }
    }

    private func finalHeroesRow(_ card: LineupCard) -> some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 12) {
                ForEach(card.detail.finalHeroes) { piece in
                    let is3Star = card.detail.level3HeroIDs.contains(piece.heroID)
                    LineupHeroChip(
                        piece: piece,
                        mode: data.selectedMode,
                        style: .circle,
                        showName: true,
                        is3Star: is3Star
                    )
                }
            }
            .padding(.horizontal, 4)
            .padding(.top, 4)
        }
    }

    private var traitOverviewRow: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(card.traits, id: \.self) { trait in
                    LineupTraitBadge(traitStr: trait)
                }
            }
            .padding(.horizontal, 2)
        }
    }

    private var transitionSection: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.small) {
            sectionTitle("早期过渡")
            
            HStack(alignment: .top, spacing: 16) {
                // 前期过渡
                if !card.detail.earlyHeroes.isEmpty {
                    VStack(alignment: .leading, spacing: Theme.Spacing.small) {
                        Text("前期")
                            .font(Theme.Font.title3)
                            .foregroundStyle(Theme.Color.gold)
                        
                        // 圆形打工英雄一览
                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: 8) {
                                ForEach(card.detail.earlyHeroes) { piece in
                                    let is3Star = card.detail.level3HeroIDs.contains(piece.heroID)
                                    LineupHeroChip(
                                        piece: piece,
                                        mode: data.selectedMode,
                                        style: .circle,
                                        showName: true,
                                        is3Star: is3Star
                                    )
                                }
                            }
                        }
                        
                        ChessboardView(pieces: card.detail.earlyHeroes, mode: data.selectedMode, compact: true)
                    }
                    .padding(12)
                    .background(Theme.Color.cardBackground)
                    .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                    .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.05), lineWidth: 1))
                }

                // 中期过渡
                if !card.detail.midHeroes.isEmpty {
                    VStack(alignment: .leading, spacing: Theme.Spacing.small) {
                        Text("中期")
                            .font(Theme.Font.title3)
                            .foregroundStyle(Theme.Color.accent)
                        
                        // 圆形打工英雄一览
                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: 8) {
                                ForEach(card.detail.midHeroes) { piece in
                                    let is3Star = card.detail.level3HeroIDs.contains(piece.heroID)
                                    LineupHeroChip(
                                        piece: piece,
                                        mode: data.selectedMode,
                                        style: .circle,
                                        showName: true,
                                        is3Star: is3Star
                                    )
                                }
                            }
                        }
                        
                        ChessboardView(pieces: card.detail.midHeroes, mode: data.selectedMode, compact: true)
                    }
                    .padding(12)
                    .background(Theme.Color.cardBackground)
                    .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                    .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.05), lineWidth: 1))
                }
            }
        }
    }

    private var equipmentAnalysisSection: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.small) {
            sectionTitle("装备分析")
            
            VStack(alignment: .leading, spacing: Theme.Spacing.medium) {
                // 1. 抢装顺序
                if !card.detail.equipmentOrderIDs.isEmpty {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("抢装顺序")
                            .font(Theme.Font.caption.weight(.semibold))
                            .foregroundStyle(Theme.Color.textTertiary)
                        HStack(spacing: 8) {
                            ForEach(0..<card.detail.equipmentOrderIDs.count, id: \.self) { idx in
                                let eqId = card.detail.equipmentOrderIDs[idx]
                                if let eq = data.getEquip(eqId) {
                                    HStack(spacing: 6) {
                                        RemoteIcon(url: eq.picture, size: 32, cornerRadius: 4)
                                            .help(eq.name)
                                        if idx < card.detail.equipmentOrderIDs.count - 1 {
                                            Image(systemName: "chevron.right")
                                                .font(.system(size: 11, weight: .bold))
                                                .foregroundStyle(Theme.Color.textTertiary)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 2. 主C装备
                let carries = card.detail.finalHeroes.filter { $0.isCarryHero }
                if !carries.isEmpty {
                    Divider().background(Color.white.opacity(0.05))
                    VStack(alignment: .leading, spacing: 8) {
                        Text("主C装备")
                            .font(Theme.Font.caption.weight(.semibold))
                            .foregroundStyle(Theme.Color.textTertiary)
                        ForEach(carries) { piece in
                            let is3Star = card.detail.level3HeroIDs.contains(piece.heroID)
                            HStack(spacing: 12) {
                                LineupHeroChip(
                                    piece: piece,
                                    mode: data.selectedMode,
                                    style: .circle,
                                    showName: true,
                                    is3Star: is3Star
                                )
                                
                                Text("推荐神装:")
                                    .font(Theme.Font.caption)
                                    .foregroundStyle(Theme.Color.textSecondary)
                                HStack(spacing: 4) {
                                    ForEach(piece.equipmentIDs, id: \.self) { eqId in
                                        if let eq = data.getEquip(eqId) {
                                            RemoteIcon(url: eq.picture, size: 28, cornerRadius: 3)
                                                .help(eq.name)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 3. 其他英雄装备
                let others = card.detail.finalHeroes.filter { !$0.isCarryHero && !$0.equipmentIDs.isEmpty }
                if !others.isEmpty {
                    Divider().background(Color.white.opacity(0.05))
                    VStack(alignment: .leading, spacing: 8) {
                        Text("其他英雄装备")
                            .font(Theme.Font.caption.weight(.semibold))
                            .foregroundStyle(Theme.Color.textTertiary)
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 135), spacing: 8)], spacing: 8) {
                            ForEach(others) { piece in
                                let hero = data.hero(for: piece.heroID, mode: data.selectedMode)
                                HStack(spacing: 10) {
                                    SafeAsyncImage(urlString: hero?.picture ?? "", size: 26, cornerRadius: 13)
                                    HStack(spacing: 3) {
                                        ForEach(piece.equipmentIDs, id: \.self) { eqId in
                                            if let eq = data.getEquip(eqId) {
                                                RemoteIcon(url: eq.picture, size: 18, cornerRadius: 2)
                                            }
                                        }
                                    }
                                }
                                .padding(6)
                                .background(Theme.Color.cardBackground)
                                .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.chip))
                            }
                        }
                    }
                }
                
                // 4. 文字装备分析说明
                if !card.detail.equipmentInfo.isEmpty {
                    Divider().background(Color.white.opacity(0.05))
                    Text(card.detail.equipmentInfo)
                        .font(Theme.Font.body)
                        .foregroundStyle(Theme.Color.textSecondary)
                        .lineSpacing(4)
                        .padding(10)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Theme.Color.cardBackground)
                        .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.chip))
                }
            }
            .padding(12)
            .background(Theme.Color.cardBackground)
            .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
            .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.05), lineWidth: 1))
        }
    }

    private func detailIconSection<Content: View>(
        title: String,
        ids: [String],
        @ViewBuilder item: @escaping (String) -> Content?
    ) -> some View {
        Group {
            if !ids.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    sectionTitle(title)
                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 116), spacing: 8)], spacing: 8) {
                        ForEach(ids, id: \.self) { id in
                            item(id)
                        }
                    }
                }
            }
        }
    }

    private func textSection(_ title: String, _ text: String) -> some View {
        Group {
            if !text.isEmpty {
                VStack(alignment: .leading, spacing: Theme.Spacing.small) {
                    sectionTitle(title)
                    Text(text)
                        .font(Theme.Font.body)
                        .foregroundStyle(Theme.Color.textSecondary)
                        .lineSpacing(5)
                        .fixedSize(horizontal: false, vertical: true)
                        .padding(12)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Theme.Color.cardBackground)
                        .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                }
            }
        }
    }

    private func sectionTitle(_ title: String) -> some View {
        Text(title)
            .font(Theme.Font.title3)
            .foregroundStyle(Theme.Color.textPrimary)
    }
}

// MARK: - 棋盘

/// ChessboardView 阵容棋盘视图
/// 核心职责：
/// - 按金铲铲 4×7 错列棋盘展示站位
/// - 按 hero_id 回查棋子头像与名称
/// - 在棋子下方展示携带装备图标
enum LineupChipStyle {
    case circle
    case hexagon
}

private struct ChessboardView: View {
    let pieces: [LineupPiece]
    let mode: String
    var compact = false

    @State private var containerWidth: CGFloat = 0

    private let rows = [4, 3, 2, 1]
    private let cols = Array(1...7)

    private var activeWidth: CGFloat {
        if containerWidth > 0 {
            return containerWidth
        }
        return compact ? 320.0 : 500.0
    }

    private var cellWidth: CGFloat {
        let maxW = compact ? 360.0 : 550.0
        let effectiveW = min(activeWidth, maxW)
        return effectiveW / (compact ? 8.2 : 7.6)
    }

    private var cellHeight: CGFloat {
        cellWidth * 1.1547
    }

    private var rowSpacing: CGFloat {
        -cellHeight * 0.25
    }

    private var colSpacing: CGFloat { 0 }

    private var board: [String: LineupPiece] {
        Dictionary(uniqueKeysWithValues: pieces.map { ($0.locationKey, $0) })
    }

    var body: some View {
        VStack(spacing: rowSpacing) {
            ForEach(rows, id: \.self) { row in
                let pads = rowPadding(for: row)
                HStack(spacing: colSpacing) {
                    ForEach(cols, id: \.self) { col in
                        let key = "\(row),\(col)"
                        if let piece = board[key] {
                            LineupHeroChip(piece: piece, mode: mode, style: .hexagon, compact: compact, customWidth: cellWidth, customHeight: cellHeight)
                                .frame(width: cellWidth, height: cellHeight)
                        } else {
                            HexTile(fill: Color.white.opacity(0.015))
                                .frame(width: cellWidth, height: cellHeight)
                                .overlay(HexTile(stroke: Color.white.opacity(0.05), lineWidth: 1))
                        }
                    }
                }
                .padding(.leading, pads.leading)
                .padding(.trailing, pads.trailing)
            }
        }
        .padding(compact ? 8 : 12)
        .frame(maxWidth: .infinity, alignment: .center)
        .background(
            GeometryReader { geo in
                Color.clear
                    .onAppear {
                        let w = geo.size.width
                        if w > 0 { containerWidth = w }
                    }
                    .onChange(of: geo.size.width) { _, newWidth in
                        if newWidth > 0 { containerWidth = newWidth }
                    }
            }
        )
        .background(
            LinearGradient(
                colors: [Color.black.opacity(0.18), Color.purple.opacity(0.12)],
                startPoint: .top,
                endPoint: .bottom
            )
        )
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .overlay(RoundedRectangle(cornerRadius: 10).stroke(Color.white.opacity(0.06), lineWidth: 1))
    }

    private func rowPadding(for row: Int) -> (leading: CGFloat, trailing: CGFloat) {
        if row.isMultiple(of: 2) {
            return (cellWidth * 0.5, 0)
        } else {
            return (0, cellWidth * 0.5)
        }
    }
}

private struct LineupHeroChip: View {
    let piece: LineupPiece
    let mode: String
    var style: LineupChipStyle = .circle
    var showName = false
    var is3Star = false
    var compact = false
    var customWidth: CGFloat? = nil
    var customHeight: CGFloat? = nil
    @StateObject private var data = GameDataService.shared

    private var hero: HeroModel? { data.hero(for: piece.heroID, mode: mode) }

    private var chipWidth: CGFloat {
        if let w = customWidth {
            return w
        }
        return style == .circle ? 50 : (compact ? 40 : 50)
    }

    private var chipHeight: CGFloat {
        if let h = customHeight {
            return h
        }
        return style == .circle ? 50 : (compact ? 46 : 58)
    }

    private var avatarWidth: CGFloat {
        style == .circle ? 50 : chipWidth
    }

    private var avatarHeight: CGFloat {
        style == .circle ? 50 : chipHeight
    }

    private var equipSize: CGFloat {
        if style == .circle {
            return 14
        } else {
            return chipWidth * 0.24
        }
    }

    var body: some View {
        VStack(spacing: style == .circle ? 4 : 0) {
            ZStack(alignment: style == .circle ? .topLeading : .bottom) {
                // Circular Avatar or Hexagon Avatar
                if style == .circle {
                    if let urlString = hero?.picture, !urlString.isEmpty {
                        RemoteIcon(url: urlString, size: avatarWidth, cornerRadius: avatarWidth / 2)
                            .overlay(Circle().stroke(piece.isCarryHero ? Theme.Color.gold : Color.white.opacity(0.15), lineWidth: piece.isCarryHero ? 1.8 : 1))
                    } else {
                        Circle()
                            .fill(Color.white.opacity(0.05))
                            .frame(width: avatarWidth, height: avatarWidth)
                            .overlay(Circle().stroke(piece.isCarryHero ? Theme.Color.gold : Color.white.opacity(0.15), lineWidth: piece.isCarryHero ? 1.8 : 1))
                    }

                    // Stars above Circle Avatar
                    HStack(spacing: 1) {
                        ForEach(0..<(is3Star ? 3 : 2), id: \.self) { _ in
                            Image(systemName: "star.fill")
                                .font(.system(size: 6.5))
                                .foregroundStyle(Theme.Color.gold)
                        }
                    }
                    .offset(x: 14, y: -6)

                    // C Badge for carry
                    if piece.isCarryHero {
                        Text("C")
                            .font(.system(size: 8.5, weight: .bold))
                            .foregroundStyle(.white)
                            .frame(width: 15, height: 15)
                            .background(Color.orange)
                            .clipShape(Circle())
                            .overlay(Circle().stroke(Color.white, lineWidth: 1))
                            .offset(x: -2, y: -2)
                    }
                } else {
                    // Hexagon Avatar Group
                    ZStack(alignment: .bottom) {
                        if let urlString = hero?.picture, !urlString.isEmpty {
                            RemoteIcon(url: urlString, width: avatarWidth, height: avatarHeight, cornerRadius: 0)
                                .clipShape(HexagonShape())
                        } else {
                            HexagonShape()
                                .fill(Color.white.opacity(0.05))
                                .frame(width: avatarWidth, height: avatarHeight)
                        }

                        // Hexagon Border
                        HexagonShape()
                            .stroke(piece.isCarryHero ? Theme.Color.gold.opacity(0.95) : Color.white.opacity(0.15), lineWidth: piece.isCarryHero ? 1.6 : 1)
                            .frame(width: avatarWidth, height: avatarHeight)

                        if piece.chessType == "pet" {
                            Image(systemName: "pawprint.fill")
                                .font(.system(size: 6))
                                .foregroundStyle(.white)
                                .padding(2)
                                .background(Color.black.opacity(0.6))
                                .clipShape(Circle())
                                .offset(x: avatarWidth * 0.35, y: -avatarHeight * 0.35)
                        }

                        // Equipment row overlapping the bottom edge
                        HStack(spacing: 1) {
                            ForEach(Array(piece.equipmentIDs.prefix(3)), id: \.self) { id in
                                if let equip = data.getEquip(id) {
                                    RemoteIcon(url: equip.picture, size: equipSize, cornerRadius: 1)
                                        .overlay(RoundedRectangle(cornerRadius: 1).stroke(Color.black.opacity(0.8), lineWidth: 0.5))
                                }
                            }
                        }
                        .offset(y: equipSize * 0.3)
                    }
                    .scaleEffect(0.90)
                }
            }
            .frame(width: avatarWidth, height: avatarHeight)
            .padding(.top, style == .circle ? 6 : 0)

            // Equipment row below Circle Avatar
            if style == .circle {
                HStack(spacing: 1) {
                    let equips = piece.equipmentIDs.compactMap { data.getEquip($0) }
                    if !equips.isEmpty {
                        ForEach(Array(equips.prefix(3))) { equip in
                            RemoteIcon(url: equip.picture, size: equipSize, cornerRadius: 1)
                        }
                    } else {
                        Color.clear.frame(width: avatarWidth, height: equipSize)
                    }
                }
                .frame(height: equipSize)
            }

            // Name text for Circle
            if style == .circle && showName {
                Text(hero?.name ?? "?")
                    .font(Theme.Font.caption)
                    .foregroundStyle(Theme.Color.textSecondary)
                    .lineLimit(1)
            }
        }
        .frame(width: style == .circle ? avatarWidth : nil)
        .help(hero?.name ?? piece.heroID)
    }
}

private struct HexTile: View {
    var fill: Color? = nil
    var stroke: Color? = nil
    var lineWidth: CGFloat = 0

    var body: some View {
        HexagonShape()
            .fill(fill ?? .clear)
            .overlay(
                HexagonShape()
                    .stroke(stroke ?? .clear, lineWidth: lineWidth)
            )
            .scaleEffect(0.90)
    }
}

private struct HexagonShape: Shape {
    func path(in rect: CGRect) -> Path {
        let insetY = rect.height * 0.25
        let midX = rect.midX
        var path = Path()
        path.move(to: CGPoint(x: midX, y: rect.minY))
        path.addLine(to: CGPoint(x: rect.maxX, y: rect.minY + insetY))
        path.addLine(to: CGPoint(x: rect.maxX, y: rect.maxY - insetY))
        path.addLine(to: CGPoint(x: midX, y: rect.maxY))
        path.addLine(to: CGPoint(x: rect.minX, y: rect.maxY - insetY))
        path.addLine(to: CGPoint(x: rect.minX, y: rect.minY + insetY))
        path.closeSubpath()
        return path
    }
}

// MARK: - 通用组件

private struct LineupRowBackground: View {
    var body: some View {
        LinearGradient(
            colors: [
                Color(red: 0.19, green: 0.12, blue: 0.37).opacity(0.86),
                Color(red: 0.12, green: 0.10, blue: 0.22).opacity(0.72),
            ],
            startPoint: .leading,
            endPoint: .trailing
        )
    }
}

private struct QualityBadge: View {
    let quality: String

    var body: some View {
        Text(quality)
            .font(.system(size: 16, weight: .black, design: .rounded))
            .foregroundStyle(textColor)
            .frame(width: 38, height: 42)
            .background(HexTile(fill: fillColor.opacity(0.92)))
            .overlay(HexTile(stroke: Color.white.opacity(0.22), lineWidth: 1))
    }

    private var fillColor: Color {
        switch quality.uppercased() {
        case "S": return .yellow
        case "A": return Color(red: 0.62, green: 0.78, blue: 1.0)
        default: return .gray
        }
    }

    private var textColor: Color {
        quality.uppercased() == "S" ? .black.opacity(0.72) : .white.opacity(0.92)
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

private struct IconTextItem: View {
    let icon: String
    let title: String
    let subtitle: String

    var body: some View {
        HStack(spacing: 8) {
            RemoteIcon(url: icon, size: 28, cornerRadius: 5)
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.system(size: 10, weight: .semibold)).foregroundStyle(.primary).lineLimit(1)
                Text(subtitle).font(.system(size: 8)).foregroundStyle(.tertiary).lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(8)
        .background(Color.white.opacity(0.04))
        .clipShape(RoundedRectangle(cornerRadius: 7))
    }
}

private struct FilterPill: View {
    let text: String
    let isSelected: Bool
    let action: () -> Void

    init(_ text: String, isSelected: Bool, action: @escaping () -> Void) {
        self.text = text
        self.isSelected = isSelected
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            Text(text)
                .font(.system(size: 10, weight: isSelected ? .semibold : .regular))
                .foregroundStyle(isSelected ? .black : .secondary)
                .padding(.horizontal, 10)
                .padding(.vertical, 5)
                .background(isSelected ? Color.yellow.opacity(0.9) : Color.white.opacity(0.06))
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }
}

private extension Text {
    func badgeStyle() -> some View {
        self.font(.system(size: 10))
            .foregroundStyle(.secondary)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(Color.white.opacity(0.06))
            .clipShape(Capsule())
    }
}

// MARK: - 仿官网大卡片设计 SeasonTabCard
private struct SeasonTabCard: View {
    let name: String
    let id: String
    let isSelected: Bool
    
    private var iconName: String {
        switch id {
        case "17": return "sparkles"
        case "4": return "crown.fill"
        case "16": return "shield.fill"
        default: return "star.fill"
        }
    }
    
    private var backgroundGradient: LinearGradient {
        if isSelected {
            switch id {
            case "17": // 星神
                return LinearGradient(
                    colors: [Color(red: 0.2, green: 0.28, blue: 0.38), Color(red: 0.08, green: 0.1, blue: 0.15)],
                    startPoint: .top,
                    endPoint: .bottom
                )
            case "4": // 天选福星
                return LinearGradient(
                    colors: [Color(red: 0.55, green: 0.12, blue: 0.12), Color(red: 0.18, green: 0.04, blue: 0.04)],
                    startPoint: .top,
                    endPoint: .bottom
                )
            case "16": // 英雄联盟传奇
                return LinearGradient(
                    colors: [Color(red: 0.28, green: 0.12, blue: 0.55), Color(red: 0.08, green: 0.04, blue: 0.2)],
                    startPoint: .top,
                    endPoint: .bottom
                )
            default:
                return LinearGradient(
                    colors: [Color(red: 0.2, green: 0.2, blue: 0.25), Color.black],
                    startPoint: .top,
                    endPoint: .bottom
                )
            }
        } else {
            return LinearGradient(
                colors: [Color.white.opacity(0.04), Color.white.opacity(0.01)],
                startPoint: .top,
                endPoint: .bottom
            )
        }
    }
    
    private var borderGradient: LinearGradient {
        if isSelected {
            switch id {
            case "17": // 星神 (Cyan / Silver)
                return LinearGradient(
                    colors: [Color(red: 0.7, green: 0.85, blue: 1.0), Color(red: 0.3, green: 0.45, blue: 0.6)],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            case "4": // 天选福星 (Gold / Orange)
                return LinearGradient(
                    colors: [Color(red: 1.0, green: 0.8, blue: 0.3), Color(red: 0.9, green: 0.2, blue: 0.1)],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            case "16": // 英雄联盟传奇 (Purple / Gold)
                return LinearGradient(
                    colors: [Color(red: 0.85, green: 0.4, blue: 0.95), Color(red: 0.5, green: 0.1, blue: 0.7)],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            default:
                return LinearGradient(
                    colors: [Theme.Color.gold, Color.orange],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            }
        } else {
            return LinearGradient(
                colors: [Color.white.opacity(0.12), Color.white.opacity(0.06)],
                startPoint: .top,
                endPoint: .bottom
            )
        }
    }
    
    private var glowColor: Color {
        guard isSelected else { return .clear }
        switch id {
        case "17": return Color(red: 0.5, green: 0.7, blue: 1.0).opacity(0.3)
        case "4": return Color(red: 1.0, green: 0.6, blue: 0.2).opacity(0.3)
        case "16": return Color(red: 0.8, green: 0.3, blue: 1.0).opacity(0.3)
        default: return .clear
        }
    }
    
    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: iconName)
                .font(.system(size: 11, weight: .bold))
                .foregroundStyle(
                    isSelected ? 
                    (id == "4" ? Color(red: 1.0, green: 0.8, blue: 0.3) : (id == "17" ? Color(red: 0.7, green: 0.95, blue: 1.0) : Color(red: 0.9, green: 0.6, blue: 1.0))) : 
                    Theme.Color.textSecondary
                )
            
            Text(name)
                .font(.system(size: 13, weight: .bold))
                .foregroundStyle(isSelected ? .white : Theme.Color.textSecondary)
                .tracking(1.0)
            
            Image(systemName: iconName)
                .font(.system(size: 11, weight: .bold))
                .foregroundStyle(
                    isSelected ? 
                    (id == "4" ? Color(red: 1.0, green: 0.8, blue: 0.3) : (id == "17" ? Color(red: 0.7, green: 0.95, blue: 1.0) : Color(red: 0.9, green: 0.6, blue: 1.0))) : 
                    Theme.Color.textSecondary
                )
        }
        .frame(width: 140, height: 42)
        .background(
            ZStack {
                backgroundGradient
                
                // Add texture overlay
                RoundedRectangle(cornerRadius: 6)
                    .stroke(borderGradient, lineWidth: isSelected ? 1.5 : 1)
            }
        )
        .clipShape(RoundedRectangle(cornerRadius: 6))
        .shadow(color: glowColor, radius: 8, x: 0, y: 0)
    }
}

// MARK: - 阵容推荐详情页 羁绊徽章 LineupTraitBadge
private struct LineupTraitBadge: View {
    let traitStr: String
    @StateObject private var data = GameDataService.shared
    
    var body: some View {
        let parsed = parseTrait(traitStr)
        let traitModel = data.traits.first(where: { $0.name == parsed.name })
        let tier = traitTier(name: parsed.name, countStr: parsed.count)
        
        HStack(spacing: 0) {
            // Left part: Hexagon emblem with icon
            ZStack {
                HexagonShape()
                    .fill(tier.color)
                    .frame(width: 20, height: 23)
                
                if let trait = traitModel, !trait.picture.isEmpty {
                    RemoteIcon(url: trait.picture, size: 13, cornerRadius: 0)
                        .colorMultiply(.black.opacity(0.85)) // Matches dark emblem icons
                } else {
                    Image(systemName: "shield.fill")
                        .font(.system(size: 8))
                        .foregroundStyle(.black.opacity(0.85))
                }
            }
            .padding(.leading, 1)
            
            // Right part: Text "Count Name"
            HStack(spacing: 4) {
                if !parsed.count.isEmpty {
                    Text(parsed.count)
                        .font(.system(size: 11, weight: .bold))
                        .foregroundStyle(tier == .gold ? Color(red: 0.95, green: 0.77, blue: 0.35) : .white)
                }
                
                Text(parsed.name)
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(Theme.Color.textPrimary)
            }
            .padding(.horizontal, 8)
        }
        .frame(height: 24)
        .background(Color.white.opacity(0.05))
        .clipShape(RoundedRectangle(cornerRadius: 3))
        .overlay(
            RoundedRectangle(cornerRadius: 3)
                .stroke(tier.color.opacity(0.4), lineWidth: 1)
        )
    }
    
    private func parseTrait(_ traitStr: String) -> (count: String, name: String) {
        let trimmed = traitStr.trimmingCharacters(in: .whitespacesAndNewlines)
        if let firstChar = trimmed.first, firstChar.isNumber {
            var numberStr = String(firstChar)
            var remaining = trimmed.dropFirst()
            while let nextChar = remaining.first, nextChar.isNumber {
                numberStr.append(nextChar)
                remaining = remaining.dropFirst()
            }
            let name = remaining.trimmingCharacters(in: .whitespacesAndNewlines)
            return (numberStr, name)
        }
        return ("", trimmed)
    }
    
    private func traitTier(name: String, countStr: String) -> TFTTraitTier {
        guard let count = Int(countStr) else { return .grey }
        let matches = data.traits.filter { $0.name == name }
        guard let trait = matches.first else {
            if count >= 6 { return .gold }
            if count >= 4 { return .silver }
            if count >= 3 { return .bronze }
            return .grey
        }
        
        let thresholds = trait.thresholds.sorted()
        if thresholds.isEmpty {
            return .grey
        }
        
        var activeIndex = -1
        for (idx, thresh) in thresholds.enumerated() {
            if count >= thresh {
                activeIndex = idx
            }
        }
        
        if activeIndex == -1 {
            return .grey
        }
        
        let totalLevels = thresholds.count
        if activeIndex == totalLevels - 1 {
            return .gold
        } else if activeIndex >= totalLevels / 2 {
            return .silver
        } else {
            return .bronze
        }
    }
    
    private enum TFTTraitTier {
        case gold
        case silver
        case bronze
        case grey
        
        var color: Color {
            switch self {
            case .gold: return Color(red: 0.95, green: 0.77, blue: 0.35)
            case .silver: return Color(red: 0.75, green: 0.75, blue: 0.8)
            case .bronze: return Color(red: 0.7, green: 0.45, blue: 0.25)
            case .grey: return Color(red: 0.4, green: 0.4, blue: 0.45)
            }
        }
    }
}
