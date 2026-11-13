# pj4edk2

面向 PocketJS、UEFI 与 Linux 启动交接的公共组件仓库。
`main` 仅提供分支导航；源码与构建入口分别维护在对应组件分支。

| 分支 | 内容 |
| --- | --- |
| [uefi](https://github.com/Yeamika/pj4edk2/tree/uefi) | 公共 ABI、UEFI 宿主、默认页面、共享 UI 包与构建工具 |
| [linux-efi](https://github.com/Yeamika/pj4edk2/tree/linux-efi) | EFI → Linux 的公共显示描述、范围校验与 DT 写入辅助 |
| [kernel-boot-display](https://github.com/Yeamika/pj4edk2/tree/kernel-boot-display) | Linux 启动显示描述、终端绘制与内建 console 适配 |

另有 `qemu-arm64-small` 和 `webui-viewer` 实验分支，分别用于 ARM64 虚拟启动链与浏览器 UI 预览。
它们的远端可用性以仓库分支列表为准；实验分支不属于产品板级固件。

## 获取组件

克隆时显式选择所需分支，例如：

```sh
git clone --branch uefi --single-branch https://github.com/Yeamika/pj4edk2.git
git -C pj4edk2 submodule update --init --recursive
```

消费项目通过 Git 子模块的 gitlink 固定具体提交。
`.gitmodules` 的 `branch` 只说明更新来源，不代替固定提交；更新组件后需在消费项目记录新 gitlink。
各组件的依赖、构建方式和验证范围以该分支文档为准，分支名称本身不保证接口兼容。

## 项目边界

UEFI 页面只提交启动请求；Linux EFI 层负责载荷与 DT 交接；内核显示层负责接收并呈现启动日志。
initrd 范围、rootfs PARTUUID 和启动显示描述是三类不同参数，接收方与生命周期不能混用。

W202DS 的板级驱动由 [edk2-ums9620](https://github.com/Yeamika/edk2-ums9620) 维护，
加载器与固件组合由 [w202ds-boot](https://github.com/Yeamika/w202ds-boot) 维护，
Linux 板级显示适配由 [w202ds-linux](https://github.com/Yeamika/w202ds-linux) 维护。
公共层不包含这些设备的固定地址、寄存器或刷写策略。
