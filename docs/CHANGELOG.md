# 变更留痕台账

> 用途：集中记录所有正式文档的变更历史。追加式，只增不改。
> 边界：docs/index.md 只做导航和当前生效文档列表，不含变更历史；所有变更留痕统一记在这里。

## 变更记录

| 日期 | 文档路径 | 版本号 | 类型 | 说明 |
|---|---|---|---|---|
| 2026-09-25 | docs/official/rules/graphify-workflow.md | 1.0.0 | 新增 | 初始版本：Graphify 代码图谱工作流 |
| 2026-09-25 | docs/official/requirements/strokelet-demo.md | 1.0.0 | 新增 | 确定首版 Demo 用户行为、范围与验收要求 |
| 2026-09-25 | docs/official/design/strokelet-demo.md | 1.0.0 | 新增 | 确定首版组件边界、运行流程与安全约束 |
| 2026-09-25 | docs/official/specs/gesture-recognition.md | 1.0.0 | 新增 | 确定上划识别状态、阈值与判据 |
| 2026-09-25 | docs/official/design/strokelet-demo.md | 1.1.0 | 修改 | 运行时目录权限由 0750 改为 0770，使目标用户能绑定 socket |
| 2026-09-26 | docs/official/requirements/strokelet-demo.md | 1.1.0 | 修改 | 安装包纳入范围：用户服务、自动选择会话和鼠标 |
| 2026-09-26 | docs/official/design/strokelet-demo.md | 1.2.0 | 修改 | 安装包以 root 只创建运行时目录，uinput 走 uaccess |
| 2026-09-26 | docs/official/design/strokelet-demo.md | 1.2.1 | 修改 | 运行时目录改由登录时的 root 钩子创建，用户服务不能提权 |
