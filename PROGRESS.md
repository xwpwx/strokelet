# 项目进度

> 用途：跨会话保存当前工作状态。每次长任务开始前先读本文件；每次会话结束前更新本文件。
> 边界：这里只记录执行状态、阻塞、验证结果和交接；完整方案/需求/架构推演写入 docs/drafts/ 或 docs/official/，并在“相关文档”中链接。

## 当前状态

- 最新 commit: 运行 `git rev-parse HEAD` 查询
- 测试状态: `make test` 已有具名测试通过
- Lint: `make lint` 通过
- 类型检查: `make check` 包含 `cargo check --locked --offline`
- 完整验证: `make check` 通过
- Graphify 图谱: 未建立

## 当前目标

- T01、T03 已经独立验收；当前执行 `TASKS.md` T02，整理首版正式需求/设计文档。

## 相关文档

- 生效依据：`docs/official/rules/graphify-workflow.md`
- 草稿/方案：`docs/drafts/2026-09-25-strokelet-demo-design.md`、`docs/drafts/2026-09-25-strokelet-demo-plan.md`（非生效版本）
- 重要决策：DECISIONS.md
- 图谱导航：graphify-out/GRAPH_REPORT.md（若存在）

## 已完成

- [x] 建立 docs/ 目录骨架与权威分层
- [x] 建立 docs/index.md 与唯一变更台账 docs/CHANGELOG.md
- [x] 建立 Graphify 正式规则文档
- [x] 建立 AGENTS.md / CLAUDE.md 与状态文件
- [x] T01：Rust crate 骨架与最小测试，独立评估通过
- [x] T03：标准 Makefile 入口，独立评估通过

## 进行中

- [ ] T02：首版 Demo 正式需求/设计文档

## 已知问题

- `make setup` 只在当前机器验证；全新机器预装 Rust 的步骤尚未验证。
- 首版产品设计仍在草稿区，还不是开发依据。

## 阻塞项

| 阻塞项 | 原因 | 需要谁处理 | 下一步 |
|---|---|---|---|
| 全新机器初始化未验证 | 尚无干净机器验证 Rust 安装前提 | 后续维护者 | 具备全新环境时运行 `make init`、`make setup`、`make check` |
| 无生效需求/设计文档 | 仅有 drafts | 后续文档会话 | 启动 T02，提升正式文档 |

## 下一步

1. 新会话先读 AGENTS.md、docs/index.md、INIT_CONTRACT.md、TASKS.md、本文件与 DECISIONS.md。
2. 完成 T02 的正式文档、索引与变更台账，并验证 Frontmatter。
3. T02 独立验收后，再按正式需求拆分业务任务；未拿到完成证据前不要并行开工。

## 最近验证结果

| 时间 | 命令 | 结果 | 备注 |
|---|---|---|---|
| 2026-09-25 | 文档结构与 Frontmatter 人工检查 | 通过 | 仅 harness 文件；无 make/cargo 可跑 |
| 2026-09-25 | graphify | 不适用 | 空仓库初始化，图谱未建立 |
| 2026-09-25 | `PATH="$HOME/.cargo/bin:$PATH" cargo test --offline` | 通过 | 1 个具名启动测试；先观察到失败，再通过 |
| 2026-09-25 | 独立评估：`cargo fmt`、`cargo clippy`、`cargo check`、`cargo test`、`cargo run`（锁定/离线） | 通过 | T01：1 个具名测试，binary 输出 `strokelet: scaffold only`，锁文件已跟踪 |
| 2026-09-25 | `make setup`；`make init && make test && make check` | 通过 | 当前机器；1 个具名测试，fmt / clippy / typecheck 成功 |
| 2026-09-25 | `HOME=/tmp/strokelet-missing-rust PATH=/usr/bin:/bin make init` | 按预期失败 | 明确提示缺少 rustc 和安装入口 |
| 2026-09-25 | 独立评估：`make init`、`make setup`、`make test`、`make check` | 通过 | T03：1 个具名测试；缺 Rust 提示有效；全新机器未实测 |

## Graphify 图谱状态

- graphify-out 是否存在：否
- 最近更新命令：未运行
- 最近图谱健康：未检查
- 需要重建或更新的原因：项目尚未出现稳定代码结构或 3 个以上核心模块

## 决策摘要

| 决策 | 为什么这样做 | 放弃的方案 | 影响范围 |
|---|---|---|---|
| CLAUDE.md 引用 AGENTS.md | 避免两份规则漂移 | 复制全文 | 后续只改 AGENTS.md |
| CHANGELOG 作为唯一台账 | 保持 index 精简 | 在每篇文档维护 changelog | 追溯只查 docs/CHANGELOG.md |
| 遗留 superpowers 文档降为草稿 | 缺 Frontmatter，且未经正式确认 | 直接当作 official | 开发不得引用其为唯一依据 |

> 注：这里只记录影响当前执行的关键决策摘要；重要设计决策的完整“为什么”请写入 DECISIONS.md；完整方案或推演请写入 docs/drafts/ 或 docs/official/，并在“相关文档”中链接。

## 会话交接记录

### 2026-09-25

- 本次做了什么：初始化文档治理体系、agent 地图、变更台账、Graphify 规则、初始化契约与状态文件；将已有 superpowers 文档移入 drafts。
- 重要发现：仓库原先不是 git 仓库；除两份过程文档外无代码。
- 未完成事项：Rust 工程骨架、正式需求/设计文档、Makefile 验证入口、首次之外的业务实现。
- 下次开工建议：按 TASKS.md 只激活 T01 或 T02。

### 2026-09-25：T01 工程骨架

- 本次做了什么：建立最小 Rust library + binary、工具链选择器、Cargo lockfile 和具名启动测试。
- 验证信号：先观察到启动测试失败（stdout 为空），实现后测试通过；`cargo check --offline` 与 Clippy 通过。
- 未完成事项：T01 仍待独立验收并由 supervisor/harness/人工 reviewer 更新状态；T03 标准 Makefile 入口尚未建立。
- 下次开工建议：先独立验收 T01 的测试与锁文件；确认完成后再选下一任务。
- 图谱：目前仅有最小 crate，未形成稳定多模块结构，不需要建立图谱。

### 2026-09-25：T03 标准入口

- 本次做了什么：加入 Makefile 的 setup/init/test/lint/check，并更新启动契约和 README。
- 验证信号：当前机器全部标准入口通过；模拟缺少 Rust 时 `make init` 给出明确提示。
- 未完成事项：全新机器的 Rust 安装前提尚未实测；T02 正式需求/设计文档待整理。
- 图谱：未改变代码模块或依赖关系，图谱未受影响。
