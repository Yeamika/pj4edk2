# 启动显示公共层与平台边界

本分支已有显示协议、可复用终端与 Linux 内建 console 适配源码。
[目录与测试入口](readme.md) 描述当前实现；它们不是仅供未来迁移的空接口。

| 层 | 当前实现 | 责任 |
| --- | --- | --- |
| 描述与编解码 | `include/pj4edk2_boot_display.h` | 80 字节 DT 协议、版本、字节序、长度、几何和溢出校验 |
| 终端与绘制 | `include/pj4edk2_terminal.h` | 有界文本、换行、滚动、可变字形、旋转、像素格式与脏区域 |
| Linux 通用适配 | `linux/pj4edk2_console.c` | reserved/coherent RAM 接收、早期 console 注册与异步绘制 |
| 平台适配 | 消费项目 | 显存授权、映射、缓存一致性、DPU 提交、时钟/电源和所有权交接 |
| 原生 DRM | 对应平台驱动 | 接管显示资源并停止前一 owner 的绘制 |

## 内存与生命周期

协议合法只说明字段和范围一致，不授权访问任意物理地址。
通用 Linux 适配额外要求 DT 指向完整 memblock 保留区，并声明 coherent scanout；
不满足这些条件的显存或 MMIO 必须由平台自己的驱动接入。

早期 console 在 `console_initcall` 注册，从 `core_initcall` 启动异步绘制，必须内建。
它不能改成依赖 rootfs 后才加载的 `.ko`，也不应与其他驱动同时覆盖同一 framebuffer。

W202DS 继续拥有 DPU、缓存清理、时钟与原生 DRM 交接策略。
公共终端不替代这些硬件行为。需要接管时，平台先停止并等待旧绘制，再释放映射和时钟资源。

## 验证与许可

`tests/protocol.c` 检查协议与错误输入，`tests/terminal.c` 检查像素格式、旋转、滚动、步幅与区域保护。
QEMU 分支提供通用适配的虚拟启动环境；硬件持续显示、缓存与 DRM 接管另行验收。
本次文档整理不代表重新编译或执行上述测试。

协议头保留 `BSD-2-Clause OR GPL-2.0-only`；从 Linux 实现抽出的终端和内核适配保留 `GPL-2.0-only`。
具体文件的 SPDX 声明优先。

协议与 rootfs 选择的边界见 [DT 交接说明](docs/efi_dtbchosen.md)。
