# EFI 实例交接 ABI

状态：结构定义和消费端转调已实现。驻留服务、专用堆和真实跨 EFI 租约交接由宿主实现。

接口定义位于 [`src/abi/`](../src/abi)，生成头文件为
[`include/pocketjs_boot_context.h`](../include/pocketjs_boot_context.h)。ABI
主版本、GUID、结构布局和操作码沿用既有定义。

## 发现与兼容

GUID 为 `6988824e-3b93-4bd4-bf07-65517b0f906d`。生产者通过 `InstallConfigurationTable`
发布 `ContextTable`，消费者按 GUID 查找；不改变 `EFI_SYSTEM_TABLE` 的布局。 其他 loader 可忽略此条目。

调用前核对签名、结构大小、ABI 主版本、体系结构、指针宽度、必要能力、非零代次和指针。更大的同主版本结构可附带未知尾部字段。 可选 `pjb_call`
检查基本调用头与能力；它不能证明任意指针有效，也不替代服务端的租约和状态检查。

## 所有权与操作

`owner_image_handle` 标识保有原代码和回调的映像，`runtime_instance` 是不透明实例指针，`heap_ranges`
描述所有者保持有效的 arena。 消费端不能转换为自己编译的 Rust/QuickJS 对象，也不能直接释放其对象。

| 操作      | 约定                                             |
| --------- | ------------------------------------------------ |
| `ACQUIRE` | 以当前消费者的真实 EFI image handle 获取独占租约 |
| `PUMP`    | 使用原实例执行一次有界更新；不初始化另一份引擎   |
| `RELEASE` | 释放租约，保留实例和堆                           |
| `QUIESCE` | 停止服务并撤销租约，后续不能重新取得此实例       |

父阶段先释放，子阶段再取得；返回时反向交接。不能并发重入原实例或把父映像句柄伪装为子句柄。 配置表和 image handle
本身不固定代码寿命，所有者必须在消费者使用期间保持装载。

## 退出与验证

最终内存映射之前完成停止、撤表与需要的资源清理。成功 `ExitBootServices` 后不再调用原入口或 Boot
Services，即使原物理字节仍存在。 跨内核阶段继续运行需要另外定义代码、堆和驱动的寿命。

验证方法见[接入与验证](getting-started.md)。完整的既有协议语义保存在[协议专题](../../docs/uefi-handoff.md)，其语义变更需与本工程同步；该专题不是另一份结构定义。

## 可选充电控制

`ChargeControlProtocol` 使用独立 GUID `69f253de-281e-4aca-a788-8c27cec41710`， 不改变既有
`PlatformProtocol` revision 1。两个回调读写 0/1 使能值，遵循 Boot Services
生命周期、TPL_APPLICATION 单调用方约定。读数指充电通路的硬件使能， 与电池实际 Charging/Discharging
状态分开；错误时视为未知，写成功要求硬件读回。 板级实现决定能否开启、异常停充和交接策略。调用者不能把开关当作 PD 协商、快充或电池旁路。

页面通过 `SetChargingEnabled` 请求，宿主回 `ChargeConfigured` 并定期发布 `ChargeState`；
未发布协议的平台隐藏控件。充电请求不写入 BootSettings，不随重启自动恢复。
