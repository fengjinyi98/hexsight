# HexSight

HexSight 是一个面向 macOS 的《金铲铲之战》实时辅助原型，采用 **SwiftUI + Rust** 双栈架构：Swift 负责悬浮窗、权限和截屏，Rust 负责规则、识别、记忆与 FFI 输出。

## 当前能力

| 模块 | 说明 |
|---|---|
| Swift 悬浮窗 | 单窗口展示决策、英雄、装备、羁绊、海克斯 |
| Rust 工作区 | `core / vision / memory / engine / llm / ffi` 六个 crate |
| 本地数据 | `config/game_data` 内置多个 mode 的静态资料库 |
| 数据抓取 | `scripts/jcc_api.py` 可拉取官网资料与阵容数据 |
| FFI 桥接 | Swift 通过 `hexsight_ffi` 调用 Rust 引擎 |

## 技术架构

| 层 | 路径 | 职责 |
|---|---|---|
| App UI | `swift/Sources/HexSight` | 窗口、视图、权限、数据展示 |
| FFI | `crates/hexsight-ffi` | Rust 与 Swift 的 C ABI 边界 |
| 决策引擎 | `crates/hexsight-engine` | 阵容与规则决策 |
| 视觉识别 | `crates/hexsight-vision` | OCR、图标、数字识别 |
| 记忆层 | `crates/hexsight-memory` | 状态与快照 |
| 基础类型 | `crates/hexsight-core` | 公共类型与错误定义 |
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
| 英雄图鉴 | 展示层按 `HeroCatalog.displayHeroes(from:)` 去重 |
| 羁绊图鉴 | 展示层按 `checkId` 折叠，取最小 `level` |
| 悬浮窗 | 当前使用普通层级窗口，允许其他应用覆盖 |

## 当前状态

这是一个早期原型仓库，核心目标是打通：

1. 视觉输入
2. 本地规则与数据支持
3. Swift 悬浮窗输出
4. Rust 决策结果桥接

后续演进可以继续补强识别精度、阵容规则、端侧 LLM 推理与交互体验。
