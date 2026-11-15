# pj4edk2 Linux EFI

公共 EFI → Linux 显示交接构建与 DT 写入逻辑。上游仓库为
[Yeamika/pj4edk2](https://github.com/Yeamika/pj4edk2) 的 `linux-efi` 分支。

- `include/pj4edk2_boot_display.h`：与 Linux 接收端一致的 80 字节显示协议。
- `include/pj4edk2_linux_display.h`：从已验证的 surface 和逻辑区域构建显示描述，不假设平台地址或 BootDisplay ABI。
- `include/pj4edk2_linux_chosen.h`：initrd 开区间和 PARTUUID 校验。
- `include/pj4edk2_linux_config.h`：LinuxBoot 配置解析，校验路径、必填字段、重复项和长度；不读取文件系统或 GPT。
- `include/pj4edk2_linux_fdt.h`：使用调用方 libfdt 写入显示描述和 initrd 范围。
- `edk2/DisplayBridge.c`：通用 EDK II EFI 应用。读取 pj4edk2 BootDisplay 表和线性 GOP，验证 EFI reserved/WB 内存，将保留显示和区域写入复制的 DT，再启动 Linux EFI stub。

桥接应用的 LoadOptions 为：`绝对 EFI 路径 空格 Linux 命令行`。只接受标准
RGBX/BGRX 32 位 GOP 和完整 EFI reserved RAM，不处理 BLT-only GOP、任意 MMIO、
显示硬件初始化、固定加载地址或板级 watchdog。子程序返回时恢复原 DT 配置表。
运行中的 Linux 显示驱动不能同时覆盖同一保留画面；示例关闭 efifb 并使用 quiet
抑制 EFI stub 对固件文本控制台的输出，Linux 自身的 printk 仍正常工作。

36 字节 BootDisplay 表只是区域请求，
不等于 DT 显示描述，也不单独授权物理内存访问。rootfs 分区标识使用 `pj4edk2,root-partuuid` 属性。

协议版本 1 为 RGB565；版本 2 支持 BGRX8888/RGBX8888。公共协议头与
`kernel-boot-display` 分支保持一致。QEMU ARM64 Small 工程提供桥接应用的实际编译和运行示例。

在本分支根目录执行，先设置源码树外的 `BUILD_DIR`，并准备主机 C 编译器：

```sh
mkdir -p "$BUILD_DIR"
for test in protocol initrd surface config; do
  cc -std=c11 -Wall -Wextra -Werror -Iinclude "tests/$test.c" -o "$BUILD_DIR/$test"
  "$BUILD_DIR/$test"
done
```

纯协议与 surface 转换无需 UEFI/Linux API。FDT helper 由调用方提供 libfdt；EDK II
桥接应用由消费工程提供 INF 并声明依赖；本分支只提供桥接源码。
平台加载策略和硬件访问继续由平台实现。

分层与平台边界见 [ABSTRACTION.md](ABSTRACTION.md)，字段语义见 [DT 交接说明](docs/efi_dtbchosen.md)。

本页列出当前源码与检查入口，不代表最新代码已完成构建或物理设备验收。

## LinuxBoot 配置

平台加载器可使用公共配置解析器读取同目录 `linuxboot.conf`：

```ini
kernel=/EFI/Linux/Image
initramfs=/EFI/Linux/initramfs.cpio.lz4
dtb=/EFI/Linux/board.dtb
metadata=/EFI/Linux/payload.bin
root-partuuid=01234567-89ab-cdef-0123-456789abcdef
options=systemd.unit=graphical.target
```

前五项必填，options 可省略或为空；文件最大 4096 字节，支持 LF/CRLF 与整行 `#` 注释。
路径是 ESP 内绝对路径，不接受空段、`.` 或 `..`。根分区和启动目标可只修改配置；
内核、initramfs 或 DTB 内容改变时，消费端必须更新对应完整性元数据。
元数据格式、文件加载、内存布局与命令行写入由平台加载器负责。
通用 `DisplayBridge.c` 仍使用上述 LoadOptions 接口，不会因新增解析器自动切换配置格式。

配置不是显示协议的一部分。EFI 通过 DT 传递 PARTUUID、initrd 地址和 bootargs；
initramfs 挂载根分区，systemd 消费 `systemd.unit=`。公共显示接收端不读取 ESP 配置。

## 配置与协议验证边界

`tests/config.c` 覆盖 ESP 启动项配置解析；公共库不扫描 GPT，也不绑定某个发行版根分区。交接中的面板固有方向与请求旋转必须分别解释，不能把旧模式切换当作协议要求。主机协议测试通过不代表硬件显示接管已完成。消费工程应固定本分支的提交，并与 `kernel-boot-display` 使用相同协议版本。
