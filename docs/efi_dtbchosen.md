# EFI 与 Linux 的 DT 交接

本页区分 initrd 内存范围、rootfs 分区选择和启动显示描述。
公共 helper 的存在不代表所有板级加载器、内核与 initramfs 已完成接入。

| 参数 | DT `/chosen` 属性 | 接收方 |
| --- | --- | --- |
| initrd 内存范围 | `linux,initrd-start` / `linux,initrd-end` | Linux 内核 |
| rootfs 分区 | 兼容现有消费者的 `w202,root-partuuid` | 平台 initramfs |
| 显示描述 | `pocketjs,boot-display` | 早期 console / 对应显示驱动 |

## initrd

起始地址和结束地址使用大端 64 位值，结束地址是开区间，必须对应已实际加载的载荷。
零长度、地址加法溢出和不符合平台内存策略的范围必须拒绝。
EFI 写入实际范围，Linux 定位 initrd；EFI 不执行发行版 rootfs 程序。

## rootfs PARTUUID

PARTUUID 标识 GPT 分区，不是文件系统 UUID。`w202,root-partuuid` 是保留的兼容命名，不能当作通用标准属性。
公共层提供标识校验；加载器选择与写入、initramfs 等待块设备、身份检查及挂载仍属于消费项目。
根目录子路径等扩展必须由双方显式约定，不能因为文档出现字段就假定已实现。

## 显示描述

`pocketjs,boot-display` 为 20 个大端 u32，共 80 字节。
EFI 内部 BootDisplay 表是另一份区域请求，必须结合实际 GOP、显存范围及内存策略转换，不能直接当作同一内存结构。

| 索引 | 内容 |
| --- | --- |
| 0–1 | 协议标记与版本 |
| 2–3 | framebuffer 物理地址高低 u32 |
| 4–5 | 分配容量与有效画面字节数 |
| 6–9 | 物理宽、高、stride 字节数与像素格式 |
| 10–11 | 逻辑画布宽、高 |
| 12–15 | 日志区域 x、y、宽、高 |
| 16–19 | 旋转、面板安装方向、每像素字节数及标记 |

版本 1 保持 RGB565 契约；版本 2 支持 BGRX8888 / RGBX8888。
准确常量与字段约束以 [协议头](../include/pj4edk2_boot_display.h) 为准。
EFI 与内核两侧的协议头需要保持一致，不能只更新一侧。

## 内存授权与接管

描述校验不等于映射授权。通用 RAM 适配还需要
`/chosen/pocketjs,boot-display-memory` 指向覆盖 framebuffer 的 reserved-memory，
由内核确认 memblock 完整保留，并声明 `pocketjs,coherent-scanout`。
W202DS 等具有特定显存、DPU 或缓存策略的平台使用自己的适配。

缺失或无效描述按消费端既定策略回退，不能继续使用上次启动的陈旧记录。
早期显示与原生 DRM 不能同时写同一画面；接管前应停止并等待旧绘制，再转移资源所有权。

## 验证范围

纯协议检查、EFI 桥接构建、内核构建、QEMU 启动与真实硬件验收分别记录。
本说明描述当前源码契约，不将某一平台的历史验证推广到其他设备。
