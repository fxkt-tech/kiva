# 三维物理

## Scope / Trigger

修改 `simulation/physics.rs`、角色控制、支撑、道具碰撞、空间查询或固定步顺序时阅读。
Avian 0.8 使用 `3d` / `parry-f32` 最小特性；原生和 WASM 共用模拟，不依赖窗口或渲染 Mesh。

## Signatures

- `Body::new(Vec2, speed, radius)` 为平面出生快捷构造，`Body::at(Vec3, speed, radius)` 为三维脚底出生位置。
- `Body::{world_position,interpolated_world,velocity}` 返回物理求解后的三维脚底快照；`position/interpolated/step_displacement` 明确为 x/z 投影。
- `Body::teleport_to(Vec3)` 排队权威重置；`teleport_actor(&mut World, Entity, Vec3)` 立即重置姿态、速度、动作和显示快照。
- `spawn_box(&mut World, center: Vec3, size: Vec3, dynamic: bool) -> Entity`，size 为完整尺寸。
- `spawn_terrain(&mut World, vertices: Vec<Vec3>, triangles: Vec<[u32;3]>) -> Entity`。
- `PhysicsPose::{position,rotation,interpolated_position,interpolated_rotation}` 是道具中心姿态的只读快照。
- `SurfaceHit { entity: Entity, position: Vec3, normal: Vec3 }` 使用世界坐标。
- `surface_below(&SpatialQuery, origin, max_distance, excluded) -> Option<SurfaceHit>`；无命中必须保留 None。
- `MotionState::{is_grounded,surface,support_normal,jumps_used}` 暴露接触状态；高度 / 竖直速度适配器只读求解结果。

## Contracts

- 物理组件 Position / Rotation / 速度是唯一运动权威，Body 和 PhysicsPose 只缓存结果。不得在客户端修改物理实体 Transform，也不得重新积分一份高度。
- 角色为直立动态胶囊，站立总高 1.73m、蹲伏 1.40m，物理中心与脚底相差当前半高。姿态变化保住脚底，起身先检查净空。
- 控制目标沿支撑面切向，以有限加速度追踪；实际位移由接触求解产生。默认重力沿 -Y 为 18m/s²。
- 支撑根据脚底接触 / 小距离探测、世界法线与相对分离速度判定，包含休眠接触。绝对海拔不能用于判断接地。
- 动态支撑点速度包含 `linear + angular × (point - center_of_mass)`；停止输入不能抵消脚下物体的移动。
- 有效起跳立即离地，脚下残留探测不能补次数；两次跳跃均提供 6.4m/s 起跳速度，真正落地才恢复次数。
- 高速胶囊和动态箱体的 CCD 过滤包含 Dynamic；不能依赖只扫 Static / Kinematic 的默认配置。
- `CollisionWorld` / `SolidBox` 只兼容旧无窗口夹具的几何装配。产品关卡直接装配三维实体，不以旧二维 slide 结算位移或遮挡。

### 调度与暂停

Avian 使用 `PhysicsPlugins::new(FixedUpdate)`，保持固定 60Hz。动作 / motor 在 Prepare 前，物理求解后更新当前查询与支撑 / 快照，再结算战斗与瞬移清理。
发布依赖本步空间查询的结果时必须排在 `SpatialQuerySystems` 之后；`PhysicsSystems::Last` 并不代表组件化查询已完成。

`SimulationActive` 共同门控 motor、`PhysicsSystems::StepSimulation`、快照发布、战斗及 AI。首次暂停不能只调用物理时钟 pause 后假定旧 delta 已清空。
恢复只推进当前固定步，不补算暂停期间的重力 / 冷却。客户端暂停时直接取 current，避免插值循环。
狙击 HitStop 与失锁共用整条规则门控：固定步开头确定 blocked_tick 后，本步门控保持不变；战斗请求下一步开始停顿，不能跳过命中同一步的瞬移清理。0.04s 预算在 60Hz 向上取整为 3 步，失锁不消耗预算，不修改物理时钟倍速或速度；显示在停顿时使用 current。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 非有限 / 非正尺寸、速度、出生点 | 内容构造 assert |
| 无支撑、移走脚下箱子、坡度超过上限 | 离地 / 下滑，不把高度钳回地面 |
| 上升时仍探测到脚下表面 | 不重新接地，不补跳跃次数 |
| 头顶净空不足 | 保持蹲伏，不扩大胶囊穿入实体 |
| 无可用落点 | 返回 None，不能伪造世界 y=0 命中 |
| 暂停 / 恢复 | 物理姿态、速度和额度冻结，无首次漏步或积累补算 |
| 瞬移 / 复活 | 权威中心、速度、动作、支撑和前后快照一起重置 |

## Good / Base / Bad Cases

Good：站在非零海拔或运动箱子上，支撑与相机一致；走出边缘后自然下落。
Base：平地移动、二段跳、落地恢复、推箱及暂停恢复。
Bad：只把显示根升高、以 y==0 接地，或者模型插值反馈到 Position。

## Tests Required

`make check`、WASM Clippy、临时目录 Trunk 构建。无窗口夹具运行真实完整 FixedUpdate / Avian 求解，按物理容差断言。
手动运行 schedule 的 App 夹具应在全部插件安装后调用 `app.finish()`；Avian 的空间查询 / 诊断资源在插件 finish 中建立。
覆盖箱顶落地、支撑丢失、上下坡、山顶与陡坡、头顶净空、两次额度、动态箱子受力 / 倾倒 / 堆叠、休眠支撑、移动 / 转动随动和高速不穿透。
首次暂停、暂停中的落体 / 箱体、恢复无补算、瞬移与复活分别验证；外部 `tests/physics_terrain.rs` 使用真实庭院地形。
构建和代码测试不替代浏览器画面与手感验收。

## Wrong vs Correct

错误：`view.Transform` 回写物理、按解析曲线强制落到 y=0，或用出生坐标查询移动箱子遮挡。
正确：意图驱动权威胶囊，Avian 求解后发布只读快照；视线和落点查询当前三维碰撞世界。
