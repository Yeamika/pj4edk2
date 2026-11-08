# 默认页面与共享 UI 接口

本页说明默认 PocketJS 页面、宿主输入输出与动画时序。桌面演示与后续固件宿主使用同一套页面和 Rust 配置类型。

> [!NOTE]
> 状态：页面与桌面适配已实现。UEFI I/O、实际 EFI 程序启动、恢复工具和电源管理后端仍需实现。桌面演示不提供 EFI 系统表，也不验证跨
> loader 堆复用。

## 模块

| 源码                                                                         | 职责                                                |
| ---------------------------------------------------------------------------- | --------------------------------------------------- |
| [`boot-api/src/config.rs`](../pocketjsforedk2/crates/boot-api/src/config.rs) | 模式、版本、视口、启动项与电池参数，输入校验        |
| [`boot-api/src/event.rs`](../pocketjsforedk2/crates/boot-api/src/event.rs)   | 宿主事件与页面请求；可观测状态在参考 `snapshot.rs`  |
| [`crates/boot-ui/src/name_card.rs`](../default-ui/src/name_card.rs)          | 名称贴图的参数与尺寸限制                            |
| [`boot-api/src/services/`](../pocketjsforedk2/crates/boot-api/src/services)  | 列表、设置、参数、boot 服务与输出区域契约           |
| [`crates/boot-ui/src/scripts.rs`](../default-ui/src/scripts.rs)              | 按依赖顺序嵌入默认 JS                               |
| [`ui/default/theme.js`](../default-ui/pages/theme.js)                        | 调色板、节点与文字绘制                              |
| [`ui/default/battery-status.js`](../default-ui/pages/battery-status.js)      | 顶栏电量图标、百分比与 BSP 电源状态同步             |
| [`ui/default/low-battery.js`](../default-ui/pages/low-battery.js)            | 无文字黑底低电量保护页，只有电池与接电图形          |
| [`ui/default/chrome.js`](../default-ui/pages/chrome.js)                      | 顶栏、九宫格淡化加载、阶段与倒计时底栏              |
| [`ui/default/settings.js`](../default-ui/pages/settings.js)                  | 自动启动秒数设置                                    |
| [`ui/default/settings-debug.js`](../default-ui/pages/settings-debug.js)      | 会话调试开关、等待确认与实际状态                    |
| [`ui/default/boot.js`](../default-ui/pages/boot.js)                          | 启动项节点、选择与确认输入                          |
| [`ui/default/icons.js`](../default-ui/pages/icons.js)                        | 系统、Windows、终端、恢复、电源图标与复选框勾选图形 |
| [`ui/default/boot-default.js`](../default-ui/pages/boot-default.js)          | 默认项请求、读回确认、错误状态和复选框              |
| [`ui/default/boot-actions.js`](../default-ui/pages/boot-actions.js)          | Recovery 页面入口与 Shutdown 宿主请求               |
| [`ui/default/boot-geometry.js`](../default-ui/pages/boot-geometry.js)        | 渐隐抛物线、旋转卡片与黑色区域的几何                |
| [`ui/default/boot-image.js`](../default-ui/pages/boot-image.js)              | 生成一次名称贴图并使用原有图片节点旋转              |
| [`ui/default/boot-motion.js`](../default-ui/pages/boot-motion.js)            | 辅助内容淡出、收拢、旋转与启动动画                  |
| [`ui/default/recovery.js`](../default-ui/pages/recovery.js)                  | 恢复操作入口                                        |
| [`ui/default/charging.js`](../default-ui/pages/charging.js)                  | 电量、电源状态与充电呼吸动画                        |
| [`ui/default/main.js`](../default-ui/pages/main.js)                          | 初始化、事件分派、时间推进与场景切换                |
| [`ui/default/stage.js`](../default-ui/pages/stage.js)                        | 同一 realm 中保存父页面、释放子页面与恢复父阶段     |

这些 JS 使用 PocketJS 的 `ui` 操作，不依赖浏览器、DOM 或定时器。自研宿主、接口与字体烘焙工具使用 Rust；上游 QuickJS 仍包含
C。字体来自 Inter，许可证见
[`ui/assets/LICENSE-Inter.txt`](../default-ui/assets/LICENSE-Inter.txt)。

## 宿主接入

共享 crate 为 `no_std + alloc`，使用 serde 编码数据。它不创建 QuickJS，也不依赖 EFI ABI
crate。宿主先挂载上游 canonical UI bindings、`bootEmit(text)` 与 `bootBakeName(text)`，再按
`scripts()` 顺序求值页面。

| 调用                            | 参数与结果                                              |
| ------------------------------- | ------------------------------------------------------- |
| `rimuksInit(configJSON)`        | 一次性读取 `UiConfig` 的 JSON 字符串，建立界面          |
| `rimuksTick(nowMs, eventsJSON)` | 接收单调毫秒数和 `UiEvent` 数组，推进状态与动画         |
| `rimuksState()`                 | 返回 `UiSnapshot` JSON，供诊断与验证                    |
| `bootEmit(actionJSON)`          | 页面调用宿主，提出 `UiAction` 请求                      |
| `bootBakeName(specJSON)`        | 宿主生成一次名称贴图，返回 `{handle,width,height}` JSON |

`bootBakeName` 接收共享的 `NameCardSpec`，包含文字、逻辑尺寸、圆角、ABGR 颜色、字体槽和文字起点。宿主使用同一字体与上游绘图
API 生成带透明补边的贴图，并以最近邻采样上传到当前 UI 实例，使用 `upload_texture` 或 flags=0，不启用
`FLAG_LINEAR`。旋转直接选取最近的纹素，不混合相邻纹素。默认桌面字体使用二值覆盖，圆角保留贴图生成时的边缘覆盖。返回尺寸是补齐到 2
的幂后的纹理尺寸；页面按这些尺寸建立图片节点，不拉伸卡片。默认卡片最多 512×64×4 字节，即 128 KiB，只在确认启动时生成一次，场景销毁时通过
`ui.freeTexture` 释放。请求不会重建 QuickJS 实例。

事件与求值在拥有该实例的同一线程执行，不能并发或重入。宿主负责固定频率推进时间、绘制及呈现。页面以 1280×720
为基准等比缩放；点击坐标使用实际视口像素，由页面转换。

`UiConfig`
支持六种预设：`uefi`、`loader`、`uefi-loader`、`recovery`、`charging`、`low-battery`。前三者共用
boot 页面，顶栏分别显示 `uefi`、`loader`、`uefi + load`。顶层视口范围为 640×360 至 2560×2560，子页面允许
320×180 起的输出区域，自动启动范围为 0 至 120 秒，最多 8 个启动项。超出可见区域的启动项通过滚轮或方向键访问。

`Boot` 包含启动项 ID、路径和输出区域列表，`SetTimeout` 包含秒数，`Recovery` 包含操作标识，`EnterBoot` 表示进入
boot 场景。宿主决定是否以及如何执行请求；来自 UI 的路径不能替代固件的卷定位、映像校验与启动流程。

宿主通过 `BootServices` 读取列表、读取和保存设置、读取参数并执行 boot。boot
请求携带实际运行时的借用、自定义参数和设备像素区域；保存结果通过 `SettingsSaved` 或 `SettingsSaveFailed`
回送页面。类型、所有权及子页面的坐标转换见[启动服务与页面嵌套](boot-services.md)。

## 交互与动画

初始选中默认项用于自动启动，但没有手动确认。第一次点击某项会选中并确认该项；再次点击同一项才开始启动。两次点击不要求连续快速完成。方向键改变选择并取消前一项的确认，Enter
第一次确认、第二次启动。桌面宿主忽略按键自动重复，长按 Enter 不会代替两次按下。

配置了 `default_entry_id` 时优先选中该项，未配置时使用第一项；保存的条目已不存在时暂停自动启动。右侧 40×40
逻辑像素复选框只修改默认项，不改变当前选择或触发启动。页面等待 `DefaultEntryRead` 的实际读回 ID
才移动勾选，保存或读取失败保持原值。图标通过 `BootEntry.icon` 指定，支持
`system`、`windows`、`terminal`、`recovery`，未知图标名回退到系统图标。

列表底部按钮为 `Recovery` 和
`Shutdown`，分别配有恢复十字和电源图标。前者进入恢复页面并记住返回模式；后者发出一次关机请求，实际电源控制归宿主/BSP。默认项写入期间暂停启动及模式切换。

任何点击、按键或滚轮输入都会取消本轮自动启动，底栏倒计时和进度条随即消失。打开或保存设置也不会恢复本轮倒计时。鼠标仅移动不取消倒计时。事件先于超时判定处理，因此恰好到达截止时间的输入仍能取消自动启动。0
秒表示关闭自动启动。

界面采用统一的浅蓝背景、细边框与两级卡片状态。名称使用 20 像素字体，路径为 14 像素，标题为 32
像素；悬停、按下与键盘选择有独立反馈。顶栏中央只显示方框和粗体阶段文字，没有圆点。字体槽 7 使用宿主烘焙的加粗 18 像素字形。

顶部九宫格保持固定位置，只在等待自动启动、启动动画、设置保存、调试开关等待确认或宿主通过 `Waiting`
声明忙碌时，沿外圈顺时针循环淡化。停止等待后，外圈八格常亮，中心格每 500 毫秒切换明暗；不再持续播放对角波形。子页面运行时也继续更新父页面的指示灯。

默认桌面宿主在字体资源烘焙时将字形覆盖量化为 0 或 255，并通过公开 `RenderResources`
接口选择最近邻采样，包含缩放时的字形纹理。曲线使用独立的物理像素坐标层，每行一个无圆角的实色线段，避免子像素高度被布局舍入后断开。横向没有柔边，沿线保留渐隐；这些规则在资源、页面和宿主实现，无需修改引擎。

确认启动后用 1000 毫秒完成分段过渡，移动采用两端平滑的五次缓动：

1. 初始为右凸深度 32 像素的浅抛物线，从 y=132 延长至 640，两端逐渐融入蓝色背景。每个文本框的左边缘位于对应圆点右侧 36
   像素，因此各行沿弧线错开排列。启动后弧线拉直并收至左侧 44 逻辑像素处，仍保留两端渐隐。
1. 标题、路径、状态与辅助提示在前 160 毫秒淡出，未选中项在前 210 毫秒淡出，避免黑色区域切断文字。
1. 从第 40 毫秒开始，卡片高度由 72 缩至 48，长度按名称收至 184 至 340 像素，框中只保留名称；第 260
   毫秒完成缩小。文本框与对应圆点使用同一条曲线的坐标，点击和悬停区域也采用各行的实际位置。
1. 第 260 至 950 毫秒，圆点沿当前曲线滑动，名称框同步围绕圆点逆时针向上旋转
   90°。缩小完成后切换到同尺寸的名称贴图，通过原有图片节点旋转。默认项向下滑动，其他项由原位置滑至停靠点。最终竖直名称框居中于顶栏和底栏之间，位于
   x=20 至 68，垂直中心为 y=370；圆点位于名称框下方，相隔 20 像素。默认 184 像素名称框为 y=278 至 462，圆点中心为 (44,
   482)；长名称自动调整圆点位置，框的中心不变。渐隐连线同步拉直。
1. 右侧黑色区域向左展开，始终避开旋转卡片的实际右边缘，保留 16 像素间距。最终停在 x=84，基准 1280 像素宽度下占屏幕宽度约 93.4%。
1. 黑色区域从顶栏下沿延伸至屏幕底部，顶栏与九宫格加载继续显示。启动期间设置按钮显示为不可用状态。
1. 动画结束才发出一次 `Boot` 请求；过渡期间忽略重复输入。

桌面目标为 60 FPS，实际帧率取决于宿主。动画按单调时间计算，丢帧不会延长启动等待。恢复与充电返回 boot 时只重建场景节点，继续使用同一个
QuickJS 实例。

`Pointer` 事件携带视口坐标与按下状态。移动只更新悬停，按下取消自动启动；Windows 宿主在释放时生成
`Click`，拖动不作为点击。设置面板的时间设置保持一行“Auto boot delay＋数字输入框＋秒单位”，下方增加 `PocketJSDebug`
按钮，保存按钮位于右下角。数值改动显示 `*`，保存成功确认后清除，失败时保留输入允许重试；支持数字、删除、Ctrl+A、方向键调整和 Tab 切换焦点。Tab
顺序是时间输入、调试按钮、保存；调试按钮聚焦时 Enter 切换开关，其余焦点的 Enter 保存时间。
面板保留短距离滑入和背景淡化，关闭动画结束前阻止点击穿透。

调试开关初始为 OFF，用户选择后立即请求宿主切换，等待期间显示 `...`，实际状态确认后才显示 ON/OFF；连接失败用错误边框提示。
它不写入配置，不改变时间设置的 `*`，也不因打开或保存设置而开启。BSP 连接责任及数据接口见[调试数据通道](debug-channel.md)。

## 电量与低电量保护

boot 与 recovery 顶栏在设置按钮左侧显示电池图标和百分比；电量未知时显示 `--`。初始值来自
`UiConfig.battery_percent` 和 `external_power`，运行期间 BSP 可发送
`UiEvent::PowerState { battery_percent, external_power }` 更新。电量是 0 至 100
的可选整数，超范围事件被忽略；遥测不视为用户操作，不取消倒计时。嵌套页和父页面同步使用当前值。Windows 数值为演示参数，不读取系统电池。

`low-battery` 是第六个入口。页面使用纯黑背景，只画居中的电池轮廓、红色低电量条和下方接电图形；没有文字、百分比、顶底栏、设置或按钮。
图形保持静态，数据更新时才改变电量条与接电图形的颜色。未知或极低电量保留一个红色图形标记，不伪造百分比。

该场景不运行自动启动，也不接受 Enter、点击或 F2 进入 boot/设置。连接充电器或电量增加不会自动解除保护；进入、解除、关机与阈值均由 BSP
的电源策略决定。 界面不直接访问电池或 PMIC。

## 验证与限制

实际运行与验证命令见独立 Windows demo（外部桌面参考工程，未纳入本仓库）。`--self-test` 在真实 QuickJS
中检查选择、倒计时取消、动画完成、单次请求、场景复用与字体覆盖；`--preview` 用同一渲染器生成 PNG 和 GIF。

名称旋转使用上游已有图片路径，UI 核心和软件光栅器均保持原样。桌面宿主参考实现与源码校验见依赖说明（外部桌面参考工程，未纳入本仓库）。

## UEFI 图形接入

UEFI 可以用 CPU 执行三维变换、透视投影和三角形光栅化，再通过 GOP 帧缓冲或 Blt 显示结果。GOP 提供显示模式和像素输出，没有通用 GPU
3D 加速接口。PocketJS 现有软件路径可用于旋转面片和轻量三维界面；实际帧率取决于 CPU、分辨率、覆盖面积与场景复杂度。

当前名称条只需要二维图片旋转。接入固件时仍需实现 GOP 像素格式、帧缓冲步长、呈现和输入适配，固件里无需 Windows
窗口或显卡驱动。本项目尚未完成这些后端，桌面演示的 `uefi` 模式仅切换页面阶段，不能作为 UEFI 实机验证。

默认字体包只包含可打印 ASCII 字符，默认页面使用英文。自定义中文名称需要宿主补齐对应字体覆盖；当前 demo
没有中文输入和字体回退。电池值来自启动参数，不读取 Windows 电源状态。设置保存仅在桌面宿主实现，固件变量持久化尚未实现。
