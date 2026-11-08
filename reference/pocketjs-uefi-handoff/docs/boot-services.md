# 启动服务与页面嵌套

本页说明页面和平台之间的 Rust 服务接口，以及已有 PocketJS 实例中的阶段切换。

> [!NOTE]
> 状态：Rust 接口和桌面适配已实现，宿主循环检查通过。嵌套页面只演示 loader 的界面和服务调用；尚无可执行的 EFI loader，也未执行
> `LoadImage`、`StartImage` 或真实租约交接。

## 服务接口

接口位于
[`pocketjs-boot-api/src/services/`](../../../pocketjsforedk2/crates/boot-api/src/services)，使用
`no_std + alloc`。参考 `boot-ui` 重新导出这些唯一的接口类型。页面读取宿主准备的 `UiConfig`，发出
`UiAction`；文件、固件变量和映像加载归宿主处理。

| 方法                | 输入和结果                                                |
| ------------------- | --------------------------------------------------------- |
| `read_entries`      | 返回启动项 ID、名称、路径、图标名及该项的自定义参数       |
| `read_settings`     | 返回 `BootSettings`，包含自动启动秒数与默认项 ID          |
| `save_settings`     | 持久化设置，成功后才能确认页面改动已保存                  |
| `read_parameters`   | 返回全局自定义参数和显示设备描述                          |
| `boot`              | 接收启动项、合并参数、当前运行时借用和输出区域列表        |
| `shutdown`          | 由平台/BSP 执行关机；Windows 演示关闭图形窗口             |
| `set_debug_enabled` | 显式选择后通知 BSP 接通或断开调试传输；PJB 只提供数据通道 |

`BootServices` 有 `Error` 和 `Runtime` 两个关联类型。桌面适配的 `Runtime` 是实际的
`Engine`，`BootRequest.runtime` 是对同一实例的可变借用，调用返回后仍由原宿主持有。固件适配应绑定已有 `ContextTable`
或持有它的宿主对象，通过驻留 `invoke` 使用实例。Rust trait 和借用只用于同一宿主内，不作为 C/EFI ABI。

`boot` 在页面 tick 返回后调用，不能从正在执行的 QuickJS 回调中重入实例。当前服务调用同步发生在拥有 UI 的线程。跨核队列和固件 I/O
调度仍按[引导优先与多核](architecture.md)的设计实现。

## 自定义参数与显示区域

`BootParameters.custom` 是保留 JSON 类型的键值表，可使用带模块前缀的键。全局参数先读取，启动项的 `parameters`
再覆盖同名键；合并后最多 16 KiB。整数、字符串、布尔值、列表和对象按原类型传递。运行时和指针不进入这个 JSON 表。

桌面演示支持以下参数文件：

```json
{
  "kernel.args": "quiet",
  "rimuks.debug": false,
  "vendor.options": {"profile": "normal"}
}
```

在 demo 目录运行：

```sh
cargo run --release --locked -- --mode uefi --parameters parameters.json
```

启动列表的每项可添加同名键值表：

```json
[
  {
    "id": "loader",
    "name": "RIMUKS loader",
    "path": "\\EFI\\RIMUKS\\BOOTX64.EFI",
    "parameters": {"kernel.args": "debug"}
  }
]
```

`BootParameters.displays` 描述设备 ID 和物理尺寸。桌面适配中的设备是程序窗口，不声称已枚举实际显示器或 GOP
句柄。`BootRequest.display_regions` 是绘制区域列表：

```json
[
  {
    "id": "boot-output",
    "display_id": "main",
    "position": [84, 84],
    "size": [1196, 636]
  }
]
```

坐标相对对应显示设备，以物理像素计。页面在动画完成时报告黑框的实际矩形；宿主检查设备存在、区域非空、ID
唯一和边界不溢出。嵌套页面输出的区域继续使用设备坐标，包含父区域的偏移。当前页面产生一个区域，接口允许传递多个。

## 保存确认

设置面板的倒计时为一行文字和数字输入框，另有本次运行的 `PocketJSDebug` 按钮，保存按钮在右下角。数值改变后显示 `*`；0
表示关闭自动启动，可输入 0 至 120。直接输入数字、Backspace、Delete、Ctrl+A、Tab 和方向键均由宿主事件驱动。

页面发出 `SetTimeout` 后等待结果。宿主写入成功，回送 `SettingsSaved`，页面才更新当前设置并清除 `*`；失败回送
`SettingsSaveFailed`，保留输入和标记，允许重试。保存不会重新开始本轮倒计时。桌面适配先写临时文件，再提交到设置文件。

`BootSettings.default_entry_id` 是可选字符串，缺少此字段的旧配置仍可读取。点击默认项复选框发出
`SetDefaultEntry`，宿主先读取现有设置、只更新默认 ID、写入，再重新调用 `read_settings`；通过
`DefaultEntryRead` 返回实际读到的 ID。写入成功但读回不同值时，界面按读回值显示。写入或读回失败发送
`DefaultEntrySaveFailed`，勾选保持原值。修改超时也先读现有配置，避免覆盖已保存的默认项。

## 电源状态

BSP 通过
`UiEvent::PowerState { battery_percent: Option<u8>, external_power: Option<bool> }`
提交电池遥测，初始值在 `UiConfig` 中。 它不触发 boot 服务或取消倒计时。boot/recovery 顶栏显示百分比，黑底
`low-battery` 只画图形；保护与解除由 BSP 判断。
详见[默认页面](default-ui.md#%E7%94%B5%E9%87%8F%E4%B8%8E%E4%BD%8E%E7%94%B5%E9%87%8F%E4%BF%9D%E6%8A%A4)。

## 调试状态

`PocketJSDebug` 在新运行时中默认关闭，不属于 `BootSettings`，不接受自定义启动参数自动开启。用户显式操作产生
`SetDebugEnabled`；宿主先改变数据门控，再调用 BSP 的 `set_debug_enabled`，最后用 `DebugConfigured`
回送实际状态。 `DebugState`
用于同步当前运行时状态，不产生连接请求，也不清除等待中的操作。失败回滚、队列和连接代次规则见[调试数据通道](debug-channel.md)。

## 同实例中的子页面

桌面 boot 服务在 `uefi` 或 `uefi-loader` 模式下，将 loader 页面放入收到的第一个输出区域。它保存父页面函数和状态，在同一
QuickJS realm 中创建子页面节点，不重新初始化 QuickJS。子页面裁剪到目标区域，可用区域小于 960×540
时等比缩小内容；较大区域保持原像素布局。鼠标坐标减去区域偏移后再除以内容比例，输出区域仍按设备物理坐标传递。名称贴图仍使用最近邻采样。

按 Escape 返回父页面；子页面设置打开时，Escape 先关闭设置。返回会释放子页面节点和名称贴图，恢复父 UEFI 页面，保留运行时及 realm
中的其他对象，自动启动保持暂停。嵌套深度最多 8，顶层视口最小 640×360，嵌套输出区域最小 320×180。相关实现位于
[`ui/default/stage.js`](../../../default-ui/pages/stage.js)
和桌面阶段适配（外部桌面参考工程，未纳入本仓库）。

黑框定义绘制目标，不能隔离程序、提供 CPU 执行入口或代替固件映像加载。界面嵌套成功不代表 EFI 程序嵌套成功。

## EFI 入口与句柄

“UEFI 内存空间起始地址”没有可直接调用的 loader 入口语义。固件加载 PE/COFF 映像时要建立映像、处理重定位，并通过 `StartImage`
调用已加载映像的入口；也可以由 `LoadImage` 从正确的映像缓冲区加载。堆基址、帧缓冲地址、`ImageBase` 和映像入口是不同概念。

父、子 `EFI_HANDLE` 应分别使用各自实际的映像句柄；不把父句柄伪装成子句柄。驻留表的 `owner_image_handle` 和
`runtime_instance` 保持所属实例不变，消费者句柄随当前 loader
改变。父阶段释放租约后子阶段取得租约，返回时反向交接，不能让两个消费者同时操作实例。具体语义见[UEFI 交接协议](uefi-handoff.md)。

现有 GUID、`ContextTable`、堆范围和生成头文件沿用原协议。真实固件 boot 服务还需实现显示设备到 GOP
的绑定、映像加载、参数封装和租约调度；本次不增加一个虚构的运行时地址或修改 `EFI_SYSTEM_TABLE`。

## 验证

桌面项目运行 `cargo test --locked`，覆盖 32 轮父 UEFI 页面→子 loader 页面→返回父页面。每轮均通过同一 boot
服务分派，检查运行时地址、QuickJS realm 哨兵对象、自定义参数类型及逐轮令牌、嵌套区域坐标和单次请求；共 64 次 boot 服务调用。

原 ABI 的[宿主检查](getting-started.md)另执行 32 轮父、子句柄经生成 C 头文件进入 Rust
回调的转发，检查所有者、实例、堆描述、代次和租约未变。回调是测试替身，只验证 ABI 传递，不实现或验证固件租约服务。

测试还覆盖输入校验、保存失败后重试、成功确认、设置文件往返、最近邻图片旋转，以及等待时的九宫格变化与空闲时八格常亮。Windows 原生 DPI、窗口输入和真实
EFI 引导仍需对应环境验证。
