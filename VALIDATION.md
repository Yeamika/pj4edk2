# 2026-11-08 源码拆分验证

接口、运行时和默认页面以 `uefi-controls-20261105` 对应源码为基线；固定 PocketJS 上游快照的 197 个登记文件在迁移前已逐一验证。引擎实现未作本板修改，字体和上游许可随源代码保留。

- Rust 阶段在现有 `NativeCmpEnv` / `p90-rust-builder` 上使用隔离的 Rust 1.93.0，未改变编译机原有 1.99.0 默认环境。
- 运行时测试通过：20 项库测试、1 项 BootDisplay、4 项旋转、4 项设置，共 29 项。
- 公共接口 workspace 的 5 项 Boot API 测试通过。
- `default-ui` 独立 `Cargo.lock` 已同步到当前路径依赖，`cargo test --locked` 和文档测试成功；该 crate 本身没有独立测试用例，页面行为由运行时测试覆盖。
- UEFI C 硬件桥接、启动日志及设置文件故障注入共 3 项测试通过。
- 远端 Rust 归档、QuickJS 源码和源码身份清单传回后，接收端哈希验证通过；本地主机未依赖 Rust 编译器完成 C 组件构建。
- EDK II / libc ABI 支持库和完整 `PocketJS.a` 构建成功，并在最终 FD 中通过无未定义符号链接检查。

完整归档 `PocketJS.a` SHA-256：

```text
b77868e3beaab1915b97fb343b586a61c1c756ea67e83dd656b0834a2ab1af90
```

`runtime-build.json` 记录当前源码与归档、ABI 清单的对应关系；`abi-support/manifest.json` 记录固定 EDK II / libc 版本及 39 个文件的哈希。重新构建受工具链和构建路径影响，不承诺与历史发布二进制逐字节一致。本次未做新固件真机验证。

## 2026-11-08 外部 UI 集成验证

在同一台 Rust 编译机上使用隔离的 Rust 1.93.0 验证本次修改：

- 6 项 UI 输入测试通过，包含实际 Cargo 消费工程；确认脚本加载顺序、JS 与字体更新、切换带空格的 UI 路径均在共享目标目录下正确更新嵌入内容。
- 输入清单可随 UI 目录迁移，哈希身份保持一致；缺文件、重复脚本、越界路径和目录外符号链接被拒绝。
- 默认 UI 的 29 项运行时测试、5 项 Boot API 测试及 `default-ui` crate/文档测试通过。
- Rust-only 入口的旧位置参数和新 `--output` / `--default-ui` 参数均完成 ARM64 构建。
- 外部 UI 在原默认页面后增加独有标记脚本；标记只存在于外部 UI 归档，默认 UI 归档中不存在。
- 将外部 UI 与 Rust bundle 传回不同绝对路径后，源码、UI 和文件哈希校验通过；本地无需 Rust，完成 UEFI C 组件编译、完整 `PocketJS.a` 和最终固件链接。
- 实际产物的 UI 与重新选择的 UI 不一致时，`w202ds-boot` 拒绝复用。

本次外部 UI 测试归档 `PocketJS.a` SHA-256：

```text
1790325bb3cdf481835cf31b046f0dd89220493e59b0fb206b170923e18b24c3
```

新版 `rust-build.json` 和 `runtime-build.json` 使用 schema 2，记录有序脚本、字体及所有所选 UI 输入的哈希。编译机临时目录、私有工具链和缓存已清理，原有默认 Rust 1.99.0 保持。本次未做新固件真机验证。
