# HexSight Agent Rules

## 沟通规则

1. 面向用户的回复统一使用简体中文。
2. 输出顺序默认采用：结论 → 修改内容 → 验证结果 → 风险。
3. 优先给可执行结论，少写空泛描述。

## 工作方式

1. 先看运行现象和当前代码，再动手修改。
2. 修 bug 时优先定位根因，避免堆补丁。
3. 涉及展示数据时，先区分“原始数据层”和“图鉴展示层”。
4. 改动保持最小、可回退、可验证。

## 项目结构约定

| 区域 | 路径 | 说明 |
|---|---|---|
| Swift App | `swift/Sources/HexSight` | 视图、窗口、权限、桥接、服务 |
| Swift Tests | `swift/Tests/HexSightTests` | Swift 单测与回归测试 |
| Rust Workspace | `crates/*` | 核心类型、识别、记忆、引擎、LLM、FFI |
| 配置数据 | `config/` | 游戏数据、阵容、区域配置 |
| 脚本 | `scripts/` | 官网数据抓取 |
| 文档 | `docs/` | 产品与数据管线说明 |

## 代码规则

### Swift / SwiftUI

1. 禁止在 SwiftUI 渲染路径执行副作用写操作。
2. `body`、同步计算属性、格式化层默认视为纯函数层。
3. 开发模式下的本地配置路径统一走 `ProjectPaths`。
4. 图鉴展示逻辑优先复用现有归一化规则，例如 `HeroCatalog`。
5. 注释使用中文职责型头部注释。

### Rust

1. 优先沿用 workspace 现有 crate 边界。
2. 公共类型放 `hexsight-core`，跨层接口通过 FFI 边界暴露。
3. 识别、记忆、规则、推理分层保持清晰。

## 验证规则

| 变更类型 | 必做验证 |
|---|---|
| Swift 代码 | `cd swift && swift test && swift build` |
| Rust 代码 | `cargo test --workspace` |
| 全链路构建 | `make build` |
| 数据加载 | 检查控制台 `[HexSight]` 与 `[GameData]` 日志 |

## 数据规则

1. 英雄原始数据允许保留多星级与变体。
2. 英雄图鉴展示按 `name` 折叠。
3. 羁绊图鉴展示按 `checkId` 折叠，并取最小 `level`。
4. `race.js` 与 `job.js` 主要用于 ID→名称映射。

## Git 规则

1. 提交前确认构建产物未入库。
2. 构建目录保持忽略：`target/`、`swift/.build/`。
3. 提交信息优先简洁明确。
