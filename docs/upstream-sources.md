# 选择上游源码

`build.sh`、`build-rust.sh` 和 `w202ds-boot/build.py` 使用同一组上游源码选项。
构建默认从上游获取已固定的提交，首次下载后可以复用外部缓存。

| 名称        | 默认仓库                                     | 默认提交                                   |
| ----------- | -------------------------------------------- | ------------------------------------------ |
| `edk2-libc` | `https://github.com/tianocore/edk2-libc.git` | `73056078b60c9378af08dd122a29913e25e84e1c` |
| `pocketjs`  | `https://github.com/pocket-nexus/pocketjs`   | `e6770376626a4de5c93eaa6fb048e7e9ffb81f9f` |
| `quickjs`   | `https://github.com/quickjs-ng/quickjs.git`  | `fd0a0210b7be00957751871e7e01b8291268fc29` |

默认值以根目录 `sources.lock.json` 为准。QuickJS 使用当前 `rquickjs-sys 0.12.2` 对应的 QuickJS-NG
提交；其来源可在
[rquickjs 的 Git 子模块记录](https://github.com/DelSkayn/rquickjs/tree/v0.12.2/sys/quickjs)
核对。构建脚本直接编译选择的 QuickJS Git 源码；该选择不受 Cargo 中的测试解释器版本替代。 EDK II 本身继续使用
`edk2-ums9620` 固定的平台版本。

## 参数

- `--source-ref NAME=REV`：选择完整提交、标签或分支，可重复指定不同依赖。
- `--source-ref NAME=latest`：选择所选仓库默认分支的最新提交。
- `--source-repo NAME=URL`：选择上游或自己的 fork，可与对应的 `--source-ref` 配合。
- `--source-cache DIR`：源码 Git 缓存，必须位于源码树和 UI 目录之外；也可设置
  `PJB_SOURCE_CACHE`。不指定时放在构建输出内。
- `--source-lock FILE`：用之前生成的 `upstream.lock.json` 重放具体提交。 仍可用上述仓库和引用参数覆盖其中某一项。

不传选择参数时始终使用默认固定提交。选择其他仓库而不传引用时，仍使用该依赖的默认提交， 因此 fork
必须包含该提交。`latest`、分支及标签每次联网解析，解析失败时构建失败。 固定提交在缓存完整时无需联网；源码缓存被修改时会拒绝复用。 Git
使用调用者现有的代理和认证配置，脚本不修改全局网络设置。

## 示例

固定默认版本并共享下载缓存：

```sh
./build-rust.sh --output /path/to/build/rust-pinned \
  --source-cache /path/to/cache/upstream
```

选择 PocketJS fork 的某个分支，并更新 QuickJS：

```sh
./build-rust.sh --output /path/to/build/rust-custom \
  --source-cache /path/to/cache/upstream \
  --source-repo pocketjs=https://github.com/example/pocketjs.git \
  --source-ref pocketjs=my-integration \
  --source-ref quickjs=latest
```

重放同一批源码：

```sh
./build-rust.sh --output /path/to/build/rust-replay \
  --source-cache /path/to/cache/upstream \
  --source-lock /path/to/build/rust-custom/upstream.lock.json
```

也可先单独下载并解析依赖：

```sh
python3 tools/upstream.py resolve \
  --output /path/to/build/upstream.lock.json \
  --source-cache /path/to/cache/upstream \
  --source-ref edk2-libc=latest
```

## 接入、编译机和构建记录

源代码缓存按仓库和实际提交区分。每次构建在输出内创建 `source-tree/`， 把本项目代码与选定的 PocketJS 源码接在一起，并限制
PocketJS workspace 为固件相关成员。 保留选定上游的包元数据和依赖；不改写原仓库、原子模块或已下载的上游源码。 默认 UI 仍可从外部传入。

Cargo 在私有源码树中解析所选版本的依赖，随后使用锁定结果编译。 实际 `Cargo.lock` 随产物保存；源码树内原有的锁文件保持不变。
上游的接口、目录或最低编译器版本改变时，所选版本可能需要适配；脚本会报告实际构建错误， 不会改回其他提交。

Rust 编译机阶段传回 `Rust.a`、`quickjs/`、`Cargo.lock`、`upstream.lock.json` 和
`rust-build.json`。接收端使用 `--source-lock` 指向同一份清单， 避免 `latest`
在两个阶段之间变化。接收端核对项目源码、UI、上游提交、QuickJS 源文件和归档哈希。

`runtime-build.json` 使用 schema 3，记录固定后的上游身份和实际 Cargo 锁哈希。
`packages/source-manifest.json` 记录 C 组件的实际源码版本；完整固件的 `manifest.json`
也保留上游清单。升级前的 schema 2 Rust/运行时归档需要重建。
