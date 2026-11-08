# pocketjsforedk2

本工程只维护 PocketJS 启动环境的抽象接口：跨 EFI 实例交接 ABI，以及宿主与页面之间的启动、输入、配置和显示区域契约。 Rust
是接口定义的来源，跨 loader 的 C 头文件由 Rust 生成。

电源采样、标准触控和退出交接契约见 [硬件接口](docs/hardware.md)。外接电源状态支持
`null`（未知），不把读取失败当作未接电源，也不把接电当作正在充电。

## 工程结构

```text
pocketjsforedk2/
├── Cargo.toml                   # ABI 库与接口 workspace
├── src/                         # no_std EFI ABI、消费端辅助函数
├── crates/boot-api/             # no_std + alloc 的服务和数据契约
├── include/                     # 自动生成的 C 头文件
├── tools/                       # 头文件生成与互调检查
├── checks/                      # C/Rust ABI 验证
└── docs/                        # 接口说明、接入和职责
```

| 接口                        | 内容                                                                                     |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| `pocketjs-boot-context-abi` | GUID、堆范围、实例与映像句柄、代次、独占租约和单一调用入口；沿用既有 crate 名称          |
| `pocketjs-boot-api`         | 启动项、设置、自定义参数、显示设备与区域、输入/电源事件、页面动作和 `BootServices` trait |

主工程不依赖 PocketJS 引擎、QuickJS、渲染器、设备驱动或桌面库。设置默认值和启动项由调用方提供。

## 入门与实现

从[接入与验证](docs/getting-started.md)开始。协议语义见[EFI ABI](docs/abi.md)与[宿主服务](docs/services.md)，项目职责见[边界说明](docs/boundaries.md)。

本组件位于 `pj4edk2` 仓库内。默认页面 `default-ui`、UEFI 宿主 `uefi-runtime` 和通用库 `crates`
通过路径依赖使用这里定义的类型。 板级固件、硬件驱动和设备验收由对应平台工程维护。

## 当前状态

接口已从参考实现中抽离。既有 EFI GUID、ABI 布局、操作码与生成头文件内容保持不变。 本工程定义交接契约；真实驻留服务、堆生命周期和跨 EFI
租约调度由宿主实现，不能将接口编译通过视作固件执行验证。
