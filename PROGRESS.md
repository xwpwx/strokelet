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


| 阻塞项          | 原因                             | 需要谁处理 | 下一步                                             |
| ------------ | ------------------------------ | ----- | ----------------------------------------------- |
| 全新机器初始化未验证   | 尚无干净机器验证 Rust 安装前提             | 后续维护者 | 具备全新环境时运行 `make init`、`make setup`、`make check` |
| 实机设备与覆盖层尚未验证 | 当前会话不能操作真实鼠标，也不能在 Shell 外创建 St | 用户    | 按 `docs/acceptance/demo-checklist.md` 做一次上划和粘贴  |




## 下一步

1. 新会话先读 AGENTS.md、docs/index.md、INIT_CONTRACT.md、TASKS.md、本文件与 DECISIONS.md。
2. 用户先停掉旧的演示和设置窗口，再用新的 `target/debug/strokelet` 重新启动。在设置里录一条「先按左键再按右键」的组合并保存，确认演示打出 `reloaded gesture rules`。不要把 T10 标成 passing，除非这一步真的发生。
3. 不要把 T09 标成 passing。请用户按 `docs/acceptance/demo-checklist.md` 在 Wayland 桌面做一次上划和粘贴。



## 最近验证结果


| 时间         | 命令                                                                               | 结果    | 备注                                                             |
| ---------- | -------------------------------------------------------------------------------- | ----- | -------------------------------------------------------------- |
| 2026-09-25 | 文档结构与 Frontmatter 人工检查                                                           | 通过    | 仅 harness 文件；无 make/cargo 可跑                                   |
| 2026-09-25 | graphify                                                                         | 不适用   | 空仓库初始化，图谱未建立                                                   |
| 2026-09-25 | `PATH="$HOME/.cargo/bin:$PATH" cargo test --offline`                             | 通过    | 1 个具名启动测试；先观察到失败，再通过                                           |
| 2026-09-25 | 独立评估：`cargo fmt`、`cargo clippy`、`cargo check`、`cargo test`、`cargo run`（锁定/离线）    | 通过    | T01：1 个具名测试，binary 输出 `strokelet: scaffold only`，锁文件已跟踪        |
| 2026-09-25 | `make setup`；`make init && make test && make check`                              | 通过    | 当前机器；1 个具名测试，fmt / clippy / typecheck 成功                       |
| 2026-09-25 | `HOME=/tmp/strokelet-missing-rust PATH=/usr/bin:/bin make init`                  | 按预期失败 | 明确提示缺少 rustc 和安装入口                                             |
| 2026-09-25 | 独立评估：`make init`、`make setup`、`make test`、`make check`                           | 通过    | T03：1 个具名测试；缺 Rust 提示有效；全新机器未实测                                |
| 2026-09-25 | `gnome-shell --version`、`lsb_release -ds`、`echo "$XDG_SESSION_TYPE"`             | 通过    | GNOME Shell 50.1、Ubuntu 26.04.1 LTS、Wayland；仅环境事实，不代表功能验收      |
| 2026-09-25 | `python3` + PyYAML 校验 official Frontmatter/topic/index/CHANGELOG/链接；`make check` | 通过    | 4 份 active 文档元数据和链接一致；1 个 Rust 测试通过                            |
| 2026-09-25 | 独立评估：official Frontmatter/topic/index/CHANGELOG/链接与 `make check`                 | 通过    | T02：3 份新增 active 文档，无重大遗漏                                      |
| 2026-09-25 | `make test`（实现前）                                                                 | 按预期失败 | `tests/gesture_cases.rs` 找不到 `Decision` / `Gesture` / `Limits` |
| 2026-09-25 | `cargo fmt --all` 后 `make check`                                                 | 通过    | 25 个手势测试 + 1 个启动测试；fmt / clippy / typecheck 成功                 |
| 2026-09-25 | `make check`                                                                     | 通过    | 补上绘制超时提前进入取消；30 个手势测试 + 1 个启动测试                                |
| 2026-09-25 | `make test`（门禁实现前）                                                               | 按预期失败 | `tests/copy_gate_cases.rs` 找不到 `CopyGate` 等类型                  |
| 2026-09-25 | `make check`                                                                     | 通过    | T05：6 个门禁测试 + 既有 31 个测试；fmt / clippy / typecheck 成功            |
| 2026-09-25 | `make check`                                                                     | 通过    | 58 个测试：手势 30、门禁 6、输入 8、协议 13、启动 1                              |
| 2026-09-25 | `gjs -m scripts/check-gjs-socket.js`                                             | 通过    | 打印 `strokelet: gjs unix socket round-trip ok`                  |
| 2026-09-25 | `scripts/check-extension-install.sh`                                             | 通过    | 外来目录拒绝安装和删除；非 Wayland 拒绝 `run-demo.sh`                         |
| 2026-09-25 | `make check`                                                                     | 通过    | 67 个测试：配置 6、门禁 6、手势 31、输入 9、协议 14、启动 1                         |
| 2026-09-25 | `gjs -m settings/app.js`（无显示器）                                                   | 按预期停住 | Gtk 报 Failed to open display；脚本已加载。窗口点击未做                      |
| 2026-09-25 | `cargo test --offline --locked --all-targets` 与 clippy `-D warnings`          | 通过    | 72 个测试。快捷键录制的窗口点击未做                                          |
| 2026-09-26 | `cargo test --locked --offline` 与 `cargo clippy --locked --offline --all-targets -- -D warnings` | 通过    | 组合起始键改为先按下的鼠标键。调试二进制已重新编译。桌面上还没按过先左后右。 |




## Graphify 图谱状态

- graphify-out 是否存在：否
- 最近更新命令：未运行
- 最近图谱健康：未检查
- 需要重建或更新的原因：代码已超过 3 个模块，但本会话未运行 graphify；若本机没有该命令，保持未建立



## 决策摘要


| 决策                     | 为什么这样做                | 放弃的方案             | 影响范围                   |
| ---------------------- | --------------------- | ----------------- | ---------------------- |
| CLAUDE.md 引用 AGENTS.md | 避免两份规则漂移              | 复制全文              | 后续只改 AGENTS.md         |
| CHANGELOG 作为唯一台账       | 保持 index 精简           | 在每篇文档维护 changelog | 追溯只查 docs/CHANGELOG.md |
| 遗留 superpowers 文档降为草稿  | 缺 Frontmatter，且未经正式确认 | 直接当作 official     | 开发不得引用其为唯一依据           |


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



### 2026-09-25：快捷键录制计划

- 本次做了什么：写下非生效草稿 `docs/drafts/2026-09-25-strokelet-shortcut-capture-plan.md`。用设置窗口里的一次按键录制替换固定按键名单。没有改代码。
- 未完成事项：计划尚未确认，不得当作实现依据。T10 窗口仍未在桌面点过。T09 仍 blocked。
- 图谱：只新增一份草稿索引，图谱未受影响。

### 2026-09-25：快捷键录制

- 本次做了什么：用户确认录制计划后，设置窗口改为按下组合来记录快捷键。左右修饰键分开保存。旧的名字配置仍能读。演示进程仍不读键盘。
- 验证信号：见最近验证结果。窗口里的按键录制还没在桌面点过。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。

### 2026-09-25：录制时独占键盘

- 本次做了什么：Ctrl+Alt+T 这类系统快捷键会先被 GNOME 处理，窗口因此失焦。录制改为短时独占物理键盘，录完或取消后放开。演示进程仍不读键盘。
- 验证信号：`cargo test` 与 clippy 通过。真实键盘独占还没在这次会话里按过。
- 图谱：未建立。命令不存在。

### 2026-09-25：自定义轨迹计划

- 本次做了什么：写下非生效草稿 `docs/drafts/2026-09-25-strokelet-custom-stroke-plan.md`。用录制的鼠标轨迹替换只认四个直线方向。没有改代码。
- 未完成事项：计划尚未确认。T09 仍 blocked。T10 仍 active。
- 图谱：只新增一份草稿索引，图谱未受影响。

### 2026-09-25：自定义轨迹

- 本次做了什么：用户确认后，松开时按整条轨迹选规则。设置窗口可以画出轨迹。旧的四个方向仍按直线读取。
- 验证信号：`cargo test` 与 clippy 见最近验证。桌面上的画轨迹还没在这次会话里做过。
- 图谱：未建立。命令不存在。

### 2026-09-26：画轨迹改由正在运行的演示回传

- 本次做了什么：用户画的时候，录制进程抢不到已被演示独占的鼠标。现在「画出轨迹」向正在运行的演示要下一笔；演示没抓住鼠标时才自己独占。
- 验证信号：见最近验证结果。桌面上还没重画过。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。
- 图谱：未建立。命令不存在。

### 2026-09-26：屏幕上显示规则名称

- 本次做了什么：规则可以填写屏幕名称。快捷键发出后，扩展在指针旁显示这个名字；留空则不显示。
- 验证信号：见最近验证结果。屏幕上的文字还没在桌面看过。改了 `extension.js`，需要注销一次。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。
- 图谱：未建立。命令不存在。

### 2026-09-26：鼠标组合键计划

- 本次做了什么：写下非生效草稿 `docs/drafts/2026-09-26-strokelet-button-chord-plan.md`。按住触发键再按另一个鼠标键可以单独对应一条快捷键。没有改代码。
- 未完成事项：计划尚未确认。T09 仍 blocked。T10 仍 active。
- 图谱：只新增一份草稿索引，图谱未受影响。

### 2026-09-26：鼠标组合键

- 本次做了什么：用户确认后，按住触发键再按另一个已配置的鼠标键会立刻注入快捷键，并吞掉该键。没配置的键仍交回应用程序。设置窗口可以录这个组合。
- 验证信号：见最近验证结果。桌面上还没按过这个组合。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。
- 图谱：未建立。命令不存在。

### 2026-09-26：组合的起始键可以是先按下的任意鼠标键

- 本次做了什么：鼠标组合记下起始键和第二个键。左键、右键、中键和两个侧键都可以先按。轨迹触发键的下拉菜单也加上了左键，并改成中文。
- 验证信号：`cargo test --locked --offline` 通过；`cargo clippy --locked --offline --all-targets -- -D warnings` 通过。调试二进制已重新编译。桌面上还没按过「先左后右」。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。需要停掉正在运行的演示和设置窗口，再用新二进制重新打开。
- 图谱：未建立。命令不存在。

### 2026-09-26：设置窗口更好认、更好点

- 本次做了什么：设置窗口改成规则列表。点一条进入编辑：先在「画出轨迹」和「鼠标组合」里选一种，再录快捷键和屏幕名字。没点完成不会改原来的规则。没保存就关窗口会提醒。
- 验证信号：`gjs -m settings/app.js` 能打开主窗口和编辑窗口，退出码 0，没有报错。窗口里的点击还没在这次会话里逐项点过。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。
- 图谱：未建立。命令不存在。

### 2026-09-26：README 写上适用场景

- 本次做了什么：在 `README.md` 写明适用场景，并按 Shell 大版本写运行环境。Wayland 加上 Shell 50 的小版本是同一代。随后补上 2026-09 软件源里 Fedora、Debian、Arch、openSUSE 的对应版本。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。
- 图谱：未受影响。

### 2026-09-26：扩展声明并适配 Shell 45–50

- 本次做了什么：`metadata.json` 的 `shell-version` 改为 45 到 50。指针读数和屏幕名字尺寸按这几代可能不同的返回值来取。带查询串的模块加载失败时，改回直接加载文件。
- 验证信号：扩展脚本通过了 `node --check`。本机只登录过 Shell 50.1，45–49 还没有实机登录。改了 `extension.js`，要重新安装扩展并注销一次。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。
- 图谱：未建立。命令不存在。

### 2026-09-26：打出开箱即用的 deb

- 本次做了什么：`scripts/build-deb.sh` 打出 deb。安装后用户服务准备运行目录、找到 Wayland 会话，并在只有一只可用鼠标时自动接管。多只鼠标时在设置里选择。ydotool 的虚拟设备不作为鼠标。
- 验证信号：`cargo test --locked --offline` 通过；`cargo clippy --locked --offline --all-targets -- -D warnings` 通过；`node --check settings/app.js` 通过。`dpkg-deb -c` 看到程序、扩展、用户服务和 udev 规则，包内目录是 0755。还没有执行 `apt install`。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。装上之后要注销一次。本机有三只真实鼠标，安装后要在设置里选一只再保存。
- 图谱：未建立。命令不存在。

### 2026-09-26：顶栏图标换成一笔上划

- 本次做了什么：面板上的「划」字换成圆点加向上一笔。没连上或暂停时变淡。
- 验证信号：`node --check extension/strokelet@local/indicator.js` 通过。`scripts/reload-extension.sh` 已重新加载扩展。顶栏外观还没在这次会话里看过。
- 未完成事项：T10 仍 active。T09 仍 blocked。本次改动未提交。改的是 `indicator.js`，重新加载扩展即可，不用注销。
- 图谱：未建立。命令不存在。

