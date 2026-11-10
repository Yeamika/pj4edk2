# pj4edk2 Linux boot-display middleware

Remote: `https://github.com/Yeamika/pj4edk2.git`, branch `kernel-boot-display`.

本分支承载 pj4edk2 与 Linux 显示交接的公共内核侧中间件。初始化提交只包含本说明；尚未迁入 C 代码、Kconfig 或 Kbuild 文件，不是已经可编译的内核模块。

## 仓库与接入位置

- 独立本地克隆：`/state/workspace/pushup/pj4edk2-linux-boot-display`。
- W202DS 子模块：`w202ds-linux/6.18.35/Devices/drm/source/pj4edk-boot-display/common`。
- W202DS 实际接收实现：`Devices/drm/source/pj4edk-boot-display/`，以及原生 DRM 的相应显示接管代码。
- EFI 侧配套公共接口在同一远程的独立 `linux-efi` 分支。

消费项目以 Git submodule 提交号固定版本。两个中间件分支独立初始化，只提交各自文档，不混入 pj4edk2 主分支运行时、板级内核或构建结果。远程仍为 pj4edk2，使用 `.gitmodules` 和固定提交 gitlink；本地入口使用指向独立克隆的相对符号链接。Git 索引保留 gitlink，不把符号链接提交为普通链接文件。

## 公共层与板级实现

公共层拟定义显示描述版本与字段、长度/溢出/几何校验、解析结果结构及接管生命周期接口。它不执行 JavaScript，不依赖 PocketJS 运行时，也不把 UEFI 指针带入 Linux。

W202DS 负责显存映射、保留内存确认、RGB565 与旋转适配、DPU/DSI 寄存器、时钟与电源引用、缓存清理、console 绘制及原生 DRM 接管。其他硬件应提供自己的适配，不能沿用 W202DS 的物理地址或寄存器常量。

这里的“内核模块中间件”指内核侧组件，不意味着必须生成可加载 `.ko`：当前启动日志发生在 rootfs 与原生 DRM 接管之前，W202DS 使用 `CONFIG_PJ4EDK_BOOT_DISPLAY=y` 内建。后续拆分必须保持这个时序，不能改成依赖 rootfs 才能加载的服务。

## 现有传输协议

Linux 从 DT `/chosen/pocketjs,boot-display` 读取 80 字节、20 个大端 u32。EFI 内部 BootDisplay 是另一份 36 字节描述，必须由 EFI 适配层校验并转换；不能将两者作为同一内存结构解释。

| 索引 | 字段 |
| --- | --- |
| 0–1 | 当前协议标记 `1, 2` |
| 2–3 | 64 位 framebuffer 物理地址的高低 u32 |
| 4–5 | 保留区字节数、有效画面字节数 |
| 6–9 | 物理宽、高、stride 字节数、像素格式 |
| 10–11 | 逻辑画布宽、高 |
| 12–15 | 日志框 x、y、宽、高 |
| 16–19 | 旋转、面板安装方向、每像素字节数、当前固定标记 |

当前 W202DS 接收端严格校验本机显存、格式、旋转与几何；这些约束目前仍在板级实现。公共化时应区分通用格式校验和板级接受策略，不直接放宽现有安全边界或更改线上编码。

## 生命周期与许可边界

有效描述才注册可选启动 console；缺失或损坏时维持既有回退行为。原生 DRM 接管前须停止并等待旧绘制工作，之后释放旧 owner 的时钟和映射，不能继续写已交出的 framebuffer。通用层不暴露 W202DS DRM 私有符号作为跨项目 ABI。

现有 Linux 实现标有 `GPL-2.0-only`，后续抽出时保留原始许可与来源；EFI 侧的许可不自动适用于内核代码。初始化说明不替代源码许可。

## 当前状态与后续验收

本子模块只建立版本与目录边界，尚未被 W202DS Kbuild 引用；当前 Linux 显示实现继续在板级项目中编译。后续先抽出可测试的协议与校验接口，再修改板级适配调用。验收覆盖缺失、截断、版本错误、越界、溢出、两侧编码一致和实际内核构建；显示持续性、时钟与 DRM 接管仍须实机验证。

## 交接基线

以 REIMUKS 的 `boot/doces/efi_dtbchosen.md` 为本轮对接依据，本地参考 `/state/workspace/pushup/reimuks/boot/doces/efi_dtbchosen.md`。initrd 范围交给 Linux 内核，PARTUUID 交给 initramfs，显示描述交给早期控制台与 DRM；三者不混用。`w202,root-subdir` 仍是未实现的扩展，不纳入本轮已实现能力。

## 现有模块内部的公共依赖

本轮远端只提交本 `readme.md`。不提交源码、额外文档或构建结果；详细笔记和接口快照只留在本地。现有模块内部的 `common/` 仅登记这份公共接口约定，尚未进入实际编译依赖。

# 从现有 boot-display 模块内部抽出公共层

唯一板级模块保持为 `w202ds-linux/6.18.35/Devices/drm/source/pj4edk-boot-display`。本分支只作为该模块内部 `common/` 的依赖；不会在顶层 components 建立第二个显示驱动。

## 分层与边界

| 层 | 归属 | 责任 |
| --- | --- | --- |
| 公共描述与校验 | kernel-boot-display 分支，板级模块的 common/ | 80 字节 DT 描述的字段、版本、字节序、边界、溢出与通用几何校验 |
| 可复用的显示控制逻辑 | 后续逐步评估抽出 | 文本状态、脏区域及所有权生命周期；以回调隔离平台行为 |
| W202DS 适配 | 原 pj4edk-boot-display 模块 | 显存与保留区确认、旋转和 RGB565 布局、DPU present、时钟/电源、缓存清理、console/workqueue 接入 |
| 原生 DRM | W202DS DRM 项目 | 接管 aperture、取得自身资源、继续显示并终止旧 owner |

## 现有文件如何拆分

- `pj4edk-boot-display.c` 里的 DT 读取应调用未来公共解析/校验接口；`memblock` 检查和固定显存约束仍由板级执行。
- `pj4edk-boot-display-core.h` 不能整体当作公共库迁入：它既有终端逻辑，也硬编码了 1200×1920、旋转、RGB565 和特定 framebuffer 地址。先拆协议校验与板级接受策略，再逐项参数化终端和绘制逻辑。
- `pj4edk-boot-display-clocks.h`、DPU 寄存器和 ARM64 cache maintenance 保留在板级，公共层不包含这些地址和硬件时序。
- `source/baseline/w202_boot_console.c` 是原生 DRM 侧的另一位协议消费者。公共解析接口稳定后可让早期 console 与 DRM 复用同一套描述校验，避免各自维护字段解释；两者仍是不同生命周期的显示 owner。
- 原 Kconfig/Makefile 继续作为唯一内核接入。早期显示当前必须内建，不能为了“模块化”改为等 rootfs 启动后才加载 .ko。

## 后续接口形状

先提供不依赖 printk、MMIO、分配器的描述解码与验证；输入明确包含数据长度，输出为已校验描述。平台另外执行显存全覆盖、保留区及支持格式检查。

如果继续抽出绘制与调度，公共逻辑只能通过平台提供的映射、flush、present 与 release 等操作访问硬件，必须明确每个操作的执行上下文、并发规则和资源所有权。不得在尚未完成这部分设计时用一个空 ops 表包住现有驱动便宣称通用化。

缺失或不合法描述保留当前回退行为。DRM 接管必须先停止并等待绘制工作，再释放旧映射与时钟；公共层不得引入绕过 aperture 所有权的新全局接口。

## 本轮交付与下一步

本轮仅交付 MD、层次拆分和现有模块内的依赖入口，尚未修改 C 源码、Kconfig 或 Makefile，也未产生第二套驱动。

下一步先抽出协议描述及纯校验测试，覆盖两个有效布局、截断、未知版本、越界、整数溢出和两侧编码一致；之后才接入现有早期 console 与 DRM 消费方并进行实际内核构建。显示、时钟与接管时序仍须硬件验收。原 GPL-2.0-only 源码的许可与来源须保留。
