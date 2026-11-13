# Linux EFI 公共层与平台边界

本分支已有协议头、范围校验、surface 转换、libfdt 辅助函数和通用 EFI 桥接应用源码。
[目录与测试入口](readme.md) 是当前实现入口；本页说明公共逻辑与平台适配的边界。

| 层 | 当前实现 | 责任 |
| --- | --- | --- |
| 显示描述协议 | `include/pj4edk2_boot_display.h` | 版本、编码、长度、几何与溢出校验 |
| surface / 区域转换 | `include/pj4edk2_linux_display.h` | 将调用方已验证的 framebuffer 与逻辑输出区域转换为显示描述 |
| initrd / PARTUUID | `include/pj4edk2_linux_chosen.h` | initrd 开区间与分区标识校验；不负责查找设备或挂载 rootfs |
| DT 写入 | `include/pj4edk2_linux_fdt.h` | 使用调用方 libfdt 写入显示和 initrd 范围 |
| 通用 EFI 桥接 | `edk2/DisplayBridge.c` | 校验线性 GOP 与保留 RAM，复制 DT 并启动 Linux EFI stub；子程序返回时恢复原表 |
| 平台加载器 | 消费项目 | 镜像格式、固定地址、硬件状态、保留内存策略、watchdog 与直接内核入口 |

## 已实现与限制

纯协议和转换逻辑不依赖 UEFI、Linux、MMIO 或固定板级地址。
通用 EFI 桥接只支持满足其 reserved/coherent RAM 条件的 32 位 RGBX/BGRX GOP；
BLT-only 显示、任意 MMIO 与 W202DS RGB565/DPU 仍由各平台处理。

PARTUUID 校验函数不等于完整根分区启动链。实际属性写入、initramfs 接收、内核与模块身份检查由消费项目组合，
不能因存在公共 helper 就声称所有系统已经接入。

W202DS 可复用公共转换与 DT 辅助，保留自己的镜像、地址、DPU、缓存/MMU、PMIC 看门狗与 CPU 交接策略。
`LinuxHandoff.c` 等硬件事务不应作为所有平台默认行为。

## 验证

协议、initrd 和 surface 模型测试位于 `tests/`，合成 DT 生产端位于 `examples/`。
通用 EFI 桥接的构建与启动环境由 QEMU ARM64 Small 分支提供。
测试源码、编译成功、虚拟机启动和物理板级启动是不同验证范围；本次文档整理不新增执行结果。

initrd、rootfs 与显示字段的区别见 [DT 交接说明](docs/efi_dtbchosen.md)。
