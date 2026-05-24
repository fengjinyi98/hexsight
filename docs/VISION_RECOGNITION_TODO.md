# 画面识别未完成 TODO

## 目标

对齐 PRD「模块2：画面识别与数据解析」，把当前画面识别链路中尚未完成的工程项单独收口，便于后续按优先级推进。

## 当前结论

| 识别层 | 当前状态 | 主要缺口 | 优先级 |
|---|---|---|---|
| 商店 / HUD 文本 OCR | Swift Vision 链路已闭环 | 海克斯选择界面样本不足，字段级准确率统计不足 | P0 |
| 棋盘英雄头像 | Rust 模板匹配第一版已接入 | 只覆盖棋盘格，缺少费用、星级、装备字段 | P0 |
| 数字识别 | Rust `digit.rs` 仍为 stub | 金币、等级、人口等 7-segment 模板解析未实现 | P0 |
| 条状扫描 | Rust `bar.rs` 仍为 stub | 血量条、经验条像素列扫描未实现 | P0 |
| 装备图标 | Rust `icon.rs` 仍为 TODO | 散件、成装、转职图标模板库与匹配逻辑未实现 | P0 |
| 羁绊图标 | Rust `icon.rs` 仍为 TODO | 羁绊图标模板库与匹配逻辑未实现 | P1 |
| 备战席识别 | 未实现 | 缺 bench ROI、模板匹配、真实截图标注 | P1 |
| 对手阵容识别 | 未实现 | 当前只做对手侧栏 OCR，缺棋盘阵容识别 | P2 |

## P0 必做项

| TODO | 文件 / 区域 | 完成标准 |
|---|---|---|
| 实现 7-segment 数字解析 | `crates/hexsight-vision/src/digit.rs` | 金币、等级、人口 ROI 能输出真实整数；加入单测和真实截图回归 |
| 建立数字模板目录 | `config/digit_templates/` | 0-9 模板可加载，支持至少 1920x1080 标准截图 |
| 实现血量条扫描 | `crates/hexsight-vision/src/bar.rs` | 根据填充色比例输出血量，误差可量化 |
| 实现经验条扫描 | `crates/hexsight-vision/src/bar.rs` | 输出经验百分比或可供等级策略使用的进度值 |
| 实现装备图标匹配 | `crates/hexsight-vision/src/icon.rs` | 装备席、英雄身上装备能识别散件/成装/转职 |
| 建立装备模板目录 | `config/vision_templates/equipment/` | 模板来源、下载脚本、排除规则明确 |
| 补齐海克斯选择 OCR 样本 | `docs/ocr_samples/` | 至少 15-25 张海克斯选择界面样本，覆盖三选、刷新、不同等级 |
| 输出字段级准确率 | `swift run HexSight --ocr-eval` | 评测结果能按 round/shop/trait/opponent/augment 分字段统计 |

## P1 必做项

| TODO | 文件 / 区域 | 完成标准 |
|---|---|---|
| 实现羁绊图标匹配 | `crates/hexsight-vision/src/icon.rs` | 左侧羁绊图标可映射到 trait id / name |
| 建立羁绊模板目录 | `config/vision_templates/traits/` | race/job 图标模板可加载并回归 |
| 增加备战席 ROI | `config/regions.json`、`RegionConfig` | bench slots 可随分辨率缩放 |
| 识别备战席英雄头像 | `crates/hexsight-vision/src/lib.rs` | `RecognizedFrame` 能包含 bench heroes 或独立字段 |
| 棋盘英雄补费用 / 星级 | `crates/hexsight-vision` | 英雄识别结果能区分 cost、star，供规则引擎消费 |
| 扩展真实截图回归集 | `docs/OCR_SAMPLE_COLLECTION_TODO.md` | 内测集达到 80-120 张 |

## P2 后续项

| TODO | 范围 | 完成标准 |
|---|---|---|
| 对手阵容棋盘识别 | 对手棋盘 / 可控切屏方案 | 可统计同行核心卡与阵容方向 |
| 选秀阶段装备识别 | 选秀画面 ROI | 能输出优先选秀目标所需装备候选 |
| 场景指纹路由 | HUD / 商店 / 海克斯 / 战斗 / 选秀 | 不同场景自动选择识别策略 |
| 多分辨率模板校准 | 1080P / 2K / 窗口化 | ROI 和模板匹配在常见窗口尺寸稳定 |
| 性能基准 | 单帧识别链路 | 识别延迟达到 PRD 15-30ms 目标，端到端 ≤100ms |

## 关联现有资料

| 文档 / 文件 | 用途 |
|---|---|
| `docs/PRD.md` | 画面识别目标来源 |
| `docs/OCR_SAMPLE_COLLECTION_TODO.md` | OCR 样本采集和回归计划 |
| `crates/hexsight-vision/src/digit.rs` | 数字识别 stub |
| `crates/hexsight-vision/src/bar.rs` | 血量 / 经验条扫描 stub |
| `crates/hexsight-vision/src/icon.rs` | 装备 / 羁绊 TODO，英雄头像第一版 |
| `crates/hexsight-vision/src/ocr.rs` | Rust OCR 兜底 stub |
| `swift/Sources/HexSight/OCR/` | Swift Vision OCR 当前可用链路 |

## 验收命令

| 检查项 | 命令 | 通过标准 |
|---|---|---|
| Rust 视觉测试 | `cargo test -p hexsight-vision` | 0 failures |
| Rust 全量测试 | `cargo test --workspace` | 0 failures |
| Swift OCR 回归 | `cd swift && swift run HexSight --ocr-eval ../docs/ocr_samples` | 输出评测 JSON，字段统计可读 |
| Swift 测试 | `cd swift && swift test` | 0 failures |
