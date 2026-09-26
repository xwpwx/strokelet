# Strokelet

在 GNOME Shell 45 到 50 的 Wayland 桌面上，用一笔轨迹或两个鼠标键发出一组快捷键，并在屏幕上画出不抢焦点的轨迹。普通点击仍然可用。

## 安装

在仓库里打出安装包，再装上：

```sh
scripts/build-deb.sh
sudo apt install ./dist/strokelet_0.1.0_amd64.deb
```

装好后注销一次再登录。之后每次登录都会接管鼠标。点右上角那一笔标记，选「设置」，或者从应用列表打开「Strokelet」，都可以改规则。

只有一只可用的相对鼠标时会自动用它。有多只时，打开「Strokelet」，在「使用这只」里选中要接管的鼠标，再点保存。也可以把路径写入 `~/.config/strokelet/device`。

## 适用场景

适合手经常放在鼠标上、又不想为少数几个常用快捷键把右手挪回键盘的时候。动作发生在当前窗口里，焦点不会被切走。

- 选中文字后，按住右键向上划，松开后发出一次 Ctrl+C。只是点一下右键时，菜单仍会打开。这是没有配置文件时的默认行为。
- 把自己的一笔，或「先按住一个鼠标键、再按另一个」，配成常用快捷键。例如打开终端、粘贴、关闭窗口。最多 16 条。
- 希望做完时在指针旁边看到自己起的名字，例如「复制」。名字留空则什么都不显示。
- 终端里的复制可以录成这个终端实际使用的组合。Ptyxis 里是 Ctrl+Shift+C。

发出去的是录下的那组快捷键。当前应用会照常处理它：有选中文字时 Ctrl+C 会复制，没有选中时只是按下这组键。

## 运行环境

要同时满足两件事：当前会话是 Wayland，GNOME Shell 的大版本在 45 到 50。

扩展声明了 `"shell-version": ["45", "46", "47", "48", "49", "50"]`。这几代用的是从 Shell 45 开始的模块导入，轨迹用 `St.DrawingArea` 的 Cairo。46 到 50 的官方移植说明里，扩展入口没有不兼容改动。同一大版本里的小版本不用再各写一次，所以 50.0、50.1、50.2 都算 Shell 50。本机登录跑过的是 Ubuntu 26.04.1 和 Shell 50.1。45 到 49 按这套接口做了适配，还没有在那些版本上登录验收。

| 系统 | 桌面 | Wayland | 这份程序 |
| --- | --- | --- | --- |
| Ubuntu 22.04 | GNOME Shell 42 | 可以登录 Wayland | 对不上。早于 Shell 45 的模块写法 |
| Ubuntu 24.04 | GNOME Shell 46 | 默认就是 Wayland，登录界面仍可以选 Xorg | 声明已覆盖。本机没登录过 Shell 46 |
| Ubuntu 24.10 | GNOME Shell 47 | 可以登录 Wayland | 声明已覆盖。本机没登录过 Shell 47 |
| Ubuntu 25.04 | GNOME Shell 48 | 可以登录 Wayland | 声明已覆盖。本机没登录过 Shell 48 |
| Ubuntu 25.10 | GNOME Shell 49 | 桌面会话只有 Wayland | 声明已覆盖。本机没登录过 Shell 49 |
| Ubuntu 26.04 及其 26.04.x | GNOME Shell 50.x | 桌面会话只有 Wayland | 符合。本机跑过 50.1 |
| 下一个开发版（当前软件源里的 Shell 51） | GNOME Shell 51 | Wayland | 大版本超出 45–50。要先核对接口再声明 |

鼠标识别走 Linux 的 evdev 和 logind，不绑定某一家的版本号。别的发行版也看这两件事：GNOME 会话是 Wayland，Shell 大版本在 45 到 50。下面是 2026-09 软件源里的版本。滚动发行版升到 Shell 51 之后就离开这一段。这些系统还没有在这里跑过。

| 发行版 | 现在的 GNOME Shell | 这份程序 |
| --- | --- | --- |
| Fedora 43 | 49.x | 声明已覆盖。本机没登录过 |
| Fedora 44 | 50.x | 声明已覆盖。Workstation 的 GNOME 会话是 Wayland |
| Fedora 45、Rawhide | 51 | 超出 45–50 |
| Debian 12 | 43 | 早于 Shell 45 |
| Debian 13 | 48.x | 声明已覆盖。本机没登录过 |
| Debian testing / unstable（2026-09） | 50.x | 声明已覆盖 |
| Debian experimental | 51 | 超出 45–50 |
| Arch Linux 正式源 extra | 50.x | 声明已覆盖。GNOME 会话是 Wayland |
| Arch 的 gnome-unstable | 51 | 超出 45–50 |
| openSUSE Tumbleweed | 50.x | 声明已覆盖 |
| openSUSE Leap 16.0 | 48.x | 声明已覆盖。本机没登录过 |
| openSUSE Leap 15.6 | 45.x | 声明已覆盖。本机没登录过 |

KDE Plasma、Cinnamon、Xfce、COSMIC 没有这份 GNOME 扩展，Shell 对不上，扩展不会被启用。Linux Mint 默认是 Cinnamon，不是这份 Shell。

启动脚本看到 `XDG_SESSION_TYPE` 不是 `wayland` 会直接退出。Shell 50 起，GNOME 自己也不再提供 Xorg 桌面会话。

动作针对原生 Wayland 窗口，例如 GNOME 文本编辑器。XWayland 窗口只能当作补充。鼠标要是一只相对位移设备，启动时用 `--device` 指定，一次只接管这一只。演示是前台进程。进程在跑、扩展连着、会话未锁定，并且面板上没有暂停时，快捷键才会发出。

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

改规则：另开一个终端，运行 `./target/debug/strokelet settings`。点「添加」或列表里的一条规则。一条规则选「画出轨迹」或「鼠标组合」（先按下的那个键是起始键，左键、右键、中键或侧键都可以，再按另一个键），再在「快捷键」旁点「录制」。屏幕上显示可以填一个名字，例如「复制」；留空则识别成功后不显示文字。点「完成」回到列表，再点「保存」。轨迹触发键可以选左键、右键、中键或侧键。演示正在运行时，按住轨迹触发键在屏幕上画一笔即可，不用先暂停。演示没开时，画轨迹会短暂独占鼠标。保存后写入 `~/.config/strokelet/gestures.json`；如果演示正在运行，会让它重新读这份文件。没有配置文件时，默认仍是右键直线上划 Ctrl+C。以前只写了方向的配置还会当成直线用。旧的鼠标组合如果没写起始键，仍表示按住当时的轨迹触发键。

暂停：点面板右上角的一笔标记，打开「暂停」。停止：等超时，或在运行终端按 Ctrl+C。卸载：`scripts/remove-extension.sh`，它只删除带本项目标记的扩展目录。

## 调试

不用每次都注销。

- 改 Rust 后重新 `cargo build`，再跑 `scripts/run-demo.sh`。Ctrl+C 会放开鼠标并删掉 socket。
- 改 `overlay.js`、`indicator.js`、`transport.js` 后运行 `scripts/reload-extension.sh`。它会把扩展复制进用户目录，再让当前 Shell 关掉并重新打开。
- 只有第一次安装，或者改了 `extension.js` 本身，才需要注销一次。GNOME 50 不能在 Wayland 上热重载扩展入口文件。

默认阈值在识别规格里：起步 12 counts，上划至少 80 counts，最长 2500 ms。轨迹要移动约 12 个屏幕像素后才显示，线宽 3。

生效文档见 [docs/index.md](docs/index.md)。当前任务见 [TASKS.md](TASKS.md)。
