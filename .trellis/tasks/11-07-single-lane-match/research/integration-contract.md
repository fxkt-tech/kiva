# 本轮集成契约（实现协调，接口变更须通知主代理）

## 已确定的通用模拟接口
通用模拟接口已落地，玩法与客户端复用以下边界。

- UnitClass::{Actor, Structure}；UnitStats新增class与attack_delivery，默认Actor及原英雄弹丸。
- BasicAttackDelivery::{Direct, Projectile { speed, radius }}。
- SimulationSet::Step包住既有完整FixedUpdate链。
- preview_cast(&World, Entity, CastSource, AbilityTarget) -> Result<AbilityTarget, Rejection>：无副作用复用开始施法预检；提交仍复核。
- is_dashing(&World, Entity) -> bool。
- DamageSpec::estimate_damage(self, &EffectiveAttributes, &EffectiveAttributes, health:f32) -> Option<f32>：复用伤害公式。
- RecoverySpec { radius, health_fraction_per_second, mana_fraction_per_second, out_of_combat_ticks:u32 }。
- restore_in_zone(&mut World, source:UnitLifeRef, target:UnitLifeRef, RecoverySpec) -> Result<RecoveryAmount, RecoveryError>：结构源、友方Actor、生命/范围/脱战/上限/暂停/每目标每tick一次由模拟检查。

## 玩法层公开契约

- GameplayPlugin：资源和固定步编排，不引入GPU/窗口/DOM。AiPlugin独立提供小兵/英雄决策，客户端默认安装两者，headless对局时钟测试可仅安装GameplayPlugin；不增加隐藏运行模式。
- prepare_world(&mut World)：准备默认模式地图、角色/技能/装备目录与Waiting状态；重复调用不能覆盖已有局面或注册冲突。
- start_match(&mut World, player_character:CharacterId) -> Result<MatchHandle, MatchError>：仅Waiting；先预检双方基地、英雄与完整首波计划，仅构造基地/英雄/首对兵共6根后提交Running；其余按待发序列逐对生成；失败回滚新增实体，身份分配器不回退。
- reset_match(&mut World)：调用reset_simulation，清模式状态/AI/波次/结果，保留静态定义及单调match_id分配器。
- MatchHandle { id:MatchId, player:Entity }；MatchId使用u64，Web必须Display为字符串而非JS number。
- MatchState只读getter：id/phase/player_character/hero(team)/base(team)/wave_number/next_wave_tick/finished_tick；具体返回Option以Waiting合法。
- MatchPhase Waiting/Running/Finished(MatchOutcome)，MatchOutcome Winner(Team)/Draw。完整Step之后一次判定；同tick双基地死亡Draw。
- UnitRole Hero/MeleeMinion/RangedMinion/Base，与Controller Human/HeroAi/MinionAi分离；它们是模式元数据，不拥有HP、CD、位置。
- MapDefinition getters包括half_size/lane axis/base_position/hero_spawn/creep_spawns；场景与NavigationMap共用同一配置。

追加调整后默认地图边长64、基地Ally(-8,24)/Enemy(8,-24)，道路宽7，出生与路线从道路纵/横轴派生。镜头首次/重开定位到英雄一次，不强制全图显示。

MatchDefinition新增minion_spawn_interval_ticks()/with_minion_spawn_interval_ticks(u64)，默认30tick；wave_interval仍3600。每队slot0..5在波开始后的0/30/60/90/120/150tick出兵，3近战720HP、3远程480HP。MatchRecord私有PendingWave保存wave/next_slot/next_tick；每波先全量预检，每对双方原子生成。Waiting/暂停/终局门控及错误/重开清理见tuning.md和现行gameplay规范。

## 所有者与并行边界

1. 基础实现者：src/simulation及必要UnitStats构造迁移，公开规则测试和模拟spec。
2. 对局实现者：src/gameplay、src/lib.rs导出、模式无窗口测试；先实现固定步门控/地图/构造/波次/终局和初始内容。不碰client或Web文件。
3. 客户端实现者：src/client、web、index.html和必要客户端测试；使用模式真实接口。不写gameplay/simulation运行态。
4. 兵线/AI实现者后续接手独立ai子模块与软避让；与玩法实现者协调mod.rs接线，不并发修改同文件。
5. 主代理：任务文档、最终集成审查、浏览器脚本与验收；不与上述源所有者交叉写。

并行工作只用于独立文件准备；基础API和完整模式质量门仍按依赖顺序通过，不在半成品上宣布完成。所有代理不提交、不推送、不操作用户trunk服务。

## 客户端关键边界

- Selecting/Active还是捕获与人类绑定所有者；Launch通过start_match取得player，完整成功才变Active。
- 加入结果重开意图，带match_id及player上下文；ReturnToSelection和Restart经模式reset/start，不通过刷新页面。
- 终局冻结不能被捕获/继续覆盖；按MatchPhase+用户暂停合成有效paused。结果帧同步最终根一次，冻结插值与骨骼，不能只让SimulationTick停而继续用overstep重新显示。
- 英雄/兵/基地用显式外观描述，标签与拾取锚点集中配置，并继续投影同一UnitVisual显示根。
- 保留相机模式/边缘滚屏/缩放/跟随；地图边长已按追加要求扩大到64，不固定全图。
- 浏览器测试必须真实输入、只读投影；不新增可写World调试桥，不为测试保留隐藏旧试炼模式。

## 已放行基线

基础make check183项（原173+10新增）和WASM Clippy通过，可接lib/client。AI实施者独占gameplay/ai/**和simulation普通移动软避让；gameplay/mod.rs仍由核心实施者统一接线。
