# 局域网联机模块说明

## 模块目标

- 在保留本机输入响应能力的前提下，增加局域网内的在线发现与输入同步。
- 远端消息只用于驱动远端展示状态，不得进入本机系统输入、快捷键或窗口控制链路。

## 当前实现边界

- Rust 后端联机核心位于 `src-tauri/src/core/lan.rs`。
- 本机键盘/鼠标事件在 `src-tauri/src/core/device.rs` 中转发到联机模块。
- 本机手柄事件在 `src-tauri/src/core/gamepad.rs` 中转发到联机模块。
- 前端联机状态集中在 `src/stores/lan.ts`，联机生命周期封装在 `src/composables/useLanSync.ts`。
- 本地与远端猫共享实例化的 Live2D 渲染控制器，公共能力位于 `src/utils/live2d.ts`。
- 键位映射与按键贴图切换规则统一收敛在 `src/utils/modelInput.ts`。
- 偏好设置中的联机开关、昵称、端口和状态展示位于 `src/pages/preference/components/general/index.vue`。
- 远端猫渲染组件位于 `src/components/remote-cat/index.vue`，由主窗口按远端客户端列表动态渲染。

## 协议约定

- 当前协议使用 UDP 广播，默认端口为 `47832`。
- 消息固定包含：`version`、`clientId`、`nickname`、`messageType`、`sequence`、`timestamp`、`payload`。
- 当前消息类型为：`hello`、`heartbeat`、`input`、`leave`。
- 当前协议版本为 `1`；未知版本或未知消息类型应直接忽略。
- `sequence` 用于去重和乱序保护；同一客户端的旧消息不得覆盖新状态。

## 输入状态约定

- 同步范围仅包含输入状态：
  - 键盘按下/释放
  - 鼠标按键按下/释放
  - 鼠标坐标
  - 手柄按钮值
  - 手柄摇杆轴值
- heartbeat 必须携带当前输入快照，用于丢包后的状态恢复。
- 不同步文本内容、剪贴板、窗口标题、进程信息、文件路径等隐私数据。

## 稳定性约束

- 需忽略本机自己广播的消息。
- 需限制鼠标移动和手柄轴广播频率，避免高频消息拖垮局域网或 UI。
- 需限制单条消息大小和远端客户端数量。
- 需在远端客户端超时后清理对应状态。

## 后续扩展注意事项

- 如果继续实现“多只远端猫”渲染，应优先复用现有模型/输入映射规则，避免在前端出现两套独立动作逻辑。
- 远端输入只能驱动 `src/components/remote-cat/index.vue` 内部的展示状态，不得接入本机输入监听、快捷键、窗口控制或系统调用链路。
- 如果调整协议字段、消息类型、端口默认值或前后端事件名，必须同步更新本文档与相关 checklist。
