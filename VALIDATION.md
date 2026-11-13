# 历史验证记录

以下记录对应 2026-11-08 的源码和构建。后续子模块迁移、共享 UI 包、显示与 Fastboot 修改需要重新验证；本文不将历史结果视为当前版本通过。

## 2026-11-08 源码拆分验证

接口、运行时和默认页面以 `uefi-controls-20261105` 对应源码为基线；固定 PocketJS 上游快照的 197
个登记文件在迁移前已逐一验证。引擎实现未作本板修改，字体和上游许可随源代码保留。

- Rust 阶段使用 Rust 1.93.0。
- 运行时测试通过：20 项库测试、1 项 BootDisplay、4 项旋转、4 项设置，共 29 项。
- 公共接口 workspace 的 5 项 Boot API 测试通过。
- `default-ui` 独立 `Cargo.lock` 已同步到当前路径依赖，`cargo test --locked` 和文档测试成功；该 crate
  本身没有独立测试用例，页面行为由运行时测试覆盖。
- UEFI C 硬件桥接、启动日志及设置文件故障注入共 3 项测试通过。
- 远端 Rust 归档、QuickJS 源码和源码身份清单传回后，接收端哈希验证通过；本地主机未依赖 Rust 编译器完成 C 组件构建。
- EDK II / libc ABI 支持库和完整 `PocketJS.a` 构建成功，并在最终 FD 中通过无未定义符号链接检查。

完整归档 `PocketJS.a` SHA-256：

```text
b77868e3beaab1915b97fb343b586a61c1c756ea67e83dd656b0834a2ab1af90
```

`runtime-build.json` 记录当前源码与归档、ABI 清单的对应关系；`abi-support/manifest.json` 记录固定 EDK
II / libc 版本及 39 个文件的哈希。重新构建受工具链和构建路径影响，不承诺与历史发布二进制逐字节一致。本次未做新固件真机验证。

## 2026-11-08 外部 UI 集成验证

在同一台 Rust 编译机上使用隔离的 Rust 1.93.0 验证本次修改：

- 6 项 UI 输入测试通过，包含实际 Cargo 消费工程；确认脚本加载顺序、JS 与字体更新、切换带空格的 UI 路径均在共享目标目录下正确更新嵌入内容。
- 输入清单可随 UI 目录迁移，哈希身份保持一致；缺文件、重复脚本、越界路径和目录外符号链接被拒绝。
- 默认 UI 的 29 项运行时测试、5 项 Boot API 测试及 `default-ui` crate/文档测试通过。
- Rust-only 入口的旧位置参数和新 `--output` / `--default-ui` 参数均完成 ARM64 构建。
- 外部 UI 在原默认页面后增加独有标记脚本；标记只存在于外部 UI 归档，默认 UI 归档中不存在。
- 将外部 UI 与 Rust bundle 传回不同绝对路径后，源码、UI 和文件哈希校验通过；本地无需 Rust，完成 UEFI C 组件编译、完整
  `PocketJS.a` 和最终固件链接。

本次外部 UI 测试归档 `PocketJS.a` SHA-256：

```text
1790325bb3cdf481835cf31b046f0dd89220493e59b0fb206b170923e18b24c3
```

新版 `rust-build.json` 和 `runtime-build.json` 使用 schema 2，记录有序脚本、字体及所有所选 UI
输入的哈希。该阶段未做新固件真机验证。

## 2026-11-08 运行时目录整理验证

UEFI 宿主移到 `uefi-runtime/`，Cargo 包名为 `pocketjs-uefi-runtime`；通用执行层移到
`crates/embedded-runtime/`，包名为 `pocketjs-embedded-runtime`。调试通道和字体资源库归入
`crates/`，专题归入 `docs/`，上游快照与来源清单归入 `vendor/`。

- 原上游目录的 200 个文件逐一核对，内容保持一致；其中来源清单登记的 197 个文件全部通过哈希校验。
- Cargo 路径依赖、构建脚本、Python 语法和 160 个文档内部链接检查通过，迁移文档通过固定版本 mdformat 检查。
- 固定 Rust 1.93.0 下，29 项 UEFI 运行时、5 项 Boot API、10 项通用执行层、2 项调试通道测试通过；默认 UI crate
  和文档测试通过。
- 6 项 UI 输入与真实 Cargo 集成测试通过，覆盖外部脚本和字体更新及共享缓存下切换 UI 目录。
- 本地 3 项实际 UEFI C 适配测试通过。
- 默认和外部 UI 均完成 ARM64 构建；新归档记录全部新源码目录，旧目录的归档被拒绝复用。
- 编译机归档传回后校验通过，完成 UEFI C 组件构建；当前未做真机验证。

外部 UI 集成验证归档 `PocketJS.a` SHA-256：

```text
c75d0b87ac76d7648d56ec7a42440efabfec6f884b18d8e77861e13146b80eb1
```


## 2026-11-08 上游源码选择验证

当时的构建脚本直接获取 `edk2-libc`、PocketJS、QuickJS-NG 的 Git 源码；当前 PocketJS 和 edk2-libc 已改用完整 Git 子模块，见 [上游说明](docs/upstream-sources.md)。 默认沿用固定提交；QuickJS-NG 固定为
`rquickjs-sys 0.12.2` 对应的 `fd0a0210b7be00957751871e7e01b8291268fc29`。

- 三个默认仓库均已真实下载。PocketJS 的 197 个登记文件逐一匹配旧快照； QuickJS 的核心解释器源码和头文件与旧归档一致。
- 11 项源码选择、UI 输入和真实 Cargo 集成测试通过，覆盖固定提交、标签、分支、
  自定义仓库、移动的默认分支、固定清单离线重放、缓存被改及不存在的引用。
- 从下载的 PocketJS 准备独立源码树，Rust 1.93.0 的 ARM64 交叉构建通过； 该源码树的 29 项运行时、10 项嵌入式执行层、5 项
  Boot API、2 项调试通道测试通过， 默认 UI crate 和文档测试通过。
- 远端归档传回后，通过源码、实际提交、QuickJS 文件和 Cargo 锁校验； 使用下载的 edk2-libc 和 QuickJS 完成 C
  组件编译及完整固件链接和封装。
- 实际 QuickJS `latest` 解析并下载为 `d4187c7e4b57dc62c2dcbee14d25e0bd8162b690`；
  使用该选择复用默认版本 Rust bundle 时被拒绝。 此项验证下载和身份校验，未验证此最新版本的完整固件兼容性。

schema 3 的 `rust-build.json`、`runtime-build.json` 和最终固件清单记录实际选择，
`upstream.lock.json` 支持重放，实际 Cargo 锁保存在产物中。

本次默认版本 `PocketJS.a` SHA-256：

```text
19b2422e2959cae191697601a056cf37cf461718b5ffcda3352fed68644fa841
```

本次未刷写或新增真机验证。

## 2026-11-08 通用 EDK II 独立构建验证

移除对板级仓库的默认路径和固定版本约束。在只有 `pj4edk2` 的独立目录中构建， 未传入 `--edk2`，也未设置
`EDK_TOOLS_PATH`；通用 EDK II、私有 BaseTools、 完整 `PocketJS.a` 和 ABI
支持库均成功生成。实际源码路径不包含板级仓库。

- 16 项源码选择与 UI 集成测试通过，覆盖通用 EDK II 获取、外部 checkout 的不同提交、 两个 Brotli 浅克隆、固定
  gitlink、缓存修改拒绝及 Rust 阶段不下载 EDK II。
- Rust 编译机使用隔离的 Rust 1.93.0 完成 AArch64 构建；传回的源码身份、UI、 QuickJS 和归档哈希校验通过，原有默认
  Rust 1.99.0 保持。
- EDK II 和第一份 Brotli 从上游下载；第二份 Brotli 下载连接停滞后， 复用本轮已下载的相同 Git 对象。两个子模块均核对为 EDK
  II 指定的 `f4153a09f87cbb9c826d8fc12c74642bb2d879ea`。
- 使用独立 EDK II 头文件完成 3 项 UEFI C 适配测试。

独立 `PocketJS.a` SHA-256：

```text
70d4d034d9c1c5ea22e29b5501f038b7d689b6dda374521a669aebb6cdfd08a1
```

本次验证默认固定版本和 AArch64 构建，未验证其他 EDK II 版本或 CPU 架构的完整兼容性，未刷写设备。
