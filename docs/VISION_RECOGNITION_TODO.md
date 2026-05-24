# 画面识别收口状态

## 目标

对齐 PRD「模块2：画面识别与数据解析」，把画面识别链路中的工程项、模板资产、样本回归和字段级评测收口到可验证状态。

## 当前结论

| 识别层 | 当前状态 | 主要缺口 | 优先级 |
|---|---|---|---|
| 商店 / HUD 文本 OCR | Swift Vision 链路已闭环，`--ocr-eval` 已输出字段级统计 | 海克斯选择界面真实样本仍不足 | P0 |
| 棋盘英雄头像 | Rust 模板匹配已接入，并补费用、星级字段 | 真实棋盘截图标注仍需扩充 | P0 |
| 数字识别 | Rust `digit.rs` 已实现 7-segment ROI 解析与模板目录校验 | 需要真实金币/等级/人口截图标注持续校准 | P0 |
| 条状扫描 | Rust `bar.rs` 已实现血量条、经验条列扫描 | 需要真实截图误差统计 | P0 |
| 装备图标 | Rust `icon.rs` 已实现装备模板库与匹配，装备席与英雄身上装备已接入 | 英雄身上装备仍需真实截图标注校准 | P0 |
| 羁绊图标 | Rust `icon.rs` 已实现羁绊模板库与匹配入口 | race/job 真实图标资产仍需批量补齐 | P1 |
| 备战席识别 | `bench_slots` 与 `bench_heroes` 已接入 | 真实截图标注仍需扩充 | P1 |
| 对手阵容识别 | 已补对手棋盘 ROI 与 `opponent_heroes` 输出 | 缺对手视角真实截图标注 | P2 |

## P0 必做项

| TODO | 文件 / 区域 | 当前状态 | 证据 |
|---|---|---|---|
| 实现 7-segment 数字解析 | `crates/hexsight-vision/src/digit.rs` | 已完成 | `digit::tests::parse_digits_reads_multiple_seven_segment_numbers_from_roi` |
| 建立数字模板目录 | `config/digit_templates/` | 已完成 | `digit::tests::load_templates_accepts_config_directory_with_digit_patterns` |
| 实现血量条扫描 | `crates/hexsight-vision/src/bar.rs` | 已完成 | `bar::tests::scan_hp_bar_returns_green_fill_percentage` |
| 实现经验条扫描 | `crates/hexsight-vision/src/bar.rs` | 已完成 | `bar::tests::scan_exp_bar_returns_blue_fill_percentage` |
| 实现装备图标匹配 | `crates/hexsight-vision/src/icon.rs` | 已完成装备席接入 | `tests::recognize_populates_equipment_bench_cost_and_star_fields` |
| 建立装备模板目录 | `config/vision_templates/equipment/` | 已完成最小模板与目录约定 | `icon::tests::match_equipment_and_trait_return_loaded_template_names` |
| 补齐海克斯选择 OCR 样本 | `docs/ocr_samples/` | 未完成 | 缺 15-25 张真实海克斯选择 PNG |
| 输出字段级准确率 | `swift run HexSight --ocr-eval` | 已完成 | 输出 `fieldAccuracy`，字段含 `round/shop/trait/opponent/augment` |

## P1 必做项

| TODO | 文件 / 区域 | 当前状态 | 证据 |
|---|---|---|---|
| 实现羁绊图标匹配 | `crates/hexsight-vision/src/icon.rs` | 已完成匹配入口 | `icon::tests::match_equipment_and_trait_return_loaded_template_names` |
| 建立羁绊模板目录 | `config/vision_templates/traits/` | 已完成最小模板与目录约定 | `config/vision_templates/traits/README.md` |
| 增加备战席 ROI | `config/regions.json`、`RegionConfig` | 已完成 | `config/regions.json` 的 `bench_slots` |
| 识别备战席英雄头像 | `crates/hexsight-vision/src/lib.rs` | 已完成 | `RecognizedFrame.bench_heroes` 与集成测试 |
| 棋盘英雄补费用 / 星级 | `crates/hexsight-vision` | 已完成 | 英雄费用索引 + 星标扫描集成测试 |
| 扩展真实截图回归集 | `docs/OCR_SAMPLE_COLLECTION_TODO.md` | 未完成 | 当前 30 张，目标 80-120 张 |

## P2 后续项

| TODO | 范围 | 当前状态 | 完成标准 |
|---|---|---|---|
| 对手阵容棋盘识别 | 对手棋盘 / 可控切屏方案 | 已补 `opponent_board_grid`、`opponent_heroes` 和回归测试 | 可统计同行核心卡与阵容方向 |
| 选秀阶段装备识别 | 选秀画面 ROI | 已补 `carousel_slots`、`carousel_equipment` 和回归测试 | 能输出优先选秀目标所需装备候选 |
| 场景指纹路由 | HUD / 商店 / 海克斯 / 战斗 / 选秀 | 已补 Rust 场景指纹路由与 Swift augment ROI | 不同场景自动选择识别策略 |
| 多分辨率模板校准 | 1080P / 2K / 窗口化 | ROI 缩放路径可用，真实多分辨率标注不足 | ROI 和模板匹配在常见窗口尺寸稳定 |
| 性能基准 | 单帧识别链路 | 已补 `recognize_measured` 单帧耗时入口 | 识别延迟达到 PRD 15-30ms 目标，端到端 ≤100ms |

## 关联现有资料

| 文档 / 文件 | 用途 |
|---|---|
| `docs/PRD.md` | 画面识别目标来源 |
| `docs/OCR_SAMPLE_COLLECTION_TODO.md` | OCR 样本采集和回归计划 |
| `crates/hexsight-vision/src/digit.rs` | 数字识别实现 |
| `crates/hexsight-vision/src/bar.rs` | 血量 / 经验条扫描实现 |
| `crates/hexsight-vision/src/icon.rs` | 装备 / 羁绊 / 英雄头像模板匹配 |
| `crates/hexsight-vision/src/ocr.rs` | Rust OCR 兜底 stub |
| `swift/Sources/HexSight/OCR/` | Swift Vision OCR 当前可用链路 |

## 当前阻塞项

| 阻塞项 | 影响 | 所需外部输入 |
|---|---|---|
| 海克斯选择真实样本不足 | 无法证明三选、刷新、不同等级场景的字段级准确率 | 15-25 张真实海克斯选择 PNG |
| 内测回归集不足 | 无法证明 80-120 张内测集覆盖 | 50-90 张新增真实截图与可选标注 JSON |
| 对手棋盘 / 选秀画面缺样本 | P2 工程出口已具备，真实准确率仍无法证明 | 对手视角切屏方案、选秀阶段截图和标注 |

## 验收命令

| 检查项 | 命令 | 通过标准 |
|---|---|---|
| Rust 视觉测试 | `cargo test -p hexsight-vision` | 0 failures |
| Rust 全量测试 | `cargo test --workspace` | 0 failures |
| Swift OCR 回归 | `cd swift && swift run HexSight --ocr-eval ../docs/ocr_samples` | 输出包含 `frames` 与 `fieldAccuracy` 的评测 JSON |
| Swift 测试 | `cd swift && swift test` | 0 failures |
| Swift 构建 | `cd swift && swift build` | Build complete |
