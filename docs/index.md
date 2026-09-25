# 文档索引

> 本文件是全局地图：只做导航和当前生效文档列表。变更历史统一写入 [CHANGELOG.md](CHANGELOG.md)。

## 目录权威

| 目录 | 角色 | 可否作为开发依据 |
|---|---|---|
| [docs/official/](official/) | 唯一真相源。只有 `status: active` 的文档才是开发、设计、需求的合法依据 | 是 |
| [docs/drafts/](drafts/) | 临时草稿区。仅存放头脑风暴、备选方案、过程记录 | 否。引用时必须标注“非生效版本” |
| [docs/archive/](archive/) | 废弃归档区。所有内容均已失效，仅用于历史回溯 | 否 |

仓库是唯一事实来源：agent 看不到的内容，对它来说就不存在。

## 当前生效文档

> 每次在 `docs/official/` 新建或废弃正式文档，必须同步更新本表。本表不记录变更历史。

| 分类 | 文档 | 版本 | topic | 说明 |
|---|---|---|---|---|
| rules | [Graphify 代码图谱工作流](official/rules/graphify-workflow.md) | 1.0.0 | graphify-workflow | 图谱导航层的使用、更新与验证规则 |
| requirements | [Strokelet 首版 Demo 需求](official/requirements/strokelet-demo.md) | 1.0.0 | strokelet-demo-requirements | 用户行为、范围与验收 |
| design | [Strokelet 首版 Demo 架构](official/design/strokelet-demo.md) | 1.0.0 | strokelet-demo-architecture | 组件、数据流和安全边界 |
| specs | [Strokelet 手势识别规格](official/specs/gesture-recognition.md) | 1.0.0 | gesture-recognition | 识别状态、阈值与判据 |

以上文档为首版 Demo 的当前生效依据；草稿区材料仍是非生效版本。

## 文档分类

| 分类 | 目录 | 用途边界 |
|---|---|---|
| requirements | [official/requirements/](official/requirements/) | 跨模块、跨产品的正式需求与验收目标 |
| design | [official/design/](official/design/) | 已确认的设计决策、数据流、接口与架构取舍 |
| specs | [official/specs/](official/specs/) | 可执行规格：协议、状态机、阈值、行为细节 |
| rules | [official/rules/](official/rules/) | 全局工作规则与不可违反的协作约束 |

## 就近文档

模块级约束优先放在对应代码目录旁，本文件只索引，不搬运全文。

- 当某个代码模块出现稳定职责、接口或硬约束时，再在该模块目录下创建简短文档。不要为尚不存在的代码目录凭空建文档。
- `ARCHITECTURE.md`：该模块职责、边界、依赖方向、关键设计决策。
- `CONSTRAINTS.md`：不可违反的硬约束、常见坑、认证/数据/并发/安全规则。
- 目标长度 30–80 行，让 agent 读代码时能顺手读到约束。
- 跨模块、跨产品、正式需求和全局规则仍放在 `docs/official/`，并由本文件索引。

推荐示例（目录出现后再创建，当前不要预建）：

```text
src/api/ARCHITECTURE.md
src/db/CONSTRAINTS.md
```

当前尚无代码模块，因此没有已索引的就近文档。

## 草稿与过程材料

以下为非生效版本，不得作为开发依据：

- [docs/drafts/2026-09-25-strokelet-demo-design.md](drafts/2026-09-25-strokelet-demo-design.md) — 首版 Demo 设计（非生效版本）
- [docs/drafts/2026-09-25-strokelet-demo-plan.md](drafts/2026-09-25-strokelet-demo-plan.md) — 首版 Demo 实施计划（非生效版本）

## Graphify

`graphify-out/` 是代码库与文档库的结构导航层和可再生图谱缓存，不是需求、设计或业务规则的唯一事实源。

- 正式依据仍以 `docs/official/` 中 `status: active` 的文档为准。
- 若存在 `graphify-out/GRAPH_REPORT.md`，回答架构、模块关系、影响面、跨文件路径问题前应先读取它或使用 `graphify query`。
- 不得把图谱推断替代正式需求/设计文档。
- 完整规则见 [official/rules/graphify-workflow.md](official/rules/graphify-workflow.md)。

## 相关入口

- 变更台账：[CHANGELOG.md](CHANGELOG.md)
- Agent 地图：[../AGENTS.md](../AGENTS.md)
- 启动契约：[../INIT_CONTRACT.md](../INIT_CONTRACT.md)
- 任务分解：[../TASKS.md](../TASKS.md)
- 当前进度：[../PROGRESS.md](../PROGRESS.md)
- 设计决策：[../DECISIONS.md](../DECISIONS.md)
- 质量状态：[../QUALITY.md](../QUALITY.md)
