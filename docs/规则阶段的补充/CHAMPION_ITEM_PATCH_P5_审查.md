结论：**P5 可收口；版本公告修正模块具备结构化加载、人工覆写、阵容评分修正入口和硬验收回归测试。**

## 2026-05-25 收口结论

| 维度 | 判断 | 证据 |
|---|---|---|
| 核心模块 | 已完成 | `crates/hexsight-engine/src/patch_knowledge.rs`、`crates/hexsight-engine/src/patch_modifier.rs` |
| 配置产物 | 已完成 | `config/rules/S18.1/patch_knowledge.json`、`config/rules/S18.1/patch_knowledge_overrides.json` |
| 配置边界 | 已澄清 | `patch_overrides.json` 保留给旧规则包/伤害/阈值/权重覆写；P5 只读取 `patch_knowledge*.json` |
| 版本知识加载 | 已完成 | `PatchKnowledgeLoader::load` 合并 `patch_knowledge.json` 与 `patch_knowledge_overrides.json` |
| 阵容评分修正 | 已完成 | `PatchModifier::apply_to_score` 支持 champion / item / trait / augment / lineup / system |
| 解释输出 | 已完成 | 正向修正进入 `reasons`，负向修正进入 `risks` |
| 收口判断 | 可收口 | P5 目标测试覆盖加载、硬验收方向、无关改动隔离和解释输出；默认 `RuleOutput` 主链路接入归入 P7 |

## 已完成项

| P5 验收项 | 当前状态 | 证据 |
|---|---|---|
| 加载结构化版本公告 | 已完成 | `loads_patch_knowledge_and_patch_knowledge_overrides_entries` |
| 人工公告覆写独立文件 | 已完成 | `patch_knowledge_overrides.json` 被加载，`patch_overrides.json` 中误填 `entries` 被忽略 |
| 版本改动影响阵容评分 | 已完成 | `patch_modifier_changes_lineup_score_and_explains_sources` |
| 主 C 削弱后相关阵容评分下降 | 已完成 | `carry_nerf_reduces_lineup_total_and_base_score` |
| 装备加强后阵容装备分上升 | 已完成 | `core_item_buff_increases_lineup_item_fit_and_total_score` |
| 无关改动不影响阵容 | 已完成 | `unrelated_patch_entries_do_not_change_lineup` |
| 棋子改动映射阵容 | 已完成 | champion entry 命中 carry/final/tank/early/mid hero IDs |
| 装备改动映射阵容 | 已完成 | item entry 命中 core/tank/equipment order IDs |
| 羁绊改动映射阵容 | 已完成 | trait entry 命中 `trait_targets` |
| 海克斯改动映射阵容 | 已完成 | augment entry 命中 recommended/replacement hex IDs |

## 问题处理

| 原问题 | 处理结果 | 当前证据 |
|---|---|---|
| `patch_overrides.json` 与 P5 `PatchEntry` 语义混用 | 已拆分为 `patch_knowledge_overrides.json` | `PatchKnowledgeLoader` 只读取 `patch_knowledge.json` 和 `patch_knowledge_overrides.json` |
| 主 C 削弱验收缺口 | 已补测试 | `carry_nerf_reduces_lineup_total_and_base_score` |
| 装备加强验收缺口 | 已补测试 | `core_item_buff_increases_lineup_item_fit_and_total_score` |
| 真实配置入口为空 | 保留模板入口和独立覆写入口 | `patch_knowledge.json`、`patch_knowledge_overrides.json` 均有明确职责说明 |

## 验证结果

| 命令 | 结果 |
|---|---|
| `rustfmt --check crates/hexsight-engine/src/patch_knowledge.rs crates/hexsight-engine/src/patch_modifier.rs crates/hexsight-engine/tests/patch_modifier_p5.rs` | 通过 |
| `cargo test -p hexsight-engine --test patch_modifier_p5` | 通过：5 passed，0 failed |
| `cargo test --workspace` | 通过：engine 130 passed + P5 integration 5 passed + P4 integration 12 passed + P3 integration 12 passed + ffi 5 passed，0 failed；doctest 全部通过 |
| `make build` | 通过：Rust release 构建成功；Swift debug build 成功 |
| `cd swift && swift test` | 通过：17 tests，0 failures |
| `cargo test -p hexsight-engine --test patch_modifier_p5 && git diff --check` | 通过：5 passed，0 failed；diff 无空白错误 |

## P5 当前边界

| 范围 | 结论 |
|---|---|
| 阵容评分 | 已提供 `PatchModifier::apply_to_score/apply_to_scores` 修正入口；默认 `RuleOutput` 主链路统一留到 P7 集成 |
| 装备建议排序 | 当前通过阵容 `item_fit_score` 体现装备版本影响；单棋子装备候选排序接入留给后续阶段 |
| 真实公告数据 | 当前提供结构化入口与覆写入口；后续版本更新时按 entries 录入 |
| 目标文档口径 | 已同步为 `patch_knowledge.json` + `patch_knowledge_overrides.json`，并把装备加强验收改为阵容装备适配分和总分上升 |
