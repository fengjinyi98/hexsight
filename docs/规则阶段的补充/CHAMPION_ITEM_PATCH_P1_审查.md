结论：**P1 可以收口，可以进入 P2。**  
当前 P1 已满足阶段验收：真实阵容推荐装备进入 Rust 阵容评分入口，主 C 装备刚需、替代装、当前散件可合成度、缺核心惩罚会共同影响阵容推荐分，并有真实数据端到端测试保护。

## 本轮收口结论

| 维度 | 当前判断 | 证据 |
|---|---|---|
| 主 C 装备评分模块 | 已完成 | `crates/hexsight-engine/src/champion_item_fit.rs` |
| 官网推荐装强先验 | 已进入 P1 阵容评分入口 | `LineupFitScorer::score_all_with_item_context` 调用 `ChampionItemFitScorer::score_with_item_context` |
| 当前散件合成判断 | 已按合成表判断 | `ItemSynthesisIndex::from_equipment` / `can_synthesize` 使用 `synthesis1`、`synthesis2` |
| 替代装配置 | 已被规则上下文读取并参与评分 | `LineupItemFitContext::from_rule_config` 读取 `item_replacement_groups.json` |
| 棋子装备覆写 | 已进入 KnowledgeBase 构建入口 | `KnowledgeBaseBuilder::build_all_with_rule_config` 读取 `champion_item_overrides.json` |
| 缺核心装影响阵容评分 | 已接入 | `item_fit_with_context` 将直接成装、可合成核心装、替代装转成 `item_fit_score`，并写入 `reasons` / `risks` |
| 真实数据 E2E | 已补 | `real_data_p1_item_context_scores_lineup_from_rule_config` 串起真实 lineup + KB + 替代组配置 + 合成表 + 阵容评分 |
| P1/P2 边界 | 已清理 | 单件评分移除严格度无条件扣分；移除 `conflict_group == damage_profile` 判断 |
| 配置 ID 化 | 已支持 | `ReplacementGroup` 支持 `itemIds`、`itemNames`、legacy `items` |
| 旧缺口入口 | 已标注 legacy | `assess_item_gaps` 已 deprecated，指向 `score_with_item_context` |

## 已完成补齐项

| 原问题 | 当前处理 | 验收证据 |
|---|---|---|
| `ChampionItemFitScorer` 未进入阵容评分入口 | 新增 `LineupItemFitContext`，阵容评分可消费 P1 scorer | `crates/hexsight-engine/src/lineup_fit_scorer.rs` 中 `item_fit_with_context` 调用 `score_with_item_context` |
| `ItemReplacementGroupLoader` 未被业务调用 | `LineupItemFitContext::from_rule_config` 从版本规则目录加载替代组 | 真实数据 E2E 测试通过 |
| 真实阵容推荐装没有进入 P1 E2E | 从真实 mode17 lineup profile 读取主 C 核心装并参与 P1 评分 | `real_data_p1_item_context_scores_lineup_from_rule_config` |
| 缺装惩罚仍是简化命中率 | P1 上下文按直接命中、散件可合成、替代装分层计算 `item_fit_score` | `p1_item_context_distinguishes_alternative_synthesis_and_completed_core` |
| 默认 KnowledgeBase 绕过覆写 | 保留 legacy `build_all`，新增规则链路入口 `build_all_with_rule_config` | `real_rule_config_item_overrides_enter_knowledge_base` |
| 装备严格度无条件扣分 | 严格度改为缺核心惩罚倍率的一部分 | `score_item` 不再按 high strictness 直接扣 15 |
| P1 混入冲突组语义 | 移除 `conflict_group == champ.damage_profile` 判断 | 冲突逻辑进入 P2 专门 scorer |
| 替代组只写名称 | 支持 `itemIds` + `itemNames` + legacy `items` | `replacement_group_drives_gap_alternatives` 覆盖 ID 配置路径 |

## 关键实现证据

| 文件 | 证据 |
|---|---|
| `crates/hexsight-engine/src/champion_item_fit.rs` | `ItemSynthesisIndex`、`score_with_item_context`、替代组 ID/名称映射、legacy `assess_item_gaps` 标注 |
| `crates/hexsight-engine/src/lineup_fit_scorer.rs` | `LineupItemFitContext`、`score_all_with_item_context`、`item_fit_with_context`、真实数据 E2E |
| `crates/hexsight-engine/src/knowledge_builders.rs` | `build_all_with_item_overrides`、`build_all_with_rule_config` |
| `crates/hexsight-engine/src/lib.rs` | 导出 P1 scorer、loader、合成索引、阵容 P1 上下文 |
| `config/rules/S18.1/item_replacement_groups.json` | 替代组配置，支持 `itemNames` 并保留 legacy `items` |
| `config/rules/S18.1/champion_item_overrides.json` | 棋子装备严格度覆写配置 |

## 验证结果

| 命令 | 结果 |
|---|---|
| `cargo test --workspace` | 通过：engine 107 tests + ffi 5 tests，0 failed |
| `cargo test -p hexsight-engine champion_item_fit -- --nocapture` | 通过：7 tests，0 failed |
| `cargo test -p hexsight-engine lineup_fit_scorer -- --nocapture` | 通过：6 tests，0 failed |
| `make build` | 通过：Rust release + Swift build 成功 |
| `cd swift && swift test` | 通过：17 tests，0 failed |

## 非阻塞提醒

| 项 | 处理建议 |
|---|---|
| `score_all` legacy 入口仍保留 | 保持兼容；后续 P7 / RuleOutput 集成时统一让实时决策入口使用 `score_all_with_item_context` |
| `build_all` legacy 入口仍保留 | 保持测试和旧链路兼容；规则链路使用 `build_all_with_rule_config` |
| `assess_item_gaps` legacy 入口仍保留 | 已 deprecated；后续清理旧调用时删除或改成私有 |

## 阶段判断

| 项 | 判断 |
|---|---|
| P1 是否可以收口 | 可以 |
| 是否可以进入 P2 | 可以 |
| 当前 P1 阻塞项 | 无 |

P2 可以开始，目标聚焦装备冲突与边际收益：重伤、灼烧、减抗、回蓝、续航、护盾、控制免疫等效果的覆盖率和重复收益降权。
