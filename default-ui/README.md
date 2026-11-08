# 默认 PocketJS 页面

本组件位于 [pj4edk2 仓库](../README.md)，为 UEFI 宿主提供默认页面。

| 目录 | 内容 |
| --- | --- |
| `pages/` | boot、UEFI、组合模式、recovery、charging、low-battery 页面及动画与设置 |
| `assets/` | 默认字体及其许可证 |
| `src/` | 脚本装载顺序、页面预设、名称卡片描述与可观测状态 |

Rust crate 名称保持 `pocketjs-boot-ui`，使用 `no_std + alloc`。公共服务和数据类型来自
[`pocketjs-boot-api`](../pocketjsforedk2/crates/boot-api)，本组件重新导出供宿主使用。
引擎实例、输入设备、设置存储和实际启动由宿主提供。

脚本入口及页面行为见[默认页面说明](../reference/pocketjs-uefi-handoff/docs/default-ui.md)，运行与验证见
[仓库构建与测试说明](../README.md)。

电池旁的充电按钮由宿主的 `ChargeState` 能力事件开启，显示硬件使能状态；等待回执时
不重复发送，失败显示重试，状态未知时只允许请求停充。电压、有符号电流和实际充电状态
独立显示。主菜单从第一项按 Up（或末项按 Down）选中充电按钮，Enter 切换；继续 Up/Down
返回列表。实际键映射由平台提供。页面不直接访问硬件、不持久保存充电开关。

Recovery 的诊断、启动日志、恢复启动设置在二次确认后发给宿主；等待期间不重复发送。
`RecoveryResult` 显示真实结果与失败信息，支持触控和 Up/Down 分页，Enter/Escape 关闭。
只有宿主确认写入并读回成功的设置才更新界面。桌面预览没有固件维护后端时明确返回不可用。
