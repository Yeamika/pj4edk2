# pj4edk2

面向 PocketJS、UEFI 与 Linux 启动交接的公共组件仓库。`main` 仅保留此导航说明，各组件在对应独立分支维护。

| 分支 | 内容 | 独立本地克隆 |
| --- | --- | --- |
| [uefi](https://github.com/Yeamika/pj4edk2/tree/uefi) | UEFI 固件侧公共运行时、ABI、默认 UI 与构建接口；具体板级固件由对应设备项目组装 | `/state/workspace/pushup/pj4edk2` |
| [linux-efi](https://github.com/Yeamika/pj4edk2/tree/linux-efi) | EFI 到 Linux 的公共交接中间件；当前仅初始化文档 | `/state/workspace/pushup/pj4edk2-linux-efi` |
| [kernel-boot-display](https://github.com/Yeamika/pj4edk2/tree/kernel-boot-display) | Linux 内核侧启动显示协议与公共组件；当前仅初始化文档 | `/state/workspace/pushup/pj4edk2-linux-boot-display` |

消费项目通过 `.gitmodules` 指定对应分支，并以 gitlink 固定具体提交。本地开发入口使用指向独立克隆的相对符号链接；不要将本地符号链接作为普通文件提交到远程。

W202DS 的真实 Linux EFI 加载器在 `w202ds-boot/pocketjs4edk2_uefi/efi/LinuxBoot`；真实内核显示适配在 `w202ds-linux/6.18.35/Devices/drm/source/pj4edk-boot-display`。公共层与板级实现的边界，以及 initrd 范围、rootfs PARTUUID、`pocketjs,boot-display` 三类 DTB 交接参数，见两个中间件分支的 README 与 `docs/efi_dtbchosen.md`。
