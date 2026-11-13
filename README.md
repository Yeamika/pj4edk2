# pj4edk2 QEMU ARM64 Small

ARM64 `virt` 测试工程：精简 Linux、virtio-gpu DRM、带自定义 DRM 模块的 initramfs，
以及 pj4edk2 UEFI 菜单。用于验证公共交接协议和完整启动流程。

```text
simulator/             QEMU 启动、截图、键盘、GPT/FAT32 磁盘生成
kernel/
  source/              上游 Linux 源码子模块，固定 v6.12.116
  kernel.config        allnoconfig 基础上的精简配置
  build.py             内核、模块、initramfs 构建
initramfs/
  init.c               静态 PID1，无发行版依赖
  drm/                 虚拟 DRM/KMS 验证模块源码
  common/              kernel-boot-display 分支子模块（内建早期 console/终端）
uefi/
  pj4edk2/             pj4edk2 uefi 分支子模块
  QemuDisplay.c        指定 GOP/ramfb 分辨率的 QEMU 适配
  build.py             通用 EDK II EFI 应用工程
  config.json          默认 ARM64 菜单
efi/
  common/              linux-efi 分支子模块
  DisplayBridge.inf    独立 EFI 加载器工程
  build.py             仅编译 DisplayBridge.efi
  fixture.c            公共协议 FDT 示例生产端
  baremetal/           退出 Boot Services 后直接绘制八色条纹
```

## 依赖与构建

Linux 主机，需要 Python 3.12+、GNU make/GCC、AArch64 GNU GCC/binutils、Flex、Bison、
BC、mtools、dosfstools、QEMU ARM64 和 AAVMF 固件。UEFI 部分还需要 Rust 1.93.0、
`rust-src`、`aarch64-unknown-none`、UUID 开发头和 Git；具体依赖见各目录说明。
网络与下载代理通过调用方环境配置，工程不写入机器专属设置。

```sh
git submodule update --init --recursive
python3 build.py --output /path/to/build --jobs 8
# 只构建内核、initramfs 和 DRM 模块：
python3 build.py --output /path/to/build --jobs 8 --kernel-only
```

源码和编译结果分离。内核版本、下载哈希及产物尺寸记录在 `linux/build.json`；
内核源码子模块未初始化时也可按固定哈希下载归档。公共项目可由 `kernel/build.py`
的 `--boot-display`、`--linux-efi` 指定，支持其他 checkout。

## 快速运行与截图

直接内核模式验证 DRM/模块，跳过 UEFI 菜单：

```sh
python3 simulator/run.py start --direct --build /path/to/build/linux --run /path/to/run-direct
python3 simulator/check.py --run /path/to/run-direct
python3 simulator/run.py screen --run /path/to/run-direct --output /path/to/screen.png
python3 simulator/run.py stop --run /path/to/run-direct
```

默认完整路径：UEFI 菜单 → 显示交接桥 → Linux 内建早期 console → initramfs/DRM：

```sh
python3 simulator/disk.py --build /path/to/build/linux \
  --menu-efi /path/to/build/uefi/PocketJsBoot.efi \
  --bridge-efi /path/to/build/efi/DisplayBridge.efi \
  --baremetal-efi /path/to/build/efi/Baremetal.efi --output /path/to/disk.img
python3 simulator/run.py start --disk /path/to/disk.img --run /path/to/run-efi \
  --firmware-code /path/to/AAVMF_CODE.fd --firmware-vars /path/to/AAVMF_VARS.fd
python3 simulator/run.py screen --run /path/to/run-efi --output /path/to/menu.png
python3 simulator/run.py key --run /path/to/run-efi --key ret
python3 simulator/run.py key --run /path/to/run-efi --key ret
python3 simulator/check.py --run /path/to/run-efi --require-handoff
```

磁盘是含 GPT 和 FAT32 ESP 的普通文件，默认 128 MiB。配置与设置位于
`EFI/BOOT/pj4edk/`；默认写入持久化，临时实验可用 `run.py --snapshot`。
QEMU 关闭后再修改磁盘。演示磁盘的 `EFI/BOOT/BOOTAA64.EFI` 用于启动菜单，
不改变板级固件的回退策略。截图无需 Pillow，使用 Python 标准库。

默认分辨率为 **1920×1200**，完整 EFI 模式默认使用 ramfb，直接启动内核模式
默认使用 virtio-gpu。`run.py --size WIDTHxHEIGHT` 可覆盖尺寸。EFI 应用内的
QEMU 适配先选择标准 GOP 模式；AAVMF 缺少所需 ramfb 模式时，使用 fw_cfg
建立新的 reserved-RAM GOP surface。这些虚拟硬件操作由 QEMU 适配层实现。

Linux 的 console 在 `console_initcall` 内建注册，随即接收 printk，
从 `core_initcall` 开始由 worker 绘制，采用 8×16 字形。它不等待 initramfs 或
DRM 模块加载。`--require-handoff` 同时检查 console 注册发生在 `/init` 运行前。

## 交接协议示例

```sh
python3 simulator/run.py dump-dtb --run /path/to/dtb-export
/path/to/build/linux/make-fixture /path/to/dtb-export/virt.dtb /path/to/fixture.dtb
python3 simulator/run.py start --direct --build /path/to/build/linux \
  --dtb /path/to/fixture.dtb --run /path/to/run-protocol
```

也可在生成 UEFI 演示磁盘时传 `disk.py --dtb /path/to/fixture.dtb`。
DTB 必须来自同一 QEMU 机器参数。示例写入有效 RGB565 描述，保留一段测试 RAM；
fixture 消费端只校验，不映射或冒充 virtio-gpu 的真实 framebuffer。
`pj4edk2_drm.ko` 创建独立的虚拟 DRM/KMS 设备；直接模式的画面来自 virtio-gpu。
完整 ramfb 模式则使用 EFI 实际保留的 GOP framebuffer，由内建 console 写入原 UI 的
boot-output 区域，保留顶部和侧边。桥接命令行关闭 efifb，避免第二个驱动覆盖画面。

成功标记包括 `PJ4EDK2_DRM PASS driver=virtio_gpu`、
`PJ4EDK2_MODULE PASS virtual DRM/KMS registered`、`PJ4EDK2_FB PASS`、
`PJ4EDK2_SMALL_READY`。提供 fixture 时还应出现 `PJ4EDK2_DT PASS`。
这些验证仅覆盖 QEMU 虚拟硬件环境。

## 子模块与版本一致性

通过子模块 gitlink 固定 UEFI、Linux EFI、内核显示公共层和上游 Linux 的版本。
初始化后应检查各依赖是否包含当前构建入口所需的协议和源文件；`.gitmodules` 的分支名不保证版本匹配。
消费方应在依赖提交发布后记录对应 gitlink，不将开发用符号链接作为子模块提交。

此分支是独立 QEMU 测试工程。源码存在不代表最新依赖组合已经跑过完整启动链，
截图、成功标记、软件帧率与物理设备显示验收分别处理。

默认六个启动项为 LinuxA、LinuxB、Android、Windows、Darwin、Baremetal。
LinuxA/B 使用同一个小内核，日志级别分别为 7 和 6。Baremetal 退出 Boot Services
后直接绘制八色竖条并停留；`simulator/check-bars.py` 可检查实际截图。
Android、Windows、Darwin 只有示例 EFI 路径，未附带对应系统或加载器，
其中 Darwin 项不表示本工程已具备 XNU 引导支持。菜单由 `uefi/config.json`
统一提供给嵌入默认配置和磁盘生成器。
