import Foundation

/// LineupRepository 阵容数据仓库
/// 核心职责：
/// - 封装 Rust FFI 调用，隔离数据获取与 UI 展示
/// - 将 Rust JSON 输出解码为 Swift 展示模型
/// - 提供 @MainActor 可观测的数据流

@MainActor
final class LineupRepository: ObservableObject {
    static let shared = LineupRepository()

    @Published var lineups: [LineupCard] = []
    @Published var isLoading = false
    @Published var errorMessage: String?

    private let bridge = RustBridge.shared

    private init() {}

    /// 加载某模式的阵容列表（本地缓存）
    func loadLineups(mode: String) async {
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }

        let rawList = bridge.getLineups(mode: mode)
        if rawList.isEmpty {
            errorMessage = "阵容数据为空"
        }
        lineups = rawList.compactMap { LineupCard(rustDict: $0) }
    }

    /// 获取阵容详情（从 Rust）
    func loadDetail(mode: String, lineupId: String) -> LineupDetailData? {
        guard let raw = bridge.getLineupDetail(mode: mode, lineupId: lineupId) else {
            return nil
        }
        return LineupDetailData(rustPayload: raw)
    }

    /// 获取阵容规则上下文（从 Rust）
    func loadRulesContext(mode: String, lineupId: String) -> [String: Any]? {
        bridge.getLineupRulesContext(mode: mode, lineupId: lineupId)
    }

    /// 获取 Rust 知识决策 RuleOutput
    func loadKnowledgeRuleOutput(mode: String, lineupId: String) -> [String: Any]? {
        bridge.getKnowledgeRuleOutput(mode: mode, lineupId: lineupId)
    }

    /// 刷新远端阵容缓存（Rust 负责 CDN URL 拼装和 HTTP 请求）
    func refreshRemoteCache(mode: String) -> Bool {
        bridge.refreshLineups(mode: mode)
    }

    /// 校验数据对齐
    func validateSnapshot(mode: String) -> [String: Any]? {
        bridge.validateDataSnapshot(mode: mode)
    }

    /// 获取支持的模式列表
    func supportedModes() -> [[String: Any]] {
        bridge.getSupportedModes()
    }
}
