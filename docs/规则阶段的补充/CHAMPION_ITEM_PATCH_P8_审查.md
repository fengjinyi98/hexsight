结论：**P8 可以收口；Rust 知识决策 `RuleOutput` 已通过 FFI 暴露给 Swift，Swift 侧只解析结构化 JSON。**

## 2026-05-25 复查结论

| 维度 | 判断 | 证据 |
|---|---|---|
| Rust FFI 出口 | 已实现 | `crates/hexsight-ffi/src/data_ffi.rs` 新增 `hexsight_get_knowledge_rule_output_json` |
| Swift 桥接 | 已实现 | `swift/Sources/HexSight/Bridge/RustBridge.swift` 新增 `getKnowledgeRuleOutput` |
| C 头文件声明 | 已补齐 | `swift/Sources/HexSight/Bridge/hexsight.h` 新增函数声明 |
| 仓库层入口 | 已实现 | `LineupRepository.loadKnowledgeRuleOutput(mode:lineupId:)` |
| Swift 展示边界 | 已保持 | Swift 只调用 FFI 并解析 JSON 字典，规则计算仍在 Rust |
| 目标文档 | 已同步 | `docs/CHAMPION_ITEM_PATCH_KNOWLEDGE_TARGET.md` 新增 P8 阶段和依赖链 |

## 已完成项

| P8 验收项 | 当前状态 | 证据 |
|---|---|---|
| Rust 返回知识决策 JSON | 已完成 | FFI 返回包含 `knowledgeActions`、`itemAction`、`augmentAction`、`pivotConditions` 的 `RuleOutput` |
| Swift 能读取 Rust 输出 | 已完成 | `LineupRepositoryTests.testKnowledgeRuleOutputLoadsViaRustFFI` |
| Swift 不参与知识计算 | 已完成 | Swift 侧无装备收益、版本修正或知识决策逻辑，仅解析 FFI JSON |
| Release 静态库可链接 | 已完成 | `cargo build -p hexsight-ffi --release` 后 Swift 测试通过 |

## 已执行验证

| 命令 | 结果 |
|---|---|
| `cargo test -p hexsight-ffi test_get_knowledge_rule_output_json -- --nocapture` | 通过：1 passed，0 failed |
| `rustfmt --check crates/hexsight-ffi/src/data_ffi.rs` | 通过 |
| `cargo build -p hexsight-ffi --release` | 通过 |
| `cd swift && swift test --filter LineupRepositoryTests/testKnowledgeRuleOutputLoadsViaRustFFI` | 通过：1 test，0 failures |
| `cargo test --workspace` | 通过：engine 130 passed、P4/P6/P3/P7/P5 集成测试通过、FFI 6 passed、doc-tests 通过 |
| `cd swift && swift test` | 通过：18 tests，0 failures |
| `cd swift && swift build` | 通过 |
| `git diff --check` | 通过 |

## 阶段判断

P8 已具备收口条件。后续 UI 面板可以直接消费 `RuleOutput.knowledgeActions` 做展示，不需要在 Swift 侧重建规则推理。
