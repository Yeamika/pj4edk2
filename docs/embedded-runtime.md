# 嵌入式 PocketJS 中间件

本页说明单片机、ARM/x86 裸机、boot 与 UEFI 共用的执行层，以及 BSP 的链接责任。普通 framebuffer
和硬件加速的接入见[裸机显示与加速](rendering.md)。

> [!NOTE]
> 状态：已实现 `no_std + alloc` 共享中间件，并以真实 QuickJS、官方 guest、UI、软件输出和模拟异步加速器完成宿主测试。 MCU
> 和其他裸机 target 的检查覆盖 Rust 库；本仓库的 UEFI 宿主及默认页面已使用该执行层，完成 ARM64
> 离线构建与最终链接。实体设备范围见[验证记录](../VALIDATION.md)；外部 Windows 演示独立维护。

## 官方嵌入式实现

PocketJS 上游提供
[`engine/guest-quickjs`](https://github.com/pocket-nexus/pocketjs/tree/e6770376626a4de5c93eaa6fb048e7e9ffb81f9f/engine/guest-quickjs)，
负责 QuickJS 与 Rust runtime 之间的官方 UI 绑定。它不是桌面 `RquickjsGuest` 的别名。上游的
[ESP-IDF 集成](https://github.com/pocket-nexus/pocketjs/blob/e6770376626a4de5c93eaa6fb048e7e9ffb81f9f/docs/ESP_IDF.md)
使用这个 guest，加上 ESP-IDF 的分配、显示和输入适配；可选 runner 提供 FreeRTOS 任务。解释器本身不要求 Windows 或
Linux。

当前版本复用上游提交 `e6770376626a4de5c93eaa6fb048e7e9ffb81f9f`，选取的源码及哈希见
[`vendor/UPSTREAM.json`](../vendor/UPSTREAM.json)。官方 C guest、生成绑定、Rust UI
和渲染器源码保持原样；vendor 根 manifest 仅缩小 workspace 成员并移除不在子集内的默认成员。 GLES
源码仅用于解析可选依赖路径，本中间件不启用它们。

## 组成

[`crates/embedded-runtime/`](../crates/embedded-runtime) 提供可直接创建的 `PocketJs`，无需
BSP 再实现解释器后端。

| 模块                                           | 职责                                                                |
| ---------------------------------------------- | ------------------------------------------------------------------- |
| `vm.rs`                                        | 持有 QuickJS 与 canonical runtime，创建、替换和释放用户 context     |
| `ffi.rs`                                       | QuickJS-NG 指针接口；JSValue 留在官方 C guest 内                    |
| `memory.rs`                                    | 用 Rust `GlobalAlloc` 实现 QuickJS 的分配器回调                     |
| `budget.rs`                                    | 对解释器与 Promise job 执行施加中断轮询预算                         |
| `input.rs`                                     | 复用官方触控命中编码，转交按钮、键盘、轴与运动数据                  |
| `output.rs`                                    | 复用官方 Renderer/DisplayOutput/Coordinator，协调输出和纹理生命周期 |
| `output/framebuffer.rs`                        | 可选的直接 CPU framebuffer，描述格式和 stride，不分配像素           |
| `services.rs`                                  | 通过官方 `svcOpen`、`svcPoll`、`svcSend` 传递有界宿主消息           |
| [`../debug-channel/`](../crates/debug-channel) | 默认关闭的双向调试数据队列，物理连接由 BSP 提供                     |

普通依赖不包含 `std`、桌面窗口、线程、操作系统文件接口或 `rquickjs`。`cc` 构建脚本在开发机执行；测试专用的 `rquickjs-sys`
只提供本机 QuickJS C 库，不进入裸机运行依赖。`pocket-runtime-cabi` 关闭默认 feature，使用
`external-runtime`，由最终 BSP 提供 allocator、panic 和入口。

这里使用官方低层 embedding 接口，由本中间件管理 guest 生命周期、UI tick 和驱动呈现，没有调用完整包启动状态机。
传入的是可信源脚本；`.pocket` 包的读取、准入和资源注册需要另行接入，不能把 `replace_user` 当作包校验入口。

## BSP 接入

BSP 安装 Rust 全局分配器，提供匹配的 QuickJS-NG C 库、官方 guest 和 C 兼容层，然后注册输出并调度中间件。 以下示例中的
`fb_memory` 是 BSP 已映射、具有一致性且可供 CPU 独占写入的 `&'static mut [u8]`：

```rust,ignore
use pocketjs_embedded_runtime::{Config, InputFrameBuilder, PocketJs, output::*};

let mut js = PocketJs::new(Config {
    width: 1280,
    height: 720,
    tick_hz: 30,
    js_heap_bytes: 512 * 1024,
    js_stack_bytes: 64 * 1024,
    interrupt_polls: 32,
    frame_buffer_budget: 1280 * 720 * 4,
})?;
let display = FramebufferDisplay::new(
    fb_memory, 1280, 720, 1280 * 4, PixelFormat::Argb8888,
)?;
js.add_display(display, RendererId::SOFTWARE)?;
let page = js.replace_user(&[
    "ui.setProp(1, 64, 0xffaa7733); globalThis.frame = () => {};",
])?;
let mut input = InputFrameBuilder::new();
// 到期时：BSP 先填充 input 的按钮、触控、键盘等事实。
js.tick(page, &input.frame())?;
// 下一次采样前 input.clear()，避免累加旧的文本和相对轴事件。
input.clear();
```

该例没有固件入口、驱动或 allocator。应用可以从 flash、分区或固件文件加载脚本；中间件不选择存储设备。 `replace_user_with`
在挂载 guest 前提供一次 `&mut Ui`，用于加载字体 atlas、样式和纹理；不需要把资源编码进 JS 字符串。

`replace_user` 加载脚本后由调用方显式呈现。`advance` 每次推进一个固定时间步，`present` 提交输出，`tick` 连续执行两者。
中间件不创建线程或睡眠；同步输出会在呈现内等待设备完成，异步输出可延迟到下一次呈现或显式完成。 单个实例只允许一个执行核拥有；引导核和 UI 核之间由 BSP
提供队列。GPU、blitter、DMA 以及 framebuffer 的所有权规则见[显示与加速](rendering.md)。

## 页面替换与实例保留

`engine_handle()` 与 `runtime_handle()` 在 `PocketJs` 生命周期内保持不变。 `replace_user`
先完成旧输出设备工作，再检查待处理 job，然后销毁旧 guest 与 JSContext，重建用户 UI、资源、输入捕获及双向消息队列，最后创建新
JSContext。 QuickJS runtime、其分配器和中间件对象保留；用户全局变量、模块、节点、字体解析结果和纹理不保留。静态资源原始字节可以继续放在
flash 或驻留内存中。

每次页面创建使用新 `UserGeneration`。旧代次的 advance、present、tick 和消息操作被拒绝。它是实例内部的页面标识，不能替代
EFI 引擎句柄，也不用于跨实例或跨 loader 的 Rust ABI。

QuickJS job 队列属于 runtime。当前官方接口没有按 context 取消 job 的功能。正常 evaluate/frame 会在预算内排空
job；如果异常之后仍有 job，卸载或替换返回 `PendingJobs` 并永久禁止该实例继续执行，随后释放整个实例。清理过程不会执行旧脚本。
普通脚本异常或预算中断后停止该页面；没有遗留 job 时仍可显式替换为新页面。

通过 `engine_handle()` 取得地址不代表堆可直接迁移。UEFI 交接仍须让原服务代码、分配器和所有内存保持有效，并通过现有
[C ABI](uefi-handoff.md) 调用服务；不能跨 loader 传 Rust trait 对象，也不能从某段堆的起始地址执行代码。

## 调试通道

`debug_channel()` 返回运行时拥有的[有界数据通道](debug-channel.md)。创建实例时默认关闭，应用宿主消息或配置不会自动开启。
只有宿主处理用户显式选择时才改变门控，并通知 BSP 接通或断开传输。PJB 只提供不透明字节队列，不创建设备，也不执行收到的命令。
通道属于运行时，`replace_user` 保留其状态；新实例恢复关闭。

## 宿主消息

宿主 `send(page, text)` 对应 JS 的 `ui.svcPoll()`；JS 的 `ui.svcSend(text)` 对应宿主
`receive(page)`。 没有消息时，官方 guest 的 `ui.svcPoll()` 返回 `undefined`。每个方向最多 16
条消息，每条最多 4096 UTF-8 字节。

自定义参数、设备区域列表、设置结果和 boot 请求可以编码为 JSON。中间件只转发消息；调用方解析、校验并交给
[`BootServices`](boot-services.md)，避免在解释器回调内重入引导服务。较大的参数需要调用方拆分或专用
provider，不能超过消息上限。 现有 RIMUKS 的 `bootEmit`、`bootBakeName`、字体和 `rimuksTick`
仍需页面适配层接入；这不需要 BSP 重写 QuickJS 绑定。

## C 库与内存边界

上游 ESP-IDF guest 使用 QuickJS-NG；此处 FFI 对应 NG 的 `JS_NewRuntime2` 五项分配器表，不能直接换成
Bellard QuickJS 的不同 allocator ABI。 官方 guest 必须与最终 QuickJS 使用同一套头文件、目标 ABI、编译选项和
JSValue 表示。

设置 `POCKETJS_QUICKJS_INCLUDE` 为包含 `quickjs.h` 的目录时，构建脚本用 `cc` 原样编译官方 guest；BSP
设置目标 C 编译器及其 sysroot。 不设置此变量时，最终链接方自行提供官方 guest 的符号。QuickJS C 库始终由最终链接方提供。

QuickJS 和官方 guest 仍需 C 内存、字符串、格式化、数学等兼容函数；guest 默认 console 还使用 C stdio。 单片机可以使用
SDK/newlib/picolibc 等实现，UEFI 使用其兼容层，不要求操作系统。 `memory.rs` 把 QuickJS 对象分配转给 Rust
全局堆，guest 自身与脚本临时副本仍走 C allocator，BSP 要让两者均可用。

`js_heap_bytes` 只限制 QuickJS 记账的堆，不是整个中间件的 RAM 上限；UI、guest、资源与驱动存储另外占用内存。 核心不强制分配
CPU 帧缓冲，实际格式、stride 和存储由输出驱动描述；设备私有目标可以只传句柄。选择 CPU framebuffer 时按实际大小预留预算，例如
1280×720 的 32 位单帧为 3686400 字节。 Rust UI 分配失败仍按 BSP 的 OOM 策略处理；并非所有内部 Vec
分配都能返回错误。

执行预算按 QuickJS 中断轮询次数计，不是硬实时毫秒时限；解析、C 原生函数、GC、UI 布局和渲染不保证在该预算内结束。 引导优先级和硬件
watchdog 由 BSP 的调度与隔离策略保证。

## 验证

在项目根目录，使用与测试依赖 `rquickjs-sys = 0.12.2` 配套的 QuickJS-NG 头文件：

```sh
export POCKETJS_QUICKJS_INCLUDE=/path/to/rquickjs-sys-0.12.2/quickjs
cargo test --manifest-path crates/embedded-runtime/Cargo.toml --locked
```

六个完整路径测试覆盖 64 次用户 context 创建、稳定 VM/runtime 地址、旧全局变量与 UI 消失、旧代次和消息失效、
软件像素、触控命中、按钮、键盘、自定义 JSON 参数、异常清理、队列上限和循环脚本/Promise job 的预算中断。
另外验证调试默认关闭、双向字节传递、用户 context 替换后保留连接代次，以及新实例恢复关闭。
另有四项[输出驱动测试](rendering.md#%E9%AA%8C%E8%AF%81)，覆盖 32 位 strided
fb、设备目标、异步加速和纹理/页面释放顺序。 这些测试直接调用官方 C guest 和 Rust runtime/UI；硬件路径的驱动是受控模拟，不代表实体
GPU 验证。

Rust 库的无操作系统构建示例：

```sh
unset POCKETJS_QUICKJS_INCLUDE
rustup target add thumbv7em-none-eabihf riscv32imac-unknown-none-elf
cargo build --manifest-path crates/embedded-runtime/Cargo.toml --locked --release --lib --target thumbv7em-none-eabihf
cargo build --manifest-path crates/embedded-runtime/Cargo.toml --locked --release --lib --target riscv32imac-unknown-none-elf
```

同一库也检查 AArch64/X64 的 `unknown-none` 与 `unknown-uefi` target，不需要在核心中切换平台 feature。
这些是 Rust 库构建，不包含目标 QuickJS/guest C 库的最终链接、显示驱动或实体设备启动。
