# 俯视探索 Demo 架构

本项目是单 crate 的 Bevy 游戏。依赖保持 `client → gameplay → simulation`，客户端可以直接读取模拟状态。
旧 MOBA 实现、资源和活跃规范已移出当前版本，完整历史在 `lol` 分支；Trellis 归档只作历史参考。

## 状态所有权

| 模块 | 唯一负责的状态 / 行为 |
|---|---|
| `simulation/movement.rs` | `Body` 权威位置、前一固定帧位置；`MoveIntent` 世界平面移动意图 |
| `simulation/geometry.rs` | `CollisionWorld` 静态扩张 AABB 扫掠与滑动 |
| `gameplay/mod.rs` | `Level`、`Prop`、`Player`，关卡和玩家创建 |
| `client/controls.rs` | `ViewYaw`，设备输入转世界方向；捕获失效时停止意图 |
| `client/scene.rs` | 几何网格、材质、灯光和插值后的显示 `Transform` |
| `client/camera.rs` | 从相同显示位置与朝向派生跟随镜头 |
| `web/input.js` | 浏览器锁定确认、焦点、鼠标位移单位与一次性消费 |
| `web/startup.js` | 在 WASM 加载前记录成功 / 失败，给晚订阅者重放 |

`Body` 字段私有，只提供读取；客户端只写 `MoveIntent`，不能用模型 `Transform` 反写权威位置。
`Level.props` 同时生成模型尺寸与 `SolidBox`，避免两份障碍坐标漂移。地面边框内侧与关卡边界一致。

## 一帧中的顺序

1. Startup 顺序执行：玩法装配 → 三维场景 → 相机 → 通知 Web 引擎就绪。
2. Bevy 输入更新后，在 `RunFixedMainLoopSystems::BeforeFixedMainLoop` 采样控制。
   鼠标位移直接改 `ViewYaw`；WASD 按该朝向换算后提交 `MoveIntent`。
3. `FixedUpdate` 以 60Hz 推进运动和碰撞，保存前后两次权威位置。无论显示帧执行零个或多个固定步，输入只采样一次。
4. Update 顺序执行场景同步和相机跟随。模型用 `Time<Fixed>::overstep_fraction()` 插值，
   相机读取同一个显示根位置，朝向使用当前 `ViewYaw`，不追加平滑延迟。
5. Bevy 完成全局变换传播和渲染。

## 控制与构图

世界平面用 `Vec2(x,z)`，向前为局部 `-Z`；`Quat::from_rotation_y(yaw)` 同时决定前进和横移方向。
鼠标右移令 yaw 减小，以 `0.0025 rad / CSS px` 直接累计，绝不再乘帧时间。
移动意图长度不超过 1，速度为 6m/s。人物使用半宽 0.35m 的保守方形占地，扫掠可防长步穿透，
去掉法线方向的剩余位移保留沿墙滑行；地图边界也约束整个占地。

固定相机距离：相对人物身体中心后方 12m、上方 18m，望向身体中心前方 4m，垂直 FOV 50°。
人物身体中心为显示根上方 0.9m，实际投影在屏幕高度约 65%，不通过偏移人物模型来伪造构图。

## 浏览器边界

`startup.js`、`input.js` 必须同步安装在 Trunk 生成的模块之前，脚本不依赖 DOM 已创建。
`window.kivaInput` 只暴露 `isActive(): boolean`、`takeLook(): number`、`keys(): number`、`ready(): void`。
`takeLook()` 返回并清空本显示帧水平位移；引擎在启动完成后调用 `ready()`。
只有收到 `pointerlockchange` 确认且页面有焦点才能操作，Promise 完成不代表捕获成立。
Esc 解锁或失焦会清累计，失焦后必须重新点击，延迟到达的旧请求不能恢复控制。

未锁定时从可信鼠标的 `clientX` 与 `movementX` 学习 CSS 单位增益；DPR 只用于判定校准是否失效，
不直接用于除法。合并事件与父事件二选一，排除窗口边缘截断样本。浏览器保留系统鼠标加速。
Web 桥记录有效捕获期间的可信 WASD 按键，keys() 返回 W=1 / S=2 / A=4 / D=8 的位掩码；失锁清零，
忽略新的重复 keydown，防止快速失焦再捕获重放旧按键。原生按 WindowFocused 事件消费中间的失焦，
不能只看最终 Window.focused。原生版使用 Bevy 输入及 `CursorOptions`，Windows 使用 Confined，其他平台使用 Locked。

## 扩展原则

新游戏规则放在模拟层，玩法层定义关卡和装配，客户端负责意图和表现。添加会改变位置的机制时，
仍通过模拟更新 `Body`；修改障碍时以 `Level` 数据为源。现阶段不引入为未来战斗、联网或物理准备的空框架。
新增方向、碰撞或时序行为应有无窗口回归，Web 生命周期使用实际桥接脚本的 Node 测试。
