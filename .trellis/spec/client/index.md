# 客户端

## Scope / Trigger

`src/client/`、`web/`、`index.html` 拥有输入、程序化表现和镜头，修改控制 / 构图 / Web 启动时阅读。

## Signatures

- `controls::ViewYaw::yaw()` 供 scene 和 camera 读取；鼠标样本只能由 controls 消费一次。
- `controls::AimDistance` 是客户端落点选择状态（默认 6m），随客户端插件初始化；最终落点仍由模拟校验。
- `stickman::spawn(world, owner, body_material, eyes_material)` 为玩家 / 敌人共用的纯显示构造器，创建 RigPose 和骨骼附件，返回可关联受击变色的身体部件。
- `AnimationLibrary::{bone,duration,sample,model_pose}` / `blend` 采样随包骨骼资产；`RigPose::new(&AnimationLibrary)` 保存局部混合及模型空间姿态。
- `stickman::{attach_weapon,sync_attachments}` 绑定右手腕，并在姿态采样后同步几何；WeaponMode 分别校正近战柄轴和远程枪轴。
- `scene::sync_player` → 敌人显示根 / 武器显隐 → `animation::animate` → `stickman::sync_attachments` → HUD → `camera::follow` 在 Update 链式执行。
- `ActorView { owner }` / `PlayerView` / `PropView { owner }` 标记纯显示根；输入、战斗身份留在物理实体。
- Web 桥 `window.kivaInput.isActive(): boolean` / `takeLook(): number` / `takeAim(): number` / `keys(): number` / `takeActions(): number` / `ready(): void`。
- wasm-bindgen 绑定在 `client/web.rs`，原生不需要 JS；加载观察器为 `window.kivaStartup.subscribe(callback)`。
- `window.kivaHud.render(persona, weapon, healthFraction, health, combo, cooldown, context)`；绑定在 `client/hud.rs`，只有显示方向。

## Contracts

输入在 `BeforeFixedMainLoop` 采样，W/S 沿视角前后、A/D 横移；斜向归一化。
鼠标右移令 yaw 减小；0.0025 rad/CSSpx，不乘 dt，不重复除 DPR。垂直移动不改变镜头俯角，
只在火炮远程且未跃进时按 0.012m/CSSpx 调整落点距离（默认 6m，限制 1–8m），其他模式消费后丢弃。
scene 根据 Body 的三维脚底前后快照和 fixed overstep 插值，camera 读取同一 PlayerView 位置。
物理根与显示根独立，客户端不写物理实体的 Transform / Position；箱子通过 PhysicsPose 插值中心位置和旋转，绑带挂在同一 PropView 下。
`SimulationActive=false` 时所有角色、血条和标记统一取当前快照，避免暂停时插值循环抖动；朝向不延迟平滑。
相机：FOV 50°，身体中心为根位置 + Y*0.9，相机在后方 3.5 / 上方 5；后退距离固定为高度的 0.7 倍，
调整高度时同比缩放以保持俯角。根据透视投影推导俯角，
使身体中心投影在水平 50%、从顶部算垂直 2/3 的位置。
网格 / 材质复用；使用随包 CC0 动画资源，无运行时网络资源加载，移动权威在模拟。

Web startup / input / hud 三个 head 脚本先于 Trunk 模块同步安装，DOM 就绪后才绑定元素。
捕获必须收到 pointerlockchange 且文档有焦点，requestPointerLock Promise 完成不算确认。
失锁清水平 / 垂直累计；失焦失效整个请求，重新点击才能恢复。takeLook / takeAim 各返回并清空，合并事件和父事件不能重复计数。
Web keys() 的 u32 位掩码为 W=1 / S=2 / A=4 / D=8 / sprint=16 / block=32，仅捕获期间可信非重复输入建立按住状态；
失锁清零，keyup 释放。原生消费 WindowFocused 的中间失焦事件，避免同帧失焦恢复遗漏清理。
自由鼠标用 clientX / movementX 校准 CSS 增益，丢弃边缘截断样本；DPR 改变清校准，不盲目用它换算。
takeActions() 一次返回并清空 jump=1 / dodge=2 / attack=4 / skill=8 / combo=16 / switch=32 / pair1=64 / pair2=128 / target=256 / crouch=512。
KeyC 是切换指令：完整按压按 XOR 奇偶性锁存，keyup 不取消，重复按住不能多次切换；原生 C 同样消费有序 KeyboardInput。
Rust 将离散位提交到 Intent 锁存，直到真实固定步消费；失锁清移动 / 动作 / 战斗意图并暂停模拟。
Shift 位表示物理按住而非已进入疾跑；聚合左右 Shift 的首次按下另提交一次 dodge 位，Ctrl 不再绑定闪避。
原生按事件顺序消费当前窗口 KeyboardInput，不能只比较相邻帧 pressed 快照，否则会漏掉帧间松开再按下；
左右 Shift 重叠按住不产生第二次闪避，重复事件、捕获首帧、失焦与其他窗口事件不能建立输入。
`ActionIntent::set_movement_held(held & 15 != 0)` 传递物理方向键事实，不能用归一化方向是否为零代替。
失活只清理 Intent；MotionState 仍只读，疾跑取消请求由恢复后的固定步消费。
滚轮以 160ms 安静间隔区分手势；捕获用点击不同时触发普攻；阻止默认 Tab / 空格 / 滚轮等行为只在控制激活时执行。
战斗显示复用网格和材质，按 CombatFeedback 单调序号消费，临时特效到期 despawn；HUD 仅 textContent 和 progress.value。
火柴人使用 act 的骨架比例（待机头顶约 1.73m），手部挂点由骨骼动态计算；模型替换不能反写 Body 或改变相机锚点。
资产为 `assets/animations/stickman.anim`（23 骨骼 / 19 动画），保留同目录 CC0 许可证；`include_bytes!` 嵌入，显式小端解码。
身体姿态通过 ActorView.owner 读取 Body 的实际固定步位移、MotionState、Health 和 Combatant，动画反馈按物理 CombatEvent.actor/sequence 归属与去重。
禁止把动画根位移叠加到权威位置；空中不重复抬升骨盆。暂停冻结姿态、尸体保留可见播放 Death01，复活清理旧姿态。
跳跃姿态读取 MotionState::jumps_used()，第二次起跳重启空中姿态计时并使用 NinjaJump_Idle_Loop；落地继续播放 Jump_Land。
蹲姿和滑铲根据 MotionState 只读状态选择，0.12s 平滑权重调用 act 的 crouch_pose 腿部 IK，下移骨盆 0.33m 且保持脚掌和腿长。
局部混合缓存必须保存 IK 前姿态，对副本做蹲姿 IK 后生成模型空间骨骼，避免蹲起过渡重复降低骨盆。
地形 Mesh 与 Collider 消费玩法层同一份原始顶点 / 三角形。目标圈、瞄准与防御标记使用支撑表面位置 / 法线，不能固定世界 y=0.05。
地面攻击与敌人蓄力特效从真实事件位置向下查询表面并匹配法线，旋斧保留空中事件位置；表面查询排除角色胶囊和传感器，未命中时不退回世界零高度。
外框固定 16:9，按窗口最大可容纳面积缩放，最小留白 12–24px；新拟态使用参考下拉框的 3px 双向阴影和 1px 描边，随系统明暗。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 未就绪 / 未捕获 / 失焦 | 清持续状态和离散请求，停止全部 Intent，水平累计返回 0，暂停模拟 |
| 捕获失败 | 显示重试入口，不激活控制 |
| 旧捕获请求在失焦后成功 | 立即释放，不恢复控制 |
| 已确认捕获后旧 Promise 报错 | 忽略 |
| 非有限 / 非可信 / 非鼠标样本 | 不产生转向 |
| WASM 失败 / 无 WebGPU | 加载失败提示与重载按钮；真正 ready 可覆盖早期失败 |
| HUD 在 DOM 就绪前收到状态 | 保存最新快照，就绪后重放；不注入 HTML |
| 一帧经过多个固定步 / 效果到期 | 全部新序号各消费一次；回收临时实体，不新增网格 / 材质 |

## Good / Base / Bad Cases

Good：边走边转向，人物及镜头读取相同 yaw / 显示位置，无跟随拖尾。
Base：点击进入、WASD 行走、Esc 释放、点击继续。
Bad：先 await 捕获再假定锁定、把 movementX 乘 dt、把垂直位移当俯仰。

## Tests Required / Quality Check

`make check`、WASM Clippy；入口改变再执行临时目录 Trunk 构建。
Rust 测试验证相对方向、斜向速度、分帧转角一致和真实透视投影；Node 直接执行桥脚本，验证加载重放、
捕获确认 / 失败 / 失焦、旧请求、动作一次消费、滚轮手势、合并事件和缩放单位；HUD 验证早期重放、数值约束和安全文字。
客户端无窗口场景测试验证装配、武器切换显示、反馈消费及特效回收和资产复用；真实画面 / 手感不由这些测试代替。
Shift 输入回归覆盖完整点按先于采样结束、左右键重叠 / 换手、帧间松开再按下，以及捕获 / 重复事件隔离。
火柴人无窗口测试覆盖杆件端点连接、跨角色网格复用、父子关系及不改变物理姿态。
骨骼测试覆盖全部动画有限性 / 无根位移、走跑跳跃等实际姿态变化、事件归属、武器跟手、死亡 / 复活 / 暂停及不改模拟高度。
C 回归覆盖短按 / 松键保持、完整双按奇偶性、重复过滤和失焦清理；蹲伏 IK 验证多片段的脚掌位置 / 旋转 / 腿长和武器握持。

## Wrong vs Correct

错误：独立平滑相机、把显示 Transform 写回物理 Position，或将 view 实体 ID 当作战斗 actor。
正确：JS 只管理输入边界，物理实体消费 Intent；owner 关联的显示根读取快照，模型和相机共用显示位置。
