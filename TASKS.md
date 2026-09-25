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
| T01 | 建立 Rust crate 骨架、锁定依赖，并提供最小可运行测试 | 无 | passing | `make test` 或 `cargo test` | `rustc --version`；至少一个测试通过 | 独立评估：1 个测试通过；`Cargo.lock` 已跟踪；binary 启动通过 |
| T02 | 将首版 Demo 设计从草稿提升为正式需求/设计文档 | T01 非硬依赖，可并行于文档会话 | passing | 检查 `docs/index.md` 生效列表与 frontmatter | `docs/CHANGELOG.md` 有对应新增记录 | 独立评估：3 份新增 active 文档，索引、台账、链接一致 |
| T03 | 建立 Makefile 标准入口（setup/init/test/lint/check） | T01 | passing | `make init && make test && make check` | 新会话只读仓库能回答怎么跑、怎么测 | 独立评估：init/setup/test/check 通过；缺 Rust 提示明确 |
| T04 | 相对位移手势产生一次 RightClick/Copy/Cancel 决策 | T02 | passing | `make test && make check` | 边界、取消、重复释放测试通过 | `make check`：30 个手势测试 + 1 个启动测试通过；无设备、socket 或剪贴板调用 |
| T05 | Copy 决策只在桌面状态确认安全后允许注入一次 | T04 | passing | `make test && make check` | 未知、超时、暂停、断连、锁屏和 IPC 请求均不注入；成功仅一次 | `make check`：6 个门禁测试通过 |
| T06 | 协议、会话状态和固定 Ctrl+C 输出拒绝不安全输入 | T05 | passing | `make check` | 13 个协议测试通过 | `make check`：握手、超长行、断连、健康超时、复制和弦、路径和 peer 测试通过 |
| T07 | 输入帧透传右键以外的事件，并只在右击决策重放右键 | T06 | passing | `make check` | 8 个输入序列测试通过 | `make check`：透传、右击重放、上划、滚轮、SYN_DROPPED、合帧和设备过滤通过 |
| T08 | 扩展安装脚本拒绝外来目录，GJS 能收发一行 JSON | T07 | passing | `scripts/check-extension-install.sh`；`gjs -m scripts/check-gjs-socket.js` | 两条命令都打印成功 | 2026-09-25 两条命令成功；Shell 内轨迹未实机验证 |
| T09 | 在真实桌面完成上划复制、负例、热插拔和锁屏验收 | T08 | blocked | 按 `docs/acceptance/demo-checklist.md` 操作真实鼠标 | 编辑器与 Firefox 粘贴结果、轨迹和普通右击 | 等待用户实机验证；模拟测试不能代替 |
| T10 | 设置窗口可改触发键、直线方向和对应快捷键，运行中的演示能重新读取 | 用户确认草稿计划；T09 仍 blocked | active | `make check`；打开 `strokelet settings` 后保存并让正在运行的演示打出 reloaded | 配置往返、未知键、重复方向、reload 行测试通过；窗口本身要在桌面点过 | 自动测试已通过。窗口尚未在本次会话里点过，不能标 passing |

## T01: 建立 Rust crate 骨架与最小测试

- 目标：让后续会话能在本仓库安装工具链、锁定依赖，并跑通至少一个示例测试。
- 范围：创建最小 library+binary crate、提交 `Cargo.lock`、提供 `cargo test` 可观察通过信号。不实现手势识别或设备接管。
- 依赖：无
- 状态：passing
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
- 状态：passing
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
- 状态：passing
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

## T04: 纯手势识别状态机

- 行为：右键按下后按 SYN_REPORT 帧累计相对位移，根据正式手势规格在释放时返回一次 RightClick、Copy 或 Cancel；重复释放返回 None。
- 依赖：T02 的 `docs/official/specs/gesture-recognition.md`。
- 状态：passing
- 范围：`src/gesture.rs` 与 `tests/gesture_cases.rs`，覆盖阈值、折返、直线度、超时及取消。不连接 evdev/uinput，不绘制轨迹，不注入 Ctrl+C。
- 验证命令：`make test && make check`。
- 完成证据：`make check` 通过。30 个具名手势测试覆盖阈值、折返、直线度、超时和取消；重复释放返回 None。`src/gesture.rs` 不访问设备、socket 或剪贴板。

### 冲刺合同

- 范围：只输出决策和当前是否进入 Drawing，供后续输入与轨迹模块消费。
- 验证标准：79/80 counts、2500/2501 ms、横向偏移和直线度边界；普通右击、下上往返、横/下划、取消均有可回放测试。
- 排除项：真实设备接管、GNOME 扩展、协议和复制注入。
- 运行时信号：测试进程退出码与各具名测试结果；无外部资源或设备 FD。

## T05: 复制注入门禁

- 行为：识别结果为 Copy 时，仅当目标会话归属正确、活动且未锁定，扩展已连接，未暂停，修饰键状态与通信结果已知且未超时，权限检查通过，才允许注入一次 Ctrl+C。同一决策的再次检查、非 Copy 决策，以及 IPC 直接请求，都不注入。
- 依赖：T04 的 `Decision`；依据为 `docs/official/design/strokelet-demo.md` 的会话、连接、暂停、修饰键和权限边界。
- 状态：passing
- 范围：`src/copy_gate.rs` 与 `tests/copy_gate_cases.rs`。不打开 socket，不读 logind，不写 uinput。
- 验证命令：`make test && make check`。
- 完成证据：`make check` 中 6 个门禁测试通过。不安全或不完整的桌面状态不能注入，成功路径只有一次。

## T06: 协议、会话与固定复制

- 行为：JSON 行协议拒绝未知版本、超长行、截断 JSON 和重复 Ready；断连后旧手势 id 失效；复制输出固定为 Ctrl、C 的按下与抬起；运行时路径拒绝符号链接和错误属主。
- 依赖：T05。
- 状态：passing
- 范围：`src/protocol.rs`、`src/copy.rs`、`src/session.rs`、`src/server.rs` 与 `tests/protocol_cases.rs`。
- 验证命令：`make check`。
- 完成证据：13 个具名协议测试通过。

## T07: 输入帧与右键控制

- 行为：同一 SYN_REPORT 内的相对位移合并成一次 motion；右键被暂存，只在右击决策重放；滚轮、侧键和 SYN_DROPPED 取消手势并转发原事件。
- 依赖：T06。
- 状态：passing
- 范围：`src/input.rs` 与 `tests/input_sequences.rs`。fake sink，不抓真实设备。
- 验证命令：`make check`。
- 完成证据：8 个具名输入测试通过。

## T08: 扩展安装与 GJS socket

- 行为：安装脚本只覆盖带 `STROKELET_OWNED` 标记的本项目扩展；卸载脚本拒绝删除外来目录；GJS 能通过 Unix socket 收发一行 JSON。
- 依赖：T07。
- 状态：passing
- 范围：`extension/strokelet@local/`、`scripts/install-extension.sh`、`scripts/remove-extension.sh`、`scripts/check-gjs-socket.js`。
- 验证命令：`scripts/check-extension-install.sh` 与 `gjs -m scripts/check-gjs-socket.js`。
- 完成证据：两条命令成功。GNOME Shell 内的轨迹、暂停菜单和焦点行为仍属于 T09。

## T09: 实机验收

- 行为：在 Wayland 桌面上，右键直线上划显示轨迹并复制一次；普通右击、负例、停止、热插拔和锁屏按验收清单确认。
- 依赖：T08，以及用户能操作真实鼠标。
- 状态：blocked
- 范围：`src/main.rs` 前台循环、`scripts/run-demo.sh`、`docs/acceptance/demo-checklist.md`。
- 验证命令：按验收清单操作真实鼠标。
- 完成证据：尚无。当前会话不能操作真实鼠标，不能把 `make check` 记成实机通过。

## T10: 手势与快捷键设置窗口

- 行为：`strokelet settings` 打开 GTK 窗口，编辑一个触发键和最多四条直线方向快捷键。保存写入 `~/.config/strokelet/gestures.json`。正在运行的 `run` 收到 `reload` 后换上新规则，不替换扩展连接。缺文件时用右键上划 Ctrl+C；损坏文件拒绝启动。
- 依赖：用户确认 `docs/drafts/2026-09-25-strokelet-gesture-gui-plan.md`。T09 仍 blocked，这次是用户明确要求先做设置。
- 状态：active
- 范围：`src/config.rs`、`src/copy.rs`、`src/gesture.rs`、`src/main.rs`、`settings/app.js`。不升格正式文档，不标 T09 通过。
- 验证命令：`make check`。窗口路径要在桌面打开并保存一次。
- 完成证据：自动测试通过。窗口尚未在桌面上点过，所以保持 active。

### 冲刺合同

- 范围：把一次手势决策和一份桌面观察结果变成 `None` 或 `CopyOnce`。
- 验证标准：新鲜安全状态注入一次；二次检查、失败后重试、右击、取消、IPC 请求均不注入。会话、扩展、暂停、修饰键、通信、权限的未知和超时都拒绝。
- 排除项：真实 logind、Unix socket、虚拟键盘和轨迹层。
- 运行时信号：测试进程退出码与各具名测试结果；无外部进程或设备 FD。
