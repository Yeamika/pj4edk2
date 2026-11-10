# pj4edk2 Linux EFI middleware

Remote: `https://github.com/Yeamika/pj4edk2.git`, branch `linux-efi`.

本分支承载 pj4edk2 的公共 Linux EFI 交接中间件。初始化提交只包含本说明，尚未迁入中间件代码，也不代表已经产出可执行 EFI 应用。

## 仓库与接入位置

- 独立本地克隆：`/state/workspace/pushup/pj4edk2-linux-efi`。
- W202DS 子模块：`w202ds-boot/pocketjs4edk2_uefi/efi/LinuxBoot/common`。
- W202DS 实际加载器：`w202ds-boot/pocketjs4edk2_uefi/efi/LinuxBoot`。
- 配套 Linux 侧公共接口在同一远程的独立 `kernel-boot-display` 分支。

两个中间件分支各自独立初始化，不包含主分支运行时、固件、板级源码或构建结果。消费项目以 Git submodule 的提交号固定版本；`.gitmodules` 的 branch 用于说明更新来源，并不代替固定提交。远程使用 `.gitmodules` 与固定提交的 gitlink；本地对应入口使用指向上述独立克隆的相对符号链接。Git 索引登记 gitlink，不把本地符号链接提交为普通链接文件。

## 职责边界

公共层拟提供启动载荷描述、长度与范围检查、DT `/chosen` 写入辅助接口、BootDisplay 描述转换和交接阶段约束。真正加载文件、选择内存布局、确认硬件状态、完成 ExitBootServices 和跳转内核，由板级 EFI 应用组合实现。

W202DS 的 ESP 路径与 GUID、固定加载地址、DT compatible、GOP 与 DPU 寄存器校验、显存保留区、PMIC 看门狗和缓存/MMU/CPU 交接操作继续归属 `efi/LinuxBoot`。不得把这些固定硬件参数作为所有设备的公共默认值。

## 与 REIMUKS 的交接

REIMUKS 接收板级内核与匹配模块，制作 initramfs 和 rootfs，再按加载器支持的格式组装启动包。当前 W202DS 加载器读取 Android v4 `boot.img`、32 字节 `payload.bin` 和 DTB；REIMUKS 的原始 initramfs/rootfs 产物不能直接视为此启动包。

拟固定的 DT 属性：

| 属性 | 写入方与消费方 | 要求 |
| --- | --- | --- |
| `linux,initrd-start` / `linux,initrd-end` | LinuxBoot → Linux 内核 | 大端 64 位起止地址，结束地址为开区间；对应真实已加载载荷 |
| `w202,root-partuuid` | W202DS 启动项 → initramfs | NUL 结尾 GPT PARTUUID 字符串；初始保留现有板级命名，不擅自改为新的公共属性 |
| `pocketjs,boot-display` | EFI BootDisplay 转换层 → Linux 显示接收端 | 现有版本为 20 个大端 u32 / 80 字节；与接收端版本一致 |

rootfs 由 initramfs 挂载，EFI 不执行发行版用户态。根分区选择不依赖可被 `CONFIG_CMDLINE_FORCE` 丢弃的外部 bootargs。

## 当前状态与后续验收

当前实际功能仍在 W202DS `efi/LinuxBoot`，本子模块位于实际模块内部的 `common/`；文档初始化阶段尚未被构建引用。现有加载器已经写入 initrd 范围及显示属性，但尚未写入 `w202,root-partuuid`；REIMUKS 已新增该根分区属性的接收逻辑，完整链路未验收。

后续按可构建步骤抽出公共接口并修改消费方，不能仅增加子模块便宣称完成接入。验收至少覆盖载荷越界、DT 容量与属性错误、无显示请求时清除旧记录、根分区标识、双方显示协议一致性，以及实际 EFI 构建。硬件启动另行验收，不在初始化提交中宣称完成。

## 交接基线

以 REIMUKS 的 `boot/doces/efi_dtbchosen.md` 为本轮对接依据，本地参考 `/state/workspace/pushup/reimuks/boot/doces/efi_dtbchosen.md`。initrd 范围交给 Linux 内核，PARTUUID 交给 initramfs，显示描述交给早期控制台与 DRM；三者不混用。`w202,root-subdir` 仍是未实现的扩展，不纳入本轮已实现能力。

## 现有模块内部的公共依赖

本轮远端只提交本 `readme.md`。不提交源码、额外文档或构建结果；详细笔记和接口快照只留在本地。现有模块内部的 `common/` 仅登记这份公共接口约定，尚未进入实际编译依赖。

# 从 LinuxBoot 内部抽出公共交接层

唯一板级加载器保持为 `w202ds-boot/pocketjs4edk2_uefi/efi/LinuxBoot`。本分支作为该模块内部 `common/` 的依赖，不在顶层 components 下建立另一套 LinuxBoot。

## 分层与边界

| 层 | 归属 | 责任 |
| --- | --- | --- |
| 页面与固件运行时 | pj4edk2 的 uefi 分支 | 产生启动选择与 BootDisplay 配置表 |
| 公共交接层 | linux-efi 分支，板级模块的 common/ | 描述载荷与 DT 参数；通用解析、范围和协议校验；无板型常量的 DT 编码辅助 |
| 实际加载器与硬件适配 | W202DS efi/LinuxBoot | 文件访问、ESP 与路径、加载地址、DT 与 GOP/DPU/显存确认、PMIC 看门狗、退出 UEFI 与跳转内核 |
| 根文件系统启动 | REIMUKS initramfs | 等待 PARTUUID 对应设备、验证内核身份、挂载并 switch_root |

## 现有文件如何拆分

- `LinuxImage.c/.h`：逐个区分通用镜像格式与边界检查、板级固定地址和加载策略；只把前者抽成公共解析接口。
- `LinuxDisplay.h`：拆成 BootDisplay 与 DT 描述的通用几何/编码部分，以及 W202DS 的 1920×1200、RGB565、旋转和固定地址接受策略。后者留在板级。
- `LinuxBoot.c`：继续作为板级入口和流程组合点；DT 通用写入辅助可抽出，但文件读取、MMIO 检查、保留内存校验及启动项策略留在此项目。
- `LinuxHandoff.c`：当前是 UMP9620 ADI 看门狗事务，不属于通用交接库；继续留在板级。
- `EnterKernel.S`：ARM64 交接中的架构公共部分可以单独评估，但不能在未验证缓存/MMU和CPU状态的情况下作为通用实现迁入。

## 两侧共同遵守的协议

按 `docs/efi_dtbchosen.md` 区分 initrd 范围、rootfs PARTUUID 和显示描述。EFI 层写入实际加载位置；initramfs 消费根分区选择；内核消费显示属性。内核显示中间件不参与 rootfs 挂载，EFI 也不调用 rootfs 程序。

## 本轮交付与下一步

本轮只有 MD 与模块内的依赖入口，不改变 LinuxBoot 的 INF 或 C 文件，不声称实现了公共 C API。

后续按“可独立测试的协议结构与解析 → EFI 编码辅助 → 板级调用 → INF 构建接入”逐步实施。公共接口应通过明确参数接收布局与策略，不能读取 W202DS 私有全局变量。接入前验证现有边界与失效路径，接入后执行真实 EFI 构建及另行安排的硬件交接验收。
