# PocketJS for EDK II

独立维护 UEFI 引导菜单的公共接口、PocketJS 运行时和默认页面，输出 `PocketJS.a` 及配套的 UEFI ABI 支持库。 依赖通用
EDK II 和 UEFI 协议，可由不同平台接入。板级驱动、菜单配置和最终固件组合由消费项目提供； 当前验证的架构为 AArch64。

本页对应 `uefi` 分支。其他组件与实验环境见仓库的
[分支导航](https://github.com/Yeamika/pj4edk2/tree/main)。

## 源码组成

| 目录                       | 内容                                                                  |
| -------------------------- | --------------------------------------------------------------------- |
| `pocketjsforedk2/`         | `no_std` 公共 ABI、生成的 C 头文件、Boot API 和接口测试               |
| `default-ui/`              | 默认菜单、设置、恢复和充电页面，以及字体资源                          |
| `uefi-runtime/`            | Rust 运行时和 UEFI C 胶合：绘制、输入、硬件协议、设置存储及子应用启动 |
| `crates/embedded-runtime/` | 通用嵌入式执行层：JS 实例、页面上下文、输入和渲染适配                 |
| `crates/rendering/` | UEFI 与 WASM 共用的像素对齐与纹理采样规则 |
| `crates/ui-package/` | 共享 `.pocket` UI 包的读取、校验与资源访问 |
| `tools/` | 依赖获取、UI 输入处理、包构建及测试工具 |
| `crates/debug-channel/`    | 有界调试消息队列                                                      |
| `crates/boot-assets/`      | 字体图集等共享资源的构建                                              |
| `vendor/pocketjs/`         | 完整上游源码的固定提交 Git 子模块，供固件与 WASM 共用     |
| `docs/`                    | 模块、接入说明及跨 EFI 实例交接等设计文档                             |
| `vendor/edk2-libc/`               | 固定提交 Git 子模块，构建直接读取此目录                     |

版本固定在 [sources.lock.json](sources.lock.json)、各 `Cargo.lock` 和
`rust-toolchain.toml`。PocketJS 与 edk2-libc 的版本由 Git 子模块记录，构建清单记录实际 HEAD。接口只在 `pocketjsforedk2`
定义，消费端不另存一份公共 ABI。

UEFI 宿主包名为 `pocketjs-uefi-runtime`，通用执行层包名为 `pocketjs-embedded-runtime`。
运行时代码按职责直接放在上述目录中；跨 EFI 共享实例仍是设计，见[模块与状态说明](docs/modules.md)。

## 构建

需要 Git、Rust 1.93.0（含 `rust-src` 和 `aarch64-unknown-none`）、Cargo、Python 3.11+、GNU
make、主机 GCC/G++、UUID 开发头文件和 AArch64 GNU GCC/binutils。Rust 构建保留 `x18`，使用
`panic=abort`，不启用 LTO。

```sh
git submodule update --init vendor/pocketjs vendor/edk2-libc
./build.sh --output /path/to/build/pocketjs \
  --source-cache /path/to/cache/upstream
```

脚本独立下载通用 EDK II 及其固定的 BaseTools、MdeModulePkg Brotli 子模块，并在输出目录编译 BaseTools。
无需任何板级仓库。构建输出必须位于源码树外；Cargo 缓存使用正常的 `CARGO_HOME`，目标文件默认放在输出目录。

PocketJS 和 edk2-libc 直接使用已初始化的 `vendor/` 子模块；构建不会下载或切换这两个仓库。
切换版本时，先在对应子模块中 checkout 指定提交；更新后由父项目记录新的 gitlink。
EDK II 与 QuickJS-NG 仍支持指定上游、提交或最新版本：

```sh
./build.sh --output /path/to/build/pocketjs \
  --source-cache /path/to/cache/upstream \
  --source-ref quickjs=latest
```

构建把实际提交写入 `upstream.lock.json`，可用 `--source-lock FILE` 重放；子模块必须已检出清单中的提交。
Rust 阶段不下载 EDK II；平台组合工程可指定自己使用的 EDK II。具体用法见
[上游源码选择](docs/upstream-sources.md)。

其他项目可以传入自己的默认 UI 目录和输出目录：

```sh
/path/to/pj4edk2/build.sh \
  --default-ui /path/to/my-project/ui \
  --output /path/to/my-project/build/pocketjs \
  --edk2 /path/to/generic-edk2
```

UI 目录包含 `ui.json`、按清单排列的 JavaScript 文件和字体，无需 `Cargo.toml` 或 Rust 源码。 可从仓库的
`default-ui/ui.json`、`pages/`、`assets/` 开始修改，目录格式和宿主入口见
[UI 集成说明](default-ui/README.md)。省略 `--default-ui` 时使用仓库自带 UI；也可以通过
`PJB_DEFAULT_UI` 环境变量指定，命令行参数优先。此变量也适用于直接调用 Cargo。

`--edk2 DIR` 使用调用方提供的 EDK II Git checkout，不检查它是否等于本项目的默认提交， 也不下载或修改它；使用时不能同时指定
`--source-ref edk2=...` 或 `--source-repo edk2=...`。 外部 EDK II 的两个 Brotli
子模块需由调用方初始化。`EDK_TOOLS_PATH` 可复用已编译的工具； 也可单独运行
`python3 tools/build-basetools.py --edk2 DIR --output /path/to/build/basetools`。
运行时 ABI 支持库与平台 EDK II 的版本匹配由最终组合工程检查。

JS 源码和字体图集仍然在构建时嵌入 `PocketJS.a`。UI 路径、清单、JS 或字体变化都会触发 Cargo 更新相应内容。输出与
`CARGO_TARGET_DIR` 必须位于源码和 UI 目录外；默认所有目标文件 都进入 `--output`，也可设置外部
`CARGO_TARGET_DIR` 来共享编译缓存。

| 输出                            | 用途                                                            |
| ------------------------------- | --------------------------------------------------------------- |
| `PocketJS.a`                    | Rust 引擎/页面、QuickJS 和 UEFI C 胶合的完整静态归档            |
| `abi-support/`                  | 与这次构建匹配的 EDK II / libc 库、入口对象、链接脚本和哈希清单 |
| `packages/source-manifest.json` | 编译所用 QuickJS、引擎及 EDK II 版本信息                        |
| `runtime-build.json`            | 源码、所选 UI 输入及最终归档的对应哈希                          |
| `upstream.lock.json`            | 四个上游仓库及这次实际使用的完整提交                            |
| `Cargo.lock`                    | 所选源码实际使用的 Rust 依赖锁                                  |

`PocketJS.a` 交给平台组合工程链接进 FD；它本身不是可刷写镜像。板级菜单 JSON 不固化在这个独立归档中，而由最终链接阶段提供。 W202DS
的消费端是 `w202ds-boot`；`edk2-ums9620` 仅负责该平台驱动，不是本仓库的依赖。

## 使用 Rust 编译机

可以只在编译机完成 Rust 阶段，再在拥有 AArch64 C 工具链和 EDK II 的主机完成公共 UEFI 组件编译：

```sh
# 编译机：使用同一份 pj4edk2 源码和 UI 文件
./build-rust.sh --default-ui /path/to/my-project/ui --output /path/to/build/rust

# 传回 Rust.a、quickjs/、Cargo.lock、upstream.lock.json、rust-build.json
# 本地主机：不需要本地安装 Rust
./build.sh \
  --default-ui /path/to/my-project/ui \
  --source-lock /path/to/received/rust/upstream.lock.json \
  --rust-build /path/to/received/rust --output /path/to/build/pocketjs
```

接收端可另加 `--edk2 /path/to/generic-edk2`，或使用默认的独立 EDK II 下载。 Rust 编译机阶段不需要 EDK
II、BaseTools 或板级源码。 接收端会校验 Rust 版本、构建源码、所选 UI 和所有传回文件的哈希。两端 UI 的绝对路径可以不同，
清单和输入文件内容必须一致；缺文件或 UI 不匹配会拒绝构建。schema 3 同时校验选定上游和 QuickJS 源码；旧版归档需要重新构建。
`build-rust.sh OUTPUT_DIRECTORY` 的旧调用方式仍可用。默认页面和字体的许可证随对应源码保留。

## 测试

```sh
export CARGO_TARGET_DIR=/path/to/build/native-target
export PJB_EDK2_ROOT=/path/to/generic-edk2
bash test.sh
```

`test.sh` 自动向本机测试提供固定 QuickJS 头文件，并编译官方 guest。只在 Rust 编译机运行时可以传入
`--rust-only`，UEFI C 测试在本地另行执行。 设置 `PJB_RUN_CARGO_TESTS=1` 可额外验证外部 UI 的真实嵌入，以及共享
Cargo 缓存下脚本、字体和目录切换后的重编译。

底层模块文档区分已实现功能与设计目标，当前构建入口以本页为准。历史验证范围见
[VALIDATION.md](VALIDATION.md)。

## 共享 UI 包

[共享 `.pocket` UI 包](docs/ui-package.md) 使用 PocketJS v2 容器，包含源 JavaScript、原字体和烘焙字形图集。
UEFI 与浏览器预览读取相同的 UI 包；这不代表两者具有相同的执行环境或性能。

```sh
python3 tools/pack-ui.py --output "$BUILD_DIR/ui.pocket" --target-dir "$CARGO_TARGET_DIR"
```

调用前将 `BUILD_DIR` 和 `CARGO_TARGET_DIR` 设置为源码树外的输出目录。
UEFI 可从选定 ESP 的 `EFI/BOOT/pj4edk/ui.pocket` 读取包；缺失时使用嵌入页面，存在但无效时拒绝执行。
Web 预览由独立的 `webui-viewer` 分支维护，QEMU 完整启动环境由 `qemu-arm64-small` 分支维护。
这些实验分支的发布状态以远端分支列表为准。

## 待办

- 可选 Fastboot 协议已有实现：菜单逐帧轮询，传输时暂停自动启动和关机，进入 EFI 子程序前停止 USB。接口见 `pocketjsforedk2/include/pj4edk2_fastboot.h`，通用运行时不依赖 LK 或具体芯片。
- W202DS 上菜单与 USB 同时运行、拔插重连及刷写仍需实机验收。宿主协议测试不能代替板级验证。
