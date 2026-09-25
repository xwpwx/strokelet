# 项目进度

> 用途：跨会话保存当前工作状态。每次长任务开始前先读本文件；每次会话结束前更新本文件。
> 边界：这里只记录执行状态、阻塞、验证结果和交接；完整方案/需求/架构推演写入 docs/drafts/ 或 docs/official/，并在“相关文档”中链接。

## 当前状态

- 最新 commit: `8562bcb`；手势方向、配置和设置窗口尚未提交
- 测试状态: 设置窗口的自动测试已通过；`make check` 见最近验证结果
- Lint: clippy `-D warnings` 已通过
- 类型检查: 同上
- 完整验证: 窗口脚本能被 gjs 加载；没有显示器时停在 Gtk “Failed to open display”。桌面点击尚未做
- Graphify 图谱: 未建立。本机没有 graphify 命令

## 当前目标

- T01–T08 已有可执行证据。T09 实机验收为 `blocked`。T10 设置窗口正在做，自动测试已过，等用户打开窗口。

## 相关文档

- 生效依据：`docs/index.md` 所列的首版需求、架构、手势识别规格和 Graphify 规则
- 草稿/方案：`docs/drafts/2026-09-25-strokelet-demo-design.md`、`docs/drafts/2026-09-25-strokelet-demo-plan.md`、`docs/drafts/2026-09-25-strokelet-gesture-gui-plan.md`（均非生效版本）
- 重要决策：DECISIONS.md
- 图谱导航：graphify-out/GRAPH_REPORT.md（若存在）

## 已完成

- [x] 建立 docs/ 目录骨架与权威分层
- [x] 建立 docs/index.md 与唯一变更台账 docs/CHANGELOG.md
- [x] 建立 Graphify 正式规则文档
- [x] 建立 AGENTS.md / CLAUDE.md 与状态文件
- [x] T01：Rust crate 骨架与最小测试，独立评估通过
- [x] T03：标准 Makefile 入口，独立评估通过
- [x] T02：首版正式需求、架构和识别规格，独立评估通过
- [x] T04：纯手势识别状态机，`make check` 通过后记为 passing
- [x] T05–T08：门禁、协议、输入序列、扩展安装与 GJS socket

## 进行中

- [ ] T09：实机验收（blocked，等待用户）
- [ ] T10：手势与快捷键设置窗口（active；自动测试已过，窗口还没在桌面点过）

## 已知问题

- `make setup` 只在当前机器验证；全新机器预装 Rust 的步骤尚未验证。
- grab 前没有读取物理按键状态；`grab_allowed` 的按键参数目前固定为未按下。
- 复制门禁里的权限观察在前台循环中固定为已知且通过，没有单独的权限探测。
- GNOME Shell 内的轨迹、焦点和菜单尚未实机运行。St 不能在 Shell 外实例化。

## 阻塞项

| 阻塞项 | 原因 | 需要谁处理 | 下一步 |
|---|---|---|---|
| 全新机器初始化未验证 | 尚无干净机器验证 Rust 安装前提 | 后续维护者 | 具备全新环境时运行 `make init`、`make setup`、`make check` |
| 实机设备与覆盖层尚未验证 | 当前会话不能操作真实鼠标，也不能在 Shell 外创建 St | 用户 | 按 `docs/acceptance/demo-checklist.md` 做一次上划和粘贴 |

## 下一步

1. 新会话先读 AGENTS.md、docs/index.md、INIT_CONTRACT.md、TASKS.md、本文件与 DECISIONS.md。
2. 让用户打开 `./target/debug/strokelet settings`，保存一条规则，并确认正在运行的演示打出 `reloaded gesture rules`。不要把 T10 标成 passing，除非这一步真的发生。
3. 不要把 T09 标成 passing。请用户按 `docs/acceptance/demo-checklist.md` 在 Wayland 桌面做一次上划和粘贴。

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
| 2026-09-25 | `gnome-shell --version`、`lsb_release -ds`、`echo "$XDG_SESSION_TYPE"` | 通过 | GNOME Shell 50.1、Ubuntu 26.04.1 LTS、Wayland；仅环境事实，不代表功能验收 |
| 2026-09-25 | `python3` + PyYAML 校验 official Frontmatter/topic/index/CHANGELOG/链接；`make check` | 通过 | 4 份 active 文档元数据和链接一致；1 个 Rust 测试通过 |
| 2026-09-25 | 独立评估：official Frontmatter/topic/index/CHANGELOG/链接与 `make check` | 通过 | T02：3 份新增 active 文档，无重大遗漏 |
| 2026-09-25 | `make test`（实现前） | 按预期失败 | `tests/gesture_cases.rs` 找不到 `Decision` / `Gesture` / `Limits` |
| 2026-09-25 | `cargo fmt --all` 后 `make check` | 通过 | 25 个手势测试 + 1 个启动测试；fmt / clippy / typecheck 成功 |
| 2026-09-25 | `make check` | 通过 | 补上绘制超时提前进入取消；30 个手势测试 + 1 个启动测试 |
| 2026-09-25 | `make test`（门禁实现前） | 按预期失败 | `tests/copy_gate_cases.rs` 找不到 `CopyGate` 等类型 |
| 2026-09-25 | `make check` | 通过 | T05：6 个门禁测试 + 既有 31 个测试；fmt / clippy / typecheck 成功 |
| 2026-09-25 | `make check` | 通过 | 58 个测试：手势 30、门禁 6、输入 8、协议 13、启动 1 |
| 2026-09-25 | `gjs -m scripts/check-gjs-socket.js` | 通过 | 打印 `strokelet: gjs unix socket round-trip ok` |
| 2026-09-25 | `scripts/check-extension-install.sh` | 通过 | 外来目录拒绝安装和删除；非 Wayland 拒绝 `run-demo.sh` |
| 2026-09-25 | `make check` | 通过 | 67 个测试：配置 6、门禁 6、手势 31、输入 9、协议 14、启动 1 |
| 2026-09-25 | `gjs -m settings/app.js`（无显示器） | 按预期停住 | Gtk 报 Failed to open display；脚本已加载。窗口点击未做 |

## Graphify 图谱状态

- graphify-out 是否存在：否
- 最近更新命令：未运行
- 最近图谱健康：未检查
- 需要重建或更新的原因：代码已超过 3 个模块，但本会话未运行 graphify；若本机没有该命令，保持未建立

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

### 2026-09-25：T02 正式文档

- 本次做了什么：从非生效设计草稿整理需求、架构和识别规格三份正式文档，并更新索引与唯一变更台账。
- 验证信号：目标环境版本已核实；正式文档的 Frontmatter、索引、台账和交叉链接已本地检查通过，待独立复核。
- 未完成事项：T02 已独立验收；T04 纯识别状态机已启动，设备与轨迹仍未实现。
- 图谱：仅文档内容新增，当前仍无稳定多模块代码结构，图谱未受影响。

### 2026-09-25：T04 纯识别状态机

- 本次做了什么：补全可回放手势测试，并实现 `src/gesture.rs`。释放时返回一次 RightClick、Copy 或 Cancel；重复释放返回 None。
- 验证信号：实现前测试因类型缺失失败；实现后 `make check` 通过。复查时补上绘制超时：超过 2500 ms 立即离开 Drawing，倒退时间戳不能恢复，未进入绘制的长按仍是普通右击。当前 30 个具名手势测试通过。
- 未完成事项：T04 仍待独立验收，不能由本次实现会话改为 `passing`。设备、轨迹和复制注入尚未开始。本次改动未提交。
- 下次开工建议：先独立验收 T04，确认完成后再选下一任务。
- 图谱：仍只有一个识别模块，未形成 3 个以上核心模块，图谱未受影响。

### 2026-09-25：T04 收口并开始 T05

- 本次做了什么：按已通过的 `make check` 将 T04 记为 `passing`，并实现复制注入门禁。Copy 只有在会话、扩展、暂停、修饰键、通信和权限都确认安全时才返回一次 `CopyOnce`。
- 验证信号：门禁测试先因类型缺失失败；实现后 `make check` 通过，6 个具名测试覆盖重复注入、失败后重试、非 Copy 决策、IPC 请求，以及未知、超时和不安全桌面状态。
- 未完成事项：T05 仍待复核后才能改为 `passing`。协议、设备和轨迹尚未开始。本次改动未提交。
- 下次开工建议：先复核 T05，再选下一任务。
- 图谱：识别与门禁仍不足 3 个核心模块，图谱未受影响。

### 2026-09-25：实施计划中的可自动部分

- 本次做了什么：补完协议、输入帧、前台 `run` 循环、GNOME 50 扩展和安装/卸载/演示脚本。Begin/End/Cancel 写入客户端 socket。暂停时解除 grab。进程退出删除自己的 socket。正式架构把运行时目录改为 0770。
- 验证信号：`make check` 58 个测试通过。GJS socket 往返和扩展安装脚本通过。
- 未完成事项：T09 等待用户实机验证。grab 前未读物理按键；权限观察在循环里固定为通过。本次改动未提交。
- 下次开工建议：用户按验收清单操作真实鼠标。不要把模拟测试写成实机通过。
- 图谱：模块已超过 3 个，本会话未建立图谱。

### 2026-09-25：设置窗口

- 本次做了什么：直线识别扩成上下左右；快捷键改成可配置和弦；新增 `strokelet settings` 和 `~/.config/strokelet/gestures.json`。运行中的演示用单独连接上的 `reload` 重新读配置。
- 验证信号：clippy 通过。配置往返、未知键、重复方向和 reload 行有测试。`gjs -m settings/app.js` 在没有显示器时停在 Gtk 打不开显示，说明脚本能加载。
- 未完成事项：窗口没有在桌面上点过。T09 仍 blocked。本次改动未提交。
- 下次开工建议：用户先 Ctrl+C 停掉旧进程，用新二进制启动，再开设置窗口。
- 图谱：未建立。`command -v graphify` 为空，图谱未受影响是因为命令不存在，不是因为结构没变。
