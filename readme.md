# pj4edk2 Linux boot display

公共启动显示协议和 Linux fbcon 交接显示。上游仓库为
[Yeamika/pj4edk2](https://github.com/Yeamika/pj4edk2) 的 `kernel-boot-display` 分支。

- `include/pj4edk2_boot_display.h`：显示描述校验、大端编解码、边界和容量检查。
- `include/pj4edk2_terminal.h`：文本、换行、滚动、可选字形尺寸、区域缩放、四方向旋转、脏区域绘制。无 Linux/UEFI API、分配、MMIO 或固定物理地址。
- `include/pj4edk2_fbcon_viewport.h`：将交接区域映射成 framebuffer 内的有界窗口，处理步幅、内存旋转和八像素内边距。
- `include/pj4edk2_fbcon.h`：Linux fbdev/fbcon 适配；字符、字体、滚动、旋转和企鹅由内核 fbcon 处理，像素操作使用 `sys_fillrect/sys_copyarea/sys_imageblit`。平台通过 atomic-safe damage 回调安排缓存刷新和显示提交。
- `linux/pj4edk2_console.c`：reserved/coherent RAM 的通用内建适配，在 `device_initcall` 注册 fbdev，并通过通用 aperture API 与原生 DRM 交接。

显示描述为 20 个大端 u32（80 字节）。版本 1 保持 RGB565 契约；版本 2
扩展为 BGRX8888（格式 2）与 RGBX8888（格式 3）。两侧协议头须保持一致。
Linux 绘制只更新 boot-output 窗口，保留周围 UI；字体由标准 `fbcon=font:VGA8x16`
等内核参数选择。旧 `pj4edk2_terminal.h` 保留给需要独立文本绘制的消费者，Linux
两条适配路径均不再注册自定义 printk 绘制器。

通用 Linux 适配只接受 `/chosen/pocketjs,boot-display-memory` 指向的 reserved-memory，
要求其 reg 覆盖显示分配、memblock 完整保留，并声明 `pocketjs,coherent-scanout`。
它用于一致性 RAM scanout，不适用于任意显存或 MMIO。描述合法不等于允许映射地址。
原生驱动接管时先取得自己的显示资源，再通过 aperture 移除该适配；移除回调
同步注销 fbcon 后释放映射。非一致性平台需在自己的 remove 中继续等待缓存/提交
worker 退出，随后才能释放时钟和 MMIO。

需要内建 `CONFIG_FB`、`CONFIG_FRAMEBUFFER_CONSOLE`、`CONFIG_FB_SYSMEM_HELPERS`
和 `CONFIG_APERTURE_HELPERS`；非零内存旋转还需 `CONFIG_FRAMEBUFFER_CONSOLE_ROTATION`。
`linux/Kconfig` 提供通用配置入口。开启 `CONFIG_LOGO`、`CONFIG_LOGO_LINUX_CLUT224`，
关闭 `CONFIG_FRAMEBUFFER_CONSOLE_DEFERRED_TAKEOVER`，并使用 `console=tty1` 和允许
启动日志的 loglevel。窗口在 logo initdata 释放前注册，企鹅数量由 fbcon 按在线 CPU
决定，并受窗口宽度限制；晚加载的模块按 Linux 标准行为不重新绘制启动 logo。

这个窗口保留整屏 stride，但不开放 `/dev/fbN` 的用户态访问，避免用户程序通过
行间空隙或 mmap 改写周围 UI。完整桌面 framebuffer 由原生 DRM 另行提供。

QEMU 示例使用内建 console 和 EFI ramfb。
DRM 接管及桌面显示策略由平台负责。

检查从本分支根目录执行；先将 `BUILD_DIR` 设置为源码树外的输出目录，并准备主机 C 编译器：

```sh
mkdir -p "$BUILD_DIR"
for test in protocol terminal fbcon-viewport; do
  cc -std=c11 -Wall -Wextra -Werror -Iinclude "tests/$test.c" -o "$BUILD_DIR/$test"
  "$BUILD_DIR/$test"
done
```

测试覆盖协议、独立终端及 fbcon 窗口的 RGB565/32 位格式、四个方向、步幅填充、
区域外像素保护和非法描述。真实 fbdev API 还需用消费端内核编译并启动验证。
协议头为 BSD-2-Clause OR GPL-2.0-only；从 Linux 显示实现抽出的终端和适配保留 GPL-2.0-only。

分层与平台边界见 [ABSTRACTION.md](ABSTRACTION.md)，字段语义见 [DT 交接说明](docs/efi_dtbchosen.md)。

本页列出当前源码与检查入口，不代表最新代码已完成构建或物理设备验收。

启动配置解析属于 EFI 公共层，显示接收端只消费 DT 中的显示描述。
`linuxboot.conf`、PARTUUID、initramfs 路径和 `systemd.unit=` 不进入本模块；
修改启动目标不改变显示协议，协议版本仍为 1/2。

## 方向与消费工程

面板固有方向与消费端请求旋转分开描述；内核显示接管应保持协议指定的缓冲区布局，不依赖先切回旧 GOP 模式。协议头由 EFI 发送端与 Linux 接收端共享，平台寄存器和扫描输出由板级 DRM 驱动负责。跨工程构建需固定对应提交并执行协议测试，不能仅凭文档更新推断设备方向已经验证。
