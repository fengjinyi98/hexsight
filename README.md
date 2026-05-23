# HexSight

HexSight 是一个面向 macOS 的《金铲铲之战》实时辅助原型，采用 **SwiftUI + Rust** 双栈架构：Swift 负责悬浮窗、权限和 UI 展示，Rust 负责数据提供、解析适配、规则上下文生成和 FFI 输出。

## 当前能力

| 模块 | 说明 |
|---|---|
| Swift 悬浮窗 | 单窗口展示决策、英雄、装备、羁绊、海克斯 |
| Rust 工作区 | `core / vision / memory / engine / llm / ffi` 六个 crate |
| Rust 数据层 | 阵容读取、官方适配、规则上下文、LLM 上下文、远端 CDN 刷新 |
| Rust 版本校验 | 检查阵容 hero_id/equip_id 与静态数据对齐 |
| 本地数据 | `config/game_data` 内置多个 mode 的静态资料库 |
| 数据抓取 | `scripts/jcc_api.py` 可拉取官网资料与阵容数据 |
| FFI 桥接 | Swift 通过 `hexsight_ffi` 调用 Rust 引擎（12 个 FFI 函数） |

## 技术架构

| 层 | 路径 | 职责 |
|---|---|---|
| App UI | `swift/Sources/HexSight` | 窗口、视图、权限、数据展示 |
| FFI | `crates/hexsight-ffi` | Rust 与 Swift 的 C ABI 边界（识别管线 + 数据 API） |
| 数据引擎 | `crates/hexsight-engine` | 数据加载、阵容适配、规则上下文、LLM 上下文、远端刷新、版本校验 |
| 视觉识别 | `crates/hexsight-vision` | OCR、图标、数字识别 |
| 记忆层 | `crates/hexsight-memory` | 状态与快照 |
| 基础类型 | `crates/hexsight-core` | 运行时类型 + 数据层类型 + 错误定义 |
| 数据脚本 | `scripts/jcc_api.py` | 官网数据抓取与落盘 |

## 目录结构

```text
hexsight/
├── config/                 # 游戏数据、阵容配置、区域配置
├── crates/                 # Rust workspace
├── docs/                   # PRD、数据管线文档
├── scripts/                # 数据抓取脚本
├── swift/                  # SwiftPM 应用
├── Cargo.toml              # Rust workspace root
└── Makefile                # 统一构建入口
```

## 环境要求

| 组件 | 要求 |
|---|---|
| macOS | Apple Silicon，面向 macOS 26 SDK 开发 |
| Rust | stable toolchain |
| Swift | Swift 6.2+ |
| Python | 3.12+ |

## 快速开始

### 1. 构建 Rust + Swift

```bash
make build
```

### 2. 运行应用

```bash
make run
```

### 3. 常用检查

```bash
cargo test --workspace
cd swift && swift test
cd swift && swift build
```

## 数据更新

### 拉取静态资料库

```bash
python3 scripts/jcc_api.py fetch-gamedata
```

### 拉取阵容推荐

```bash
python3 scripts/jcc_api.py fetch-lineups
```

更详细的数据来源、字段结构、去重规则与验证方式见：

- `/Users/fengjinyi/Desktop/hexsight/docs/DATA_PIPELINE.md`
- `/Users/fengjinyi/Desktop/hexsight/docs/PRD.md`

## 开发说明

| 主题 | 规则 |
|---|---|
| 路径解析 | Swift 开发模式统一走 `ProjectPaths` |
| 阵容数据 | 全链路走 Rust FFI（`LineupRepository` → `RustBridge` → `hexsight-ffi`），Swift 不直接解析阵容 JSON |
| 远端刷新 | CDN URL 拼装、HTTP 请求、缓存写入由 Rust `RemoteLineupSource` 完成 |
| 英雄图鉴 | 展示层按 `HeroCatalog.displayHeroes(from:)` 去重 |
| 羁绊图鉴 | 展示层按 `checkId` 折叠，取最小 `level` |
| 悬浮窗 | 当前使用普通层级窗口，允许其他应用覆盖 |

## 当前状态

核心链路已完成 Rust 后端化：

1. **Rust 数据层**：阵容读取、官方适配、规则上下文、LLM 上下文、远端 CDN 刷新、版本校验
2. **FFI 数据 API**：12 个 C ABI 函数（6 个识别管线 + 6 个数据提供）
3. **Swift 阵容链路**：`LineupPanel` → `LineupRepository` → `RustBridge` → Rust FFI，旧适配层已删除
4. **图鉴静态数据**（英雄/装备/羁绊/海克斯）仍由 `GameDataService` 管理，属于下阶段迁移

后续演进：规则引擎评分推理、端侧 LLM 接入、识别精度提升、图鉴静态数据下沉 Rust。
