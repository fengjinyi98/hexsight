# HexSight 统一构建系统
# 使用方式：
#   make build        — 构建 Rust + Swift
#   make build-rust   — 仅构建 Rust
#   make build-swift  — 仅构建 Swift
#   make run          — 构建并运行
#   make clean        — 清理构建产物

RUST_TARGET := $(shell rustc -vV | grep host | cut -d' ' -f2)
RUST_LIB_DIR := target/$(RUST_TARGET)/release
RUST_LIB := $(RUST_LIB_DIR)/libhexsight_ffi.a
SWIFT_BUILD_DIR := swift/.build
APP_BUNDLE := $(SWIFT_BUILD_DIR)/debug/HexSight

.PHONY: all build build-rust build-swift run clean

all: build

# === Rust 构建 ===
build-rust:
	@echo "=== 构建 Rust 引擎 ==="
	cargo build --release
	@echo "Rust 构建完成: $(RUST_LIB)"

# === Swift 构建 ===
build-swift: build-rust
	@echo "=== 构建 Swift 应用 ==="
	cd swift && swift build
	@echo "Swift 构建完成"

# === 完整构建 ===
build: build-swift

# === 运行 ===
run: build
	@echo "=== 启动 HexSight ==="
	$(APP_BUNDLE)

# === 清理 ===
clean:
	@echo "=== 清理构建产物 ==="
	cargo clean
	cd swift && swift package clean
	@echo "清理完成"

# === 检查 ===
check:
	cargo check 2>&1

# === Rust 测试 ===
test-rust:
	cargo test --workspace

# === Swift 测试 ===
test-swift:
	cd swift && swift test

# === 生成 FFI 头文件 ===
header:
	cbindgen --config crates/hexsight-ffi/cbindgen.toml \
	         --crate hexsight-ffi \
	         --output swift/Sources/HexSight/Bridge/hexsight_generated.h 2>/dev/null || \
	echo "请安装 cbindgen: cargo install cbindgen"

# === 格式化 ===
fmt:
	cargo fmt
	cd swift && swiftformat . 2>/dev/null || echo "swiftformat 未安装，跳过"
