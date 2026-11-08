# 接入与验证

在项目根目录操作。接口 workspace 需要 Rust 1.85 或以上；当前检查使用 Rust 1.93.0。 普通 Rust 检查不需要安装
PocketJS、QuickJS 或设备工具链。

## Rust 接入

按需依赖 ABI 库或 `crates/boot-api`。在兄弟工程的 Cargo manifest 中可使用：

```toml
[dependencies]
pocketjs-boot-context-abi = { path = "../pocketjsforedk2" }
pocketjs-boot-api = { path = "../pocketjsforedk2/crates/boot-api" }
```

路径相对调用方 manifest；嵌套 crate 需要调整层级。实现 `BootServices` 并提供自己的 `Error`、`Runtime`
和设备/存储后端。

```sh
cargo test --locked --workspace
rustup target add aarch64-unknown-uefi
cargo check --locked --workspace --target aarch64-unknown-uefi
```

这些检查验证接口、序列化、边界与目标编译，不执行 EFI 映像或初始化引擎。

## C 头文件与互调

头文件只覆盖跨 EFI ABI；Rust 服务 trait 不直接生成 C 接口。生产 `.h` 由固定版本 cbindgen 与 Rust 生成器产生。

```sh
cargo install cbindgen --version 0.29.2 --locked
mkdir -p /tmp/pocketjs-interface-check
rustc --edition=2024 tools/verify.rs -o /tmp/pocketjs-interface-check/verify
TMPDIR=/tmp/pocketjs-interface-check /tmp/pocketjs-interface-check/verify . --native-only
```

本机检查需要 X64 Rust 工具链和 C 编译器 `cc`，覆盖生成内容、C11 布局以及 32 轮父子句柄转发。 去掉 `--native-only`
时还需要 `aarch64-linux-gnu-gcc`，并额外检查 AArch64 C 布局。 需要重建头文件时追加
`--write-header`，然后复核差异并重新执行检查。

## 参考实现回归

抽象接口变更后，按[仓库测试入口](../../README.md#%E6%B5%8B%E8%AF%95)运行 `uefi-runtime` 和
`crates` 的调用方回归。 引擎、C guest、渲染和设备测试使用各自的工具链及依赖，不进入接口工程。
