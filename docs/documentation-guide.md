# 文档规范

本页规定 PocketJS for Boot 文档的组织、写法和检查方式，供维护者新增或修改文档时使用。

## 参考来源

参考 Firecracker 文档的分工、示例组织与自动检查方式。本次阅读基线为提交
`3468bd74c33706357c4496fdba4cbc28e8b91883`，阅读日期为 2026-10-28。

| 参考文档                        | 采用的做法                                       |
| ------------------------------- | ------------------------------------------------ |
| [README][fc-readme]             | 首页介绍项目、入门、设计、能力与限制，再链接专题 |
| [Getting Started][fc-start]     | 先列前置条件，再给操作步骤、输出和问题排查       |
| [Design][fc-design]             | 按范围、宿主集成、内部架构和组件组织设计         |
| [贡献质量要求][fc-contributing] | 公开接口有文档，修改说明包含理由和验证依据       |
| [Markdown 检查][fc-checks]      | 使用 `mdformat --check`，检查内部链接目标存在    |
| [mdformat 配置][fc-format]      | 80 列自动换行、有序列表源码不递增编号、LF 行尾   |

下列文档分类、状态标签和模板是本项目的具体约定。

## 目录与职责

| 位置                      | 职责                                   |
| ------------------------- | -------------------------------------- |
| 根目录 `README.md`        | 项目用途、当前能力、快速入口与主要限制 |
| `docs/README.md`          | 完整阅读索引和能力状态                 |
| `docs/getting-started.md` | 当前可执行的准备、生成和验证步骤       |
| `docs/design.md`          | 总体范围、宿主关系、内部架构与生命周期 |
| `docs/modules.md`         | 模块职责、源码位置和实现状态           |
| `docs/uefi-handoff.md`    | 生产者与消费者的协议约定               |
| 其他 `docs/*.md`          | ABI、多核、入口、用户层等独立专题      |

每个事实保留一个主要说明位置，其他页面链接到它。生成命令集中在入门页，协议语义集中在交接页。新增文档需要加入索引；调整路径时更新入站链接。旧
`USERSPACE.md` 保留为指向新专题的入口。

常规专题使用小写英文文件名，多词以连字符分隔。目录索引使用 `README.md`。正文使用中文，保留 UEFI、BSP、AP、QuickJS
和源码标识符的原名。

## 页面结构

每页使用一个一级标题，开头用一两句话说明内容和适用目的。二级标题划分主题，三级标题展开细节，不跳级。标题直接命名主题，不加章节序号。

按页面用途选择必要章节：

| 页面类型       | 建议顺序                                                       |
| -------------- | -------------------------------------------------------------- |
| 入门或操作指南 | 目的、前置条件、步骤、预期结果、问题排查、后续阅读             |
| 设计           | 范围、宿主集成、内部架构、生命周期、限制、验证                 |
| 模块           | 职责、输入输出、依赖、源码与状态                               |
| 协议或接口     | 发现、结构与版本、操作语义、所有权、错误、并发与退出边界、验证 |

短页面可以合并章节，不为填模板添加空标题。流程可用 Mermaid；字段、兼容矩阵和状态适合用表格。顺序操作使用有序列表，正文解释原因和约束。

## 状态与证据

设计中的能力在开头使用 GitHub 提示块标注。例如：

```markdown
> [!NOTE]
> 状态：设计。AP 调度与有界队列尚未实现。
```

状态用词统一如下：

| 标签         | 含义                                                        |
| ------------ | ----------------------------------------------------------- |
| 设计         | 描述目标行为，尚无相应实现                                  |
| 已实现       | 有源码，需要链接到对应实现并列出未验证部分                  |
| 宿主验证通过 | 当前工具验证了宿主编译、布局或互调，不能据此声称 EFI 可运行 |
| EFI 验证通过 | 需要列出实际固件/QEMU 环境、架构、场景和证据                |

同页状态不一致时，按模块或能力分别标注。区分上游已有能力与本项目已经接入的能力。记录性能结论时写明设备、固件、构建版本、测量方法及结果；设计目标不写成测量结论。

正文描述最终约定，省略聊天记录、选项编号和临时命名过程。尚未决定的事项明确列出，不写成已经存在的配置项或命令。

## 接口与示例

接口文档说明参数、返回值、版本兼容、所有权、指针有效期、调用位置、重入限制和结束边界。字段与常量以 Rust 定义为准，生成的 `.h` 不手工修改。

命令示例需要注明工作目录、所需工具与版本、变量来源及成功判据。Shell 代码块标为 `sh`，Rust 标为 `rust`，纯输出或目录树标为
`text`，图标为 `mermaid`。操作命令与输出分别放置，避免复制输出作为输入。

只将可执行步骤写进入门页。规划中的 EFI
文件、分区路径或配置结构在示例前明确标注，并解释哪些组件尚未实现。跨页面引用命令时链接到原步骤，避免复制后逐渐不一致。

## Markdown 格式

使用 GitHub Flavored Markdown。标题、段落、列表、表格和围栏代码块之间留空行，文件使用 UTF-8、LF
行尾并以换行结束。路径、字段、函数与命令用行内代码表示；引用源码时提供相对链接。

[`../.mdformat.toml`](../.mdformat.toml) 采用 Firecracker 的三项配置。 80
列由格式工具处理；URL、表格和代码行不强行截断。中文保持自然段落，格式化后检查中英文交界处和标识符是否仍然清楚。

本项目选择的工具版本固定在 [`requirements.txt`](requirements.txt)，包含表格和 GitHub 提示块支持。准备好
Python 3、pip 和 venv 后，在项目根目录运行：

```sh
python3 -m venv .venv-docs
.venv-docs/bin/python -m pip install -r docs/requirements.txt
.venv-docs/bin/mdformat --check README.md USERSPACE.md docs
```

需要修正格式时，运行同一个命令并去掉 `--check`，然后复查差异。文档工具属于开发环境依赖，不进入 EFI 映像。

## 链接与变更检查

内部链接使用文档所在目录的相对路径，链接文字描述目标内容。检查目标时移除 `#` 锚点和 `?`
查询参数后确认文件或目录存在；有锚点的链接还应确认章节没有改名。外部依赖的行为说明优先链接固定提交或版本。

交付前检查格式、相对链接、目录索引、状态标注与源码是否一致。说明变更目的、涉及内容、运行过的检查及验证边界。纯文档改动运行文档检查；接口改动再运行
[ABI 验证](getting-started.md)。

[fc-checks]: https://github.com/firecracker-microvm/firecracker/blob/3468bd74c33706357c4496fdba4cbc28e8b91883/tests/integration_tests/style/test_markdown.py
[fc-contributing]: https://github.com/firecracker-microvm/firecracker/blob/3468bd74c33706357c4496fdba4cbc28e8b91883/CONTRIBUTING.md
[fc-design]: https://github.com/firecracker-microvm/firecracker/blob/3468bd74c33706357c4496fdba4cbc28e8b91883/docs/design.md
[fc-format]: https://github.com/firecracker-microvm/firecracker/blob/3468bd74c33706357c4496fdba4cbc28e8b91883/.mdformat.toml
[fc-readme]: https://github.com/firecracker-microvm/firecracker/blob/3468bd74c33706357c4496fdba4cbc28e8b91883/README.md
[fc-start]: https://github.com/firecracker-microvm/firecracker/blob/3468bd74c33706357c4496fdba4cbc28e8b91883/docs/getting-started.md
