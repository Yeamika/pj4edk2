# PocketJS for EDK II

独立维护 UEFI 引导菜单的公共接口、PocketJS 运行时和默认页面，输出 `PocketJS.a` 及配套的 UEFI ABI 支持库。板级驱动由 `edk2-ums9620` 提供，菜单配置和最终固件组合由 `w202ds-boot` 提供。

## 源码组成

| 目录 | 内容 |
| --- | --- |
| `pocketjsforedk2/` | `no_std` 公共 ABI、生成的 C 头文件、Boot API 和接口测试 |
| `default-ui/` | 默认菜单、设置、恢复和充电页面，以及字体资源 |
| `reference/pocketjs-uefi-platform/` | Rust 运行时和 UEFI C 胶合：绘制、输入、硬件协议、设置存储及子应用启动 |
| `reference/pocketjs-uefi-handoff/crates/` | 嵌入式引擎适配和调试通道 |
| `reference/crates/boot-assets/` | 字体图集等共享资源的构建 |
| `reference/vendor/pocketjs/` | 固定上游引擎快照，保留上游许可和文件哈希；引擎源码未作本板修改 |
| `edk2-libc/` | 固定版本的 Tianocore C 库子模块 |

版本固定在 [sources.lock.json](sources.lock.json)、各 `Cargo.lock` 和 `rust-toolchain.toml`。上游引擎文件校验表位于 `reference/pocketjs-uefi-handoff/vendor/UPSTREAM.json`。接口只在 `pocketjsforedk2` 定义，消费端不另存一份公共 ABI。

## 构建

需要 Rust 1.93.0（含 `rust-src` 和 `aarch64-unknown-none`）、Cargo、Python 3.11+、AArch64 GNU GCC/binutils，以及和平台一致的 EDK II 与 BaseTools。Rust 构建保留 `x18`，使用 `panic=abort`，不启用 LTO。

```sh
git submodule update --init edk2-libc
export EDK_TOOLS_PATH=/path/to/build/basetools
./build.sh --edk2 ../edk2-ums9620/edk2 --output /path/to/build/pocketjs
```

BaseTools 可以用 `../edk2-ums9620/build-tools.sh /path/to/build/basetools` 生成。构建输出必须位于源码树外；Cargo 缓存使用正常的 `CARGO_HOME`，目标文件默认放在输出目录。

| 输出 | 用途 |
| --- | --- |
| `PocketJS.a` | Rust 引擎/页面、QuickJS 和 UEFI C 胶合的完整静态归档 |
| `abi-support/` | 与这次构建匹配的 EDK II / libc 库、入口对象、链接脚本和哈希清单 |
| `packages/source-manifest.json` | 编译所用 QuickJS、引擎及 EDK II 版本信息 |

`PocketJS.a` 交给 `w202ds-boot` 链接进平台 FD；它本身不是可刷写镜像。板级菜单 JSON 不固化在这个独立归档中，而由最终链接阶段提供。

## 使用 Rust 编译机

可以只在编译机完成 Rust 阶段，再在拥有 AArch64 C 工具链和 EDK II 的主机完成公共 UEFI 组件编译：

```sh
# 编译机：使用同一份 pj4edk2 源码
./build-rust.sh /path/to/build/rust

# 传回 Rust.a、quickjs/、rust-build.json，保留目录结构
# 本地主机：不需要本地安装 Rust
./build.sh --edk2 ../edk2-ums9620/edk2 \
  --rust-build /path/to/received/rust --output /path/to/build/pocketjs
```

接收端会校验 Rust 版本、构建源码和所有传回文件的哈希，防止拿错源码或归档。默认页面和字体的许可证随对应源码保留。

## 测试

```sh
export CARGO_TARGET_DIR=/path/to/build/native-target
export PJB_EDK2_ROOT="$(realpath ../edk2-ums9620/edk2)"
bash test.sh
```

`test.sh` 自动向本机测试提供固定 QuickJS 头文件，并编译官方 guest。只在 Rust 编译机运行时可以传入 `--rust-only`，UEFI C 测试在本地另行执行。

本仓库不依赖旧研究目录或 Windows 演示工程。底层模块文档保留接口设计与调试说明，当前构建入口以本页为准。实际验证范围见 [VALIDATION.md](VALIDATION.md)。
