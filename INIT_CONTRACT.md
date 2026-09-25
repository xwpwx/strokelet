# 初始化契约

## 启动命令

当前已建立 Rust crate，可执行 `cargo test --offline`。标准 Makefile 入口仍是目标接口，实现前不得假装可用：

- 安装依赖：`make setup`（未建立）
- 初始化检查：`make init`（未建立）
- 启动开发服务器：`make dev`（本项目为桌面手势 demo，首版可能不需要常驻 dev server）
- 运行测试：`cargo test --offline`（可用）；`make test`（未建立）
- 完整验证：`make check`（未建立；目标含 fmt / clippy / test）

在 `TASKS.md` 的 T03 完成前，新会话应报告“标准 harness 未就绪”，先补验证入口，不要开始业务功能。

## 当前状态

- 所有依赖已安装并锁定：crate 无外部依赖；`Cargo.lock` 已生成。Rust stable 工具链已安装于 `~/.cargo/bin`，当前 shell 需将该目录加入 PATH。
- 测试框架已配置：是，Rust 内置测试框架。
- 示例测试通过：是，`tests/startup.rs` 的具名启动测试。
- Lint 规则已配置：Rust stable 已包含 `rustfmt` / `clippy`，标准入口待 T03 建立。
- 类型检查已配置：是，`cargo check --offline`。
- 文档治理 harness：是。`docs/`、`AGENTS.md`、`CLAUDE.md`、状态文件已就位。

## 项目结构

- `AGENTS.md` / `CLAUDE.md` — agent 地图与入口
- `docs/` — 项目文档；`docs/index.md` 是全局地图
- `docs/CHANGELOG.md` — 唯一变更台账
- `PROGRESS.md` — 当前进度与交接
- `DECISIONS.md` — 重要设计决策
- `QUALITY.md` — 模块健康状态
- `TASKS.md` — 任务分解
- `src/` — 最小 library + binary 脚手架
- `tests/` — 启动测试
- `graphify-out/` — 可选生成的图谱导航缓存，不入库

## 初始化验收清单

- [ ] `make setup` 从零开始能成功
- [x] `cargo test --offline` 至少有一个测试通过（`make test` 待 T03）
- [ ] `make check` 可运行，或已记录当前阻塞原因
- [x] 新的 agent 会话能只看仓库回答“怎么跑”和“怎么测”（当前答案：`cargo test --offline` 可用，Makefile 待 T03）
- [x] `TASKS.md` 存在且至少有 3 个任务
- [x] 所有初始化内容已提交到 git
