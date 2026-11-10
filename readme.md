# pj4edk2 linux-efi

远程：`https://github.com/Yeamika/pj4edk2.git` 的 `linux-efi` 分支。远程初始化提交只含 `readme.md`；本地代码提交允许领先远程，未经指令不推送源码或更新远程到本地代码提交。

## 本地实现与归属

本地克隆：`/state/workspace/pushup/pj4edk2-linux-efi`。消费入口：`w202ds-boot/pocketjs4edk2_uefi/efi/LinuxBoot/common`，位于实际板级模块内部；不是第二套驱动或加载器。远程使用 .gitmodules/gitlink，本地入口为相对符号链接。

`include/pj4edk2_boot_display.h` 提供公共显示属性、矩形校验和大端编解码；`include/pj4edk2_linux_chosen.h` 提供 initrd 开区间范围及溢出检查。LinuxBoot 已调用这些接口，ESP 策略、加载地址、GOP/DPU、保留显存和看门狗留在板级。

公共实现无内存分配、UEFI/Linux API、MMIO 或板型常量。当前格式保持 20 个大端 u32、80 字节，公共格式只定义现有版本的 RGB565 描述。合法通用布局还必须通过消费方自己的硬件接受策略，不能仅凭协议合法就访问物理地址。

## 交接契约

基线：本地 `/state/workspace/pushup/reimuks/boot/doces/efi_dtbchosen.md`。`linux,initrd-start/end` 交给内核；`w202,root-partuuid` 交给 initramfs；`pocketjs,boot-display` 交给显示接收端。Root PARTUUID 写入方及 root-subdir 扩展本轮未实现。

EFI 和内核侧的显示协议头保持相同内容与版本，两侧通过公共测试数据确认编码一致。36 字节 EFI BootDisplay 表不等同于 80 字节 DT 描述。缺失或错误描述仍走既有回退策略，W202DS 现有布局接受规则保持。

## 检查

通用测试：`cc -std=c11 -Wall -Wextra -Werror -Iinclude tests/protocol.c -o build/protocol`（先创建本地 build 目录）。本轮已以 ASan/UBSan 执行，覆盖非对齐编解码、长度、版本、越界、容量与地址溢出，以及其他合法硬件布局。

另以现有 W202DS 适配器进行重构前后对比：早期 console 10021 组、EFI 10000 组（使用真实 EDK II 头及生成的 BootDisplay ABI），并通过原生 DRM 的现有协议测试。LinuxBoot.c 已进行基于 AArch64 EDK II 头的 C 语法与类型检查；未执行完整交叉编译、链接或实机启动。

公共显示头为新编写的双许可 BSD-2-Clause OR GPL-2.0-only；既有板级 Linux 源码保留其 GPL 许可，EFI 适配保留原许可。

initrd 测试：`cc -std=c11 -Wall -Wextra -Werror -Iinclude tests/initrd.c -o build/initrd`；已验证零长度、溢出、开区间结束地址和失效时输出不变。
