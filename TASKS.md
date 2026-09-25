# 任务分解

> 用途：把项目拆成有序、可验收的任务。这里外部化的是“范围表面”：任务、依赖、状态、完成证据。方案推演写入 docs/drafts/；当前执行交接写入 PROGRESS.md。

## 工作规则

- WIP 限制：任意时刻最多 1 个任务处于 `active`。
- 当前任务未拿到完成证据前，禁止启动新任务。
- 不要在实现任务 A 时“顺便”重构任务 B。
- 少做但做完优先：已验证功能数 > 代码行数；不要追求一次产出很多半成品。
- 每个功能项必须是三元组：行为描述、验证命令、当前状态。
- 状态只能是：`not_started` / `active` / `blocked` / `passing`。
- 从 `active` 到 `passing` 的唯一方式是验证命令成功；不能靠主观判断改成完成。
- agent 不能直接把功能项状态改成 `passing`；应运行验证命令并记录结果，由验证脚本/人工根据结果更新状态。
- `passing` 默认不可逆，除非后续明确新增回归失败证据。
- 完成证据必须可执行，例如测试命令、curl 行为验证、端到端验证结果；“代码看起来没问题”不算。
- 完成判定由 harness/验证命令/人工 reviewer 执行，不由生成 agent 自评决定。
- 终止标准必须按三层顺序执行：
  1. 语法与静态分析：lint / typecheck / build / 架构约束检查。
  2. 运行时行为验证：测试执行、应用启动检查、关键路径验证。
  3. 系统级确认：端到端测试、集成验证、用户场景模拟。
- 前一层没通过，不进入下一层；三层都满足后才允许声明完成。
- 单元测试通过只算第一层或局部验证；涉及跨组件、UI、API、状态传播或真实环境依赖的任务，必须有端到端确认或运行时信号。
- 跨组件修改必须跑完整流程：应用启动、关键用户路径、跨层数据流、错误传播、副作用和资源清理。
- 架构约束必须机械执行：关键依赖方向、安全边界、禁止调用等规则要有 lint/脚本/结构测试，不只写在文档里。
- 运行时反馈信号（日志、进程状态、健康检查、HTTP 响应、测试输出、副作用检查、临时资源清理结果）应记录到完成证据或最近验证结果中。
- 错误反馈必须可操作：写清楚失败点、可能原因、建议修改位置或检查命令，不要只写“Test failed”。
- 完成优先级：功能正确性 > 性能 > 风格；核心功能未通过验证前，禁止转去做重构/风格优化。
- 生成 agent 的自评不算完成证据；复杂任务应由独立 evaluator/reviewer 或客观验证命令确认。
- 每个功能项粒度应控制在单次会话可完成并验证；太粗拆分，太细合并。
- VCR（Verified Completion Rate）= `passing` 任务数 / 已启动任务数。VCR < 1.0 时，先完成已启动任务，不开新任务。
- 反向压力 = 非 `passing` 任务数。反向压力归零才表示项目任务完成。

## 最小机器可读格式（可选）

如果项目需要更强的自动化，可在 `features.json` 中维护同样信息：

```json
{
  "id": "F03",
  "behavior": "POST /cart/items with {product_id, quantity} returns 201",
  "verification": "curl -X POST http://localhost:3000/api/cart/items -H 'Content-Type: application/json' -d '{\"product_id\":1,\"quantity\":2}' | jq .status == 201",
  "state": "not_started",
  "evidence": ""
}
```

## 冲刺合同模板

每个任务开始前，先写一个轻量冲刺合同：

```markdown
# 冲刺合同: <任务名>

## 范围
- 

## 验证标准
- 

## 排除项
- 

## 运行时信号
- 应用生命周期：
- 关键路径：
- 数据流/副作用：
- 资源利用：
- 错误和异常：
```

## 评估评分标准模板

```markdown
| 维度 | A | B | C | D |
|---|---|---|---|---|
| 代码正确性 | 所有验证通过 | 主流程通过 | 部分通过 | 编译/启动失败 |
| 架构合规 | 完全合规 | 轻微偏离 | 明显偏离 | 严重违反 |
| 测试覆盖 | 主流程+边缘 | 仅主流程 | 仅骨架 | 无测试 |
| 运行时表现 | 生命周期/关键路径/副作用均正常 | 主路径正常 | 有非阻塞异常 | 关键路径失败 |
```

## 任务总览

| ID | 行为描述 | 依赖 | 状态 | 验证命令 | 确认命令/信号 | 完成证据 |
|---|---|---|---|---|---|---|
| T01 | 建立 Rust crate 骨架、锁定依赖，并提供最小可运行测试 | 无 | not_started | `make test` 或 `cargo test` | `rustc --version`；至少一个测试通过 | 测试命令输出与 lockfile |
| T02 | 将首版 Demo 设计从草稿提升为正式需求/设计文档 | T01 非硬依赖，可并行于文档会话 | not_started | 检查 `docs/index.md` 生效列表与 frontmatter | `docs/CHANGELOG.md` 有对应新增记录 | 生效文档路径 + CHANGELOG 行 |
| T03 | 建立 Makefile 标准入口（setup/init/test/lint/check） | T01 | not_started | `make init && make test && make check` | 新会话只读仓库能回答怎么跑、怎么测 | 命令输出与 `INIT_CONTRACT.md` 清单勾选 |

## T01: 建立 Rust crate 骨架与最小测试

- 目标：让后续会话能在本仓库安装工具链、锁定依赖，并跑通至少一个示例测试。
- 范围：创建最小 library+binary crate、提交 `Cargo.lock`、提供 `cargo test` 可观察通过信号。不实现手势识别或设备接管。
- 依赖：无
- 状态：not_started
- 验证命令：
  - `cargo test`
  - 若已建立 Makefile：`make test`
- 确认命令/运行时信号：
  - `rustc --version` 与 `cargo --version` 可执行
  - 至少一个具名测试从失败到通过，或新增后立即通过
- 完成证据：
  - `cargo test` 成功输出
  - `Cargo.toml` / `Cargo.lock` 已纳入 git

## T02: 提升首版 Demo 正式文档

- 目标：把 `docs/drafts/` 中的首版设计整理进 `docs/official/`，使开发有唯一生效依据。
- 范围：检索后新建或原地写入 requirements/design/specs；更新 `docs/index.md` 生效列表与 `docs/CHANGELOG.md`。不在本任务实现代码。
- 依赖：无（可与 T01 分会话进行，但任意时刻仍只能有 1 个 `active`）
- 状态：not_started
- 验证命令：
  - 确认正式文档含标准 Frontmatter，且 `status: active`
  - 确认 `docs/index.md` 列出这些文档
  - 确认 `docs/CHANGELOG.md` 追加了新增记录
- 确认命令/运行时信号：
  - 全新会话只读仓库能指出当前生效的需求/设计文档路径
- 完成证据：
  - 生效文档路径
  - CHANGELOG 对应行
  - index 生效列表已更新

## T03: 建立 Makefile 验证入口

- 目标：提供 `make setup` / `make init` / `make test` / `make check`（及必要的 lint），使初始化契约可执行。
- 范围：Makefile 或等价脚本、更新 `INIT_CONTRACT.md` 清单。不实现业务功能。
- 依赖：T01
- 状态：not_started
- 验证命令：
  - `make init`
  - `make test`
  - `make check`
- 确认命令/运行时信号：
  - 从干净环境或新会话能按 `INIT_CONTRACT.md` 完成启动检查
  - 失败时错误信息指向缺失工具或下一步，而不是空失败
- 完成证据：
  - 上述命令成功输出
  - `INIT_CONTRACT.md` 对应验收项被勾选，且与命令结果一致
