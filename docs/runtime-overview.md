# 运行时组成

[pj4edk2](../README.md) 将 UEFI 宿主、通用执行层和页面分别维护，最终组合成 `PocketJS.a`。

| 目录                                                  | 职责                                                          |
| ----------------------------------------------------- | ------------------------------------------------------------- |
| [uefi-runtime](../uefi-runtime/README.md)             | UEFI 入口、显示和输入适配、设置存储、电量与启动子应用         |
| [crates/embedded-runtime](../crates/embedded-runtime) | `no_std + alloc` 的 PocketJS 执行适配、页面上下文、输入和渲染 |
| [rendering](../crates/rendering/README.md) | UEFI 与 WASM 共用的像素对齐、最近邻纹理采样策略 |
| [ui-package](ui-package.md) | `.pocket` UI 主机 profile、包校验与字体资源读取 |
| [crates/debug-channel](../crates/debug-channel)       | 默认关闭的有界调试消息队列                                    |
| [crates/boot-assets](../crates/boot-assets)           | 字体图集和名称卡片生成                                        |
| [default-ui](../default-ui/README.md)                 | 默认页面和字体；可在构建时由外部 UI 目录替换                  |
| [pocketjsforedk2](../pocketjsforedk2/README.md)       | 唯一的公共 ABI、服务 trait 和数据类型                         |
| [vendor/pocketjs](../vendor/pocketjs)                 | 固定提交的完整上游引擎子模块                                        |

实际调用关系为 UEFI 宿主使用通用执行层加载页面并调用上游引擎。 跨 EFI 程序共享同一 PocketJS
实例的驻留服务仍为设计，见[实例交接协议](uefi-handoff.md)。 LK 与 EDK II 的硬件启动交接由平台及组合仓库维护。

构建和验证从[仓库入口](../README.md)开始；专题见[文档索引](README.md)。
