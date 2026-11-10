# Research: 运行资产与独立查看器的复用边界

- Query: 已接入的三个角色、四件武器能否脱离战斗复用；任意角色×武器、蒙皮／挂点／缓存的最小共享边界是什么？
- Scope: internal（运行代码、随包二进制、来源说明及本地 Bevy 依赖源码）
- Date: 2026-11-10

## Findings

### 结论与当前范围

当前 MVP 已由主会话确认：**三个已接入角色、四件已接入武器＋无手持武器、动画 clip 与单角色姿态**，不包含完整技能／战斗。资产层可以支持这种独立显示，不需要创建模拟 actor 或安装 `SimulationPlugin`。任意组合的“可显示”与双手握持／衣装碰撞已经正确是不同承诺；现有证据覆盖男性近战、女性远程，尚未覆盖全 3×4。

最直接复用的是 `avatar::{spawn_with_appearance,replace,sync}`、`weapon_models::{WeaponModel::shared,spawn}`、`stickman::{attach_weapon,sync_attachments}`，保留现有二进制、材质解析和重定向。不要复用完整 `combat::setup`、`companion::{setup,sync}` 或 `weapons::sync_tether` 来搭建查看场景。

### Files found

| 文件 | 用途 |
| --- | --- |
| `src/client/avatar.rs` | 三个外观、一次加载缓存、每部件 GPU palette、AV02/03、身体替换、源／结构姿态重定向 |
| `src/client/model_asset.rs` | 共用小端读入、PNG 色彩空间／采样器、StandardMaterial |
| `src/client/weapon_models.rs` | 四武器固定缓存、握点、Weapon／Spine 部件实例化 |
| `src/client/stickman.rs` | 显示构造兼容层、右腕挂点、近战／远程轴校正 |
| `src/client/companion.rs` | 现有白灵淼独立 RigPose、枪口姿态、双手目标、背部附件；编排依赖模拟 |
| `src/client/animation.rs` | RigPose 的已求值模型姿态／源姿态／双手目标及游戏状态控制器 |
| `src/client/combat.rs` / `src/client/weapons.rs` | 男性近战装配、敌人原长刀、飞斧／绳索显示与战斗耦合 |
| `assets/{choulingke,lihuowang,bailingmiao}/character.bin` | 实际嵌入的三个角色包 |
| `assets/{fulangmao,zhexingcuishan,yinbingluo,yizhiying}/weapon.bin` | 实际嵌入的四个独立武器包 |
| 各运行资产目录 `README.md`、`conversion-report.json`、`source-manifest.json` | 当前适配、未恢复效果、数值／来源证据；愁铃客来源与材质说明主要在 README |
| `scripts/import-{choulingke,naraka,prepared-models,melee-models}.py` | 已有离线转换；查看器无需新导入器 |
| `assets/prepared/naraka/**` | 冻结源 GLB／纹理／证据，不能作为当前运行效果的替代 |

### 实际资产清单

以下计数直接只读解码当前七个二进制并检查 EOF；顶点按 primitive 计，含包内原长刀，材质为源槽数（角色运行会建立 normal／hit／telegraph 三套句柄）。所有角色按 1.73m、脚底零、前方 -Z 归一，但动作改变后的画面边界不能用原始顶点边界替代。

| 角色 | 包／大小 | 身体部件／表面 | 包内顶点／三角形 | 材质／PNG／最大尺寸 | 最大单部件 palette |
| --- | --- | --- | --- | --- | --- |
| 愁铃客 | AV02；25,045,179 B | 13／15，另原长刀 1 部件／2 表面 | 105,605／143,327 | 17／49／512² | 147 |
| 李火旺（含惘心笠） | AV03；10,739,759 B | 21／21，另烛泪 1 部件／1 表面 | 51,414／66,268 | 22／9／1024² | 88 |
| 白灵淼＋莲母＋通用女性脸 | AV03；7,207,149 B | 22／22，无原长刀 | 43,801／58,524 | 22／6／1024² | 78 |

外观枚举为 `Appearance::{Choulingke,Lihuowang,Bailingmiao}`，不限制对应模拟角色（`avatar.rs:25`）。正常游戏的 F3 只换哥哥、白灵淼只用于妹妹，是现有编排约束，不是模型格式约束。

| 查看器武器 | 当前缓存键 | 部件／材质／PNG | 顶点／三角形 | 轴与用途 |
| --- | --- | --- | --- | --- |
| 伏浪锚 | HammerMusket + Melee | 2／2／2 | 5,437／7,838 | 原握柄 0，长轴 +Y，总长 1.10m |
| 谪星·摧山 | ChainAxeBow + Melee | 5／5／1 | 17,504／22,620 | 同上；主体＋4 个互补原部件的 Kiva 静态装配 |
| 引冰落 | HammerMusket + Ranged | 2／2／2 | 5,395／7,106 | 枪口原点 0，射击 -Z；几何向枪托方向 +Z 延伸约 1.348m |
| 羿之影 | ChainAxeBow + Ranged | 3／3／2 | 10,228／11,674 | 弓体 Weapon；quiver、arrowbag 为 Spine；弓体上下跨度约 1.552m |

键映射和包路径：`weapon_models.rs:36`。四包 PNG 最大均为 1024²。两近战所有部件均 Weapon；弓的背部附件不能一起挂在枪口下（`weapon_models.rs:132`，`companion.rs:54`）。谪星的四组不是四把互斥整刀；两个 FX 输入不在运行资产内。弓的 `related/` 独立箭没有恢复 Actor／材质装配，不能再数作已接入武器（`scripts/import-prepared-models.py:354`）。

“无武器”可直接不创建／隐藏手持根。角色本来包含的鞘和腰刀仍是衣装部件：愁铃客 `weapon2`＝鞘、`3_weapon`＝腰侧刀具；李火旺含原烛泪刀鞘。若用户希望连这些也隐藏，需要额外定义部件显隐，当前接口未提供该含义。

### 无需模拟的资源和调用关系

1. 初始化普通 Bevy 渲染资源（`Assets<Mesh>`、`Assets<StandardMaterial>`）和 `AnimationLibrary`；创建带 Transform／Visibility 的显示根。
2. `avatar::spawn_with_appearance(world,root,appearance)` 自行加载对应缓存、创建缺失的 `RigPose`、建立每部件 palette 与表面；既不读取 Body 也不需要 ActorView（`avatar.rs:692`）。Image／IBMs／SourceAnimationLibrary 会在加载时初始化（`avatar.rs:135`）。
3. 每帧由独立播放资源提供 RigPose 的已求值姿态；执行 `avatar::sync`，随后附件同步。`avatar::sync` 的 ActorView 是 Optional；没有 ActorView／Health 时采用正常材质（`avatar.rs:832`）。因此保留编译上的 simulation 类型引用即可，不必安装它的插件或伪造 Health／Body。
4. `avatar::replace` 只删除 AvatarPart，保留 RigPose、武器根和其他子实体（`avatar.rs:761`）。查看器换外观可延续当前时间；身体的旧 palette 实体应正常回收。
5. 武器装配直接调用 `weapon_models::spawn(...,Mount)` 或现有薄包装 `weapons::spawn`。它们只依赖资源与 parent，不读取 Combatant（`weapon_models.rs:132`、`weapons.rs:36`）。

缓存真实结构为：愁铃客保存在全局 `AvatarAssets` resource，李火旺／白灵淼分别存在 `AvatarAssetCache` 的两个 Option；不能把全局 AvatarAssets 当“当前选中角色”反复覆盖（`avatar.rs:88`、`:361`、`:380`）。`AvatarAssets::shared` 当前为私有方法，应通过 spawn 触发；规范把它列作签名并不等于外部可直接调用。武器用四槽 `WeaponModels`（私有）缓存。网格、纹理、材质、IBMs 共享；每个实例的 joint 实体与动画时间独立。

### 任意组合：可复用部分与必须补的适配

**近战单手挂点可直接复用。** `stickman::attach_weapon` 依据当前外观的 `attachment_pose`，取 hand_r 并校正源手轴（近战 +X 90°；远程 +Y 180°），增加沿手局部 Y 的 0.055m 偏移（`stickman.rs:22`、`:44`）。三个外观都由同一 23 个语义驱动命名，但各自保留真实结构锚点／骨长。换成女性近战没有格式阻碍，尚无所有片段穿模和握持验收。

**远程不能只套用旧 wrist attachment。** 枪／弓的原点是发射源，而不是右握柄：引冰落双握点约 `[0,-.0582,.763123]`、`[0,-.0282,.413123]`；弓为 `[0,0,.35]`、`[0,0,.08]`（`scripts/import-prepared-models.py:300`）。现有妹妹流程先由独立枪口变换计算两个握点，再驱动 palette IK（`companion.rs:122`）；若 viewer 把远程 origin 直接放到 wrist，会出现明显错握。需要明确“按片段保留姿态＋右手对齐武器”或“加双手 IK 检查最终握持”的表现策略，后者会改变原片段姿态。右手对齐可以用现有握点反推武器根变换，不能复制一套随意写死的手偏移。

**双手语义稳定，完整源姿态的双手输出还没有通用接口。** `AvatarAssets::driver_pose` 使用各自 anchors；`posed_drivers` 对设置了 `RigPose.companion_hands` 的任何外观运行同一保骨长 IK，并非按 Bailingmiao 枚举限定（`avatar.rs:395`、`:450`）。但 `attachment_pose` 对非女性只保证右手位置；源动画混合分支也只替换右手，不能把其 left hand 当三角色源片段下已经重定向正确的最终左手（`avatar.rs:658`）。`body_mount` 当前只读取结构姿态，不读取 `rig.source`（`:654`）。源 clip＋背部箭袋若需要准确随源脊柱运动，应共享补足该求值接口。

**源动画重定向也已存在，勿重复实现。** 愁铃客按原骨名驱动完整主树和独立 palette；两个 Naraka 外观以各自 23 anchors 消费原燕云源动作增量（`avatar.rs:582`、`:613`、`:627`），不套用燕云绝对手坐标或李火旺男性骨索引。女性 anchors 来自自身 Lianmu 女性 skeleton（`scripts/import-prepared-models.py:198`）。注意：有 source 时非愁铃客 `source_deltas` 走 source_driver_pose，绕过 posed_drivers 的 companion_hands；直接同时写 source＋hand targets 不构成完整的双手源片段 IK 支持，需要在共享求值路径中明确组合顺序。

已有相关数值回归：`avatar.rs:1371`（Naraka 源动作自身骨长／右腕／真实顶点），`:1449`（女性实际 palette 双握点／臂长），`:1641`（真实两条联动的女性枪口／握点／脚掌）；`weapon_models.rs:267`（两近战握柄／轴／缓存）。它们不是全组合画面验收，本次未运行测试。

### 建议的窄共享边界（方案建议，尚未实施）

保留现有 loader／缓存／retarget 的唯一实现。可设 `client::model_presentation` 为游戏和查看器共用的薄入口，提供：外观身体 spawn／replace、四武器模型 spawn、已求值姿态到 palette 的同步、当前外观的**右手／左手／脊柱最终挂点**与武器 origin／grips 元数据。先让游戏和 viewer 调用同一计算函数，再以只读参数适配各自状态，避免把 companion 的整套 ECS 编排搬进 viewer。若无需搬文件，直接在现有 avatar／weapon_models 内抽纯 helper 并由薄入口转调也足够；不应为“共享”重写全部模块。

播放资源只拥有播放时间／选择／暂停，向公共姿态输出传 model、可选 source、可选 hand targets；`RigPose` 的 private 游戏状态机不应成为查看器需要伪造 Combatant 的理由。具体采样接口由动作调研负责。现有模块多数为 `pub(super)`，把 viewer 保持在 client 子模块内可复用这些接口；单独 crate／外部 app 则需要额外可见性重构。

`companion::sync` 强依赖 CompanionState、SimulationActive、HitStop、fixed time，包含 performer 生命周期与可见性；`animation::animate` 读取 actor 的 Body／Health 等；`weapons::sync_tether` 要求唯一 Player／PlayerView 和 active action（`companion.rs:86`、`animation.rs:546`、`weapons.rs:66`）。查看页不应注册这些游戏系统。

### 原长刀额外项（本次四武器范围之外）

已经嵌入 **两把** 可访问原长刀：愁铃客 weapon1、李火旺烛泪。`avatar::spawn_blade_with_appearance` 从选择的包里取首个 blade 并生成刚性表面，二者均已校正为握柄原点／+Y（`avatar.rs:780`；`assets/lihuowang/README.md`）。白灵淼无 blade，对其调用会 panic。两把刀不属于四槽 WeaponModel，当前没有统一枚举／握点元数据；换刀函数 `replace_blade` 仅在 cfg(test)。愁铃客原刀仍在正常敌人显示中使用（`combat.rs:150`），李火旺刀虽嵌入但当前玩家用两件独立近战。可作为后续资产选择扩展，不能悄悄增加本次首版范围。

### 源／GPU／边界与材质限制

- 角色读入完整保存 sourceRestModel 和 IBM；AV03 源导入姿态可能不同于 bind pose。显示“参考姿态”应标明当前导入参考，不能用 inverse(IBM) 替代 AV03 rest，也不能把所有 skin 矩阵强制抵消为单位（`avatar.rs:264`，女性 importer `:270`）。RigPose::new 默认是 CC0 idle，不等于源参考姿态。
- 986 个节点是愁铃客**CPU 源层级**，不应合成一个 986-joint SkinnedMesh。当前按部件独立 palette，最大 147／88／78；u8 四槽索引、权重和 255 保留（`avatar.rs:299`）。本地 Bevy 0.20.1 `bevy_pbr/src/render/skin.rs:28` 的 MAX_JOINTS 为 256。继续复用分部件方案；不要为新 viewer 合并 palette。
- 运行 mesh 生成 tangent、skinned bounds，实例带 DynamicSkinnedMeshBounds（`avatar.rs:320`、`:742`）。观察相机框选应考虑实际姿态与手持／背部武器联合范围；不能只按静态 1.73m 或裸顶点 AABB 裁切。两近战包的 Y 边界分别约 [-.289,.811]、[-.336,.764]；枪口0并非几何中心。
- 七个模型 bin 合计 50,277,493 B（约 47.95MiB）原始嵌入量，尚不含动画等资源；这不是实际压缩 WASM 下载量或 GPU 内存。按需解码可延后首次使用的 CPU／GPU 分配，但 include_bytes 已编入包。缓存保持已用资源，不随换装卸载；GPU 峰值和切换卡顿尚未测量。
- 共用 PBR 读入显式区分 sRGB／linear，UV Repeat，alpha mask／doubleSided／cull、材质 roughness 等（`model_asset.rs:47`）。愁铃客有来源已核的原主色／normal／ORM；其他资产多为原 base／alpha 加中性 PBR，未证明 packed／ID 等图不能接作颜色或 ORM。
- Prepared GLB 中性 preview 不等于游戏运行 PBR。源完整布料／发丝物理、专用 shader、精确面容／染色、武器开火／拉弓变形及谪星 FX 尚未恢复。实际已有案例：愁铃客猜测各向异性造成真实 WebGPU 银白伪影，CPU 解码与静态休止测试检不出来（`assets/choulingke/README.md:47`）。因此查看器应复用实际 StandardMaterial／GPU skinning 管线；离线 GLB 或 PNG 只能作为来源对照，真实 GPU 画面须另验收。

### Related specs / external references

- `.trellis/spec/client/model-assets.md`：AV03 自身参考、四槽武器、双手 IK、只复用随包资源、来源与未恢复边界。
- `.trellis/spec/client/index.md`：显示根、挂点同步顺序、换装仅回收自身部件、资产复用与真实画面验证。
- `docs/architecture.md`：client → gameplay → simulation 单向依赖，显示不能写回模拟。
- `Cargo.toml`：Bevy 0.20.1，WASM 用 WebGPU；本地依赖源码 `/home/asuka/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_pbr-0.20.1/src/render/skin.rs:28` 核对 MAX_JOINTS。未进行外网检索。

## Caveats / Not Found

- 只读研究、七个二进制的结构清点；未构建、运行测试、启动游戏／服务或浏览器。
- `assets/prepared/naraka/README.md` 的“未接入”是冻结准备阶段叙述，不能覆盖现运行 include_bytes 与运行目录报告的事实；不修改冻结来源文档。
- 没有统一字符／武器 catalog、通用左手／source spine 最终挂点、所有 3×4 组合视觉验收或实际 GPU 内存预算。首版是否给 raw clip 加握持 IK 应在方案中明确，避免把适配后的姿态冒称源片段原样。
- 根据研究角色隔离约束没有读取 implement.jsonl／check.jsonl；仅使用派发的任务路径、PRD、规范与目标源码。

## 后续澄清：以实际游戏装配验收为默认（覆盖上文首版四武器范围）

用户进一步明确用途是检查**游戏实际显示的模型、武器位置与动作是否正确**。首版默认应为游戏配置预设；当前实用武器计为 **5 把＋无手持**：两件哥哥近战、两件妹妹远程、敌人愁铃客 weapon1 原长刀。李火旺烛泪虽然已经嵌入但没有当前游戏使用者，仍为后续可选项。继续仅查看一个 actor，不加入飞斧轨迹、目标 actor 或完整联动。

### 游戏预设与诊断选择的层次

| 层次 | 复现对象／预设 | 验收含义 |
| --- | --- | --- |
| 默认：游戏配置 | 哥哥李火旺／愁铃客＋伏浪锚／谪星；妹妹白灵淼＋引冰落／羿之影及原背部附件；敌人愁铃客＋原 weapon1，可体现实际 Body.scale | 同一输入必须经过实际游戏共用的身体、武器、最终姿态与挂点求值；用于检出游戏问题 |
| 诊断：自由搭配 | 选择三角色与五武器／无手持 | 可观察组合，不表示该组合可在正常游戏装备；不把“看起来握好”当游戏已正确 |
| 诊断：原始 clip | 明确来源／时间，保留或标明游戏现有适配，独立查看姿态 | 不自动添加只存在于查看器的握持 IK／偏移来美化结果；若启用适配检查，标明具体模式 |

默认预设必须保存角色身份与**实际武器根语义**，而不是统一当“手上挂一个模型”。哥哥／敌人用现有右腕校正（`stickman.rs:22`）；妹妹武器根是枪口／箭发射源，来自 `companion.rs:117` 的 actor frame、插值后 source 与局部瞄准旋转。其双握点由该枪口 frame 变换，再求女性身体 IK，背部附件另跟 body_mount。上文“从右手反推远程根”只适用于明确标注的自由搭配诊断策略，**不能替换默认妹妹预设的真实算法**。需要修复的握持／挂点算法进入游戏与查看器共用模块，双方一起变化；不允许 viewer 专用补偿掩盖游戏错位。

### 最小共享求值与 debug marker 数据

在已有 `avatar.rs:565`、`:613`、`:627`、`:654`、`:658` 和 `companion.rs:117` 的计算处抽出数据求值，游戏 ECS 与查看器只负责供应输入、应用相同输出。建议最小输出包含：

- 显示 root frame（平移／yaw／scale）与本帧模型／source 权重；每部件最终 palette model matrices。palette 用现有 joint_pose 的结果，不能另行用简化骨架“猜”出显示骨骼。
- 最终右／左手、spine 的 frame，以及现有右手武器 attachment frame。保留“实际 wrist”与“握持 attachment”两个标记：近战当前明确有手局部 Y×0.055 偏移，不能把它误报为数值误差。
- 当前武器根及其 origin 类型（Grip 或 Muzzle）、左右 grip 的局部与世界位置、远程 muzzle 世界位置与 -Z 方向、各 Spine 附件 frame。近战第二 grip 为零且未使用，应标为无 off-hand 约束，不能假造第二握点。
- 本帧可见身体／手持／背部附件的 bounds；区分 renderer 用的动态包围盒与用于数值验证的实际蒙皮顶点边界。debug bounds 在相应 transform／bounds propagation 后读取，避免拿上一帧 GlobalTransform。

当前没有完整统一的“source clip 最终双手＋spine”接口；尤其 attachment_pose 只修正 source 右手、body_mount 不消费 source。默认游戏路径先忠实保留现有行为，marker 显示实际输出；如果验收发现左手／背部错误，统一修复求值层，并验证游戏和查看器同时得到修复。不能仅为 marker 或 viewer 再做一套 source retarget。`AvatarAssets` 的自身 anchors 与 AV03 sourceRestModel 仍是唯一比例／参考依据。

妹妹求值的输入可从现有 CompanionState 拆成只读表现参数：previous/current position、facing、source_position、aim_position、pair、elapsed／shot_count／当前单角色姿态参数，加显式 display alpha；输出 body root、local_source、local_rotation、pitch、grip targets。该函数不查询 World、目标 actor 或发起技能，游戏由真实 state 供参，查看器由同语义的冻结表现帧／单角色演示供参。因此仍可不安装 SimulationPlugin。具体单角色姿态采样契约由动作方案负责，不能通过构造完整联动来获得妹妹枪姿。

### 游戏与查看器的同输入对照验收

采用一份确定的表现输入（相同角色、武器、root scale、插值 alpha、clip／姿态、时间、source blend、瞄准／grip 参数），分别经游戏适配器与查看器适配器运行共用求值。比较的是输出，不只断言两者调用同名函数：

1. root、每部件 palette、右／左手、spine、手持根、muzzle／grips、背部附件与可见性应逐项一致。root 改为零的观察场景可在统一 actor 局部空间比较，再单独检查世界合成，不能混用空间。相同 World 资源应复用相同句柄；跨 World 对比资源标识／内容，不能比较 Entity 或 Handle 数字 ID。
2. 对真实网格检验 `inverse(meshGlobal) × jointGlobal × IBM` 后的 LBS 顶点／边界；再比较世界坐标下 wrist／实际 attachment 与武器 grip，远程 muzzle 与输入 source、-Z 与瞄准方向。不只检查抽象 RigPose 手坐标。建议 f32 对照起始容差 1e-5，按实际运算顺序和已有来源量化误差确定；不是本次已测得的阈值。
3. 静止／实际动作变化、暂停／换外观、不同 yaw 和远程 pitch 各取固定样本；妹妹原点／上轴和保骨长约束沿用现有回归的意义。原始 source clip 与最终游戏姿态分开标识，避免把正确的重定向误当原动画已经恢复。
4. 同一相机、光照、材质、可见性和渲染选项下补真实 GPU 并排／截图对照，检查衣装／发丝、alpha、阴影、握持和遮挡。共享算法数值一致仍可能共享同一个错误；还要检查已知的几何约束与实际画面。只有 CPU／无窗口通过不能宣称 PBR／蒙皮画面通过。

这组验收不需要开启目标／战斗系统；预设的真实表现帧应可复现。若首版没有可靠的游戏帧捕获接口，可用双方共同调用的确定参数样本，并明确未覆盖的实时战斗编排。

### 部件显隐的首版价值与边界

现有每表面都有 Name、独立实体与 AvatarPart／AvatarSurface 所属信息（`avatar.rs:717`），武器 ModelPart 也有 Name／mount（`weapon_models.rs:16`）。首版可提供只读部件名列表和**实例级**显示／隐藏／全部恢复，便于隔离帽、发、衣物、鞘、手持与背部附件，观察蒙皮和穿插。需要在换外观时重建列表，不删除 joint、不修改共享 Mesh／Material，不把诊断显隐写成游戏资产配置。无手持预设仍保留游戏默认衣装／腰刀／鞘。

按源命名的轻量显隐有直接排错价值；语义部位分组、单独骨骼编辑、逐材质参数编辑、蒙皮权重热图、完整 986 骨层级树、导出／持久化诊断布局可留后续。debug marker 也只显示当前 actor 的必要锚点与轴，避免把完整场景调试器纳入本次范围。
