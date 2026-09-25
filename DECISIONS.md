# 设计决策日志

> 用途：记录重要设计决策及原因。只记录需要跨会话保留的关键“为什么”；完整方案请写入 docs/drafts/ 或 docs/official/。

## 记录格式

### YYYY-MM-DD: 决策标题

- 决策：
- 原因：
- 否决方案：
- 约束/影响：
- 相关文档：

## 记录

### 2026-09-25: 采用全局索引 + 就近文档的混合文档结构

- 决策：`docs/index.md` + `docs/official/` 负责全局导航和权威状态；模块级 `ARCHITECTURE.md` / `CONSTRAINTS.md` 只在对应代码目录出现稳定职责后就近创建。
- 原因：AGENTS.md 需要保持短地图；agent 读代码时应顺手看到局部硬约束，同时跨模块规则仍有唯一入口。
- 否决方案：把所有模块说明写进 AGENTS.md；或在尚无代码时预建 `src/api/` 等空文档。
- 约束/影响：没有代码目录就不要创建就近文档；index 只索引不搬运全文。
- 相关文档：docs/index.md、AGENTS.md

### 2026-09-25: docs/CHANGELOG.md 作为唯一变更台账

- 决策：正式文档变更历史只追加到 `docs/CHANGELOG.md`；`docs/index.md` 只保留当前生效列表；文档 frontmatter 不设 changelog 字段。
- 原因：index 每次会话都要读，必须精简；同一变更不应在多个位置重复维护。
- 否决方案：在每篇正式文档内维护 changelog；或把历史写进 index。
- 约束/影响：每次新建/修改/废弃 official 文档都必须追加一条台账记录，并更新 version/updated。
- 相关文档：docs/CHANGELOG.md、docs/index.md

### 2026-09-25: CLAUDE.md 引用 AGENTS.md

- 决策：CLAUDE.md 只作为入口指针，完整规则以 AGENTS.md 为准。
- 原因：两份全文会迅速漂移；当前工具链都能先读 AGENTS.md。
- 否决方案：维护两份完全相同的正文。
- 约束/影响：改规则只改 AGENTS.md；CLAUDE.md 仅在入口顺序变化时更新。
- 相关文档：AGENTS.md、CLAUDE.md

### 2026-09-25: graphify-out 不入库，但 worktree 共享

- 决策：`graphify-out/` 写入 `.gitignore`，同时写入 `.worktreeinclude`。
- 原因：图谱是可再生导航缓存，不是需求事实源；各 worktree 仍需要同一份本地缓存才能导航。
- 否决方案：把 graphify-out 作为正式文档提交进 git。
- 约束/影响：空仓库不强制建图；与 official 文档冲突时以 official 为准。
- 相关文档：docs/official/rules/graphify-workflow.md

### 2026-09-25: 遗留 superpowers 文档降为草稿

- 决策：将 `docs/superpowers/` 下缺 Frontmatter 的设计与计划移到 `docs/drafts/`，并标注非生效版本。
- 原因：它们是过程材料，未经正式确认，不能直接成为开发依据。
- 否决方案：补 Frontmatter 后立即升为 official；或留在原路径不管。
- 约束/影响：实现代码前应先完成 TASKS.md T02，或明确在任务中引用这些草稿并标注“非生效版本”。
- 相关文档：docs/drafts/2026-09-25-strokelet-demo-design.md、docs/drafts/2026-09-25-strokelet-demo-plan.md、TASKS.md

### 2026-09-25: 运行时目录使用 0770

- 决策：`/run/strokelet/<uid>/` 由 root 持有、组为目标用户主组、权限 0770；socket 仍为 0600 且属主为目标 UID。
- 原因：0750 让目标用户无法在该目录里绑定 socket。组写权限只给目标用户主组，socket 本身不对组开放。
- 否决方案：保持文档中的 0750，或让进程以 root 绑定后再降权。
- 约束/影响：正式架构文档已改为 1.1.0。实现与路径测试使用 0770。
- 相关文档：docs/official/design/strokelet-demo.md

### 2026-09-25: 复制前要求没有物理修饰键

- 决策：扩展上报的修饰键列表必须为空，才允许注入 Ctrl+C。
- 原因：实施计划要求上划复制发生在没有按住物理修饰键时，避免和用户正在按的组合键叠在一起。
- 否决方案：只要求修饰键状态已知，不要求为空。
- 约束/影响：正式设计只要求状态已知且未超时；空列表是实现选择，写在 `Link::desktop`。
- 相关文档：docs/drafts/2026-09-25-strokelet-demo-plan.md（非生效版本）、src/protocol.rs

### 2026-09-25: Demo 前台默认 120 秒

- 决策：`run` 与 `run-demo.sh` 默认 120 秒后退出，不安装 systemd 服务。
- 原因：实施计划把演示限定为一次前台运行。
- 否决方案：常驻服务，或把 120 秒写进正式需求作为硬约束。
- 约束/影响：超时只是演示入口默认值。退出时解除 grab、释放本进程注入的按键并删除自己的 socket。
- 相关文档：docs/acceptance/demo-checklist.md

### 2026-09-25: 用手势设置窗口覆盖首版“无设置界面”

- 决策：增加 `strokelet settings`。用户选择一个触发键，并为上、下、左、右各保存最多一条快捷键。配置在 `~/.config/strokelet/gestures.json`。正在运行的演示通过一条 `reload` 消息重新读取，不换掉扩展连接。默认仍是右键上划 Ctrl+C。
- 原因：用户确认了 `docs/drafts/2026-09-25-strokelet-gesture-gui-plan.md`，要求先能自己改鼠标直线动作和模拟按键。
- 否决方案：用 Rust 直接链接 GTK。本机有 GTK4 和 libadwaita 运行库，没有对应的开发头文件，安装被拦住了。窗口改由已安装的 GJS 类型库打开。
- 约束/影响：正式需求仍写着首版没有设置界面。这次是用户确认后的范围覆盖，草稿没有升格为正式文档。没有配置文件时用内置默认；文件损坏则拒绝启动，重新加载失败则保留上一份规则。
- 相关文档：docs/drafts/2026-09-25-strokelet-gesture-gui-plan.md（非生效版本）
