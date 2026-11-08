# 宿主服务契约

`pocketjs-boot-api` 使用 `no_std + alloc`，只依赖序列化库。类型定义在
[`crates/boot-api/src/`](../crates/boot-api/src)，不链接引擎或平台库。

## 服务与运行时

| `BootServices` 方法               | 责任                                     |
| --------------------------------- | ---------------------------------------- |
| `read_entries`                    | 读取真实启动项及其参数                   |
| `read_settings` / `save_settings` | 读取、保存配置并报告实际结果             |
| `read_parameters`                 | 提供全局自定义参数及显示设备描述         |
| `boot`                            | 接收启动项、参数、原运行时借用和输出区域 |
| `shutdown`                        | 执行宿主的关机后端                       |
| `set_debug_enabled`               | 响应用户显式选择，接通或断开宿主调试适配 |

`Error` 与 `Runtime` 是关联类型。`BootRequest.runtime` 在调用期间独占借用原实例，调用返回后所有权仍属于调用方。
不序列化运行时指针，也不在正在执行的 JS 回调中重入引擎。 这个 trait 用于同一宿主内；跨 EFI 调用使用[交接 ABI](abi.md)，与
UEFI 自身的 `EFI_BOOT_SERVICES` 表区分。

## 数据与事件

`BootParameters.custom` 保留 JSON 值的类型。参数合并由实现按全局值、启动项覆盖的顺序完成；接口校验合并后的大小和设备描述。
`DisplayDevice` 标识设备与尺寸；`DisplayRegion` 使用设备物理像素描述位置和大小，检查溢出、设备存在性及边界。它不指定
framebuffer 格式或设备句柄。

`UiEvent` 描述指针、按键、滚轮、电量和宿主结果；`UiAction` 描述启动、保存、默认项、调试和恢复请求。 未知电量使用
`None`，不填入模拟百分比。调试默认关闭且只响应用户显式选择的行为由实现验证；接口本身不创建队列、串口或网络连接。

`RecoveryResult { operation, success, lines, settings }` 回应同名恢复操作。 `lines`
是宿主观测到的结果文本；`settings` 仅在持久恢复成功且读回验证后提供。
页面按操作名匹配等待中的请求，失败时不更新设置。宿主不支持的操作也应回传失败结果， 避免页面一直等待；此事件本身不执行存储修复或启动其他系统。

`BootFailed { status }`（JSON `boot_failed`）用于宿主向重新建立的菜单报告启动失败， `status`
为可显示的结果文本。参考页面显示错误并取消本次自动启动，不改写持久设置； 诊断应用成功返回不发送此失败事件。是否重建页面、恢复硬件以及启动 EFI
应用由宿主实现。

`UiConfig`、`BootSettings` 不提供产品默认值。调用方显式配置；参考页面的旧预设改从
`pocketjs_boot_ui::default_config()` 和 `default_settings()` 获取。 参考 `boot-ui`
重新导出本工程的类型，因此宿主之间共享同一类型定义。

## EFI boot-output region

The optional `BOOT_DISPLAY_GUID` configuration table contains `BootDisplay`
revision 1 (36 bytes). It describes one `boot-output` rectangle on the primary
GOP: logical display width/height, rectangle x/y/width/height, and clockwise
rotation to GOP pixels (0 or 90 degrees). Coordinates include the viewport
origin and are bounded by the actual display. It carries no framebuffer pointer.

The launching host owns this immutable record only while the child EFI executes.
A consumer copies it before `ExitBootServices`, validates the revision, size,
geometry and its actual display hardware, then translates it into its own OS
contract. The table is not runtime memory. A returning child restores the prior
configuration table; a launch without a region temporarily hides any stale one.
The independent boot-output table does not alter ContextTable ownership or ABI.
