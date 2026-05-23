# 阵容数据工程化方案

## 目标

阵容、英雄、装备、羁绊、强化符文是后续规则引擎和 LLM 理解的核心输入。追版本时需要把官方字段变化收敛在数据适配层，UI 只消费稳定领域模型，规则层只消费稳定上下文。

## 分层边界

| 层级 | 路径 | 职责 | 版本变化承接点 |
|---|---|---|---|
| 官方源 | `config/lineups/*.json`、官方 CDN | 保存官方 `lineup_detail_total.json` 快照 | 原始字段允许变化 |
| Mode Profile | `swift/Sources/HexSight/Lineups/ModeProfile.swift` | 声明模式 ID、赛季、CDN 路径、玩法能力 | 新模式/新赛季只增配置 |
| Source Adapter | `swift/Sources/HexSight/Lineups/LineupSourceAdapter.swift` | 官方 JSON → 稳定 `LineupCard` / `LineupDetailData` | 字段兼容、候选字段、二次 JSON 解析 |
| Game Data Index | `swift/Sources/HexSight/Lineups/GameDataIndex.swift` | 英雄/装备/羁绊/强化符文 ID → 可读快照 | 静态数据字段兼容与规则解析 |
| Domain Models | `swift/Sources/HexSight/Models/LineupModels.swift` | 稳定领域模型 | 字段含义稳定时才变更 |
| Rules Context | `swift/Sources/HexSight/Lineups/LineupRulesContext.swift` | 给规则引擎/LLM 的结构化输入 | 随规则消费场景扩展 |
| UI | `swift/Sources/HexSight/Views/Panels/LineupPanel.swift` | 展示稳定模型 | 保持布局与展示逻辑稳定 |

## 官方字段兼容策略

| 数据块 | 当前字段 | 兼容策略 | 下游用途 |
|---|---|---|---|
| 阵容名称 | `detail.line_name`、`name` | 按优先级取首个非空值 | 列表、详情、搜索、规则标题 |
| 作者 | `lineupauthor_data.name`、`author_littlelegend.desc`、`author` | 按优先级取首个非空值 | 列表、详情可信来源标识 |
| 最终站位 | `hero_location` | 解析为 `LineupPiece` | 棋盘、装备归属、羁绊反推 |
| 过渡站位 | `y21_early_heros`、`y21_metaphase_heros` | 独立解析早期/中期 | 运营规则、阶段推荐 |
| 羁绊总览 | `contact`、`*_contact` | 优先使用官方计数，缺失时本地反推 | 羁绊规则、阵容强度解释 |
| 强化符文 | `hexbuff.recomm`、`hexbuff.replace` | 拆分为优先/次选 ID | 符文优先级规则 |
| 装备顺序 | `equipment_order`、棋子 `equipment_id` | 拆分为全局顺序和英雄绑定 | 装备优先级、主 C 装备 |
| mode17 | `god_list`、`godreward_info` | 归入星神奖励能力 | 星神选择规则 |
| mode16 | `task_list`、`task_info` | 归入解锁任务能力 | 英雄解锁规则 |
| mode4 | `chosen_contact`、`messengerContact`、`chosen_backup` 等 | 归入天选/使者能力 | 天选备选和羁绊目标规则 |

## 追版本流程

1. 刷新官方静态数据与阵容快照到 `config/`。
2. 在 `ModeProfile` 更新模式配置、赛季、CDN 版本目录和能力声明。
3. 用 `LineupSourceAdapter` 的快照测试验证代表阵容字段完整度。
4. 用 `GameDataIndex` 验证英雄、装备、羁绊、强化符文 ID 可解析成名称、图标和描述。
5. 用 `LineupRulesContext` 测试验证规则输入包含英雄、装备、羁绊、符文、玩法机制。
6. 确认 UI 文件只消费稳定模型，继续保持原有布局。
7. 运行 `cd swift && swift test && swift build`。

## 规则上下文输出

| 输出 | 来源 | 用途 |
|---|---|---|
| `resolvedFinalHeroes` | `hero_location` + `chess.json` + `equip.json` | 英雄名称、费用、站位、主 C、携带装备 |
| `resolvedEquipmentOrder` | `equipment_order` + `equip.json` | 装备合成优先级 |
| `resolvedRecommendedHexes` | `hexbuff.recomm` + `hex.json` | 强化符文优先级 |
| `resolvedReplacementHexes` | `hexbuff.replace` + `hex.json` | 强化符文备选 |
| `resolvedTraits` | 官方 `contact` 或英雄羁绊反推 + `trait.json` | 阵容羁绊目标 |
| `modeSpecificTexts` | `godreward_info` / `task_info` / `chosen_info` 等 | 模式玩法专属规则 |
| `strategyTexts` | 运营、站位、克制、装备说明 | LLM 解释与策略生成 |

## 工程约束

| 约束 | 判定方式 |
|---|---|
| UI 不直接解析官方 JSON | `LineupPanel.swift` 不出现阵容 `JSONSerialization` / `lineup_list` 解析 |
| 玩法差异集中声明 | `ModeProfile.capabilities` 覆盖 mode17/mode16/mode4 |
| 规则层不读 UI | 规则/LLM 只读 `LineupRulesContext` |
| 字段变更可回归 | 每个重点模式至少一条适配层测试 |
| 静态数据可解析 | 每个支持模式首个缓存阵容能生成可读英雄、装备、羁绊、符文 |
| SwiftUI 渲染纯净 | 网络、缓存写入保持在 `.task` 或服务层异步入口 |
