# PocketJS for Boot 文档

本目录介绍 PocketJS 在 UEFI 中的宿主接入、实例交接、页面包与运行场景。先按阅读目的选择专题，再查看对应源码和验证范围。

## 入门

| 文档                                 | 适用目的                           |
| ------------------------------------ | ---------------------------------- |
| [项目概览](runtime-overview.md)      | 了解目标、当前能力与限制           |
| [入门与宿主验证](getting-started.md) | 检查依赖、生成头文件、运行现有验证 |

## 设计与接口

| 文档                                                | 适用目的                                              |
| --------------------------------------------------- | ----------------------------------------------------- |
| [整体设计](design.md)                               | 了解范围、宿主集成与生命周期                          |
| [模块说明](modules.md)                              | 查找模块职责、依赖和源码                              |
| [UEFI 实例交接协议](uefi-handoff.md)                | 接入 GUID、堆范围、实例和单个调用入口                 |
| [Rust ABI 与头文件](abi.md)                         | 理解 ABI 来源、调用约定和生成规则                     |
| [引导优先与多核](architecture.md)                   | 了解 BSP/AP 分工、队列、I/O 和退出协调                |
| [入口与运行场景](entry-modes.md)                    | 选择接入方式及 boot、recovery、charging 场景          |
| [用户层存储与调试](userspace.md)                    | 了解独立分区、页面包和 DevTools 接入                  |
| [调试数据通道](debug-channel.md)                    | 接入默认关闭的 PocketJSDebug、字节队列与 BSP 连接通知 |
| [默认页面与共享 UI](default-ui.md)                  | 接入页面、输入、动作及启动动画                        |
| [裸机公共层与 BSP](no-std-bsp.md)                   | 接入 `no_std` 宿主、脚本后端及板级 I/O                |
| [嵌入式 PocketJS 中间件](embedded-runtime.md)       | 复用官方 guest，在单片机与固件中执行脚本、输入和渲染  |
| [裸机显示与硬件加速](rendering.md)                  | 接入普通 framebuffer、GPU/2D、DMA 和输出同步          |
| [启动服务与页面嵌套](boot-services.md)              | 接入列表、设置、参数、运行时和显示区域，验证循环交接  |
| 独立 Windows demo（外部桌面参考工程，未纳入本仓库） | 构建、运行界面与导出预览                              |

## 文档维护

新增或修改文档遵循[文档规范](documentation-guide.md)。规范记录 Firecracker 的参考来源、本项目的编排方式，以及
Markdown 格式和链接检查方法。

## 当前实现状态

| 能力                                | 状态                 | 依据                                                                                              |
| ----------------------------------- | -------------------- | ------------------------------------------------------------------------------------------------- |
| ABI 类型、常量与消费端转调          | 已实现；宿主验证通过 | [`src/abi/`](../pocketjsforedk2/src/abi)、[`src/consumer.rs`](../pocketjsforedk2/src/consumer.rs) |
| C 头文件生成、布局和互调检查        | 已实现；宿主验证通过 | [`tools/verify.rs`](../pocketjsforedk2/tools/verify.rs)                                           |
| EFI 驻留服务、专用堆与真实实例复用  | 设计                 | [交接协议](uefi-handoff.md)                                                                       |
| 固件 I/O、AP 调度与有界队列         | 设计                 | [多核架构](architecture.md)                                                                       |
| 入口调度、recovery 与 charging 后端 | 设计                 | [入口与场景](entry-modes.md)                                                                      |
| UEFI 包读取与实体调试传输           | 设计                 | [用户层与调试](userspace.md)                                                                      |
| 调试门控、数据队列与 BSP 通知       | 已实现；宿主验证通过 | [调试数据通道](debug-channel.md)                                                                  |
| 默认页面、共享 UI 与桌面演示        | 已实现               | [页面与验证](default-ui.md)                                                                       |
| 平台无关宿主与用户上下文生命周期    | 已实现；需 BSP 接入  | [裸机公共层与 BSP](no-std-bsp.md)                                                                 |
| 官方 guest 与嵌入式脚本/渲染中间件  | 已实现；宿主验证通过 | [嵌入式中间件](embedded-runtime.md)                                                               |

“宿主验证通过”仅指当前工具覆盖的检查；没有完成 EFI 启动、实机或 QEMU EFI 实例接续验证。计划中的模块名不代表已经存在对应实现。
