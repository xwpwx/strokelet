# Strokelet

Ubuntu 26.04 / GNOME Shell 50 Wayland 上的右键直线上划 Demo：松开后发送一次 Ctrl+C，并显示不抢焦点的轨迹。普通右击仍然可用。

设置窗口可以改触发键、直线方向，以及每条方向对应的一条快捷键。没有录制、自由形状或任意命令。

## 开发验证

```sh
make init
make setup
make test
make check
```

扩展脚本检查：

```sh
gjs -m scripts/check-gjs-socket.js
scripts/check-extension-install.sh
```

## 两个终端的启动顺序

实机验收步骤见 [docs/acceptance/demo-checklist.md](docs/acceptance/demo-checklist.md)。摘要：

1. 确认当前是 Wayland。
2. root 创建一次 `/run/strokelet/<uid>`，属主 root、组为当前用户主组、权限 0770。
3. `scripts/install-extension.sh`，再 `gnome-extensions enable strokelet@local`。首次安装可能需要注销并重新登录；脚本不会自动注销。
4. 一个终端运行 `cargo build && ./target/debug/strokelet list-devices`，记下被接受的相对鼠标路径，以及 `loginctl list-sessions` 里的会话号。
5. 另一个终端保持 GNOME 会话。运行：

```sh
scripts/run-demo.sh --device /dev/input/by-id/你的鼠标 --uid "$(id -u)" --session 会话号
```

默认前台运行 120 秒，可以用 `--timeout-seconds` 加长。复制注入由该脚本打开；直接调用 `strokelet run` 时默认关闭，需要 `--inject-copy`。`--passthrough-only` 只透传。

改规则：另开一个终端，运行 `./target/debug/strokelet settings`。窗口不抓鼠标。保存后写入 `~/.config/strokelet/gestures.json`；如果演示正在运行，会让它重新读这份文件。没有配置文件时，默认仍是右键上划 Ctrl+C。

暂停：点面板上的「划」，打开「暂停」。停止：等超时，或在运行终端按 Ctrl+C。卸载：`scripts/remove-extension.sh`，它只删除带本项目标记的扩展目录。

## 调试

不用每次都注销。

- 改 Rust 后重新 `cargo build`，再跑 `scripts/run-demo.sh`。Ctrl+C 会放开鼠标并删掉 socket。
- 改 `overlay.js`、`indicator.js`、`transport.js` 后运行 `scripts/reload-extension.sh`。它会把扩展复制进用户目录，再让当前 Shell 关掉并重新打开。
- 只有第一次安装，或者改了 `extension.js` 本身，才需要注销一次。GNOME 50 不能在 Wayland 上热重载扩展入口文件。

默认阈值在识别规格里：起步 12 counts，上划至少 80 counts，最长 2500 ms。轨迹要移动约 12 个屏幕像素后才显示，线宽 3。

生效文档见 [docs/index.md](docs/index.md)。当前任务见 [TASKS.md](TASKS.md)。
