# 项目进度

> 用途：跨会话保存当前工作状态。每次长任务开始前先读本文件；每次会话结束前更新本文件。
> 边界：这里只记录执行状态、阻塞、最近一次验证和交接。变更历史看 git 与 `docs/CHANGELOG.md`，设计原因看 `DECISIONS.md`。

## 当前状态

- 最新 commit：随 `v0.1.2` 推送
- 发布：GitHub Release `v0.1.2`，附件为 `strokelet_0.1.2_amd64.deb`
- 任务：T01–T08 为 `passing`。T09 实机验收为 `blocked`。T10 设置窗口为 `active`，不要自行改成 `passing`
- 测试：最近一次 `cargo test --locked --offline` 与 `cargo clippy --locked --offline --all-targets -- -D warnings` 通过
- Graphify 图谱：未建立。本机没有 graphify 命令

## 当前目标

发布包能在登录后接管鼠标。T09 仍等实机验收清单。T10 仍等按任务里的验证命令收口。

## 相关文档

- 生效依据：`docs/index.md`
- 任务：`TASKS.md`
- 决策：`DECISIONS.md`
- 变更历史：`docs/CHANGELOG.md` 与 git log

## 已知问题

- grab 前没有读取物理按键状态。
- 复制门禁里的权限观察在前台循环中固定为已知且通过。
- Shell 45–49 只做了接口适配，没有在对应系统上登录过。本机跑过的是 Shell 50.1。
- 本机服务目前用 `~/.local/bin/strokelet`，不是已安装的 deb。源码里的运行目录由 root 在登录时准备。这台机器要装上 `v0.1.2` 并去掉这个覆盖，注销后才会按新包运行。

## 阻塞项

| 阻塞项 | 原因 | 需要谁处理 | 下一步 |
| --- | --- | --- | --- |
| T09 实机验收 | 编辑器与 Firefox 的上划、负例、热插拔和锁屏还没有按清单做完 | 用户 | 按 `docs/acceptance/demo-checklist.md` |
| 本机仍可能停在旧安装 | `v0.1.2` 还没覆盖这台机器上正在用的包，而且用户服务被本地二进制覆盖 | 用户 | `sudo apt install` Release 里的 deb，去掉 `strokelet.service.d/binary.conf`，然后注销一次 |

## 下一步

1. 读 `AGENTS.md`、`docs/index.md`、`INIT_CONTRACT.md`、`TASKS.md`、本文件和 `DECISIONS.md`。
2. 若这台机器还没用 `v0.1.2` 的 deb 覆盖安装，先装上，去掉本地服务覆盖，再注销一次。
3. 不要把 T09 或 T10 标成 `passing`，除非对应验证命令真的通过。

## 最近验证

| 时间 | 命令 | 结果 | 备注 |
| --- | --- | --- | --- |
| 2026-09-26 | `cargo test --locked --offline`；`cargo clippy --locked --offline --all-targets -- -D warnings` | 通过 | 打 deb 之前 |
| 2026-09-26 | `gh release create v0.1.0` | 通过 | https://github.com/xwpwx/strokelet/releases/tag/v0.1.0 |
| 2026-09-26 | 本机 `systemctl --user status strokelet.service` | active | 当时用的是临时跳过失败步骤，不是已安装的新包。日志里抓住了 MCHOSE |
| 2026-09-26 | `node --check` overlay.js 与 settings/app.js；`scripts/reload-extension.sh` | 通过 | 扩展状态 ACTIVE。轨迹圆滑和名字位置还没在桌面上看过 |

## Graphify 图谱状态

- graphify-out 是否存在：否
- 最近更新命令：未运行
- 最近图谱健康：未检查
- 需要重建或更新的原因：本机没有 graphify 命令

## 交接

- 轨迹改成圆滑曲线。名字默认在屏幕下方居中，粗细和位置在设置的「显示」里改，写入 `~/.config/strokelet/appearance.json`。
- 顶栏「退出」会停掉后台、关掉设置窗口，并拿掉顶栏图标。暂停是一个按钮：运行时写「暂停」，停住时写「启动」。菜单里没有轨迹自检。
- 规则可以是按住鼠标键再滚轮。设置里选「按住再滚轮」录制。
- 已安装的设置窗口还是旧的。顶栏「设置」和本机应用入口改成打开扩展目录里的新窗口，里面有「显示」。
- 更早的会话记录不在这里。看 git log 和 `docs/CHANGELOG.md`。
