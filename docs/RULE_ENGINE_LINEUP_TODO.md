# 阵容库与规则引擎未完成 TODO

## 目标

对齐 PRD「模块4：版本答案阵容库与规则引擎」，单独收口阵容库与规则引擎当前开发阶段的未完成项。当前阶段不保留旧入口兼容目标，统一向新知识规则链路收敛。

## 当前结论

| 区域 | 当前状态 | 主要缺口 | 优先级 |
|---|---|---|---|
| 新知识规则链路 | 已较完整 | 需要逐步接入真实识别上下文 | P0 |
| `KnowledgeDecisionPlanner` | 已输出 `RuleOutput.knowledgeActions` | 需要更多真实局面回归样例 | P0 |
| FFI 上下文入口 | 已支持当前局面 JSON | 识别层还未稳定填入棋盘、装备席、候选海克斯 | P0 |
| Swift 决策展示 | 已消费 Rust `RuleOutput` 摘要 | 需要从手动/默认阵容切到真实识别阵容 | P0 |
| `DecisionEngine` 骨架 | 待移除或并入新链路 | `decide()` 只设置 source，不能作为开发目标 | P1 |
| `RulesEngine` 骨架 | 待移除或并入新链路 | 开局选阵、升人口、装备补救已有新评分器替代方向 | P1 |
| `LineupDB` 骨架 | 待移除或并入新链路 | 阵容加载与推荐应统一使用 `LineupLoader` / `LineupAdapter` / `LineupFitScorer` | P1 |

## P0 必做项：新知识链路接入真实局面

| TODO | 文件 / 区域 | 完成标准 |
|---|---|---|
| 识别层填充 `KnowledgeRuleInput` | `crates/hexsight-ffi/src/data_ffi.rs`、Swift 调用上下文 | 当前棋盘、装备、散件、海克斯、经济、血量、等级、同行数据进入上下文入口 |
| 决策面板使用真实阵容 ID | `swift/Sources/HexSight/App/AppState.swift`、`DecisionPanel.swift` | 未识别阵容时保持空态；识别到阵容后调用 context 入口 |
| 扩充真实局面回归 | `crates/hexsight-engine/tests/knowledge_decision_p7.rs`、`config/rules/*/regression_cases/` | 至少覆盖连胜、连败、低血、装备错配、同行高风险、海克斯刷新 |
| 字段化规则输出 | `hexsight-core/src/rule_types.rs` | 装备、经济、海克斯、转向条件均有结构化字段，UI 少依赖文本解析 |
| 版本知识校准闭环 | `config/rules/S18.1/*` | 版本公告、装备冲突、补丁修正能通过回归样例验证 |

## P1 必做项：删除骨架入口，避免双轨

| TODO | 文件 / 区域 | 完成标准 |
|---|---|---|
| 移除 `DecisionEngine` 骨架依赖 | `crates/hexsight-engine/src/decision.rs`、`crates/hexsight-ffi/src/lib.rs` | 实时决策入口改为新 `RuleOutput` 链路，旧 `Decision` 路径不再承载业务 |
| 移除 `RulesEngine` 骨架语义 | `crates/hexsight-engine/src/rules.rs` | 开局、经济、装备补救统一由现有评分器和 `KnowledgeDecisionPlanner` 承接 |
| 移除 `LineupDB` 骨架语义 | `crates/hexsight-engine/src/lineup.rs` | 阵容读取统一由 `LineupLoader` / `LineupAdapter`，推荐统一由 `LineupFitScorer` |
| 清理旧 `Decision` 输出类型使用 | `crates/hexsight-core/src/types.rs`、`crates/hexsight-ffi/src/lib.rs` | 对外实时输出统一为 `RuleOutput` 或明确的开发态空实现 |
| 去掉重复规则语义 | `rules.rs`、`lineup_fit_scorer.rs`、`knowledge_decision_planner.rs` | 阵容评分、经济节奏、装备补救只保留一个事实来源 |

## P2 后续项：规则质量提升

| TODO | 范围 | 完成标准 |
|---|---|---|
| 同行识别策略接入 | 对手阵容识别、`rival_counts` | 同行数进入阵容评分和转向条件 |
| 过渡质量评分实战校准 | `TransitionLineupMatcher`、`BoardPowerScorer` | 能解释 2-1、2-5、3-2、4-1 节点动作 |
| 海克斯复杂选择校准 | `AugmentEffectInterpreter`、`AugmentOptionRanker` | 三个候选海克斯输出拿/刷理由和风险 |
| 装备承载者闭环 | `HolderScorer`、`BenchTransitionScorer` | 当前装给谁、何时转给主 C 可解释 |
| 性能基准 | FFI + engine 规则链路 | 单次规则输出满足 PRD 端到端 ≤100ms 预算 |

## 骨架入口当前待清理明细

| 文件 | TODO | 建议处理 |
|---|---|---|
| `crates/hexsight-engine/src/decision.rs` | `decide()` 未组装阵容推荐、操作建议、同行风险、装备路线 | 删除业务 TODO，把实时入口接到 `KnowledgeDecisionPlanner` |
| `crates/hexsight-engine/src/rules.rs` | `classify_opening()` 装备分类未实现 | 用 `LineupFitScorer` / 装备画像替代 |
| `crates/hexsight-engine/src/rules.rs` | `level_advice()` 人口节奏未实现 | 用 `EconomyPlanner` 替代 |
| `crates/hexsight-engine/src/rules.rs` | `equip_remedy()` 装备补救未实现 | 用 `ChampionItemFitScorer` / `HolderScorer` 替代 |
| `crates/hexsight-engine/src/lineup.rs` | `load_from_dir()` 目录加载未实现 | 用 `LineupLoader::load_cached_lineups` 替代 |
| `crates/hexsight-engine/src/lineup.rs` | `load_config()` 单阵容解析未实现 | 用 `LineupAdapter::card_from_raw` 替代 |
| `crates/hexsight-engine/src/lineup.rs` | `recommend()` 推荐排序未实现 | 用 `LineupFitScorer::score_all_with_item_context` 替代 |

## 关联现有资料

| 文档 / 文件 | 用途 |
|---|---|
| `docs/RULE_ENGINE_FINAL_TARGET.md` | 规则引擎最终目标 |
| `docs/RUST_BACKEND_FINAL_TARGET.md` | Rust 后端化边界 |
| `docs/规则阶段的补充/CHAMPION_ITEM_PATCH_整体收口审查.md` | 新知识链路阶段收口结论 |
| `crates/hexsight-engine/src/knowledge_decision_planner.rs` | 当前推荐的主规则输出聚合器 |
| `crates/hexsight-ffi/src/data_ffi.rs` | 当前 RuleOutput FFI 上下文入口 |
| `swift/Sources/HexSight/Services/LineupRepository.swift` | Swift 侧 RuleOutput 数据入口 |

## 验收命令

| 检查项 | 命令 | 通过标准 |
|---|---|---|
| Rust 规则测试 | `cargo test -p hexsight-engine` | 0 failures |
| FFI 规则输出测试 | `cargo test -p hexsight-ffi test_knowledge_rule_output_uses_real_p1_to_p6_chain` | 0 failures |
| Rust 全量测试 | `cargo test --workspace` | 0 failures |
| Swift 决策展示测试 | `cd swift && swift test --filter LineupRepositoryTests` | 0 failures |
