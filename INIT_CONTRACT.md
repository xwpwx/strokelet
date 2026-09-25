# 初始化契约

## 启动命令

当前仓库只有文档治理 harness，尚未建立可执行工程入口。下列命令是目标接口，实现前不得假装可用：

- 安装依赖：`make setup`（未建立）
- 初始化检查：`make init`（未建立）
- 启动开发服务器：`make dev`（本项目为桌面手势 demo，首版可能不需要常驻 dev server）
- 运行测试：`make test`（未建立；目标落地为 `cargo test`）
- 完整验证：`make check`（未建立；目标含 fmt / clippy / test）

在 `TASKS.md` 的 T01 / T03 完成前，新会话应报告“harness 未就绪”，先补环境与验证入口，不要开始业务功能。

## 当前状态

- 所有依赖已安装并锁定：否。尚未创建 `Cargo.toml` / `Cargo.lock`；本机 PATH 中可能没有 `cargo` / `rustc`。
- 测试框架已配置：否。
- 示例测试通过：否。尚无测试。
- Lint 规则已配置：否。目标为 `cargo fmt` / `cargo clippy`。
- 类型检查已配置：否。Rust 编译器即类型检查入口，但工程未建立。
- 文档治理 harness：是。`docs/`、`AGENTS.md`、`CLAUDE.md`、状态文件已就位。

## 项目结构

- `AGENTS.md` / `CLAUDE.md` — agent 地图与入口
- `docs/` — 项目文档；`docs/index.md` 是全局地图
- `docs/CHANGELOG.md` — 唯一变更台账
- `PROGRESS.md` — 当前进度与交接
- `DECISIONS.md` — 重要设计决策
- `QUALITY.md` — 模块健康状态
- `TASKS.md` — 任务分解
- `src/` — 源代码（尚未创建）
- `tests/` — 测试文件（尚未创建）
- `graphify-out/` — 可选生成的图谱导航缓存，不入库

## 初始化验收清单

- [ ] `make setup` 从零开始能成功
- [ ] `make test` 至少有一个测试通过
- [ ] `make check` 可运行，或已记录当前阻塞原因
- [x] 新的 agent 会话能只看仓库回答“怎么跑”和“怎么测”（当前答案：入口未建立，见本文件与 `TASKS.md`）
- [x] `TASKS.md` 存在且至少有 3 个任务
- [ ] 所有初始化内容已提交到 git
