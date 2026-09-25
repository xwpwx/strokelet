# Strokelet Demo 验收清单

本清单记录可自动验证的部分，以及必须由用户在真实桌面上完成的部分。模拟事件测试不能代替实机验收。

## 已自动验证

- `make check`：格式、Clippy、58 个测试（手势 30、门禁 6、输入序列 8、协议 13、启动 1）。
- `gjs -m scripts/check-gjs-socket.js`：GJS 能收发一行 JSON。
- `scripts/check-extension-install.sh`：安装脚本拒绝同 UUID 的外来目录；卸载脚本拒绝删除没有本项目标记的目录；`run-demo.sh` 拒绝非 Wayland。

## 等待用户实机验证

以下项目都未执行，不能勾选：

- [ ] 编辑器中选中文本，右键直线上划，轨迹出现且不弹出菜单；到空白处 Ctrl+V 得到原文。
- [ ] Firefox 普通页面重复上述复制。
- [ ] 连续约 20 次有效上划。
- [ ] 左、右、下、短上划、超时、按住修饰键、暂停、扩展断开，都不复制；普通右击仍能打开菜单。
- [ ] 热插拔、锁屏、停止进程后设备与按键都恢复。
- [ ] 实际多显示器布局下轨迹跟手。
- [ ] 扩展启用后轨迹不抢焦点；禁用后没有残线和定时器。
- [ ] 接管前后指针速度差异已记录，且没有改全局鼠标设置。

## 单次上划步骤

1. 确认 Wayland 会话：`echo "$XDG_SESSION_TYPE"` 应为 `wayland`。
2. 以 root 创建运行时目录（只需一次）：`sudo install -d -o root -g "$(id -gn)" -m 0770 /run/strokelet/$(id -u)`。
3. `scripts/install-extension.sh`，然后 `gnome-extensions enable strokelet@local`。首次安装可能要注销再登录；脚本不会自动注销。
4. `cargo build` 后 `strokelet list-devices`，选一个被标成 `accepted` 的相对鼠标。
5. 查会话：`loginctl list-sessions`。
6. `scripts/run-demo.sh --device /dev/input/by-id/... --uid "$(id -u)" --session SESSION`。默认 120 秒，前台运行，不安装服务。
7. 在文本编辑器选中一段文字，按住右键向上直划后松开，再到空白处按 Ctrl+V。
