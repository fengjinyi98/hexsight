import Foundation
import SwiftUI

/// 阵容攻略面板（对齐官网布局）
/// 顶部：筛选标签（羁绊分类）
/// 内容：阵容卡片网格
struct LineupPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var lineups: [LineupCard] = []
    @State private var selectedTrait: String? = nil
    @State private var expandedLineup: LineupCard?

    var body: some View {
        VStack(spacing: 0) {
            // 筛选标签
            if !traitFilters.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 4) {
                        FilterPill("全部", isSelected: selectedTrait == nil) {
                            selectedTrait = nil
                        }
                        ForEach(traitFilters, id: \.self) { trait in
                            FilterPill(trait, isSelected: selectedTrait == trait) {
                                selectedTrait = selectedTrait == trait ? nil : trait
                            }
                        }
                    }
                    .padding(.horizontal, 10)
                }
                .padding(.vertical, 8)
                Divider().background(Color.white.opacity(0.1))
            }

            // 卡片网格
            if filteredLineups.isEmpty {
                VStack(spacing: 8) {
                    Image(systemName: "square.grid.2x2").font(.system(size: 32)).foregroundStyle(.tertiary)
                    Text("加载中...").font(.system(size: 12)).foregroundStyle(.tertiary)
                    Text("运行 python3 scripts/jcc_api.py fetch-lineups")
                        .font(.system(size: 10, design: .monospaced)).foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 200), spacing: 8)], spacing: 8) {
                        ForEach(filteredLineups) { card in
                            lineupCard(card)
                        }
                    }
                    .padding(10)
                }
            }
        }
        .onAppear { loadLineups() }
    }

    // MARK: - 数据

    private var traitFilters: [String] {
        let allTraits = Set(lineups.flatMap { $0.traits })
        return allTraits.sorted()
    }

    private var filteredLineups: [LineupCard] {
        guard let trait = selectedTrait else { return lineups }
        return lineups.filter { $0.traits.contains(trait) }
    }

    private func loadLineups() {
        let path = ProjectPaths.lineupDirectory().appendingPathComponent("raw_S18.json").path
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]]
        else { return }

        lineups = json.compactMap { LineupCard(dict: $0) }
    }

    // MARK: - 卡片

    private func lineupCard(_ card: LineupCard) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            // 阵容名
            Text(card.name)
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(.primary)
                .lineLimit(2)

            // 羁绊标签
            if !card.traits.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 3) {
                        ForEach(card.traits.prefix(4), id: \.self) { t in
                            Text(t)
                                .font(.system(size: 8))
                                .foregroundStyle(.secondary)
                                .padding(.horizontal, 4).padding(.vertical, 1)
                                .background(Color.white.opacity(0.06))
                                .clipShape(Capsule())
                        }
                    }
                }
            }

            Spacer(minLength: 4)

            // 作者 + 标签
            HStack {
                Text(card.author)
                    .font(.system(size: 9))
                    .foregroundStyle(.tertiary)
                    .lineLimit(1)
                Spacer()
                if !card.tags.isEmpty {
                    Text(card.tags.first ?? "")
                        .font(.system(size: 8))
                        .foregroundStyle(.tint)
                        .padding(.horizontal, 4).padding(.vertical, 1)
                        .background(Color.accentColor.opacity(0.1))
                        .clipShape(Capsule())
                }
            }

            // 胜率 + 详情按钮
            HStack {
                if card.top4Rate > 0 {
                    HStack(spacing: 4) {
                        Text("前四率")
                            .font(.system(size: 8)).foregroundStyle(.tertiary)
                        Text(String(format: "%.1f%%", card.top4Rate))
                            .font(.system(size: 10, weight: .bold, design: .monospaced))
                            .foregroundStyle(.green)
                    }
                }
                Spacer()
                Button("查看详情") {
                    expandedLineup = expandedLineup?.id == card.id ? nil : card
                }
                .buttonStyle(.glass)
                .font(.system(size: 9))
            }
        }
        .padding(10)
        .background(Color.white.opacity(0.04))
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }
}

// MARK: - 阵容卡片模型

private struct LineupCard: Identifiable {
    let id: String
    let name: String
    let author: String
    let traits: [String]
    let tags: [String]
    let top4Rate: Double
    let rank: Double

    init?(dict: [String: Any]) {
        let rawId = lineupStringValue(dict["id"]).trimmingCharacters(in: .whitespacesAndNewlines)
        let queueId = lineupStringValue(dict["queue_id"]).trimmingCharacters(in: .whitespacesAndNewlines)
        self.name = dict["name"] as? String ?? "未知阵容"
        self.id = [rawId, queueId, self.name].first(where: { !$0.isEmpty }) ?? UUID().uuidString

        let authorData = dict["lineupauthor_data"] as? [String: Any]
        self.author = (dict["author"] as? String).flatMap { $0.isEmpty ? nil : $0 }
            ?? authorData?["name"] as? String
            ?? "未知作者"

        let detailRaw = dict["detail"] as? String ?? "{}"
        let detail = (try? JSONSerialization.jsonObject(with: Data(detailRaw.utf8))) as? [String: Any]
        let contacts = detail?["contact"] as? [[String: Any]] ?? []
        self.traits = contacts.compactMap { $0["name"] as? String }

        let smarTag = detail?["smar_lineup_tag"] as? [String: Any]
        let allTag = smarTag?["all"] as? [String: Any]
        if let tags = allTag?["tag"] as? [String] {
            self.tags = tags
        } else if let tag = allTag?["tag"] as? String, !tag.isEmpty {
            self.tags = [tag]
        } else {
            self.tags = []
        }

        let rate = allTag?["rate"] as? [String: Any]
        self.top4Rate = (rate?["top4_rate"] as? Double ?? 0) * 100
        self.rank = rate?["rank"] as? Double ?? 0
    }
}

private func lineupStringValue(_ val: Any?) -> String {
    if let s = val as? String { return s }
    if let i = val as? Int { return String(i) }
    if let d = val as? Double { return String(d) }
    return ""
}

// MARK: - 筛选标签

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
                .foregroundStyle(isSelected ? .white : .secondary)
                .padding(.horizontal, 8).padding(.vertical, 4)
                .background(isSelected ? Color.accentColor : Color.white.opacity(0.06))
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }
}
