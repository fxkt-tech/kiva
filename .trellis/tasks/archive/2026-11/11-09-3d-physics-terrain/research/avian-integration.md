# Research: Avian 三维物理接入契约

- Query: Avian 0.8 与 Bevy 0.20.1 的版本、特性、固定步、暂停、角色控制、动态支撑、三维查询、CCD 和显示权威边界。
- Scope: mixed；只研究，不修改产品代码、Cargo 清单或现有规范。
- Date: 2026-11-09

## Findings

### 1. 版本与最小特性

Avian **0.8.0** 正式 tag 的 `crates/avian3d/Cargo.toml:73` 依赖 `bevy = "0.20"`，与当前 `Cargo.toml:8` 的 0.20.1 匹配。建议公共依赖：

```toml
avian3d = { version = "0.8", default-features = false, features = ["3d", "parry-f32"] }
```

- 0.8 的单精度是默认数值类型，**没有旧版 `f32` 特性**；`parry-f32` 会启用默认 Collider 与空间查询。
- 相同特性用于原生和 WASM。先关闭 Avian 默认的 `parallel`，避免给浏览器额外引入并行要求；现有 Bevy 原生 `multi_threaded` 配置可以保留。
- 本次显式构造胶囊、箱体和地形顶点 / 索引，不需要 `collider-from-mesh`、场景加载、picking、debug-render 或关节求解特性。
- 不开 `f64`，当前 50m 庭院没有大世界精度需求。固定步不等于承诺跨架构逐 bit 确定性。

证据：[0.8.0 Cargo.toml](https://github.com/avianphysics/avian/blob/v0.8.0/crates/avian3d/Cargo.toml)。本轮没有修改依赖、生成 lock 或执行编译，原生 / WASM 实编译仍是实施首关。

### 2. 当前必须替换的所有权

| 当前代码 | 接入影响 |
|---|---|
| `src/simulation/movement.rs:6` 的 Body 保存 Vec2 权威位置 | 改成完整三维物理结果的只读快照；不能继续自行积分平面位置 |
| `src/simulation/movement.rs:99` 调用二维 slide 写位置 | 由速度 / 冲量驱动物理胶囊，求解后采样位置 |
| `src/simulation/actions.rs:125` 保存独立 height / vertical_velocity | MotionState 保留动作、次数、计时；高度与竖直速度来自物理结果，取消第二套重力积分 |
| `src/simulation/actions.rs:152` 以 height==0 判断接地 | 改成接触支撑判定，允许站在非零海拔、箱顶和坡面 |
| `src/simulation/actions.rs:321` 火炮直接按弧线写高度并归零 | 改成受碰撞约束的发射 / 受控速度；结束不能强制回世界 y=0 |
| `src/gameplay/mod.rs:71` 丢掉 Prop.size.y | 静态地形与动态箱体各装配真实三维 collider |
| `src/simulation/combat.rs:336` 的 visible 使用二维 slide | 使用同一物理世界的三维射线，不维护旧的静态遮挡副本 |

Body 的 previous/current 可以保留为渲染用派生快照，但它们只有一个求解后写入点，不再是可独立移动的状态。瞬移 / 复活必须同时重设物理位置、速度和 previous/current，避免旧位置插值。

### 3. 明确分开物理实体与显示实体

**推荐：无渲染的物理中心实体 + 关联 owner 的独立显示根。**

- 角色物理实体保留 Player / Enemy、Health、Combatant、Intent，拥有 Avian `Position`、`Rotation`、`LinearVelocity`、`RigidBody` 和 Collider；它是战斗事件中的 actor Entity。
- 显示根使用 `ActorView { owner: Entity }` 一类关联组件，包含纯显示 Transform、骨架和模型子实体；不带 RigidBody / Collider，也不挂在物理实体之下叠加两份世界位移。
- 箱子同样采用物理中心与 `PropView { owner }`，其绑带等装饰是显示根的局部子实体，随箱子完整旋转；不能仍按 Level 初始坐标生成一批互不关联的 mesh。
- 摄像机跟随同一显示根；输入 / 伤害 / 目标引用仍使用物理 actor。RigPose 消费事件时用 view.owner 匹配 CombatEvent.actor，不能把显示实体 ID 当战斗身份。
- 客户端只插值快照并写 view.Transform；不要给物理实体开启另一套显示插值后，再自行插值第二次。

原因：Avian `PhysicsTransformPlugin` 默认 **Transform→Position** 与 Position→Transform 双向同步。当前 `src/client/scene.rs:89–100` 和 `src/client/combat.rs:174–195` 在 Update 修改的恰是 actor 本体 Transform，直接沿用会把插值、脚底偏移和相机 yaw 反馈成物理移动。`scene.rs:249–257` 的骨架装配也必须移到关联的显示根。

物理内部即使自动插入 Transform，也只有物理插件使用；客户端查询应从结构上拿不到可写的物理 Transform。可额外关闭 `PhysicsTransformConfig.transform_to_position`，但这不能代替正确的实体分离与显式 Position/Rotation 初始化。

证据：[0.8.0 physics_transform/mod.rs](https://github.com/avianphysics/avian/blob/v0.8.0/src/physics_transform/mod.rs) 的配置和 Prepare / Writeback 系统；[0.8.0 interpolation.rs](https://github.com/avianphysics/avian/blob/v0.8.0/src/interpolation.rs) 的显示插值行为。

### 4. 身体中心、脚底与蹲起

为 upright 胶囊定义一个集中保存的几何配置：半径 r、总高 H、圆柱段长 H−2r。Avian `Collider::capsule(r, length)` 的第二参是圆柱段长度，不是总高。

```text
physics_center = foot_position + UP * (H / 2)
view_root = lerp(previous_center, current_center, alpha) - UP * (H / 2)
camera_body_center = view_root + 既有身体中心偏移
```

角色锁定物理转动（例如 `LockedAxes::ROTATION_LOCKED`），视觉朝向仍来自 Facing / ViewYaw。胶囊绕 Y 对称，碰撞本体无需每帧跟随鼠标旋转。

蹲起若调整物理胶囊高度，须同步调整中心以保住脚底位置；起身前做净空检测，不能伸进低顶。骨骼 IK 只改变表现，不能驱动物理中心或 Collider 尺寸。脚底 / 中心转换及尺寸调整需要专门回归，避免站上箱子后整体偏高半个人。

胶囊与转动约束用法来源：[0.8.0 dynamic_character_3d](https://github.com/avianphysics/avian/tree/v0.8.0/crates/avian3d/examples/dynamic_character_3d)、[LockedAxes](https://github.com/avianphysics/avian/blob/v0.8.0/src/dynamics/rigid_body/locked_axes.rs)。尺寸数值应以现有 1.73m 模型和 0.35m 占地为起点调校。

### 5. 使用动态角色控制器，有限加速度驱动

动态 upright 胶囊最符合本次已确认的推动 / 碰倒 / 堆叠需求：求解器能在角色、箱子和地形间处理质量与接触。运动学控制器也能实现，但需要额外实现对箱子的推力与支撑传递，并承担无限质量角色把箱子挤飞的风险。

官方 dynamic example 给出了 Dynamic + 胶囊 + 锁转动 + 向下 ShapeCaster 的基本装配，但它只展示最基础移动与跳跃，**不能整段照抄其 Update 调度、逐帧 damping 或固定宽探测距离**。

本项目控制器应以有限加速度追踪期望的地面切向速度，保留重力 / 接触产生的竖直速度；冲量、击飞及碰撞不能被每固定步强制覆写全速度而消失。输入只决定期望速度和动作，实际位移由求解器输出。摩擦、质量、加速度、子步数都需要用角色推箱与箱堆稳定性调校。

参考：[dynamic controller plugin](https://github.com/avianphysics/avian/blob/v0.8.0/crates/avian3d/examples/dynamic_character_3d/plugin.rs)、[kinematic MoveAndSlide 工具](https://github.com/avianphysics/avian/blob/v0.8.0/src/character_controller/move_and_slide.rs)。上述控制器选择及具体 motor 策略属于本项目建议，非 Avian 的现成功能承诺。

### 6. 接地、动态支撑及跳跃恢复

支撑记录至少包含 collider / body Entity、世界接触点、朝向角色的世界法线、支撑点速度。

1. 使用真实接触，加小距离脚底 shape probe 补稳定性。排除自己、传感器及侧壁；只接受 normal·UP 达到可行走坡度阈值、脚底距离在 skin 内的接触。
2. `ContactGraph::contact_pairs_with(entity)` 包含可能仅 AABB 相交的 pair，必须再判 `pair.is_touching()`。`Collisions::iter()` 同时包含 active 与 sleeping 接触；不能只扫描 active 接触，否则箱堆休眠后会丢接地。
3. `ContactManifold.normal` 为 collider1→collider2；若角色是 collider1，支撑方向取负法线，反之取正。`ShapeHitData.normal1` 已是目标表面的**世界空间**外法线，不要再次乘 Rotation。命中子 Collider 时通过 `ColliderOf.body` 找动态支撑刚体。
4. 支撑点速度取 `v_support + angular_velocity.cross(contact_point - world_center_of_mass)`。角色 motor 追踪相对支撑的切向速度；不能强制角色世界水平速度归零，否则会抵消移动 / 转动物体对脚底的传递。
5. 判定落地还需看角色相对支撑的法向速度。相对速度沿支撑法线向外时，不因 probe 仍碰到脚下物体就重设 grounded 或恢复次数。有效跳跃当步明确离地；次数只在后续重新获得有效支撑时恢复，暂停和武器切换不恢复。
6. 站在倾倒箱上时持续重算支撑法线与接触点；超过坡度限制或失去接触后自然下落 / 下滑，角色不跟随箱子倾倒。起跳保留支撑的世界速度，额外起跳量用相对速度计算。

建议自动回归：两层静止箱休眠后仍接地；脚下箱平移 / 旋转时角色正确随动；箱倾斜后脱离；同一步起跳 probe 仍贴近地面也不能恢复二段跳；从箱顶落到斜坡能真正恢复额度。

API 证据：[ContactGraph](https://github.com/avianphysics/avian/blob/v0.8.0/src/collision/contact_types/contact_graph.rs)、[Collisions](https://github.com/avianphysics/avian/blob/v0.8.0/src/collision/contact_types/system_param.rs)、[ContactPair / manifold](https://github.com/avianphysics/avian/blob/v0.8.0/src/collision/contact_types/mod.rs)、[ShapeHitData](https://github.com/avianphysics/avian/blob/v0.8.0/src/spatial_query/shape_caster.rs)、[ColliderOf](https://github.com/avianphysics/avian/blob/v0.8.0/src/collision/collider/collider_hierarchy/mod.rs)。相对速度与落地门槛是本项目须实现的控制器契约。

### 7. 固定步、查询新鲜度和暂停

当前 `src/simulation/mod.rs:33–40` 固定 60Hz，FixedUpdate 顺序为 movement→combat；测试只手动运行 FixedUpdate。建议把 Avian 明确放入同一个 FixedUpdate：`PhysicsPlugins::new(FixedUpdate)`，避免默认 FixedPostUpdate 让 combat 在物理之前读取旧位置。

建议流水线：

```text
保存 previous 快照 / 消费动作 / motor 更新速度
→ PhysicsSystems::Prepare
→ PhysicsSystems::StepSimulation
→ PhysicsSystems::Writeback
→ 空间查询更新（若读取 ShapeHits，则在 SpatialQuerySystems 之后）
→ 支撑与落地状态 / 发布 current 快照
→ combat（射线查询、命中与下一步冲量）
```

Avian 配置 `First → Prepare → StepSimulation → Writeback → Last`，并把这些集合排在 `TransformSystems::Propagate` 前；其组件化 RayCaster / ShapeCaster 查询更新在 `SpatialQuerySystems`，排在 Propagate 后。因此**不能把一个需要本步 ShapeHits 的 grounding 系统塞进 PhysicsSystems::Last**，否则仍读到前一步探测结果。若用直接 `SpatialQuery.cast_shape`，在求解与 collider tree 更新后查询，并用测试锁定同一步刷新行为。

暂停使用同一 `SimulationActive` 对 motor、物理 StepSimulation、支撑 / 快照提交、战斗共同加 run condition。恢复只消费当前固定步 dt，不补算暂停期间时间。

**重要：不能仅调用 Time<Physics>::pause() 就假定首次暂停绝对零步。** 0.8.0 的 `pause()` 只写 paused 标志；`run_physics_schedule` 在暂停时如果旧 delta 非零仍会执行一次 schedule，随后才把 delta 归零。建议直接对 `PhysicsSystems::StepSimulation` 加 `run_if(SimulationActive)`；若还使用物理时钟暂停，暂停转换时也须 `advance_by(Duration::ZERO)` 并有首步冻结回归。不要创建与 SimulationActive 分离的第二套暂停权威。

证据：[PhysicsPlugins::new](https://github.com/avianphysics/avian/blob/v0.8.0/src/lib.rs)、[schedule/mod.rs](https://github.com/avianphysics/avian/blob/v0.8.0/src/schedule/mod.rs)、[schedule/time.rs](https://github.com/avianphysics/avian/blob/v0.8.0/src/schedule/time.rs)、[spatial_query/mod.rs](https://github.com/avianphysics/avian/blob/v0.8.0/src/spatial_query/mod.rs)。

### 8. 三维查询与 CCD

- `SpatialQuery::cast_ray(origin, direction, max_distance, solid, &filter)`：武器 / 目标视线采用实际三维起终点，排除自己及传感器；射线命中的动态箱子位置来自本步物理世界。
- `cast_shape(&collider, origin, rotation, direction, &ShapeCastConfig, &filter)`：角色体积净空、落点能否站立及支撑辅助探测。火炮地面标记先向下取可站立表面，不能用从脚底向前的二维 slide 决定所有落点。
- 静态地形可用显式顶点 / 三角索引构造 collider，不需要从 Bevy Mesh 反向提取碰撞，因此模拟测试可无 renderer。
- Avian 0.8 对快速 Dynamic 默认做针对 Static / Kinematic 的 swept CCD；**默认不扫 Dynamic**。本任务高速闪避 / 跃进也会碰动态箱子，应显式配置 `SweptCcd::new().with_filter(CcdFilter::ALL)` 或选择包含 Dynamic 的过滤集。不要照抄 0.7 的 `include_dynamic` 字段，0.8 已是 `filter / mode / threshold`。
- CCD 是求解器辅助，不取代合理 Collider 厚度、动态凸形体、子步数和墙 / 箱 / 山面实际回归。

证据：[SpatialQuery 方法](https://github.com/avianphysics/avian/blob/v0.8.0/src/spatial_query/system_param.rs)、[0.8.0 CCD](https://github.com/avianphysics/avian/blob/v0.8.0/src/dynamics/ccd/mod.rs)。

### 9. 无窗口测试入口

Avian 本身不要求窗口 / GPU。官方测试使用 `MinimalPlugins`、`TransformPlugin`、`PhysicsPlugins`，并视特性添加 Asset / Mesh / Scene 插件；见 [0.8.0 tests/mod.rs](https://github.com/avianphysics/avian/blob/v0.8.0/src/tests/mod.rs)。

本项目应先做关闭非必需特性的最小原生 / WASM 编译，再建立真正包含 Avian 的无窗口 app fixture，按固定 dt 运行与产品一致的完整 FixedUpdate 流水线。不能仅调用 MotionState helper 来证明接触有效；也不能只手动运行内部 PhysicsSchedule 而漏掉外层 Prepare / Writeback、Collider 装配与查询更新。碰撞误差用物理合理容差断言，替换原二维精确位移 / 墙前停点回归时保留对应用户行为验收。

## Related Specs

- `.trellis/spec/simulation/index.md`：固定步、动作意图、位置所有权和 headless 回归；其中二维扫掠与世界 y=0 的现状需在落地后修订。
- `.trellis/spec/client/index.md`：显示只读、单份插值 / 相机位置、暂停画面、骨骼与物理高度分离。
- `.trellis/spec/gameplay/index.md`：关卡作为几何来源、出生净空、可复现装配。
- `.trellis/spec/guides/cross-layer-thinking-guide.md`、`code-reuse-thinking-guide.md`：输入→模拟→显示所有权与单一状态源。

## Caveats / Not Found

- 本研究以 0.8.0 tag 源码为准；搜索中的 `latest` 某些二级页仍返回 0.7 缓存，尤其 CCD 字段和数值类型别名，实施时不要混用。
- 未运行 Cargo resolution、headless fixture 或 WASM 构建；不能将文档兼容性当成编译 / 运行已验收。
- Avian 没有现成完整角色控制器；支撑随动、坡面粘附、跳跃离地、防贴墙补次数与稳定推箱必须由本项目控制器和回归保证。
- 动态胶囊 + 质量 / 摩擦 / 加速度的具体参数尚需运行调校；不能承诺直接抄官方 basic example 就有合格手感或稳定箱堆。
- 火炮现有到期直接回 y=0 的弧线必须改造；仅增加箱子 Collider 不足以满足本任务。
