# 用户层存储与调试设计

本页描述页面包的存储、加载和调试接入，供宿主与应用开发者参考。

> [!NOTE]
> 状态：UEFI 已实现从选定 ESP 读取共享 UI 包，详见 [UI 包说明](ui-package.md)。独立内容分区、运行期间热更新及跨 EFI 驻留实例仍为设计；数据队列与 BSP 通知已有实现，实体调试传输尚未接入。

## 页面与引擎分离

普通 PocketJS 应用由 TypeScript/TSX 等页面源码、Solid/Vue Vapor/Octane
适配层及样式、字体、图片等资源构成。构建结果为 JS 与 baked 资源，可放进上游 `.pocket` 容器。当前容器 v2 支持 manifest、目标
variant、宿主 ABI、profile hash，以及 JS、PAK、build plan 等 section。

这些文件是这里所说的“用户层”，不包含驻留 EFI 服务的代码或活的内存堆。这个名称也不意味着 UEFI 中存在操作系统式的用户态/内核态隔离。PocketJS
的 MicroTS AOT 是另一条编译路线；本方案先沿用可从外部加载 JS 包的 QuickJS 路线。

| 组成                                          | 位置                                         | 实现语言                          |
| --------------------------------------------- | -------------------------------------------- | --------------------------------- |
| 稳定的 ABI 与可选消费端辅助函数               | Rust 定义和实现，自动生成 `.h` 给其他 loader | C ABI                             |
| 驻留服务、显示/输入、分配器、包来源与调试通道 | EFI 映像                                     | 我们新增代码用 Rust；保留上游依赖 |
| 页面、交互逻辑和打包资源                      | `.pocket` 文件，可独立分区                   | JS/TS 及二进制资源                |
| 已初始化的 QuickJS/UI 状态及其堆              | 当前启动会话的 RAM                           | 原驻留服务拥有                    |

## 独立分区

可以将 EFI 代码和页面包分别存放，例如以下规划路径：

```text
ESP（FAT32）
  EFI/BOOT/BOOTAA64.EFI       # ARM64 loader；X64 对应 BOOTX64.EFI
  EFI/PocketJS/runtime.efi   # 驻留服务

PocketJS 内容分区（FAT32）
  PocketJS/boot-ui.pocket
```

这些是规划名称，并非当前已有可启动产物。Rust loader 按 GPT 分区唯一 GUID / UEFI device path 定位指定卷，再通过该卷的
Simple File System 协议读取文件。内容分区的标识独立于配置表的自定义 GUID，不能混用二者。

FAT32 适合第一版。仍需确认目标固件实际为这个卷暴露文件系统协议；只创建一个任意类型的分区不能保证固件自动识别。若没有文件系统协议，需在 Rust
中另行适配 Block I/O 和文件系统。ext4、F2FS 等也可设计支持，但不是 UEFI 通用文件服务必然提供的能力。

更新页面包无需重新编译或刷写驻留引擎。无需实际创建分区即可先用 ESP 上的文件验证相同的包读取路径。

## 首次加载与实例接续

首次使用时，Rust 服务读取 `.pocket` 到服务拥有的稳定内存，通过上游 `LoadedPackage` 校验 target、host
ABI、独立的预期 profile hash、布局和内容校验，再创建一次 JS 页面并加载资源。上游已有 `PackageSource` 和
`OwnedMemorySource`，可以复用解析及所有权逻辑；UEFI 已接入独立的 `pj4edk2-ui` profile 和 ESP 文件来源，现有其他设备的包不能直接当作 UEFI UI 包使用。

接续的 loader 发现配置表后 ACQUIRE 同一个实例，不再调用 `guest.initialize`、重复 evaluate
页面或从分区重新载入当前包。已加载包的字节、字体和图片必须与原实例一起保留；不能让它们依赖已经关闭的文件对象或前一 loader 的临时缓冲区。

同一次启动中更新磁盘文件不会自动改变已经运行的页面。切换页面包是显式更新操作，需要校验新包并在帧边界切换页面与资源；引擎可以继续驻留，但应用变量和页面状态的迁移另行定义。活堆只在
RAM 中保留，不能通过把堆写入内容分区就跨重启恢复内部指针和回调。

## 上游已有的实时调试

依据当前固定提交 `e6770376626a4de5c93eaa6fb048e7e9ffb81f9f` 的
[DEVTOOLS.md](https://github.com/pocket-nexus/pocketjs/blob/e6770376626a4de5c93eaa6fb048e7e9ffb81f9f/docs/DEVTOOLS.md)
及上游 core / UI ops，已有：

- 组件树与节点高亮；
- 日志、异常和帧信息；
- 在帧间执行 JS 的 REPL；
- 配合 JS shim 的暂停、恢复、逐帧和输入录制；回放/跳转还需要宿主实现；
- 部分宿主的截图和包推送更新，能力取决于宿主。

这不等于 QuickJS 源码行断点调试；文档明确将字节码调试 hook、源码断点和相应 source map 链路列为
deferred。页面包更新也不等于自动保留全部页面状态的 HMR。

上游调试 shim 使用 `send(line)` / `recv()`，协议为 JSON Lines；已有
`getTree`、`inspect`、`pause`、`resume`、`step`、`eval`、`dumpTape` 等命令。浏览器使用
WebSocket，部分设备使用 USB 文件邮箱或 TCP 桥接；这些传输不能原样假定在 UEFI 中存在。

PJB 现在提供默认关闭的[有界数据通道](debug-channel.md)，只有用户在设置中主动打开 `PocketJSDebug` 才请求 BSP 连接。
开关只作用于当前运行时，不写入启动配置；PJB 只转交不透明字节，不内置物理传输或命令解析。建议 BSP 在开启时接通板级通道，关闭时断开。

接入上游 DevTools 仍需打包调试 shim 并提供协议适配。协议命令应在 UI 核的帧边界执行，不在设备回调中重入
QuickJS，也不绕过实例的独占租约。 当前默认 plain JS 页面尚未打包框架调试 shim，因此不能把上游调试能力当作现有 UEFI 通道已跑通。
