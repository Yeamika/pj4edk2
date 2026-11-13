# 上游源码与版本选择

PocketJS 与 edk2-libc 是固定提交的 Git 子模块，构建直接读取它们，不自动下载或切换版本：

```sh
git submodule update --init vendor/pocketjs vendor/edk2-libc
```

| 依赖 | 来源 | 默认提交 |
| --- | --- | --- |
| PocketJS | `vendor/pocketjs`，完整上游源码 | `e6770376626a4de5c93eaa6fb048e7e9ffb81f9f` |
| edk2-libc | `vendor/edk2-libc` | `73056078b60c9378af08dd122a29913e25e84e1c` |
| EDK II | 构建缓存或 `--edk2 DIR` | `e7aac7fc137e247edad22f7ee53b9a1fba227397` |
| QuickJS-NG | 构建缓存 | `fd0a0210b7be00957751871e7e01b8291268fc29` |

子模块用父仓库的 gitlink 固定版本。升级时先在子模块中 fetch、checkout 指定提交，再由父仓库记录新 gitlink；
选择最新版本可显式运行 `git submodule update --remote vendor/pocketjs`。fork 可通过子模块的 origin 及
`.gitmodules` URL 配置。构建记录实际 HEAD，拒绝子模块内未记录的修改。

EDK II 与 QuickJS 继续支持 `--source-repo NAME=URL`、`--source-ref NAME=COMMIT|BRANCH|TAG|latest`。
这些参数不适用于上述两个子模块；请先用 Git 选好其版本。QuickJS 与 `rquickjs-sys 0.12.2` 的对应关系可在
[rquickjs 的子模块记录](https://github.com/DelSkayn/rquickjs/tree/v0.12.2/sys/quickjs)核对。

```sh
./build.sh --output /path/to/build/runtime \
  --source-cache /path/to/cache/upstream \
  --source-ref quickjs=latest
```

`--source-cache DIR` 或 `PJB_SOURCE_CACHE` 指定 EDK II/QuickJS 的外部 Git 缓存；默认位于构建输出。
固定提交已在缓存时无需联网。`latest`、分支或标签会重新解析；构建不会静默改用其他提交。

每次构建的 `upstream.lock.json` 记录实际仓库与提交。`--source-lock FILE` 重放时，子模块须已检出清单中的仓库和提交；
不匹配时明确失败，构建不会改写子模块。`sources.lock.json` 提供下载依赖的默认版本及初始子模块参考版本。

`--edk2 DIR` 接受独立的通用 EDK II checkout，与默认版本无须相同，也不下载或修改它；
不能同时使用 EDK II 的仓库/引用覆盖。调用方需先初始化其 Brotli 子模块：

```sh
git -C /path/to/generic-edk2 submodule update --init --depth=1 -- \
  BaseTools/Source/C/BrotliCompress/brotli \
  MdeModulePkg/Library/BrotliCustomDecompressLib/brotli
```

完整构建在输出目录编译 BaseTools；`EDK_TOOLS_PATH` 可复用已编译的工具。
Rust 阶段 `build-rust.sh` 不需要 EDK II，生成 `Rust.a`、`quickjs/`、`Cargo.lock`、
`upstream.lock.json` 和 `rust-build.json`。接收端用 `--rust-build DIR --source-lock FILE` 核对同一批输入。

UEFI 构建会在输出的 `source-tree/` 中准备私有副本，仅在该副本中缩小 PocketJS 的 Cargo workspace 成员列表，
避免无关平台扩展依赖图；不会删改子模块中的上游源码。WASM 等其他宿主可直接访问完整子模块中的对应工程。
Cargo 锁与实际来源随产物记录，构建输出和缓存须位于源码树及 UI 目录之外。

最终组合工程仍需核对 `abi-support/manifest.json` 的 EDK II/libc 版本与平台一致；
切换版本可能需要接口适配。当前运行时构建清单使用 schema 3，旧归档须重建。
