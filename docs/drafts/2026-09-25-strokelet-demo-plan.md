【草稿】本文档非生效版本，不作为开发、设计、需求依据。

# Strokelet 右键上划复制 Demo Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. 本计划默认在当前任务顺序执行，不要求启动子代理。

**Goal:** 在 Ubuntu 26.04 GNOME Wayland 中，实现按住右键向上直线拖动时显示全局轨迹、松开后发送一次 Ctrl+C，且普通右击可用。

**Architecture:** Rust 接管指定相对鼠标，透传正常输入并识别上划；GNOME 50 扩展提供不抢焦点的实时轨迹及会话状态。后台和扩展使用有身份验证的 Unix socket 通信；第一版不做录制 GUI、自由形状识别和任意命令执行。

**Tech Stack:** Rust stable、evdev（含 uinput）、serde/serde_json、nix 或 rustix 的 Unix 凭据/轮询接口、GJS、GNOME Shell 50、St/Clutter/Cairo、Gio Unix socket；logind 查询可使用 zbus blocking。实现开始时核实依赖版本和 API 并固定 Cargo.lock。

**Spec:** `docs/drafts/2026-09-25-strokelet-demo-design.md`（非生效版本）。

## Global Constraints

- 项目目录 `/home/xwp/strokelet`，规划时为空目录；本机确认 GNOME Shell 50.1，未在 PATH 中找到 cargo/rustc，执行前需安装 Rust 工具链。
- 目标是原生 Wayland 应用上的全局手势；XWayland 验收不能替代它。
- 固定右键、固定直线上划、固定 Ctrl+C；终端复制快捷键与完整 GUI 属于后续版本。
- 轨迹必须绘制在应用上方，不抢焦点，不拦截点击。GNOME 扩展是 demo 的必要组成。
- 原始位移单位 counts 不等同像素；识别与实际屏幕绘制使用不同坐标来源。
- 不修改用户已有鼠标手势配置，不启用常驻开机服务，不全局关闭 GNOME 扩展版本检查，不给日常用户加入 input 组。
- 特权输入进程不接受任意按键或命令请求；命中只发送固定 Ctrl+C。
- 未通过扩展就绪、输入安全和会话检查前不接管鼠标；默认前台限时运行 120 秒。
- 这是实施计划而非完成报告；所有测试框初始未勾选，实机行为必须在执行阶段验证。

---

## 文件布局

```text
Cargo.toml
Cargo.lock
src/lib.rs
src/gesture.rs             # 纯状态机和上划判定
src/protocol.rs            # 版本化 JSON 行协议和大小限制
src/input.rs               # 设备能力、抓取、虚拟鼠标、事件转发
src/copy.rs                # 固定 Ctrl+C 输出
src/session.rs             # 活动/锁定会话验证
src/server.rs              # Unix socket、凭据验证、连接健康
src/main.rs                # CLI、轮询循环、退出与资源回收
extension/strokelet@local/metadata.json
extension/strokelet@local/extension.js
extension/strokelet@local/overlay.js
extension/strokelet@local/transport.js
extension/strokelet@local/indicator.js
scripts/install-extension.sh
scripts/run-demo.sh
scripts/remove-extension.sh
tests/gesture_cases.rs
tests/input_sequences.rs
tests/protocol_cases.rs
README.md
docs/acceptance/demo-checklist.md
```

## Task 1：先验证 GNOME 50 的轨迹与通信能力

**Files:** `extension/strokelet@local/{metadata.json,extension.js,overlay.js,transport.js,indicator.js}`、`scripts/{install-extension.sh,remove-extension.sh}`、`README.md`。

**Interfaces:** `GestureOverlay.begin()` 开始跟踪真实指针；`end()`/`destroy()` 清理所有计时器和绘图对象；`DemoTransport.connect(path)`、`send(message)`、`disconnect()`；`onMessage(message)` 回调。扩展断连时停止绘制并显示未连接。

- [ ] 读取本机 GNOME 50 的 Shell/GJS 资源与 GI 方法，确认 stage、St.DrawingArea、Gio.SocketClient Unix 连接及绘图上下文可用。记录所用接口，不复制 GNOME 46 私有 API 假定兼容。
- [ ] 创建仅声明 GNOME 50 的扩展元数据：

```json
{"uuid":"strokelet@local","name":"Strokelet Demo","description":"Right-button stroke demo","shell-version":["50"]}
```

- [ ] 实现不 reactive、不 can_focus 的轨迹层，线宽初值 3 logical px；绘图前转换 stage 坐标到 actor 局部坐标。每约 16ms 采样实际指针，轨迹保留最多 512 点；禁用时取消采样并销毁 actor。
- [ ] 指示器加入明确标注的“轨迹自检（5 秒）”和暂停开关。自检只绘图，不接管设备、不发送快捷键；菜单关闭后再开始采样，确保绘图不维持菜单 grab。
- [ ] 安装脚本只复制本项目扩展到用户目录，发现同 UUID 非本项目文件时退出。说明初装可能需注销重新登录；不自动注销用户。
- [ ] 实机在编辑器和浏览器上移动鼠标，确认能显示轨迹，同时点击和焦点保持正常；验证 disable 后不留残线或定时器。
- [ ] 验证 GJS 能连接测试 Unix socket、收发 JSON 行并处理断连。若此步不可行，暂停后续输入接管，先修正传输层，不把无法验证的 overlay 留到最后。

**通过门槛：** 原生 Wayland 应用上可显示不抢焦点的轨迹。失败时不得转向仅窗口内画线并称为完成。

## Task 2：建立纯识别状态机和可回放测试

**Files:** `Cargo.toml`、`src/{lib.rs,gesture.rs}`、`tests/gesture_cases.rs`。

**Interfaces:**

```rust
pub struct Limits {
    pub start_counts: f64,
    pub min_up_counts: f64,
    pub click_slop_counts: f64,
    pub max_duration_ms: u64,
}
pub enum Decision { None, RightClick, Copy, Cancel }
pub struct Gesture; // 内部保存状态、累计位置、路径长、最大偏移和 elapsed
impl Gesture {
    pub fn new(limits: Limits) -> Self;
    pub fn press(&mut self);
    pub fn motion(&mut self, dx: f64, dy: f64);
    pub fn tick(&mut self, elapsed_ms: u64); // 参数为本次按下起的单调时间差
    pub fn release(&mut self) -> Decision;
    pub fn cancel(&mut self);
}
```

- [ ] 安装/确认 Rust stable 工具链，然后创建最小 library+binary crate；选择实际存在且兼容的依赖版本，提交 lockfile。
- [ ] 先添加以下行为测试，执行 `cargo test --test gesture_cases`，确认未实现时失败：

```rust
use strokelet::{Gesture, Limits, Decision};
fn gesture() -> Gesture {
    Gesture::new(Limits { start_counts: 12.0, min_up_counts: 80.0,
        click_slop_counts: 12.0, max_duration_ms: 2500 })
}
#[test]
fn upward_stroke_copies_once() {
    let mut g = gesture();
    g.press();
    for _ in 0..10 { g.motion(0.0, -10.0); }
    assert!(matches!(g.release(), Decision::Copy));
    assert!(matches!(g.release(), Decision::None));
}
#[test]
fn round_trip_is_not_a_click() {
    let mut g = gesture(); g.press();
    g.motion(0.0, 100.0); g.motion(0.0, -100.0);
    assert!(matches!(g.release(), Decision::Cancel));
}
#[test]
fn timeout_cannot_turn_into_copy() {
    let mut g = gesture(); g.press(); g.motion(0.0, -100.0);
    g.tick(2501);
    assert!(matches!(g.release(), Decision::Cancel));
}
```

- [ ] 实现设计文档的方向、横向偏移、直线度判据以及 Pending→Drawing 的单向转换；未按下时 release 返回 None。
- [ ] 补充并运行具名测试：`stationary_right_click`、`small_jitter_right_click`、`downward_cancel`、`horizontal_cancel`、`diagonal_cancel`、`short_upward_cancel`、`cancel_waits_for_release`。阈值边界取 79/80 counts 及超时 2500/2501ms，避免浮点模糊断言。
- [ ] `cargo test --test gesture_cases` 全部通过后再接入真实设备。

**通过门槛：** 分类是可重复的纯函数/状态机行为，且超时与取消不会误复制。

## Task 3：协议、会话与固定复制动作的安全边界

**Files:** `src/{protocol.rs,server.rs,session.rs,copy.rs}`、`tests/protocol_cases.rs`、扩展 `transport.js`。

**Interfaces:**

```text
Rust → 扩展：Hello(version=1), Begin(id), End(id,outcome), Cancel(id,reason)
扩展 → Rust：Ready(version=1), Pause(paused), State(active,locked,modifiers), Pong
Rust → 扩展：Ping
outcome: click | copy-injected | unmatched
```

JSON 每行一条，最大 4096 字节；协议不含执行命令/任意按键字段。连接认证后才能解析控制消息。每 250ms 检查连接健康，超过 1000ms 无有效回复按断连取消；`State` 过期时不得复制。姿态/轨迹点不经特权服务反向注入。

Rust 函数约定：`SessionGuard::is_active_unlocked() -> Result<bool>`；`CopyOutput::send_copy() -> Result<()>`；`CopyOutput::release_owned_keys() -> Result<()>`。注入使用左 Ctrl，按下 Ctrl/C、释放 C/Ctrl，并发送需要的 SYN_REPORT 边界；错误清理只涉及本进程已按下的虚拟键。

- [ ] 编写解析拒绝测试：未知 version、超长行、无换行超长流、未知消息、截断 JSON、重复 Ready、断连后旧 Begin ID 不再有效；正常握手能进入 ready 状态。
- [ ] 创建受限 socket 路径并验证 SO_PEERCRED；用非目标 UID 的进程进行拒绝测试。路径权限按设计文档设置，拒绝现存符号链接。
- [ ] 将 GNOME sessionMode 和指针修饰键状态送入 State；后台同时验证 logind session 归属 UID、Active、LockedHint。未知、查询失败、失活、锁定均使可用状态为 false。
- [ ] CopyOutput 用 fake event sink 测试固定输出序列为 Ctrl↓、C↓、C↑、Ctrl↑；部分写失败执行 owned-key 清理。IPC 客户端不能直接调用 CopyOutput。
- [ ] 将端到端复制开关初始设为关闭；仅在后续真实手势集成时开启。此任务不要求抓取鼠标。

**通过门槛：** 扩展与后台能通信，断连/锁屏拒绝动作，固定快捷键输出有可测试的事件顺序。

## Task 4：完整鼠标透传与右键事件控制

**Files:** `src/{input.rs,main.rs}`、`tests/input_sequences.rs`。

**Interfaces:** `InputBridge::open(path)`、`grab()`、`read_frame()`、`forward(frame)`、`replay_right_click()`、`release_all_owned()`、`ungrab()`。`read_frame` 必须以 SYN_REPORT 为边界，内部使用 evdev 已确认的真实类型；公开接口具体 Rust 类型在选定 crate 版本后一次性定义，调用端统一引用，禁止各模块自行创建不同事件类型。

- [ ] `--list-devices` 展示名称、稳定 by-id 路径、REL_X/REL_Y 和按键能力；拒绝 Strokelet 虚拟设备和不支持的绝对设备。
- [ ] 用 fake sink 建立事件序列测试，预期表如下：

| 输入 | 预期输出 |
| --- | --- |
| 普通移动/左键/滚轮及 SYN_REPORT | 相同事件顺序和帧边界 |
| 右键↓、无移动、右键↑ | 识别结束后一次右键↓/↑ |
| 右键↓、有效上划、右键↑ | 保留移动，无右键事件，一次 Copy decision |
| 右键↓、移动、超时、右键↑ | 无 Copy，无补发菜单 |
| 手势中出现滚轮或其他按键 | 取消手势，正常转发非右键事件 |
| SYN_DROPPED | 取消手势并同步物理状态；不能产生 Copy |

- [ ] 根据选定物理鼠标复制支持的相关能力创建虚拟鼠标；保留高精度滚轮、水平滚轮和侧键，避免把原始事件丢给不支持相应 capability 的设备。
- [ ] 先实现 `--passthrough-only --timeout-seconds 10`，只有扩展 ready 且目标会话有效时允许 grab，确保全部按键松开再开始。用键盘终端启动，验证移动、滚轮和各按键。
- [ ] 比较接管前后的指针速度和加速。虚拟设备不自动继承真实设备设置；发现差异时记录设备 DPI、libinput 设置，针对虚拟设备处理，不能偷偷修改用户全局鼠标设置。
- [ ] 接入 Gesture 状态机，但动作先仅打印 decision；调试日志不记录键盘文本或选区内容。
- [ ] 对 Ctrl+C/SIGTERM、EOF、设备拔出、socket 断连、SYN_DROPPED、锁屏设置明确取消和资源关闭路径；先验证限时自动退出，再验证手动停止。

**通过门槛：** 正常鼠标操作保持可用，普通右击只有一次，输入链路退出后能恢复。硬件拔出后不自动抓取新设备。

## Task 5：集成上划、桌面轨迹和复制

**Files:** `src/main.rs`、扩展 `extension.js/overlay.js/indicator.js`、`scripts/run-demo.sh`。

- [ ] 右键按下后发送 Begin，扩展采样指针但超过屏幕起步阈值才显示线条；End/Cancel 立即隐藏。使用 gesture id，迟到事件不能恢复已结束轨迹。
- [ ] 在每个输入帧累计 REL_X/Y 后调用 `Gesture::motion`，使用 Instant 计算 tick；保持物理移动持续转发，保持右键暂存。
- [ ] release 返回 Copy 时再次确认 logind 活动未锁定、扩展 State 新鲜、无物理修饰键且非暂停，再调用 send_copy；失败则记录取消原因，不报告复制成功。
- [ ] `run-demo.sh` 校验实际桌面会话与设备参数，显式传入目标 UID/session ID；默认 timeout=120 秒。先启动 socket listener，扩展连入 Ready 后才接管；绝不让脚本自动创建长期 root 服务。
- [ ] 在原生 Wayland 文本编辑器中输入并选中 `Strokelet demo 123`，右键上划，检查轨迹与无菜单；松开后在另一空白文档手动 Ctrl+V，应得到该文本。
- [ ] 在 Firefox 普通页面重复上述流程；不以终端、浏览器受限内部页作为首次复制验收场景。
- [ ] 运行负例：左/右/下划、短上划、长按超时、修饰键按住、暂停、扩展断连，均不发送复制动作；普通右击仍可打开菜单。

**通过门槛：** 用户给出的完整链路真实运行，且复制不改变当前选择或焦点。

## Task 6：验收、文档和可复现交付

**Files:** `README.md`、`docs/acceptance/demo-checklist.md`，必要时只修正前述任务中发现的缺陷。

- [ ] 运行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`；失败必须记录并修复，不能用硬件不可用解释纯逻辑测试失败。
- [ ] 在目标会话完成设计文档中的 20 次上划、每方向 10 次负例、普通点击、停止/拔插/锁屏和实际显示器布局测试。记录实际结果、设备、GNOME 版本、缩放以及未测项。
- [ ] README 写明两个终端的启动顺序、扩展安装/启用、可能需要注销的初装条件、设备选择、阈值参数、暂停/停止/卸载命令。明确不是通用桌面版，也不是完整 Easystroke 替代品。
- [ ] 提供只移除本项目扩展的脚本；默认保留用户日后可能产生的数据。无开机服务，无 input 组修改，无遗留系统 udev 规则。
- [ ] 仅在真实目标环境验收后标记 demo 完成；如果 agent 无法操作实际鼠标，将测试停在“等待用户实机验证”，提供单次上划及粘贴的明确步骤，不将模拟事件测试说成实机通过。
- [ ] 如目录是本项目 Git 仓库，每个通过验证的任务形成一个本地提交；如无 Git 仓库，可在实现开始时初始化本目录，不操作父目录仓库。此计划编写阶段不创建仓库或提交。

## 执行顺序与失败分流

Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6。

- 轨迹层失败：停在 Task 1，调查 GNOME 50 的 actor/坐标/帧回调接口。
- 透传改变鼠标行为或丢事件：停在 Task 4，先修输入能力和转发，不增加识别算法。
- 识别通过但粘贴失败：分别检查焦点、修饰键状态、虚拟键事件和目标应用快捷键，不擅自读取选区或剪贴板绕过。
- 只有实机用户操作才能证明的行为明确列为待验收，不因构建通过而自动勾选。

## 后续版本（不在本次交付内）

egui 图形化录制和动作配置、同源原始轨迹模板录制、侧键选择、方向敏感模板识别、多样本、命令执行的普通用户代理、热插拔恢复、systemd/权限 broker 正式化及 deb 打包。
