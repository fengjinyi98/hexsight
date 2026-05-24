结论：**P3 已完成审查收口；打工棋子与装备承载者链路已补齐备战席候选、装备容量过滤、真实覆写命中、默认入口散件合成、实际承载者卖出成本。**

## 2026-05-25 复审结论

| 维度 | 判断 | 证据 |
|---|---|---|
| 核心模块 | 已完成 | `crates/hexsight-engine/src/holder_scorer.rs`、`crates/hexsight-engine/src/bench_transition_scorer.rs` |
| 配置产物 | 已完成 | `config/rules/S18.1/holder_rules.json`、`config/rules/S18.1/workhorse_overrides.json` |
| 棋盘候选 | 已完成 | 场上棋子参与临时承载评分，保留当前上场加分 |
| 备战席候选 | 已完成 | 备战席棋子进入候选池，可输出“备战席可上场承载”原因 |
| 装备席散件 | 已完成 | `score_from_rule_config_with_equipment` 可由真实 `EquipmentData` 构建 `ItemSynthesisIndex` |
| 已有装备容量 | 已完成 | 满 3 件棋子被过滤，不再推荐继续承载 |
| 真实配置覆写 | 已完成 | mode16 真实棋子命中 `workhorse_overrides.json`，覆写说明进入推荐原因 |
| 卖出成本 | 已完成 | `sell_cost` 按实际推荐承载者计算 |
| 收口判断 | 可以收口 | P3 最小闭环 5 项均有测试覆盖并通过完整验证 |

## 本轮补齐项

| 原阻塞 | 当前处理 | 验收证据 |
|---|---|---|
| `candidates_for_item` 只遍历 `board_units` | 候选池扩展为 `board_units + bench_units`；场上保留 `board_bonus`，备战席输出上场承载原因 | `recommends_two_star_bench_workhorse_as_temporary_holder` |
| `HolderUnit.item_ids` 未参与评分 | 满 3 件棋子直接过滤，避免继续推荐带装 | `filters_full_item_holder_from_recommendations` |
| `workhorse_overrides.json` 真实链路未证明命中 | 增加 mode16 真实数据测试，验证覆写原因进入推荐结果 | `real_mode16_workhorse_override_enters_holder_reasons` |
| 默认配置入口未构建合成索引 | 新增 `score_from_rule_config_with_equipment`，由 `EquipmentData` 构建 `ItemSynthesisIndex` | `default_config_entry_builds_synthesis_index_from_equipment_data` |
| 转移成本按全场最高成本计算 | 新增 `BenchTransitionScorer::score_for_candidate`，按实际候选 unit 计算卖出成本 | `sell_cost_uses_actual_recommended_holder` |

## 验收清单

| P3 验收项 | 结果 | 覆盖测试 |
|---|---|---|
| 推荐打工 C | 通过 | `recommends_two_star_ad_workhorse_as_temporary_carry` |
| 推荐打工坦克 | 通过 | `recommends_frontline_tank_as_temporary_tank_holder` |
| 判断给当前二星还是等目标主 C | 通过 | `waits_for_target_carry_when_target_is_available_and_current_holder_is_one_star` |
| 输出最终转移对象和卖出成本 | 通过 | `bench_transition_scorer_outputs_final_holder_and_sell_cost` |
| 输出备用承载者 | 通过 | `returns_backup_holders_for_same_completed_item` |
| 装备席散件可合成后推荐承载者 | 通过 | `component_bench_can_build_combat_item_and_recommend_holder` |
| 真实配置可加载并输出短建议 | 通过 | `real_s18_config_loads_holder_rules_and_scores_plan` |
| 备战席二星强牌可上场承载 | 通过 | `recommends_two_star_bench_workhorse_as_temporary_holder` |
| 满装棋子不再被推荐 | 通过 | `filters_full_item_holder_from_recommendations` |
| 默认配置入口消费散件合成索引 | 通过 | `default_config_entry_builds_synthesis_index_from_equipment_data` |
| 卖出成本按实际推荐承载者计算 | 通过 | `sell_cost_uses_actual_recommended_holder` |
| 真实打工强度覆写命中 | 通过 | `real_mode16_workhorse_override_enters_holder_reasons` |

## 验证结果

| 命令 | 结果 |
|---|---|
| `rustfmt --check crates/hexsight-engine/src/holder_scorer.rs crates/hexsight-engine/src/bench_transition_scorer.rs crates/hexsight-engine/tests/holder_scorer_p3.rs` | 通过 |
| `cargo test -p hexsight-engine --test holder_scorer_p3 -- --nocapture` | 通过：12 passed，0 failed |
| `cargo test --workspace` | 通过：engine 130 passed + P3 integration 12 passed + ffi 5 passed，0 failed |
| `make build` | 通过：Rust release + Swift build 成功 |
| `cd swift && swift test` | 通过：17 passed，0 failed |

## 阶段判断

| 项 | 判断 |
|---|---|
| P3 模块能力 | 已完成 |
| P3 配置产物 | 已完成 |
| P3 棋盘输入消费 | 已完成 |
| P3 备战席输入消费 | 已完成 |
| P3 装备席散件消费 | 已完成 |
| P3 已成装消费 | 已完成 |
| P3 当前羁绊输入消费 | 已完成基础版 |
| P3 目标阵容候选消费 | 已完成 |
| P3 是否可以收口 | 可以收口 |

## 剩余风险

| 风险 | 当前状态 | 后续建议 |
|---|---|---|
| 等待目标时字段语义 | `WaitForTarget` 仍复用 `temporary_holder_id` 表示等待对象 | 后续 P7 输出集成时拆成 `recommended_holder_id` 与 `final_holder_id` |
| 目标阵容选择 | 当前按传入目标阵容顺序取最终承载者 | 后续结合阵容评分选最高候选 |
| 羁绊贡献 | 当前用 active trait ID 简单命中 | 后续计算“上备战席棋子后新增/补齐羁绊” |
| 同棋子装备冲突 | 当前先完成容量过滤，未接入 P2 单棋子冲突报告 | 后续可把 `ItemConflictScorer` 接入 holder 级别解释 |
