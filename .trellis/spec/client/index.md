# 客户端

## Scope / Trigger

`src/client/`、`web/`、`index.html` 拥有输入、蒙皮角色/环境表现和镜头，修改控制 / 构图 / Web 启动时阅读。首次加载与进入详见 [startup.md](startup.md)，输入详见 [input.md](input.md)，双角色战斗表现详见 [combat-presentation.md](combat-presentation.md)，指定模型转换／女性驱动／武器格式见 [model-assets.md](model-assets.md)。

永劫／燕云双来源、原生 Humanoid 转换、scope 与音效 cue 见 [naraka-motion.md](naraka-motion.md)。

`?view=model` 为独立的模型／武器／动作验收App，共用真实游戏姿态和挂点，不装配模拟或游戏输入；入口、时间轴、部件显隐与共享计算契约见 [model-viewer.md](model-viewer.md)。

## Signatures

- `controls::ViewYaw::yaw()` 供 scene 和 camera 读取；鼠标样本只能由 controls 消费一次。
- `avatar::spawn(world, owner)` 为玩家 / 敌人共用的纯显示构造器，创建蒙皮和骨骼附件，返回可关联受击变色的身体部件；`EnemyBody(owner)` 标记敌人的身体表面，由战斗表现着色，其余表面由 `avatar::sync` 着色。
- `AnimationLibrary::{bone,duration,sample,model_pose}` / `blend` 采样随包骨骼资产；`RigPose::new(&AnimationLibrary)` 保存局部混合及模型空间姿态。
- `stickman::{attach_weapon,sync_attachments}` 绑定右手腕，并在姿态采样后同步几何；WeaponMode 分别校正近战柄轴和远程枪轴。
- `scene::sync_player` → 敌人显示根 / 武器显隐 → `animation::animate` → `companion::sync` → `avatar::sync` → `stickman::sync_attachments` → `weapons::sync_tether` → HUD → `camera::follow` 在 Update 链式执行。
- `ActorView { owner }` / `PlayerView` 标记纯显示根，与 `EnemyBody` 一起定义在 `client/mod.rs`；客户端模块之间不得环形依赖（`tests/layering.rs`）；输入、战斗身份留在物理实体。
- Web 桥 `window.kivaInput.isActive(): boolean` / `takeLook(): number` / `keys(): number` / `takeActions(): number` / `takeDefenseEdges(): number[]` / `ready(): void`。
- wasm-bindgen 绑定在 `client/web.rs`，原生不需要 JS；加载观察器为 `window.kivaStartup.subscribe(callback)`。
- `window.kivaHud.render(persona, weapon, healthFraction, health, combo, cooldown, context)`；绑定在 `client/hud.rs`，只有显示方向。
- `window.kivaHud.renderLoadout(pair,maximum,attackReady,meleeReady,rangedReady,guardReady,comboReady)` 与 `renderResources(stamina,maximum,meleeCooldown,rangedCooldown,axeAway,defense)`；具体快照契约见 combat-presentation.md。
- `window.kivaEnemies.takeMode(): number`（0=无请求、1=靶子、2=实战）/ `render(mode)`；绑定在 `client/enemies.rs`，固定循环前把选择交给 `gameplay::set_enemy_mode`。
- `window.kivaAppearance.takeAppearance(): number`（0=无请求、1=愁铃客、2=李火旺）/ `render(value)`；绑定在 `client/appearance.rs`，选择仅属于客户端，`appearance::reconcile` 在动画前只替换哥哥身体／服饰，保留当前武器。
- `window.kivaAudio.{playSniperShot(): boolean, playEffect(name): boolean, cueOffset(name): number, setActive(bool)}`；绑定在 `client/audio.rs`，消费已结算攻击／命中／防御和妹妹射击；Shot 及伴随 SniperShot 不重复发声。

## Contracts

输入在 `BeforeFixedMainLoop` 采样，W/S 沿视角前后、A/D 横移；斜向归一化。
鼠标右移令 yaw 减小；0.0025 rad/CSSpx，不乘 dt，不重复除 DPR。垂直移动不改变镜头俯角。
具体桥签名、位掩码、有序右键及原生事件规则见 [输入契约](input.md)。
scene 根据 Body 的三维脚底前后快照和 fixed overstep 插值，camera 读取同一 PlayerView 位置。
物理根与显示根独立，客户端不写物理实体的 Transform / Position。
`SimulationActive=false` 或 `HitStop::is_active()` 时所有角色、血条和标记统一取当前快照，避免暂停时插值循环抖动；朝向不延迟平滑。
普通相机：FOV 50°，身体中心为根位置 + Y*0.9，相机在后方 4.2 / 上方 6；后退距离固定为高度的 0.7 倍，
调整高度时同比缩放以保持俯角。根据透视投影推导俯角，
使身体中心投影在水平 50%、从顶部算垂直 2/3 的位置。
两种联动相机、妹妹独立 RigPose、事件归属和枪口对齐见 [战斗表现契约](combat-presentation.md)。
普通相机和联动均用当前空间查询的 0.15m 球体扫掠避障，保留 0.03m 间隙，排除角色和传感器；只改显示，不反写物理。
序列期间丢弃转向样本，结束无积攒转向。专用冲击只读取实际事件的射击源与命中点。
冲击发光材质使用 StandardMaterial.emissive 且保留 PBR 路径；当前 Bevy 的 unlit=true 路径只输出 base_color，会忽略 emissive，导致爆闪变成暗色实体。
`impact::update` 在骨骼附件之后、相机之前消费专用事件；粒子和镜头后坐在 HitStop 期间继续表现，失锁暂停时冻结，临时实体到期回收并复用共享资产。普通特效系统忽略这两种专用事件。
网格 / 材质 / 纹理复用；使用随包角色二进制和 CC0 动画资源，音频仅加载同源随包文件，移动权威在模拟。

Web mode脚本最先固定页面模式，startup／loader／input／hud／enemies／appearance／audio／viewer先于Trunk模块同步安装，DOM就绪后才绑定元素；游戏专属脚本在查看模式先退出。专用WASM流报告真实下载进度；游戏按钮加载时可见但disabled，Rust观察首轮必要渲染准备与提交后才最终ready。捕获进行中有反馈，确认／失败与恢复保持原输入契约；完整门槛见[startup.md](startup.md)。
音效仅在可信进入点击或画布按下时创建 / 恢复 Web Audio；随包音频异步解码一次，原有程序枪声仅保留兼容接口，实际妹妹枪弓使用已验证永劫 cue。至多 4 个短音效声部及 1 个 BGM 循环（音效增益 0.45、BGM 0.18），暂停立即停止/回收短音效并保存 BGM 播放位置；恢复只创建一条循环。未解锁、不支持或播放失败时跳过，不积压旧声；暂停仍前移反馈游标。原生构建音效为空操作，不新增平台音频依赖。
进入卡片可选择靶子 / 实战，捕获期间 F2 切换；按钮请求在暂停或引擎加载时保留至一次消费，模式权威在玩法层。
外观卡片可选择愁铃客 / 李火旺，捕获期间 F3 切换，选择保存在 `AppearanceSelection`；外观只决定哥哥身体／服饰，不改变 WeaponPair 或武器。按钮请求保留至 Rust 单次消费，JS 当前显示由 Rust render 确认；原生只在有焦点且模拟激活时响应 F3。选择可在玩家显示根创建前保存，暂停按钮仍可换装。哥哥默认李火旺；`Appearance::default()` 仍为愁铃客，供敌人使用。白灵淼仅用于妹妹，不进入 F3 或 Web 选择枚举；换装不影响敌人、妹妹或当前武器。
`AvatarAssets` 保留愁铃客资源，`AvatarAssetCache` 分别缓存李火旺与白灵淼；多次换装／技能复用原句柄。角色自身的 AvatarPart 标记限定身体回收范围，不能清空玩家所有 Children 或重建 RigPose。显示根、物理 actor、动作状态、武器根的 BoneVisual 所属保持不变。
KIVAAV03 增加每材质 doubleSided 和每骨 sourceRestModel，KIVAAV02 仍按原格式读取。源导入姿态可能不同于 bind pose，蒙皮正确性使用原 LBS 数值对照，不强改逆绑定为当前姿态抵消。李火旺骨名与燕云索引独立，身体23结构驱动校准参考臂向并保留自身骨长，源指轨另按三节语义链传递；源动作与附件读取实际皮肤掌框。原 packed/ID 图未经通道证明不能接为颜色或 ORM；面容、发材质和刚性刀穗的美术适配必须在 assets/lihuowang 来源清单中明确。
原生 F2 在有焦点时切换，窗口标题显示当前模式。模式改变后先回收失效敌人的显示树与血条，再用共享网格 / 材质创建新表现。
可信捕获、原生事件、Shift／C、有序右键与取消的唯一输入契约见 [input.md](input.md)。
零固定步保留意图、多固定步一次消费；输入只改变意图与 ViewYaw，不能修改 MotionState 或已承诺技能。
战斗显示复用网格和材质，按 CombatFeedback 单调序号消费，临时特效到期 despawn。
Web HUD 顶部居中为米金边框、暗红填充的锁定敌人长血条；底部居中为玩家生命和状态；右下角为 E／Q／左键／右键／Tab 技能组、当前绑定组与离手提示，底部另有耐力条。完整按键指南留在进入 / 暂停卡片。
目标只通过 Combatant.target() 查询活敌人，不在显示层另选最近目标；无目标、目标死亡 / 被移除或玩家倒地时隐藏并清理旧值。
套装与防御阶段使用明确数字枚举，不解析中文武器名；技能可用性是当前 Health / MotionState / Combatant 的只读提示，模拟仍负责范围、落点和动作结算。
HUD 脚本只更新 textContent、progress.value、图标 href、可见性、样式类和无障碍属性，不使用 innerHTML 或独立计时器；暂停后冷却与联动显示随模拟快照冻结。
HUD 为 pointer-events:none 的纯显示层，E／Q／Tab／中键／1／2 通过实际输入桥；不能带入预览 HTML 的点击演示和模拟计时。HUD CSS 使用 hud-* 名称和 .game-hud 作用域，不能污染入口的 button / kbd / progress / p 样式。
愁铃客标准高度1.73m，原主骨架986节点已恢复；source_animation对35个具名原clip有界解码、稀疏T/R/S采样与层级求值，按原骨名驱动独立Skin调色板，CC0回退仍用23个明确结构锚点。`avatar::sync` 只更新显示骨骼；挂点位置用适配后的手腕，枪轴方向沿用原动作。13个穿戴部件、17个材质槽中的皮肤及长刀使用原主色/法线纹理，PNG显式sRGB/linear；未知原作材质参数不猜测为已恢复。靶子从 Body.scale() 读取1.45比例，模型替换不能反写 Body 或改变玩家相机锚点。
底层资产为 `assets/animations/stickman.anim`（23 骨骼 / 19 动画），保留同目录 CC0 许可证；永劫 KIVASR02 为独立63骨／44片段，燕云 KIVASR01 仍986骨／35片段；CC0 骨架 `include_bytes!` 嵌入，两份源动画由 `bundle` 运行时加载（`SourceAnimationLibrary` 为 `FromWorld`，测试用 `bundled()`），均显式小端解码。
身体姿态通过 ActorView.owner 读取 Body 的实际固定步位移、MotionState、Health 和 Combatant，动画反馈按物理 CombatEvent.actor/sequence 归属与去重。
禁止把动画根位移叠加到权威位置；空中不重复抬升骨盆。暂停冻结姿态、尸体保留可见播放 Death01，复活清理旧姿态。
跳跃读取MotionState::jumps_used()，第二次起跳重启空中计时；哥哥使用永劫一／二次起跳、下落和落地，Boss保留燕云来源，远程／缺源动作回退到CC0的NinjaJump等姿态，不额外叠加模型高度。
妹妹使用独立 RigPose 与射击／拉弓姿态，哥哥不因 Q 或联动切成远程；实际事件到达后产生反馈，物理权威保持。
`SniperImpact` 仅为受击 actor 选择 0.30s 重受击姿态，保留骨盆和下肢落点；HitStop 期间仍消费反馈但姿态时钟 dt=0，恢复后再播放反应，不修改身体物理高度。
蹲姿和滑铲根据 MotionState 只读状态选择，0.12s 平滑权重调用 act 的 crouch_pose 腿部 IK，下移骨盆 0.33m 且保持脚掌和腿长。
局部混合缓存必须保存 IK 前姿态，对副本做蹲姿 IK 后生成模型空间骨骼，避免蹲起过渡重复降低骨盆。
地形 Mesh 与 Collider 消费玩法层同一份原始顶点 / 三角形。目标圈、瞄准与防御标记使用支撑表面位置 / 法线，不能固定世界 y=0.05。
地面攻击与敌人蓄力特效从真实事件位置向下查询表面并匹配法线，空中命中保留真实事件位置；表面查询排除角色胶囊和传感器，未命中时不退回世界零高度。
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
| 音频未解锁 / 被挂起 / 不支持 | 跳过当前声音且不排队，游戏继续；挂起清理残留声部 |
| 旧 AudioContext 已关闭但 statechange 尚未到达 | 替换前主动清理旧声部，旧回调不能停止新上下文的声音 |

## Good / Base / Bad Cases

Good：边走边转向，人物及镜头读取相同 yaw / 显示位置，无跟随拖尾。
Base：点击进入、WASD 行走、Esc 释放、点击继续。
Bad：先 await 捕获再假定锁定、把 movementX 乘 dt、把垂直位移当俯仰。

## Tests Required / Quality Check

`make check`、WASM Clippy；入口改变再执行临时目录 Trunk 构建。
Rust 测试验证相对方向、斜向速度、分帧转角一致和真实透视投影；Node 直接执行桥脚本，验证加载重放、
捕获确认 / 失败 / 失焦、旧请求、动作一次消费、右键边沿／取消／溢出、合并事件和缩放单位；HUD 验证多份快照早期重放、数值约束、安全文字、目标伤害 / 清理、套装图标和可用状态、冷却与联动过期及倒地清理。
Rust 无窗口回归通过真实固定步验证选中目标的当前血量、目标死亡 / despawn / 模式替换 / 玩家倒地隐藏，以及真实套装切换、闪避、空中和联动时的可用提示；浏览器另验证血条位置、按键状态、暂停和入口样式隔离。
狙击镜头验证低位背后仰视、入场 / 回切连续、暂停稳定、击倒目标后的瞄准点保持、非零地面高度、障碍净空和普通相机构图不变；瞄准 IK 验证枪口向上及脚底不移动，HUD 验证序列期间新动作不可用。
冲击回归通过真实锤狙事件验证预瞄不触发、单次 / 重复 / 普通射击隔离、本帧枪口、空中命中方向、HitStop 中粒子继续而骨骼冻结、失锁全部显示暂停、后坐结束恢复和最终净空；重复射击后临时实体清空且网格 / 材质数量不增长。音效 Node 测试覆盖可信解锁、拒绝恢复、不支持、缓存、不削波、并发上限和节点释放；浏览器验证实际 AudioContext 播放。
客户端无窗口场景测试验证装配、武器切换显示、反馈消费及特效回收和资产复用；真实画面 / 手感不由这些测试代替。
模式切换回归检查多次切换的敌人显示根 / 血条数量、无孤立骨骼、资产数量不增长；Node 检查早期选择、一次消费和 F2 捕获 / 重复过滤。
外观换装回归通过真实World检查哥哥身体改变、新武器与妹妹和敌人不变、暂停 / 空中物理快照与 RigPose 源姿态不重置、重复切换资产和实体数量稳定；Naraka额外验证当前姿态 / 源动作的真实骨骼变形、结构骨长与手挂点一致。Node 直接执行外观桥验证加载早期 / 暂停选择、单次消费、F3可信与捕获 / 重复过滤、Rust快照和非法值隔离。
惘心笠作为原独立头饰跟随头部驱动，帽顶额外高度不改变本体转换或碰撞；回归检查实际帽网格的蒙皮顶点随头旋转、切回愁铃客完全回收、重复切换不累积。源限发字段只记录已证明的语义，原算法未恢复时不据单个 mask 值删除整套心素发型。
Shift 输入回归覆盖完整点按先于采样结束、左右键重叠 / 换手、帧间松开再按下，以及捕获 / 重复事件隔离。
蒙皮无窗口测试覆盖绑定姿态抵消、实际走跑/蹲伏/跳跃/攻击/暂停、跨角色网格/纹理/材质复用、挂点、父子关系及不改变物理姿态。
骨骼测试覆盖全部动画有限性 / 无根位移、走跑跳跃等实际姿态变化、事件归属、武器跟手、死亡 / 复活 / 暂停及不改模拟高度。
C 回归覆盖短按 / 松键保持、完整双按奇偶性、重复过滤和失焦清理；蹲伏 IK 验证多片段的脚掌位置 / 旋转 / 腿长和武器握持。

## Wrong vs Correct

错误：独立平滑相机、把显示 Transform 写回物理 Position，或将 view 实体 ID 当作战斗 actor。
正确：JS 只管理输入边界，物理实体消费 Intent；owner 关联的显示根读取快照，模型和相机共用显示位置。

`environment` 只消费Level同源地形/竹干/岩石，19个聚合网格、7个ForestMaterial；14张原竹叶/竹皮/落叶/土壤/草苔/草丛base与normal随包复用，sRGB/linear显式区分，alpha卡片有UV/tangent。ForestMaterial=ExtendedMaterial<StandardMaterial,ForestSurface>，保留标准PBR/雾、蒙皮/阴影路径；地表只通过分层纹理和有界mask区分干土/湿土/非金属积水，不用clearcoat或金属度伪造湿地。12组装饰风动不改变碰撞竹干，暂停冻结时钟。重复不透明纹理由texture_mips生成完整mip，颜色在线性空间平均、法线重归一；不直接给裁切atlas平均alpha以免远草消失。半球环境光/冷色填光/指数雾为Kiva美术适配；原编译shader用于验证mix通道，不冒称原HLSL/完整BRDF已移植。场景是主题重建，来源与局限见assets/choulingke/README.md。

rendering共用MainView标记和HDR、Msaa::Off、FXAA、DepthPrepass、DeferredPrepass，默认不透明材质deferred；游戏GameCamera另装SSR，验收页不装SSR。透光裁切植被显式Forward。不能改变相机姿态/FOV或模拟。SSR只对屏幕内深度/已完成deferred光照有效，前向透光物体和屏幕外物体不保证倒影；验收需要真实GPU pass记录与GBuffer读回，只有shader组合测试不够。湿土粗糙度不进入SSR，仅平滑积水进入，避免高频normal造成随机闪点。

当前BGM为官方艺人发行的愁铃客《惊惧一夜》，audio/music.json.file=choulingke-bgm.ogg，原MP3及随包Ogg哈希/来源见music-manifest；加载仅同源、可信点击后解码。不能称其为已恢复的本地WEM映射、原循环点或原阶段切换。attack为已核Cue/事件/媒体的原长兵器共享适配，hit/deflect仍为自制程序声，来源逐项见effects-manifest；原长刀/成功卸势完整映射仍缺。离线导入器只写其拥有的attack，程序生成器只重建hit/deflect并核对保留原attack，不能互相覆写；不得用自制或共用武器声冒称愁铃客专属。新增声音必须逐项注明实际来源，再配置本目录简单文件名并随包复制。

实战预警颜色与HUD阶段/行动文字直接读Boss只读快照，不能依赖普通Enemy.is_winding_up镜像。HUD不显示内部行动槽ID；无Boss的被动演武靶继续使用原目标血条。

SourceAnimationLibrary与RigPose源姿态仅在client，不能让simulation导入资产名或播放时钟。两来源独立身份／绑定与KIVASR02浮点时间切点见[naraka-motion.md](naraka-motion.md)。KIVASR01保留原毫秒键、祖先索引和带符号非零尺度；读入检查边界/有限/单位quat/时间单调，缺轨回原rest。原根XZ去掉、地面保留腰身Y，空中Y不与物理高度叠加。共用jump/huajie/dodge按骨名和明确父关系retarget，省略轨记录来源，不能说成愁铃客专属；具体裁剪和比例适配写animation_manifest。手臂参考方向、真实掌轴与独立指轨按[模型契约](model-assets.md)共用校准；握柄读取当前实际腕/MCP，源指节在局部旋转混合后FK，远程显式握点IK优先。源骨架缺失独立附件不猜绑定。原技能槽→clip仍未恢复，Boss.clip_name=None不凭名字补造精确映射。
普通Effect暂停冻结寿命/变换并前移游标，恢复不补播；特效、冲击、音频共用 `FeedbackCursor`，不得各自重写“暂停只消费不呈现”的判断。HUD 状态文案由 `hud::status_text(CombatStatus)` 给出，模拟层不提供显示字符串；HitStop不等于失锁，普通视觉效果可继续推进。

竹枝叶kind=3在相机3m内裁除、3–5.5m固定像素dither渐入，使用真实view.world_position；规则共用forest_view.wesl，主片元和forest_prepass.wesl同步可见深度/法线裁切，防止隐藏近叶提前遮挡。阴影pass以LOD主相机位置与阴影view区分，保留原叶atlas阴影；远叶/地面落叶不受近叶退隐影响。不改变碰撞或源alpha。
