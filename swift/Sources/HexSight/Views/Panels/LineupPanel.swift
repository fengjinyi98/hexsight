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
                    .padding(10)
                }
            }
        }
        .task { await loadLineups() }
        .onChange(of: data.selectedMode) { _, _ in
            Task { await loadLineups() }
        }
    }

    private var topFilters: some View {
        VStack(spacing: 6) {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 6) {
                    FilterPill("全部", isSelected: selectedCategory == nil) { selectedCategory = nil }
                    ForEach(categories, id: \.self) { cat in
                        FilterPill(cat, isSelected: selectedCategory == cat) {
                            selectedCategory = selectedCategory == cat ? nil : cat
                        }
                    }
                }
                .padding(.horizontal, 10)
            }

            if !traitFilters.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 4) {
                        ForEach(traitFilters, id: \.self) { trait in
                            FilterPill(trait, isSelected: selectedTrait == trait) {
                                selectedTrait = selectedTrait == trait ? nil : trait
                            }
                        }
                    }
                    .padding(.horizontal, 10)
                }
            }
        }
        .padding(.vertical, 8)
    }

    private func lineupRow(_ card: LineupCard) -> some View {
        Button { detailLineup = card } label: {
            HStack(spacing: 14) {
                lineupIdentity(card)
                    .frame(width: 210, alignment: .leading)

                HStack(spacing: 10) {
                    QualityBadge(quality: card.quality)
                    augmentPreview(ids: card.augmentIDs)
                }
                .frame(width: 150, alignment: .leading)

                heroPreview(card)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Image(systemName: "chevron.right")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
            .background(LineupRowBackground())
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Color.white.opacity(0.06), lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 4))
        }
        .buttonStyle(.plain)
    }

    private func lineupIdentity(_ card: LineupCard) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(card.name)
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(.primary)
                .lineLimit(2)
                .multilineTextAlignment(.leading)

            HStack(spacing: 6) {
                RemoteIcon(url: card.authorAvatar, size: 20, cornerRadius: 10)
                Text(card.author)
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                Spacer(minLength: 0)
            }

            if let tag = card.tags.first {
                Text(tag)
                    .font(.system(size: 9))
                    .foregroundStyle(.white.opacity(0.86))
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
                    RemoteIcon(url: hex.icon, size: 34, cornerRadius: 17)
                        .overlay(Circle().stroke(Color.yellow.opacity(0.75), lineWidth: 1.4))
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
            VStack(alignment: .leading, spacing: 16) {
                header
                Divider().background(Color.white.opacity(0.1))

                sectionTitle("阵容站位")
                finalHeroesRow(card)
                    .padding(.bottom, 4)
                ChessboardView(pieces: card.detail.finalHeroes, mode: data.selectedMode)

                if !card.traits.isEmpty {
                    traitSection
                }

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
            .padding(16)
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
                .font(.system(size: 11))
                Spacer()
                QualityBadge(quality: card.quality)
            }

            HStack(spacing: 10) {
                RemoteIcon(url: card.authorAvatar, size: 34, cornerRadius: 17)
                VStack(alignment: .leading, spacing: 4) {
                    Text(card.name)
                        .font(.system(size: 18, weight: .bold))
                        .foregroundStyle(.primary)
                        .fixedSize(horizontal: false, vertical: true)
                    HStack(spacing: 8) {
                        if let tag = card.tags.first { Text(tag).badgeStyle() }
                        Text(card.author).font(.system(size: 11)).foregroundStyle(.secondary)
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

    private var traitSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionTitle("羁绊组成")
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 72), spacing: 6)], spacing: 6) {
                ForEach(card.traits, id: \.self) { trait in
                    Text(trait)
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .frame(maxWidth: .infinity)
                        .background(Color.white.opacity(0.05))
                        .clipShape(Capsule())
                }
            }
        }
    }

    private var transitionSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            sectionTitle("早期过渡")
            
            HStack(alignment: .top, spacing: 16) {
                // 前期过渡
                if !card.detail.earlyHeroes.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("前期").font(.system(size: 12, weight: .bold)).foregroundStyle(.yellow)
                        
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
                    .padding(10)
                    .background(Color.white.opacity(0.02))
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.white.opacity(0.05), lineWidth: 1))
                }

                // 中期过渡
                if !card.detail.midHeroes.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("中期").font(.system(size: 12, weight: .bold)).foregroundStyle(.purple)
                        
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
                    .padding(10)
                    .background(Color.white.opacity(0.02))
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.white.opacity(0.05), lineWidth: 1))
                }
            }
        }
    }

    private var equipmentAnalysisSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            sectionTitle("装备分析")
            
            VStack(alignment: .leading, spacing: 12) {
                // 1. 抢装顺序
                if !card.detail.equipmentOrderIDs.isEmpty {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("抢装顺序").font(.system(size: 10, weight: .semibold)).foregroundStyle(.tertiary)
                        HStack(spacing: 6) {
                            ForEach(0..<card.detail.equipmentOrderIDs.count, id: \.self) { idx in
                                let eqId = card.detail.equipmentOrderIDs[idx]
                                if let eq = data.getEquip(eqId) {
                                    HStack(spacing: 4) {
                                        RemoteIcon(url: eq.picture, size: 24, cornerRadius: 4)
                                            .help(eq.name)
                                        if idx < card.detail.equipmentOrderIDs.count - 1 {
                                            Image(systemName: "chevron.right").font(.system(size: 8)).foregroundStyle(.secondary)
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
                    VStack(alignment: .leading, spacing: 6) {
                        Text("主C装备").font(.system(size: 10, weight: .semibold)).foregroundStyle(.tertiary)
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
                                
                                Text("推荐神装:").font(.system(size: 10)).foregroundStyle(.secondary)
                                HStack(spacing: 4) {
                                    ForEach(piece.equipmentIDs, id: \.self) { eqId in
                                        if let eq = data.getEquip(eqId) {
                                            RemoteIcon(url: eq.picture, size: 22, cornerRadius: 3)
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
                    VStack(alignment: .leading, spacing: 6) {
                        Text("其他英雄装备").font(.system(size: 10, weight: .semibold)).foregroundStyle(.tertiary)
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 120), spacing: 8)], spacing: 8) {
                            ForEach(others) { piece in
                                let hero = data.hero(for: piece.heroID, mode: data.selectedMode)
                                HStack(spacing: 8) {
                                    SafeAsyncImage(urlString: hero?.picture ?? "", size: 20, cornerRadius: 10)
                                    HStack(spacing: 2) {
                                        ForEach(piece.equipmentIDs, id: \.self) { eqId in
                                            if let eq = data.getEquip(eqId) {
                                                RemoteIcon(url: eq.picture, size: 14, cornerRadius: 2)
                                            }
                                        }
                                    }
                                }
                                .padding(4)
                                .background(Color.white.opacity(0.03))
                                .clipShape(RoundedRectangle(cornerRadius: 6))
                            }
                        }
                    }
                }
                
                // 4. 文字装备分析说明
                if !card.detail.equipmentInfo.isEmpty {
                    Divider().background(Color.white.opacity(0.05))
                    Text(card.detail.equipmentInfo)
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                        .lineSpacing(4)
                        .padding(8)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.white.opacity(0.025))
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                }
            }
            .padding(10)
            .background(Color.white.opacity(0.02))
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.white.opacity(0.05), lineWidth: 1))
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
                VStack(alignment: .leading, spacing: 6) {
                    sectionTitle(title)
                    Text(text)
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                        .lineSpacing(4)
                        .fixedSize(horizontal: false, vertical: true)
                        .padding(10)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.white.opacity(0.035))
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                }
            }
        }
    }

    private func sectionTitle(_ title: String) -> some View {
        Text(title)
            .font(.system(size: 13, weight: .bold))
            .foregroundStyle(.primary)
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
        return style == .circle ? 40 : (compact ? 40 : 50)
    }

    private var chipHeight: CGFloat {
        if let h = customHeight {
            return h
        }
        return style == .circle ? 40 : (compact ? 46 : 58)
    }

    private var avatarWidth: CGFloat {
        style == .circle ? 40 : chipWidth
    }

    private var avatarHeight: CGFloat {
        style == .circle ? 40 : chipHeight
    }

    private var equipSize: CGFloat {
        if style == .circle {
            return 10
        } else {
            return chipWidth * 0.24
        }
    }

    var body: some View {
        VStack(spacing: style == .circle ? 2 : 0) {
            ZStack(alignment: style == .circle ? .topLeading : .bottom) {
                // Circular Avatar or Hexagon Avatar
                if style == .circle {
                    if let urlString = hero?.picture, !urlString.isEmpty {
                        RemoteIcon(url: urlString, size: avatarWidth, cornerRadius: avatarWidth / 2)
                            .overlay(Circle().stroke(piece.isCarryHero ? Color.yellow : Color.white.opacity(0.15), lineWidth: piece.isCarryHero ? 1.8 : 1))
                    } else {
                        Circle()
                            .fill(Color.white.opacity(0.05))
                            .frame(width: avatarWidth, height: avatarWidth)
                            .overlay(Circle().stroke(piece.isCarryHero ? Color.yellow : Color.white.opacity(0.15), lineWidth: piece.isCarryHero ? 1.8 : 1))
                    }

                    // Stars above Circle Avatar
                    HStack(spacing: 1) {
                        ForEach(0..<(is3Star ? 3 : 2), id: \.self) { _ in
                            Image(systemName: "star.fill")
                                .font(.system(size: 6))
                                .foregroundStyle(.yellow)
                        }
                    }
                    .offset(x: 10, y: -6)

                    // C Badge for carry
                    if piece.isCarryHero {
                        Text("C")
                            .font(.system(size: 7, weight: .bold))
                            .foregroundStyle(.white)
                            .frame(width: 12, height: 12)
                            .background(Color.orange)
                            .clipShape(Circle())
                            .overlay(Circle().stroke(Color.white, lineWidth: 1))
                            .offset(x: -2, y: -2)
                    }
                } else {
                    // Hexagon Avatar
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
                        .stroke(piece.isCarryHero ? Color.yellow.opacity(0.95) : Color.white.opacity(0.15), lineWidth: piece.isCarryHero ? 1.6 : 1)
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
            }
            .frame(width: avatarWidth, height: avatarHeight)
            .padding(.top, style == .circle ? 6 : 0)

            // Equipment row below Circle Avatar
            if style == .circle && !piece.equipmentIDs.isEmpty {
                HStack(spacing: 1) {
                    ForEach(Array(piece.equipmentIDs.prefix(3)), id: \.self) { id in
                        if let equip = data.getEquip(id) {
                            RemoteIcon(url: equip.picture, size: equipSize, cornerRadius: 1)
                        }
                    }
                }
                .frame(height: equipSize)
            }

            // Name text for Circle
            if style == .circle && showName {
                Text(hero?.name ?? "?")
                    .font(.system(size: 8))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
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
