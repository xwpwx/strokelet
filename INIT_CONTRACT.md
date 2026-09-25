# 初始化契约

## 启动命令

在仓库根目录运行以下命令。Makefile 自动将 `~/.cargo/bin` 加入 PATH；当前环境的 Rust stable 版本为 1.98.1。

- 获取锁定的项目依赖：`make setup`（需预先安装 Rust stable；当前 crate 无外部依赖）
- 初始化检查：`make init`（只检查 Rust 工具链和项目清单）
- 运行测试：`make test`（锁定依赖、离线运行）
- 运行格式与静态检查：`make lint`
- 完整验证：`make check`（fmt / clippy / test / typecheck）
- 启动当前脚手架：`cargo run --locked --offline`（仅输出状态；没有常驻 dev server）

缺少 Rust 时，先按 `make init` 的错误提示安装工具链；`make setup` 不安装系统级 Rust。产品需求/设计仍待 T02 提升为正式文档，在此之前不要开始业务功能。

## 当前状态

- 所有依赖已安装并锁定：crate 无外部依赖；`Cargo.lock` 已生成。Rust stable 工具链已安装于 `~/.cargo/bin`，Makefile 自动加入 PATH。
- 测试框架已配置：是，Rust 内置测试框架。
- 示例测试通过：是，`tests/startup.rs` 的具名启动测试。
- Lint 规则已配置：是，`make lint`。
- 类型检查已配置：是，`make check` 包含 `cargo check --locked --offline`。
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

- [ ] `make setup` 在全新机器上从零成功（尚未验证；要求预装 Rust）
- [x] 当前环境 `make setup` 获取锁定项目依赖成功
- [x] `make test` 至少有一个测试通过
- [x] `make check` 可运行
- [x] 新的 agent 会话能只看仓库回答“怎么跑”和“怎么测”（见本文件与 README）
- [x] `TASKS.md` 存在且至少有 3 个任务
- [x] 所有初始化内容已提交到 git
