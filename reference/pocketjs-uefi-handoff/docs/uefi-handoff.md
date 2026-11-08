# UEFI 实例交接协议

本页定义生产者与消费者在同一次 UEFI 启动中接续实例的约定。

> [!NOTE]
> 状态：ABI 定义与消费端转调已实现，服务语义仍为设计。 EFI 驻留服务、专用堆分配器和真实跨 loader 复用尚未实现。

模块介绍与架构入口：[文档索引](README.md)。

## 实现语言

公开结构、常量、调用类型和 `pjb_call` 辅助函数均用 Rust 定义和实现，`pocketjs_boot_context.h` 由 cbindgen
与 Rust 生成器自动产生，不再手写生产 `.c`。其他 C loader 可按头文件检查版本后直接调用驻留的
`invoke`，无须自己链接另一份引擎；若要使用 `pjb_call` 辅助函数，则需链接它的 Rust 实现。

[`src/abi/`](../../../pocketjsforedk2/src/abi) 定义 `no_std` Rust ABI，
[`src/consumer.rs`](../../../pocketjsforedk2/src/consumer.rs) 实现消费端辅助函数， [`src/lib.rs`](../../../pocketjsforedk2/src/lib.rs)
统一导出接口。驻留调用入口使用 `extern "efiapi"`。我们新增的驻留服务、UEFI 显示/输入适配、堆管理、包加载和调试传输均用 Rust
实现。现有上游 QuickJS 解释器仍包含 C 代码，应用页面仍为
JS/TS；这个约定不表示重写上游解释器。生成方式见[Rust ABI 与头文件](abi.md)。

用户层独立分区和实时调试的设计见[用户层存储与调试](userspace.md)。

## 目标与发现

同一次 UEFI 启动中，多个支持本约定的 EFI loader 使用同一个已经初始化的 PocketJS
引擎，保留运行时、堆及可复用资源。各阶段重新创建用户上下文、全局变量和 UI 节点，避免重新初始化整个引擎。公共生命周期与 BSP
的清理职责见[裸机公共层与 BSP](no-std-bsp.md)。其他 loader 可以忽略这个配置表条目，继续正常启动。

自定义配置表 GUID：`6988824e-3b93-4bd4-bf07-65517b0f906d`。

生产者通过 `BootServices->InstallConfigurationTable(&Guid, Table)` 发布
`PJB_CONTEXT_TABLE`，消费者在 `SystemTable->ConfigurationTable` 中按 GUID
查找。所有指针仅属于当前固件启动会话；这个结构不保存到磁盘、不跨重启恢复、不改变 `EFI_SYSTEM_TABLE` 本身的布局。

## 为什么堆地址之外还有一个入口

堆范围和实例指针描述状态的位置，执行仍需要原实例对应的代码。现有 `RquickjsGuest` 同时持有 QuickJS
`Runtime`、`Context`；绑定的回调捕获 `OpContext`，UI 状态包含 Rust
`Rc`、`RefCell`、容器和分配器依赖。因此，另一份独立链接的引擎不能仅凭地址就取得兼容的 Rust 对象、回调和内存所有权。

`invoke(instance, request, response)` 是留在原 EFI
映像里的单个调用入口。下一阶段通过它使用原实例，无须重新链接或初始化 PocketJS。所有 QuickJS 调用、垃圾回收、Rust
对象访问及释放都由原服务完成，内部布局不作为跨 loader ABI 暴露。

如果所有参与者另外严格约定同一个原生引擎
ABI、全局状态、分配器和函数入口，也可以直接调用已知接口。这属于更强的私有约定；本草案通过一个入口承载约定，避免要求每个 loader 理解引擎内部布局。

## 结构与校验

`pocketjs_boot_context.h` 提供无外部 SDK 依赖的 C 布局；当前仅覆盖 UEFI X64 与 AArch64。GUID 字段顺序与
`EFI_GUID` 一致；X64 调用入口使用 Microsoft x64 ABI，AArch64 使用对应平台 ABI。

| 字段                                          | 含义                                             |
| --------------------------------------------- | ------------------------------------------------ |
| `signature`、`struct_size`、`abi_major/minor` | 标识和版本；更大的同主版本结构可带未知尾部字段   |
| `machine_type`、`pointer_bits`                | 当前体系结构与指针宽度；拒绝不匹配的消费者       |
| `capabilities`                                | 活实例、堆范围描述和单消费者模型                 |
| `generation`                                  | 实例的非零代次标识；用于发现过期交接，非鉴权凭据 |
| `owner_image_handle`                          | 包含入口、回调和原生全局状态的常驻 EFI 映像      |
| `runtime_instance`                            | 固定地址的实例指针；消费者只原样传回 `invoke`    |
| `heap_ranges/count/entry_size`                | 服务拥有的堆 arena 范围及元素步长；可有多个范围  |
| `invoke`                                      | 操作原实例的唯一公开函数入口                     |
| `reserved`                                    | 生产者置零，消费者忽略                           |

消费者在调用前检查签名、最低结构长度、主版本、架构、指针宽度、必要能力位、非空指针和非零代次。范围遍历按 `heap_range_entry_size`
前进，检查乘法、地址加法溢出、长度和范围重叠。一个不认识的 GUID 被忽略；认识 GUID 但发现不兼容版本时，不调用入口，也不覆盖原条目。

`pjb_call` 是可选的 C 消费辅助函数：只检查 ABI
头、必需能力、请求/响应大小、代次及基本非空条件，不遍历和解读堆。它不证明传入指针有效，也不替代生产者在发布时对 arena 范围的完整检查或 Rust
服务对租约、运行状态和操作参数的检查。

该表是固件地址空间内组件间的约定，不是对不可信 loader 的隔离机制。结构检查不证明任意指针指向有效内存。

## 堆与映像的所有权

生产者在发布之前预留一个或多个固定地址、固定容量的 arena。堆描述表列出这些 arena
的完整容量；元素、地址和容量在条目存在期间保持不变。实际分配量由服务内部管理，因此不在配置表里放易失的 `used_bytes`。

持久的 QuickJS、Rust UI 状态及资源必须由服务拥有的分配器管理，或由服务明确拥有并保持存活的映像/资源管理。当前基准使用默认
`RquickjsGuest::initialize` 和平台分配器；它尚未具备这里要求的专用堆，不能把目前所有零散分配虚构成一块连续内存。

消费者不得复制这些堆来恢复实例，不得把 `runtime_instance` 转成自己编译的 Rust/QuickJS
类型，不得直接分配或释放其中的对象。堆范围也不代表实例全部依赖：原代码映像、原生全局状态、UEFI 服务和显示资源仍要有效。

推荐生产者为返回入口后仍驻留的 EFI Boot Service Driver。若由父 EFI 应用提供服务，则父应用必须在
`StartImage(child)` 期间继续保持装载，不能返回退出或卸载。配置表本身不固定映像寿命，`owner_image_handle`
也不是自动的引用计数；服务必须阻止仍被使用时卸载自己。

## 单入口操作

每次调用提供完整的请求头和至少 16 字节响应，二者由调用者在调用期间保持有效。返回值采用 `EFI_STATUS`
编码，错误时响应不用于更新本地租约。`generation` 必须匹配；除获取租约外，消费者映像及租约必须与服务记录一致。

以下为最小初稿语义。引导优先、多核 UI 目标还需要非阻塞请求与完成通知，详见[引导优先与多核](architecture.md)；当前没有跨核调度实现。

| 操作      | 行为                                                                                  |
| --------- | ------------------------------------------------------------------------------------- |
| `ACQUIRE` | 以自身 EFI image handle 请求独占租约，输入 lease 为零；成功返回非零令牌，不初始化引擎 |
| `PUMP`    | 使用租约执行一次有界的输入轮询、原实例更新和显示提交；复用已有 JS/UI 状态             |
| `RELEASE` | 在操作完成后交还租约，保留实例和堆；成功后响应 lease 为零                             |
| `QUIESCE` | 在交给内核前停止 UI、排空进行中的操作并撤销租约；此实例终止服务，不能再次 ACQUIRE     |

`PUMP` 成功保留原租约。`QUIESCE`
成功只表示服务停止，不释放仍被配置表引用的内存；所有者负责随后撤销条目和清理。无租约的消费者不能关闭别人的实例。未知操作返回
`EFI_UNSUPPORTED`，非法参数/代次返回 `EFI_INVALID_PARAMETER`，忙碌或重入返回
`EFI_NOT_READY`，错误租约返回 `EFI_ACCESS_DENIED`。终止服务后，不再调用原 JS，后续调用返回
`EFI_NOT_READY`。

调用在正常 UEFI 应用执行级别串行发生，不能从高 TPL 事件回调中执行 JS。交给子 loader 前由父阶段先 RELEASE，子阶段
ACQUIRE；子阶段返回前 RELEASE，父阶段再 ACQUIRE。嵌套 EFI
应用退出或崩溃后不会自动交还租约，恢复需要所有者确认消费者已不再运行，不能盲目抢占。后续日志、进度、输入注入等操作另行定义，本次仅确定发现和实例接续所需的最小头部。

## 兼容与结束边界

不认识表的 loader 无须调用它。多核版计划在合作的 loader 之间保留后台 UI；进入不合作的 loader
前则先暂停辅助任务并停止写入显示，避免与对方争夺资源。其他 loader 改变 GOP
模式后，恢复时需重新检查显示目标；能适配时重建显示资源，不能适配时返回错误，不使用缓存的旧帧缓冲指针。保留 JS 实例不意味着可以跳过硬件重新绑定。

合作的内核 loader 在最终 GetMemoryMap 前 QUIESCE；所有者如需撤表/清理，必须也在最终内存映射之前完成。若不认识该表的 loader
直接调用 ExitBootServices，生产者事先注册的 ExitBootServices 通知只做不分配、不阻塞的失效标记，不运行 JS 或销毁复杂对象。

任何消费者在 ExitBootServices 成功后都不得再调用这个入口，即使映像和堆的物理字节暂时还存在。此时原状态所依赖的 Boot
Services、驱动和内存寿命已经结束；v1 不宣称内核可继续使用该实例。跨内核阶段延续需另外设计分配器、驱动、代码寿命和启动协议。

## 验证范围

本草案只覆盖 Rust ABI 和消费端辅助函数；验证包括头文件自动生成、C 的 X64/AArch64 编译和布局、Rust `no_std`
编译，以及本机临时 C 消费者调用 Rust 辅助函数与 `efiapi` 回调。该回调仅为验证 ABI 的替身。后续需要实现 EFI
驻留服务及专用分配器，再通过两个独立 EFI 消费者验证：第二个消费者看到相同实例和 JS 状态、初始化计数不增加、租约和退出边界正确。当前没有运行这些实机或
QEMU EFI 验证。
