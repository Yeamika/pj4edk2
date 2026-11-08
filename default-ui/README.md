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

外部项目通过 `build.sh --default-ui DIR --output DIR` 或 `build-rust.sh --default-ui DIR --output DIR`
选择构建时嵌入的 UI。外部目录只需要资源和如下 `ui.json`，无需复制本目录的 Rust crate：

```json
{
  "schema": 1,
  "scripts": ["pages/main.js", "pages/stage.js"],
  "font": "assets/Inter-Regular.ttf"
}
```

`scripts` 是非空、有序且不重复的 UTF-8 JavaScript 文件列表；脚本在同一个 JS 环境中依次执行。
`font` 指定有效的 TTF/OTF 文件，当前字体图集保留原有 8 个字号槽位和 ASCII 字符范围。
所有路径相对于 UI 目录，不允许绝对路径、`..` 或指向目录外的符号链接；缺文件直接报错。
清单只负责脚本和字体，其他资源需由脚本自行内嵌或使用现有宿主能力。

自定义页面需遵循现有宿主契约：提供 `rimuksInit(configJson)` 和
`rimuksTick(nowMs, eventsJson)`；宿主传入 JSON 字符串，动作通过 `bootEmit(JSON.stringify(action))`
返回。可使用现有 PocketJS `ui`、`bootBakeName` 等接口；板级菜单配置仍由最终组合项目提供。
仅更换样式时，保留自带清单顺序并修改对应页面即可。

脚本入口及页面行为见[默认页面说明](../reference/pocketjs-uefi-handoff/docs/default-ui.md)，运行与验证见
[仓库构建与测试说明](../README.md)。

电池旁的充电按钮由宿主的 `ChargeState` 能力事件开启，显示硬件使能状态；等待回执时
不重复发送，失败显示重试，状态未知时只允许请求停充。电压、有符号电流和实际充电状态
独立显示。主菜单从第一项按 Up（或末项按 Down）选中充电按钮，Enter 切换；继续 Up/Down
返回列表。实际键映射由平台提供。页面不直接访问硬件、不持久保存充电开关。

Recovery 的诊断、启动日志、恢复启动设置在二次确认后发给宿主；等待期间不重复发送。
`RecoveryResult` 显示真实结果与失败信息，支持触控和 Up/Down 分页，Enter/Escape 关闭。
只有宿主确认写入并读回成功的设置才更新界面。桌面预览没有固件维护后端时明确返回不可用。
