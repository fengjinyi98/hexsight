结论：**P7 可以收口；`KnowledgeDecisionPlanner` 已接入 P0-P6 关键产物，回归样例已从“可加载”升级为“可回放”，P6 简化收益也已进入 P7 集成测试断言。**

## 2026-05-25 复查结论

| 维度 | 判断 | 证据 |
|---|---|---|
| 核心模块 | 已实现 | `crates/hexsight-engine/src/knowledge_decision_planner.rs`、`crates/hexsight-engine/src/knowledge_regression.rs` |
| RuleOutput Schema | 已扩展 | `crates/hexsight-core/src/rule_types.rs` 新增 `knowledgeActions`、holder、combat、patch、short explanations |
| P3 承载者集成 | 已覆盖 | `p7_integrates_knowledge_outputs_into_rule_output` 断言 holder 写入 `knowledgeActions` 与 `itemAction` |
| P4 海克斯刷新集成 | 已覆盖 | `p7_integrates_knowledge_outputs_into_rule_output` 断言 `augmentAction.action == reroll` |
| P5 版本修正集成 | 已覆盖 | `PatchModifier::apply_to_scores` 被 `KnowledgeDecisionPlanner` 调用，测试断言 nerf 风险进入阵容推荐 |
| P6 简化收益集成 | 已覆盖 | `p7_integrates_knowledge_outputs_into_rule_output` 传入 `CombatValueDiff`，断言 `knowledgeActions.combat.scoreDiff`、`winner`、`itemAction.reason` 与收益转向条件 |
| 回归样例 | 已回放 | `p7_regression_cases_replay_planner_outputs` 读取 `p7_knowledge_decision_smoke.json`，调用 planner 并断言 `expectedActions` / `expectedExplanations` |
| 改动范围 | 已收窄 | 已跟踪 diff 只保留 `rule_types.rs`、`decision_planner.rs`、`lib.rs`；P7 新增文件为 planner、regression loader、P7 测试和 regression case |
| 收口判断 | 可收口 | P7 主干、P6 集成、回归回放和验证命令均有硬证据 |

## 已完成项

| P7 验收项 | 当前状态 | 证据 |
|---|---|---|
| 新增 P7 决策规划器 | 已完成 | `KnowledgeDecisionPlanner::plan` 汇总阵容、经济、装备、海克斯、承载者、版本修正、收益估算、转向条件 |
| 新增 P7 回归样例加载器 | 已完成 | `KnowledgeRegression::load_cases` 加载 `config/rules/S18.1/regression_cases/*.json`，支持 `fixture` |
| RuleOutput 输出知识动作 | 已完成 | `knowledgeActions` 包含 holder / combat / patch / shortExplanations |
| 保持旧 DecisionPlanner 兼容 | 已处理 | 旧 `RuleOutput` 构造补 `knowledge_actions: None` |
| 版本修正进入阵容排序 | 已完成 | `PatchModifier::apply_to_scores` 在 P7 planner 内调用 |
| P6 combat 集成断言 | 已完成 | P7 测试断言 `scoreDiff == -16`、`winner == right`，并验证收益差进入装备原因和转向条件 |
| 回归样例真实回放 | 已完成 | `p7_knowledge_decision_smoke.json` 通过 `fixture: p7_smoke` 绑定测试 fixture 并校验动作/解释 |

## 收口项处理

| 原待补项 | 当前处理 | 验收方式 |
|---|---|---|
| 回归样例可回放 | 已补 `fixture` 字段和回放测试 | `p7_regression_cases_replay_planner_outputs` |
| P6 combat 集成测试 | 已在 P7 主测试传入 `CombatValueDiff` | `p7_integrates_knowledge_outputs_into_rule_output` |
| 提交范围收窄 | 已回退非 P7 模块格式化/重排 diff | `git status --short` / `git diff --stat` |
| 格式化验证口径 | P7 触达 Rust 文件定向 rustfmt 通过 | `rustfmt --check ...` |

## 已执行验证

| 命令 | 结果 |
|---|---|
| `cargo test -p hexsight-engine --test knowledge_decision_p7 -- --nocapture` | 通过：3 passed，0 failed |
| `rustfmt --check --config skip_children=true crates/hexsight-core/src/rule_types.rs crates/hexsight-engine/src/decision_planner.rs crates/hexsight-engine/src/lib.rs crates/hexsight-engine/src/knowledge_decision_planner.rs crates/hexsight-engine/src/knowledge_regression.rs crates/hexsight-engine/tests/knowledge_decision_p7.rs` | 通过 |
| `cargo test --workspace` | 通过：engine 130 passed + P7 integration 3 passed + 其他集成测试和 doc-tests 通过，0 failed |
| `cd swift && swift test` | 通过：17 tests，0 failures |
| `make build` | 通过：Rust release 构建成功；Swift debug build 成功 |
| `git diff --check` | 通过 |
| `cargo fmt --check` | 未作为收口门禁；当前仓库存在历史/非 P7 格式化差异，P7 触达文件已定向验证 |

## 阶段判断

P7 已具备收口条件。后续规则争议可以继续沉淀到 `config/rules/<version>/regression_cases/*.json`，并通过回放测试固定动作与解释。
