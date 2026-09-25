---
title: Graphify 代码图谱工作流
status: active
version: 1.0.0
updated: 2026-09-25
owner: 待确认
topic: graphify-workflow
---

# Graphify 代码图谱工作流

## 定位

- `graphify-out/` 是代码库/文档库的结构导航层，用于帮助 agent 理解模块关系、关键节点、社区结构、跨文件路径和潜在影响面。
- `docs/official/` 仍是需求、设计、规则的唯一真相源；graphify 输出不得替代正式文档。
- 图谱中的 `INFERRED` / `AMBIGUOUS` 关系只能作为调查线索，不能直接作为结论。

## 使用时机

- 空仓库初始化阶段不强制生成 graphify-out；当项目已经出现稳定代码结构、已有 3 个以上核心模块，或需要做架构/影响面分析时，再运行 `/graphify .` 建立初始 baseline。
- 回答架构、模块职责、依赖方向、调用路径、影响面问题前，若存在 `graphify-out/GRAPH_REPORT.md` 或 `graphify-out/graph.json`，优先使用图谱。
- 跨模块代码修改前，优先用 `graphify query` 或阅读 `GRAPH_REPORT.md` 判断影响范围。
- 只有当本次修改影响模块结构、跨文件依赖、公共接口、路由/数据模型、目录结构，或准备阶段性交接/合并时，才需要运行 `graphify update .`；局部 bugfix、文案、注释、测试断言等小改动可标记为“图谱未受影响”。

## 输出管理

- `graphify-out/GRAPH_REPORT.md`：可读摘要，用于快速理解关键节点和社区结构。
- `graphify-out/graph.json`：机器可查询图谱，用于 `graphify query/path/explain`。
- `graphify-out/graph.html`：本地可视化查看，不作为正式文档依据。
- `graphify-out/` 是可再生导航缓存，默认不提交 git：必须把 `graphify-out/` 写入 `.gitignore`。
- 为保证每个 worktree 都能共享同一份图谱缓存，必须同时把 `graphify-out/` 写入 `.worktreeinclude`，让 worktree 创建/同步时携带该目录。
- 若项目确有需要提交图谱缓存，需显式移除对应 `.gitignore` 条目并在 PROGRESS.md 记录原因，同时标注其为“可再生导航缓存”。

## 验证规则

- 涉及跨模块结构变化的任务，完成证据中应包含 graphify 更新结果或说明为何不适用。
- 若 graphify 输出与 docs/official/ 冲突，以 docs/official/ 为准，并创建任务修正图谱或文档索引。
