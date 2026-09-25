# 项目进度

> 用途：跨会话保存当前工作状态。每次长任务开始前先读本文件；每次会话结束前更新本文件。
> 边界：这里只记录执行状态、阻塞、验证结果和交接；完整方案/需求/架构推演写入 docs/drafts/ 或 docs/official/，并在“相关文档”中链接。

## 当前状态

- 最新 commit: 待首次提交
- 测试状态: 未建立
- Lint: 未建立
- 类型检查: 未建立
- 完整验证: 未建立
- Graphify 图谱: 未建立

## 当前目标

- 完成文档治理 harness 初始化，使后续会话能从仓库地图接手工作。
- 下一步进入 `TASKS.md` 的 T01（工程骨架）或 T02（正式文档提升），任意时刻只激活一个。

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

## 进行中

- [ ] 无业务任务处于 active

## 已知问题

- 仓库尚无源代码、测试、Makefile 或锁定依赖。
- 本机可能未安装 Rust 工具链；T01 开始前需先确认 `cargo` / `rustc`。
- 首版产品设计仍在草稿区，还不是开发依据。

## 阻塞项

| 阻塞项 | 原因 | 需要谁处理 | 下一步 |
|---|---|---|---|
| 可执行验证入口缺失 | 尚无 crate / Makefile / 测试 | 后续 agent 或维护者 | 启动 T01，建立最小 `cargo test` |
| 无生效需求/设计文档 | 仅有 drafts | 后续文档会话 | 启动 T02，提升正式文档 |

## 下一步

1. 提交本次 harness 初始化。
2. 新会话先读 AGENTS.md、docs/index.md、INIT_CONTRACT.md、TASKS.md、本文件与 DECISIONS.md。
3. 在 T01 与 T02 中选择一个设为 `active`；未拿到完成证据前不要并行开工。

## 最近验证结果

| 时间 | 命令 | 结果 | 备注 |
|---|---|---|---|
| 2026-09-25 | 文档结构与 Frontmatter 人工检查 | 通过 | 仅 harness 文件；无 make/cargo 可跑 |
| 2026-09-25 | graphify | 不适用 | 空仓库初始化，图谱未建立 |

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
- 下次开工建议：先完成 git 初始化提交（若本会话已提交则跳过），再按 TASKS.md 只激活 T01 或 T02。
