# 本地模型、动作与美术资源

## 范围与入口

适用于 `src/client/imported_models/`、`src/client/world_scene.rs`、模型/贴图/图标及离线导入脚本。
增加模型目录、动作映射或 HUD 资源字段会跨越资源导入、Bevy 表现与浏览器 HUD 边界，必须维护下列签名、载荷和验证契约。
生产对局英雄使用皇子原皮与源计划寒冰，双方近战/远程小兵使用各自蓝/红模型。
无模式的模拟测试演员继续使用 [刚性骨骼](character-animation.md)，该实现也承担资源加载/失败回退。

`ImportedModelsPlugin` 只装配客户端资源和显示系统，不注册战斗规则。
`catalog` 集中保存角色/兵种到模型、比例、源动作名和实际 AbilityId 到图标的映射；
`runtime` 负责加载、原子替换、生命/动作适配和显式采样。
角色身份仍为项目自己的 `CharacterId`，Riot 资源 ID 不进入玩法。

## 所有权与装配

- `UnitVisual` 继续唯一拥有世界平面位置、朝向、插值和击飞/跳跃高度。
- 导入场景是其子树，保留源 glTF 的负 X 比例，外层旋转 π 将 +Z 前向转为游戏的 -Z。
- 外观比例：皇子 1.1、源计划寒冰 1.2、近战兵 1.0、远程兵 1.1；按实际姿态测量维持约 2m 英雄/1.2m 小兵及既有血条锚点。
- 每种资源和动画图共享句柄；每个单位拥有独立场景、骨骼 Transform、AnimationPlayer 和表现缓存。
- 异步场景在隐藏容器加载，完整场景、播放器、源动作和动画图都就绪后才替换。
  只回收单位下旧 `PickGeometry` 模型子树及旧 `RigPose`，保留地面标记、护盾和其它非模型表现。
- 普通暂停期间不切换到刚加载的新模型，恢复后再原子替换；不能以绑定姿态覆盖已暂停的角色。
- 模型资源或必需动作缺失时保留程序模型并记录错误；不能同时显示两套身体，不能静默把缺失技能动作视为加载成功。
- 整局重置通过单位根的 linked children 清理导入场景；静态目录和句柄保留，无孤立骨骼/播放器。

## 动画与时间

Bevy `AnimationPlayer` 中的实际 clip 始终暂停；客户端按
`(SimulationTick + PresentationFrame.fraction()) / TICK_RATE` 设置 seek 时间。
不使用应用墙钟，不让源动画触发伤害、指令、冷却、动作完成或重生。

真实步位移决定 run，并按实际速度校准步态；保留 MoveTo 指令但没有实际移动时不能继续走路。
普攻前摇/后摇分别映射源 Attack clip 的前 45%/后 55%，接缝只出现一个连续接触姿态。
开始阶段长度保存首次观察到的 remaining；后摇读取自身 duration，不根据变化后的攻速重算。
施法前摇与已提交技能使用同一源技能 clip 的连续准备/提交部分；瞬发也由实际
`AbilityCommitted` 被识别，按实际 AbilityId 选择动作。新的移动、攻击或生命事件清理旧提交记忆。

生命代次、表现阶段或同类动作重启时先恢复导入时的骨骼绑定局部姿态，
再由当前 clip 覆盖，避免省略轨道的源动画继承死亡、旧施法或旧生命姿态。
这一重置只修改客户端模型，不修改模拟。

死亡进度来自只读 `UnitState::death_tick()`：将源 Death 拟合为 0.65 秒倒地，
随后保持其最后姿态直到真实重生。源 clip 长度不得延长/缩短复活倒计时。
两英雄 Death 最后姿态仍位于地面上方；小兵源 Death 会沉地，但小兵仍按玩法原有规则立即回收。
普通暂停不重新采样，终局首次按 `PresentationFrame` 同步最后权威步一次，之后冻结。

源 Root 的平面移动已在离线导入中固定为绑定值；实际 leap 高度已由 UnitVisual 应用时，
在 AnimationSystems 之后、Transform 传播之前固定源 Root 的局部 Y，避免重复叠加源跳跃轨迹。
暂停的 `AnimationPlayer` 仍然每帧评估冻结曲线，因此这一步必须每帧读取冻结的 `MotionView`
重新施加，不能随 `PresentationFrame::should_sync()` 一起跳过；暂停不能使皇子 R 突然叠加第二份高度。

## 蒙皮拾取

沿用 [模型拾取](unit-picking.md) 的候选排序、6 CSS px 容错、生命/视野门控和标签排除。
刚性模型继续使用部件 Aabb 与 GlobalTransform。
蒙皮模型逐个读取 `Mesh::skinned_mesh_bounds()` 的 `JointAabb`，
使用当前已显示的 `joint GlobalTransform * inverse_bindpose` 投影各部件。
不能使用绑定姿态框或一个整网格框吸住武器与躯干之间的空白。
非有限或退化为零比例的关节变换不参与命中，避免被动画隐藏的附件留下可点的空白位置。
边界直接来自网格和当前关节，不建立第二份位置缓存。

加载使用 Bevy 0.20.1 的
`AssetServer::load_builder().with_settings(...).load(path)`，
选择 `GltfSkinnedMeshBoundsPolicy::Dynamic`。保留 Mesh 的 MAIN_WORLD 数据供拾取读取。

## 资源处理与发布

`assets/league-models.json` 是源 URL、源 SHA-256、产物 SHA-256 与大小的清单。
`python3 -B scripts/import-league-assets.py` 离线校验本地文件；
加 `--download` 从固定源重建，源内容或产物哈希变化立即报错。
处理流程由 Python 标准库实现，不在游戏运行时解析 League 私有格式：

1. 读取标准嵌入式 GLB。
2. 去除 material extras 明确 `visible=false` 的 primitive（源计划寒冰备用头和无人机）。
3. 每个可见 primitive 按实际 indices 压缩/重编号所有顶点属性。
4. 固定 Root 平面轨道并清理无用 accessor/二进制存储。
5. 保留骨骼、逆绑定矩阵、完整源动作、UV、权重和内嵌原贴图。

第 3 步是拾取契约的一部分：源 glTF 的多个 primitive 可能共用所有部位的完整顶点数组，
Bevy 的 joint bounds 会读取这些未被该 primitive 的 indices 使用的顶点。
不能只隐藏备用 primitive 而仍让其顶点影响可见部件边界。

八技能图标按实际 AbilityId 映射，不能按 QWER 槽位决定内容；头像按 CharacterId 选择。
图标/头像和模型均来自同源 `assets/`；Trunk 显式 copy-dir，运行时不依赖 CDN。
来源和重建说明见 `assets/models/README.md`。

地图 `TerrainSurface` 在贴图真正就绪后才换材质；选取原草地/石路 UV 片段铺到既有地图几何。
原图是绘制图集，不能让每格都从同一 UV 角重新开始。相邻格交替镜像、共享边两侧 UV 完全相同，
草地用较大草地片段及约 2m 的世界重复间距；石路同样保持长轴采样连续，避免棋盘接缝和放大模糊。
地图边界、导航、出生、道路长度与宽度仍取 `MapDefinition`；不可另引入召唤师峡谷几何规则。
静态纹理装配只执行一次，不随每局重开重复生成材质和地面。

## 跨层签名与载荷

客户端边界为以下 Rust 接口；不能让资源服务 ID 或源动作名成为模拟指令：

```rust
pub(in crate::client) struct ImportedModelsPlugin;
pub(in crate::client) fn ability_icon(id: AbilityId) -> Option<&'static str>;
pub(in crate::client) fn portrait(id: CharacterId) -> Option<&'static str>;

#[derive(Component, Clone, Serialize)]
pub(in crate::client) struct ImportedModelView {
    pub name: &'static str,
    pub skin: &'static str,
    pub ready: bool,
    pub animation: &'static str,
    pub error: Option<String>,
}

// 客户端只读模拟生命周期，不依据源 clip 长度决定复活。
impl UnitState {
    pub fn death_tick(&self) -> Option<u64>;
    pub fn respawn_remaining(&self) -> Option<f32>;
}
```

`ui.rs` 只在既有观察/可见性过滤后的 `UnitView` 内克隆模型信息，不增加隐藏敌人投影：

| JSON 字段 | 类型与约束 | 来源/用途 |
| --- | --- | --- |
| `UnitView.model` | `{name,skin,ready,animation,error} \| null` | `ImportedModelView`；无生产目录项时为 null |
| `model.name` / `model.skin` | string | 本地目录的模型名称/皮肤名称 |
| `model.ready` | boolean | 已完成原子替换；false 时继续显示旧模型 |
| `model.animation` | string | 源 clip 名；ready 后为当前实际采样 clip，加载时为目录默认 idle |
| `model.error` | `string \| null` | 加载/必需动作错误诊断，正常为 null |
| `UnitView.portrait` | `string \| null` | 按 `CharacterId` 选取的同源相对路径 |
| `UnitView.respawn_remaining` | `number \| null` | 模拟返回的秒数；无待复活计时为 null；0 表示计时已到但仍等待安全出生点 |
| `CharacterSummary.portrait` | `string \| null` | 角色选择、预览同用本地头像；`CharacterPreview` 扁平展开该字段 |
| `AbilityDescription.id` | number (`u16`) | 实际 `AbilityId.0`，技能换槽后仍不变 |
| `AbilityDescription.icon_asset` | `string \| null` | ID 11–14/21–24 对应八张本地图标；其它 ID 为 null，保留原 `icon` 文字符号回退 |

`AbilityDescription` 扁平进入 `AbilityMetadata`/`AbilityView`，所以 HUD 收到的是技能对象上的
`id`、`icon_asset` 字段，而非新增嵌套 `description`。HUD 可以取整显示秒数，但不能启动 JS 计时器推演复活。
全部路径为 `assets/...` 同源相对路径；不增加环境变量或远端运行依赖。

模型就绪后的载荷片段示例（其它既有字段省略）：

```json
{
  "character_id": 2,
  "portrait": "assets/icons/league/ashe-project.jpg",
  "model": {
    "name": "Ashe",
    "skin": "PROJECT",
    "ready": true,
    "animation": "Death",
    "error": null
  },
  "alive": false,
  "respawn_remaining": 1.75
}
```

## 验证与错误矩阵

| 条件 | 结果/诊断 | 必须保持的边界 |
| --- | --- | --- |
| GLB/clip/场景仍加载 | `ready=false,error=null`，保留原模型 | 不出现绑定姿态闪烁或双身体 |
| GLB 加载失败 | `error="<path>: <loader error>"`，warn 并保留原模型 | 不阻塞选角、开始、暂停或指令 |
| 必需命名动作不存在 | `error="<path> has no clip <name>"`，warn 并保留原模型 | 不将缺技能动作视为 ready |
| 无目录项的角色/原始测试演员 | `model=null`；未知头像/技能图标为 null | 原模型与文字符号仍能工作 |
| 正常模型就绪 | 完整场景与图就绪后 `ready=true,error=null` | 仅移除旧模型子树；保留护盾、根、血条锚点 |
| 死亡且复活计时有效 | Death 在 0.65 秒内落地后保持，倒计时来自模拟 | 动画不自行复活，尸体不成为可攻击目标 |
| 到期但安全点被阻塞 | `alive=false,respawn_remaining=0`，保持尸体与等待提示 | 不传送到非法位置，不把 0 当 null |
| 普通暂停 | 模拟计时和采样冻结，新模型暂不替换；Root 高度每帧后处理 | 不用墙钟或源曲线制造倒计时/位置变化 |
| 重生/重新选角重开 | 实际新生命重置绑定局部姿态；重选清理旧根全部子树 | 不继承旧生命姿态，不留孤立骨骼/播放器 |
| 打包文件缺失/哈希变化 | 导入器报 `packaged asset missing or changed: <path>` 并非零退出 | 不将本地坏资源视为校验成功 |
| 下载源/转换结果变化 | 报 `source changed: <path>` / `prepared output changed: <path>` | 校验在写入该文件前完成；不可自动接受新哈希 |

## 正常、默认与错误案例

- 正常：选择寒冰，完整 PROJECT 场景原子替换，W 按实际 ID 22 使用冰箭图标与对应源技能 clip；死亡落地后显示同一地点的尸体和模拟倒计时，实际重生恢复新生命姿态。
- 默认：离线网页仅访问同源 `assets/`。尚未加载完成或无生产目录项时，旧模型继续表现和拾取；未知技能保留原文字符号。
- 错误：破坏必需动作映射后 `ready` 始终为 false 且 `error` 有明确缺失名字；玩法仍运行，不能删除旧身体、捏造动作成功或让浏览器倒计时替代模拟。

## 错误与正确示例

```rust
// 错误：播放时间随渲染墙钟推进，并可能越过暂停/死亡生命周期。
player.play(clip).repeat();
// 正确：clip 被动显示权威表现时刻，命中与重生由模拟产生。
player.stop_all();
player.play(clip).pause().set_seek_time(presentation_time);
```

```text
错误：把 SkinnedMesh 的绑定姿态整网格 Aabb 投影，或仅隐藏备用 primitive。
正确：离线压缩到实际索引顶点；运行时按当前 jointGlobal * inverseBindpose
      分别投影各 JointAabb，排除零比例隐藏附件，再沿用既有候选排序。
```

## 验证

- 打包 GLB 的真实骨骼/贴图、必需动作名、可见 primitive、顶点 indices 边界。
- 明确角色/兵种/阵营映射；无模式测试演员仍走独立旧 rig。
- 实际 SkinnedMesh 关节移动命中，绑定位置负例及零比例隐藏附件负例。
- 真实致死事件后的 0.65 秒倒地/最终姿态保持、普攻阶段接缝和模拟倒计时独立。
- 单显示帧跨过完整高速普攻时，`AttackReleased` 必须清除旧施法记忆；不能回放已跳过的攻击或继续旧技能。
- 使用真实 R 与 `AnimationPlugin` 的暂停回归：冻结 tick/clip 后每帧模型高度相同，恢复时不叠加第二份源跳跃。
- 草地与斜向道路网格的每一共享边两侧 UV 完全相同；浏览器截图不出现硬接缝棋盘。
- `make check`、WASM Clippy，以及重建后的 WebGPU 模型/地形、动作、死亡、暂停、复活和重选。
- 真实模型拾取、血条逐帧位置、HUD 输入隔离、800×600 和 DPR2 回归继续保留。
