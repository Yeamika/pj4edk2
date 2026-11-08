# 默认页面与嵌入式参考实现

本目录属于 [PocketJS 启动参考实现](../README.md)。抽象 ABI、服务 trait 和公共类型在独立的
[pocketjsforedk2](../../pocketjsforedk2/README.md) 中维护。

- [`../../default-ui/`](../../default-ui/README.md)：默认页面、演示预设和资源描述，重新导出唯一的公共接口类型。
- `crates/pocketjs-embedded`：官方 guest 的 `no_std + alloc` 执行适配。
- `crates/debug-channel`：有界调试数据队列实现。
- `ui/`：到默认页面和资源目录的兼容链接，规范位置为 `../../default-ui/pages/` 和 `../../default-ui/assets/`。

`src/`、`tools/`、`checks/` 及头文件为到接口工程的兼容链接；本目录不再提供独立 ABI Cargo 包。

[嵌入式运行时](docs/embedded-runtime.md)、[默认页面](docs/default-ui.md)、[调试数据通道](docs/debug-channel.md)记录实现行为。
[接口接入与验证](../../pocketjsforedk2/docs/getting-started.md)取代原 ABI 入门命令。
