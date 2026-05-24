结论：**P4 已收口；海克斯拿取/刷新/兜底主框架、兜底排序 tie-break、真实官网数据 E2E 均已补齐并通过验证。**

## 2026-05-25 审查结论

| 维度 | 判断 | 证据 |
|---|---|---|
| 核心模块 | 已实现 | `crates/hexsight-engine/src/augment_reroll_scorer.rs`、`crates/hexsight-engine/src/augment_option_ranker.rs` |
| 配置产物 | 已实现 | `config/rules/S18.1/augment_thresholds.json`、`config/rules/S18.1/augment_type_weights.json` |
| 动作输出 | 已实现 | `AugmentDecisionAction::{Take,Reroll,TakeFallback}` 与 `DecisionPlanner::plan_with_augment_reroll` |
| 海克斯说明解析 | 已扩展 | `AugmentEffectInterpreter` 增加输出类型、重伤、灼烧、破甲、回蓝、续航等标签 |
| 局势修正 | 已实现 | `AugmentSituationContext` 支持血量、经济、装备缺口、核心牌、已锁阵容、已有/需要效果、输出类型 |
| 兜底排序 | 已修复 | 同 `fallback_score` 时优先覆盖率更高；覆盖率相同时优先锁方向风险更低 |
| 真实数据 E2E | 已覆盖 | `real_mode17_hex_and_lineup_recommendation_can_take` 使用 mode17 `hex.json` 与真实阵容 `recommended_hex_ids` 验证输出 `take` |
| 收口判断 | 已收口 | P0/P1 最小闭环已补齐，P7 主链路接入作为后续集成风险跟踪 |

## 验收项

| P4 验收项 | 当前状态 | 证据 |
|---|---|---|
| 三个候选都低分时建议刷新 | 已完成 | `recommends_reroll_when_three_options_are_below_stage_threshold` |
| 刷后低分时兜底拿 | 已完成 | `takes_fallback_after_reroll_when_all_options_are_still_low` |
| 兜底同分选覆盖最高 | 已完成 | `fallback_tiebreak_prefers_higher_lineup_coverage` |
| 兜底覆盖相同选风险最低 | 已完成 | `fallback_tiebreak_prefers_lower_lock_risk_after_equal_coverage` |
| 配置阈值和类型权重 | 已完成 | `real_s18_config_loads_p4_thresholds_and_type_weights` |
| 真实官网海克斯 E2E | 已完成 | `real_mode17_hex_and_lineup_recommendation_can_take` |
| RuleOutput 输出 P4 动作 | 已完成 | `rule_output_contains_p4_reroll_action` |
| 高经济偏经济/上限 | 已完成 | `situation_context_prefers_economy_when_gold_is_high` |
| 已锁阵容偏推荐海克斯 | 已完成 | `situation_context_prefers_locked_lineup_recommended_augment` |
| 已有重复效果降权 | 已完成 | `situation_context_penalizes_duplicate_effect_tags` |
| 当前需要效果时降低重复惩罚 | 已完成 | `required_effect_tags_reduce_duplicate_penalty` |
| 输出类型不匹配降权 | 已完成 | `situation_context_penalizes_damage_profile_mismatch` |

## 已处理问题

| 优先级 | 问题 | 处理结果 | 证据 |
|---|---|---|---|
| P0 | `AugmentRerollScorer` 兜底选择 tie-break 对覆盖率/锁方向风险方向可疑 | 已修复 comparator：同分优先高覆盖，再优先低风险 | `fallback_tiebreak_prefers_higher_lineup_coverage`、`fallback_tiebreak_prefers_lower_lock_risk_after_equal_coverage` |
| P1 | P4 缺少真实官网海克斯数据 E2E | 已增加 mode17 真实数据链路测试 | `real_mode17_hex_and_lineup_recommendation_can_take` |

## 保留风险

| 优先级 | 风险 | 处理策略 |
|---|---|---|
| P1 | `DecisionPlanner::plan_with_augment_reroll` 是 P4 主入口，但上层默认实时链路是否消费该入口属于后续集成范围 | P7 即时动作输出阶段接入最终实时入口 |
| P2 | `AugmentRerollContext` 与 `AugmentSituationContext` 存在血量、经济、锁阵容字段重复 | 后续提供单一 builder 或合并上下文 |

## 验证结果

| 命令 | 结果 |
|---|---|
| `rustfmt --check crates/hexsight-engine/src/augment_reroll_scorer.rs crates/hexsight-engine/src/augment_option_ranker.rs crates/hexsight-engine/tests/augment_reroll_scorer_p4.rs` | 通过 |
| `cargo test -p hexsight-engine --test augment_reroll_scorer_p4` | 通过：12 passed，0 failed |
| `cargo test --workspace` | 通过：engine 130 passed + P4 integration 12 passed + P3 integration 12 passed + ffi 5 passed，0 failed |
| `make build` | 通过：Rust release + Swift build 成功 |
| `cd swift && swift test` | 通过：17 passed，0 failed |

## 收口标准

| 标准 | 状态 | 证据 |
|---|---|---|
| 兜底排序明确验证“覆盖最高、锁方向风险最低” | 已满足 | 新增 2 条 tie-break 测试 |
| 真实 `hex.json + lineup recommended_hex_ids` E2E 能证明官网推荐海克斯在合适局面下输出 `take` | 已满足 | 新增 mode17 真实数据 E2E |
| P4 审查文档更新为最终收口结论 | 已满足 | 本文档 |
