# pj4edk2 Linux boot display

公共启动显示协议、文本布局和 framebuffer 绘制。上游仓库为
[Yeamika/pj4edk2](https://github.com/Yeamika/pj4edk2) 的 `kernel-boot-display` 分支。

- `include/pj4edk2_boot_display.h`：显示描述校验、大端编解码、边界和容量检查。
- `include/pj4edk2_terminal.h`：文本、换行、滚动、可选字形尺寸、区域缩放、四方向旋转、脏区域绘制。无 Linux/UEFI API、分配、MMIO 或固定物理地址。
- `linux/pj4edk2_console.c`：可内建的 Linux 早期 console 适配，`console_initcall` 注册并接收 printk，`core_initcall` 启动异步绘制；默认 8×16。不是 initramfs 模块。

显示描述为 20 个大端 u32（80 字节）。版本 1 保持 RGB565 契约；版本 2
扩展为 BGRX8888（格式 2）与 RGBX8888（格式 3）。两侧协议头须保持一致。
绘制只更新 boot-output 的文本区域，保留周围 UI；字形默认为 12×24，平台可改为
8×16 等尺寸。文本存储有固定上限，超大区域只使用可容纳的行列，不越界。

通用 Linux 适配只接受 `/chosen/pocketjs,boot-display-memory` 指向的 reserved-memory，
要求其 reg 覆盖显示分配、memblock 完整保留，并声明 `pocketjs,coherent-scanout`。
它用于一致性 RAM scanout，不适用于任意显存或 MMIO。描述合法不等于允许映射地址。
该适配持续拥有画面，不应与其他显示驱动同时写入同一 framebuffer。

QEMU 示例使用内建 console 和 EFI ramfb。
DRM 接管及桌面显示策略由平台负责。

检查从本分支根目录执行；先将 `BUILD_DIR` 设置为源码树外的输出目录，并准备主机 C 编译器：

```sh
mkdir -p "$BUILD_DIR"
for test in protocol terminal; do
  cc -std=c11 -Wall -Wextra -Werror -Iinclude "tests/$test.c" -o "$BUILD_DIR/$test"
  "$BUILD_DIR/$test"
done
```

终端测试覆盖 RGB565/32 位格式、旋转、可变字形、滚动、步幅填充、区域外像素保护及脏区域。
协议头为 BSD-2-Clause OR GPL-2.0-only；从 Linux 显示实现抽出的终端和适配保留 GPL-2.0-only。

分层与平台边界见 [ABSTRACTION.md](ABSTRACTION.md)，字段语义见 [DT 交接说明](docs/efi_dtbchosen.md)。

本页列出当前源码与检查入口，不代表最新代码已完成构建或物理设备验收。
