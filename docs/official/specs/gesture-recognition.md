---
title: Strokelet 手势识别规格
status: active
version: 1.0.0
updated: 2026-09-25
owner: 待确认
topic: gesture-recognition
---

# Strokelet 手势识别规格

## 输入与初值

在每个 SYN_REPORT 帧内合并 REL_X/REL_Y，按相对位移 counts 计算；向上为负 y。counts 不是屏幕像素，用户所选鼠标的 DPI 会影响阈值，实机验收时必须校准。

| 参数 | Demo 初值 | 含义 |
|---|---:|---|
| start_counts | 12 | 超过此最大离起点距离后进入绘制状态 |
| click_slop_counts | 12 | 普通右击允许的最大离起点距离 |
| min_up_counts | 80 | 有效上划的最小净上移 |
| max_duration_ms | 2500 | 手势从右键按下到松开的最大时长 |

这些值是可调命令行初值，尚未经目标鼠标校准。

## 状态与结果

- Idle：等待右键按下；无活动手势时释放返回 None。
- Pending：暂存右键，累计帧位移、总路径长度和最大离起点距离。最大距离严格大于 start_counts 时转为 Drawing；从未进入 Drawing 时，最大距离小于等于 click_slop_counts 的释放返回 RightClick，其余返回 Cancel。
- Drawing：继续累计轨迹，不允许退回 RightClick。松开时只有全部上划判据满足且未超时才返回 Copy，否则返回 Cancel。
- CancelledUntilRelease：滚轮、额外按钮、SYN_DROPPED、连接/会话失效或 Drawing 超时后进入此状态；非右键事件照常透传，右键释放返回 Cancel，然后回到 Idle。
- 每次右键按下最多产生一次决策；重复释放返回 None。取消、超时或断连不能产生 Copy。

静止或始终处于 click_slop_counts 内的右键在释放时仍按普通右击处理；max_duration_ms 约束已进入 Drawing 的手势。

## 上划判据

以按下位置为原点，释放终点的 dy 必须小于等于 -min_up_counts；整段轨迹相对起点的最大横向偏移不得超过 `max(12, -dy * 0.30)`；净上移除以逐帧累计路径长度须至少为 0.85。路径长度为各帧合并位移向量长度之和，不用终点距离代替。

判定时还必须检查累计最大离起点距离、单调计时所得时长和取消状态；只检查终点会将下上往返误判为普通右击。79/80 counts、2500/2501 ms、横向偏移和直线度边界均需有可回放测试。

本规格只产生 RightClick/Copy/Cancel 决策，不直接操作设备或剪贴板。Copy 决策进入实际输出前还须经过 [架构文档](../design/strokelet-demo.md) 的会话与修饰键检查。
