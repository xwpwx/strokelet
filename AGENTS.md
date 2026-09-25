# 项目文档协作规则

你是本项目的文档协作助手。处理项目内所有 Markdown 文档时，必须严格遵守以下规则，违反任意一条均视为输出错误。

## 项目概览

- 目的：Strokelet 在 Ubuntu / GNOME Wayland 上用右键直线上划触发一次 Ctrl+C，并显示不抢焦点的全局轨迹；普通右击仍可用。
- 技术栈（意图，尚未锁定）：Rust stable + evdev/uinput + Unix socket；GNOME Shell 扩展（GJS）。关键版本以实现时核实并写入 lockfile 为准。
- 启动/验证：见 `INIT_CONTRACT.md`。当前尚无 `Makefile`；建立前不要假装 `make setup/test/check` 可用。
- 生效文档入口：`docs/index.md`。变更历史只看 `docs/CHANGELOG.md`。
- 进度/决策/质量：`PROGRESS.md`、`DECISIONS.md`、`QUALITY.md`。任务范围：`TASKS.md`。

## 硬约束

1. 只允许引用 `docs/official/` 中 `status: active` 的文档作为开发、设计、需求依据。
2. 先检索后修改：同主题正式文档必须原地增量更新，禁止新建同主题文件、禁止文件名版本（`*-v2.md`）。
3. 修改正式文档必须同步更新 frontmatter 的 `version` / `updated`，并在 `docs/CHANGELOG.md` 追加一条记录。`docs/index.md` 与文档 frontmatter 都不维护 changelog。
4. 每次修改后必须用可观察命令验证；不能只凭“看起来完成”宣告完成。
5. 禁止 `git add .` / `git add -A`。一次 commit 对应一个完整逻辑操作。
6. 错误复盘禁止默认追加进本文件；优先写成 lint、测试、类型约束、模块 `CONSTRAINTS.md` / `ARCHITECTURE.md` 或 `docs/official/rules/`。

## 1. 文档权威优先级

1. `docs/official/`：唯一真相源。
2. `docs/drafts/`：草稿。引用必须标注“非生效版本”。
3. `docs/archive/`：已失效，仅供回溯。

## 2. AGENTS.md 的定位

本文件是地图，不是百科全书。只保留稳定入口、强制规则和验证方式。详细设计、需求、规则和历史必须放在 `docs/` 或代码旁就近文档，从 `docs/index.md` 导航。仓库是唯一事实来源：agent 看不到的内容视为不存在。硬约束、软建议、历史教训必须分层，不要混在同一层级。本文件禁止无限膨胀。

## 3. 知识可见性与就近文档

1. 全局规则、跨模块需求、正式设计放 `docs/official/`，由 `docs/index.md` 索引。
2. 模块职责、接口、局部硬约束放代码旁 `ARCHITECTURE.md` / `CONSTRAINTS.md`（30–80 行）；目录尚未出现时不要预建。
3. `docs/index.md` 只做地图和当前生效列表，不复制局部文档全文，不写变更历史。
4. 聊天记录、人脑记忆、外部零散文档如果未写入仓库，视为不存在。

Graphify：`graphify-out/` 是结构导航层，不是事实源。架构/模块/调用链/影响面问题若图谱存在，先读 `graphify-out/GRAPH_REPORT.md` 或运行 `graphify query`。`INFERRED` / `AMBIGUOUS` 只是线索。完整规则见 `docs/official/rules/graphify-workflow.md`。空仓库不强制建图；出现稳定代码结构、≥3 个核心模块或需要影响面分析时再 `/graphify .`。结构性/跨模块/公共接口/目录变化或阶段交接时才 `graphify update .`；小改动在 `PROGRESS.md` 标记“图谱未受影响”。与正式文档冲突时以 `docs/official/` 为准。

全新会话只读仓库应能在 3 分钟内回答：项目目的、技术栈、启动/测试/lint/完整验证命令、生效文档位置、进度/阻塞/决策入口。答不上来就补地图，不要猜。

## 4. Harness 六个子系统

1. 指令：本文件与 `CLAUDE.md`。
2. 工具：按最小权限开放必要工具；不要禁掉 shell、包管理器、测试工具和 git。
3. 环境：必须自描述、可复现。锁定依赖与运行时，并提供独立初始化入口（目标：`make init` 或 `./init.sh`）。初始化只检查环境，不做业务功能。
4. 状态：开工先读 `PROGRESS.md` 与 `DECISIONS.md`，收尾必须更新 `PROGRESS.md`；重要决策写入 `DECISIONS.md`。`PROGRESS.md` 只记录执行状态，不写完整方案。逻辑操作用 git commit 原子化；提交前通过验证；多 agent 用独立分支或独立进度文件隔离。
5. 反馈：必须显式列出验证命令。
6. 图谱：见第 3 节与 `docs/official/rules/graphify-workflow.md`。

当前验证命令尚未建立。建立后应能运行测试、lint 和完整验证，并在结构性变化时更新图谱。在此之前，以 `INIT_CONTRACT.md` 的阻塞记录为准，禁止伪造通过结果。

## 5. 文档更新铁律

接到文档修改或需求变更时：先在 `docs/official/` 检索同主题文档；存在则原地增量修改。修改时同步：按语义化版本更新 `version`、把 `updated` 改为当天、在 `docs/CHANGELOG.md` 追加一条（日期、路径、版本、类型、一句话说明）。只改涉及章节，不得擅自重构全文。

- patch：错别字/格式/链接/轻微表述。
- minor：新增章节或补充规则，不推翻原结构。
- major：需求方向、信息架构或约束不兼容变化。

## 6. 同主题判定

先匹配 frontmatter `topic`，再匹配 title/文件名/子目录，再看正文核心对象。版本相同但内容冲突时以较晚 `updated` 为准；仍无法判断则暂停并请求人工确认。

## 7. 新建正式文档规范

确认无同主题后再放入 `docs/official/{requirements,design,specs,rules}/`，并包含：

```yaml
---
title: 文档全称
status: active
version: 1.0.0
updated: YYYY-MM-DD
owner: 负责人
topic: 主题标识
---
```

新建或废弃：更新 `docs/index.md` 当前生效列表，并在 `docs/CHANGELOG.md` 追加记录。仅修改内容：更新 `version` / `updated` 并追加 CHANGELOG；只有标题、路径、分类或生效状态变化时才改 `docs/index.md`。正式文档 frontmatter 不设 changelog 字段。

## 8. 草稿文档规范

未确认内容一律进 `docs/drafts/`，文件名加日期前缀，正文开头标注：

【草稿】本文档非生效版本，不作为开发、设计、需求依据。

## 9. 冲突与废弃

以 `docs/official/` 中版本最新且 `status: active` 的文档为准；版本相同看 `updated`。废弃四步：正文加废弃声明并指向最新生效文档 → `status: deprecated` → 移到 `docs/archive/` 且文件名加 `deprecated-` 前缀 → `docs/CHANGELOG.md` 追加废弃记录。

## 10. 遗留文档处理

缺 Frontmatter 的 Markdown 不得当作生效依据。先判断归属：正式文档补齐 Frontmatter（owner 不明写 `待确认`）；草稿或历史材料移到 `docs/drafts/` 或 `docs/archive/` 并补充声明。不得跳过。

## 11. 绝对禁止行为

- 用文件名区分版本，或在 `docs/official/` 放草稿。
- 修改正式文档却不更新 `version`、`updated` 和 `docs/CHANGELOG.md`。
- 未经检索新建同主题文档。
- 把 `docs/drafts/` 或 `docs/archive/` 当作开发依据。
- 使用 `git add .` 或 `git add -A`。

## 12. 验证与提交

完成后检查：正式文档 Frontmatter 完整；`status` 仅为 `active` 或 `deprecated`；`version` / `updated` 与 `docs/CHANGELOG.md` 已留痕；新建/废弃已更新 `docs/index.md`；index 未混入变更历史；无同主题重复。

提交：显式 `git add` 本次文件；一个 commit 一个完整逻辑操作；相关验证失败不得提交，除非用户要求且标明 WIP。消息格式：`docs(<范围>): <一句话说明> [vX.Y.Z]`。初始化例外：`docs: 初始化项目 harness（docs + AGENTS/CLAUDE + 变更台账 + graphify + INIT/TASKS + PROGRESS/DECISIONS/QUALITY） [v1.0.0]`。

## 功能清单规则

- 功能清单：`TASKS.md`（可选机器版本：`features.json`）。
- 任意时刻最多 1 个 `active` 任务；未拿到完成证据前禁止开新任务。
- 功能项是三元组：行为描述、验证命令、当前状态（`not_started` / `active` / `blocked` / `passing`）。
- 不要手动把状态改为 `passing`；由验证脚本/人工根据验证结果更新。`passing` 默认不可逆，除非有回归失败证据。
- 完成证据必须可执行；生成 agent 自评不算。粒度应是一轮会话能完成并验证的行为单元。
- VCR = `passing` 数 / 已启动数。VCR < 1.0 时先完成已启动任务。反向压力 = 非 `passing` 数，归零才表示任务完成。

## 自动循环角色分离

- Supervisor：读功能清单、状态和验证结果，决定下一个可执行项。
- Generator：只实现当前 `active` 项，不给自己判定 `passing`。
- Evaluator：独立 session，按验证命令、运行时信号和评分标准挑错。
- `active → passing` 只能由验证成功触发，由 supervisor/harness 更新。复杂任务可使用 adversarial evaluator；多数否决时不得合并或标记完成。

## 每次会话开始时（上班）

### 阶段 0：初始化检查

只建立执行前提，不做业务功能。必须确认：可运行环境、可验证测试框架、`INIT_CONTRACT.md`、`TASKS.md`、干净 git checkpoint。

1. 读本文件、`INIT_CONTRACT.md`、`TASKS.md`、`PROGRESS.md`、`DECISIONS.md`。
2. 若涉及架构/模块/调用路径/影响面且图谱存在，先读 `graphify-out/GRAPH_REPORT.md` 或 `graphify query`。
3. 运行项目定义的初始化与验证命令。当前入口未建立则先补 harness，不要开始功能实现。
4. 检查：能看进度、能接手下一步。记录“从开始到第一次测试通过”的时间；目标是后续全新会话 3 分钟内恢复到可执行状态。

### 阶段 1：任务执行

从 `PROGRESS.md` 的“下一步”继续。开始前确认完成定义和验证方式。发现环境/工具/验证/文档入口缺失，先补 harness。

短任务（约 30 分钟内且上下文占用低）可在同一会话完成；长任务或上下文超过窗口约 60% 时必须准备交接。

## 每次会话结束前（下班）

1. 更新 `PROGRESS.md`；若有重要决策，更新 `DECISIONS.md`。
2. 结构性变化且已有图谱则 `graphify update .`；小改动标记“图谱未受影响”；应更新但无法运行则写入最近验证结果。
3. 跑已有验证命令。退出检查六个维度：构建、测试、进度、图谱、工件、启动。
4. 幂等清理临时文件/调试日志/一次性脚本。模块或图谱健康变化则更新 `QUALITY.md`。
5. 只对已完成的原子工作单元提交；未完成工作不要伪装成完成。

定期维护：即时清理每个会话做；每周/每月扫描结构性问题与质量漂移。清理脚本必须幂等。高吞吐量合并仅在验证体系足够强且审查带宽不足时适用。
