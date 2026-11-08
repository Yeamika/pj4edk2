# 模块说明

抽象接口在 [pocketjsforedk2](../pocketjsforedk2/README.md)
中统一维护，接口结构和服务类型不在运行时目录重复定义。

| 位置                                                    | 职责                                                       |
| ------------------------------------------------------- | ---------------------------------------------------------- |
| [接口库](../pocketjsforedk2/README.md)                  | EFI ABI、生成头文件、启动服务和数据契约                    |
| [boot-ui](../default-ui)                                | 重新导出公共类型；提供默认脚本、显式演示预设及名称卡片描述 |
| [默认页面](../default-ui/pages)                         | boot/recovery/charging/low-battery 页面与动画              |
| [pocketjs-embedded-runtime](../crates/embedded-runtime) | 官方 guest、长期 VM、用户 context、输入与输出适配          |
| [debug-channel](../crates/debug-channel)                | 用户显式开启的有界不透明数据队列                           |
| [boot-assets](../crates/boot-assets)                    | 共用字体图集与名称卡片渲染                                 |
| Windows demo（外部桌面参考工程，未纳入本仓库）          | 桌面服务、窗口、阶段嵌套及回归                             |
| [EFI 参考宿主](../uefi-runtime/README.md)               | 消费 UEFI 协议并加载页面与子映像                           |

当前 UEFI 宿主使用 `pocketjs-embedded-runtime`。跨 EFI 驻留服务与专用堆仍需由宿主实现，
接口或原生测试不构成设备执行证据。构建验证范围见[仓库验证记录](../VALIDATION.md)。

详见[嵌入式运行时](embedded-runtime.md)、[启动服务](boot-services.md)、[显示与加速](rendering.md)及[默认页面](default-ui.md)。
