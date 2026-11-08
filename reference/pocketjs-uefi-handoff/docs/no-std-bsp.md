# 裸机接口与 BSP

抽象契约在独立的 [pocketjsforedk2](../../../pocketjsforedk2/README.md)，具体执行层使用
[pocketjs-embedded](embedded-runtime.md)。

接口工程只定义启动项、配置、输入/电源事件、服务请求和输出区域，不指定 CPU、板型、解释器或 framebuffer 像素格式。 BSP 提供
allocator、兼容 C 环境、计时、输入驱动、显示/加速实现、存储与实际启动后端。

低层显示注册复用上游 `Renderer` / `DisplayOutput` 契约，所有参考调用方依赖同一上游源码；不会另写一份像素格式或输出驱动 API。
默认页面的高层 `DisplayRegion` 用于描述黑框的位置与大小。

长期引擎与短期用户 context 的构建、销毁和失败处理由运行时实现并验证。不能把旧物理内存仍存在当作对象有效，也不能跨映像转换 Rust/QuickJS
对象。 对外使用[EFI 交接契约](../../../pocketjsforedk2/docs/abi.md)或同一宿主内的
`BootServices::Runtime` 借用。

旧的 `ScriptEngine` / `Bsp` / `Resident`
探针保留在[历史 boot-host](../../../research/archive/legacy-boot-host)，不再作为当前集成入口。 当前构建与 guest
测试步骤见[嵌入式运行时](embedded-runtime.md)，硬件显示与同步见[显示与加速](rendering.md)。
