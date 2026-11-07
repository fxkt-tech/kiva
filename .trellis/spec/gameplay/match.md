# 单中路对局、地图与兵波

## 1. 范围与触发

适用于`src/gameplay/{map,definition,spawn,match_state,lifecycle,content}/`及客户端开局/重开接线。
玩法层依赖通用`simulation`，不读DOM、相机、窗口或模型，也不写战斗HP、法力、位置、动作或技能CD。
通用战斗入口见[模拟基础契约](../simulation/combat-foundation.md)；显示/捕获仍由客户端负责。

## 2. 接口与所有者

```text
GameplayPlugin
AiPlugin                                  // 独立的自主决策插件
prepare_world(&mut World)
start_match(&mut World, CharacterId) -> Result<MatchHandle, MatchError>
reset_match(&mut World)
MatchHandle { id: MatchId, player: Entity }
MatchId: Copy + Eq + Display               // u64身份，不提供外部构造
MatchPhase = Waiting | Running | Finished(MatchOutcome)
MatchOutcome = Winner(Team) | Draw
MatchFinished { id: MatchId, outcome: MatchOutcome, tick: u64 } // Message

MatchState::id() -> Option<MatchId>
MatchState::phase() -> MatchPhase
MatchState::is_running() / can_advance() -> bool
MatchState::player_character() -> Option<CharacterId>
MatchState::hero(team) / base(team) -> Option<Entity>
MatchState::wave_number() -> u64
MatchState::next_wave_tick() / finished_tick() -> Option<u64>
MatchState::wave_error() -> Option<&str>

UnitRole = Hero | MeleeMinion | RangedMinion | Base
Controller = Human | HeroAi | MinionAi
WaveMember::wave() -> u64
WaveMember::slot() -> u8                    // 0..2近战，3..5远程
```

上述角色/控制来源/波次是模式组件，不保存第二份战斗状态；角色种类与控制来源分离。
`MatchState`内部字段私有，只有构造/生命周期模块提交阶段与结果。客户端只绑定`MatchHandle.player`和读取状态。

`MapDefinition`是资源，提供`half_size/lane_width/obstacles/base_position(team)/lane_direction(team)/lane_lateral(team)/lane_length/hero_spawn(team)/creep_spawns(team)/navigation`。
`MatchDefinition`集中保存敌方角色、基地/兵种stats、复活延迟、波次间隔、单兵间隔与恢复参数；配置在开局前提供，当前局保留自己的配置快照。
`minion_spawn_interval_ticks()`及`with_minion_spawn_interval_ticks(u64)`提供排期配置；默认间隔由`TICK_RATE`派生为30tick。
`MatchRecord`私有的`PendingWave`只保存当前波次、下一slot和下一出兵tick，不公开修改或另建战斗状态。

## 3. 数据、调度与生命周期契约

默认方形地图边长64，half_size32，基地为Ally(-8,24)/Enemy(8,-24)，中路宽7；导航与三维场景/小地图读取同一份定义。
正方形指世界地图，保留现有滚屏、缩放、锁定和跟随；不强制正方形画布或永远全图可见。
基地低台的目标中心可导航，不能另加覆盖中心的静态障碍而破坏通用构造/普攻视线。
英雄出生由中路方向前移2、横向1.4派生，既在2.5恢复半径内，也避开后排兵槽位并适配既有相机边界。

`prepare_world`幂等：只首次准备内置技能/角色/装备目录和从地图派生的导航；之后不重置活局、目录或身份。
内置内容按定义ID注册，既有条目不覆盖；模式配置和地图应在首次准备/开局前提供。

`start_match`只允许Waiting且SimulationTick为0。先检查全部配置/角色/出生/可达性，包括双方首波全部12兵计划，
再原子构造两基地、双英雄和slot0双方首对近战兵；全部6个根成功后才提交Running、新的MatchId和剩余5对排期。
单个单位仍通过`spawn_unit/spawn_character`创建。默认近战465生命、11攻击、1.25攻速、350移速、110攻距；远程284生命、21攻击、0.667攻速、350移速、550攻距，双抗0。距离通过gameplay::units按100:1换算，基地规则保持。
构造中发生失败时删除本次新增根，不清其它静态资源，不回退来源/身份分配器；未成功的局不发布身份或半场阵容。
开始不会替客户端清除用户暂停，捕获/聚焦与有效运行由客户端接线合并。

首次兵波在0秒开始，`wave_number=1`表示已经开始的波次；随后在3600、7200等SimulationTick开始下一波并更新下一期限。
每波每方先3近战后3远程，对应slot0..5，双方同slot同时各出现一只；相对开始tick为0/30/60/90/120/150。
波次开始先全量预检12个计划，再原子创建首对；后续每对出兵时重新预检当前配置/导航，实际工厂失败只回滚该对新根，
保留早先已参与战斗的兵和已提交效果，记录错误并冻结，不能删掉早兵来假装整波从未发生。
排期保存在模式记录，固定步按SimulationTick消费，每对新兵从下一固定步行动。
单兵间隔、波次间隔必须为正；`(6-1)*单兵间隔 < 波次间隔`，时长乘法、起止tick相加、波次递增全部检查溢出。
暂停不推进SimulationTick且保留排期，不补发墙钟经过的兵或多波；终局、失败及reset清除待发序列。

`GameplaySet`顺序为`Decisions → SimulationSet::Step → Outcome → Waves → Cleanup → Recovery`。
`GameplayPlugin`只安装模式生命周期；客户端另装`AiPlugin`获得真实自主决策。模式时钟/终局测试可只安装前者以隔离战术。

每个固定步均重新检查可运行状态。Outcome在完整战斗结算后统一读双方基地：单方死亡产生胜负，双方同tick死亡为Draw，只写一次结果并发一次MatchFinished。
随后立即设置有效暂停、清CommandQueue；Waves在函数体再次检查状态，不依赖早先已求值的共享run_if，终局当步不会再出兵。
即使外部错误清掉paused，Finished仍阻止后续Step；有效暂停不能作为可恢复终局的依据。
显示层需同步最终状态一次再冻结根和骨骼，不能只停SimulationTick而继续用变化的overstep重采样。

基地为Structure，生命与受效规则在模拟层；英雄沿既有3秒复活。恢复只向双方各自英雄调用模拟`restore_in_zone`：
半径2.5、脱战120tick、每秒最大生命/法力各20%（每0.5秒各10%）；源/目标生命、阵营、范围、脱战与钳制仍由模拟裁决。

死亡小兵在本步完整结算后删除根及linked children，英雄/基地不按兵清理；已提交的独立Projectile/AreaEffect继续原快照生命周期。
`reset_match`复用`reset_simulation`并清模式/结果消息；保留地图、内容和单调MatchId分配器，新局装备与AI组件随新根重新创建。

## 4. 校验与错误矩阵

| 输入/边界 | 结果 |
| --- | --- |
| 未prepare即开局 | `NotPrepared`，不创建单位 |
| 活局或已终局未reset又开局 | `AlreadyStarted`，保留现局 |
| 非零SimulationTick开新局 | `InvalidDefinition`，须先reset |
| 未知玩家/对手角色 | `UnknownCharacter`，无部分阵容或MatchId |
| 波次/单兵间隔为0、排放时间跨到下波、时长乘法溢出、恢复定义非法、结构/兵种分类错误 | `InvalidDefinition`，无构造 |
| 非法stats/技能绑定/出生或不可达路径 | 复用纯`validate_unit_spec`及NavigationMap，返回构造/定义错误，不复制校验公式 |
| 构造过程中工厂失败 | 回滚本次所有已创建根，分配器保持单调 |
| 后续波次全量预检失败 | 不创建该波首对，保留早波；记录`wave_error`且冻结，`can_advance=false` |
| 中途单对预检/工厂失败、排期/计数溢出 | 当前对无半边新兵，早先兵保留；清待发序列并冻结，不重复补发、不允许继续坏局 |
| 基地同时被摧毁 | 完整结算后一次Draw，不按Died消息顺序抢胜 |
| 暂停/终局/Waiting | 不产生新战斗tick、波次或AI决策 |

## 5. 正常、默认与错误案例

默认：准备后Waiting且无模式单位；开局6根、每方1近战，tick150排完每方3近战3远程；玩家角色1、配置对手角色2，双方英雄真实stats和技能保持原定义。
正常：tick29暂停后经过任意宿主时间仍为29，恢复到30只生成slot1双方一对；tick3600只开始第二波首对，下一波开始期限7200。
正常：敌基地在tick3600归零，本步先结束对局，wave_number仍为1，没有第二波。
错误：给AI维护私有基地HP、在首条Died回调直接宣布胜负、每个显示帧判断是否满60秒、构造半波后继续发布Running。

## 6. 验证与断言点

`cargo test --locked --test match_mode`覆盖：

- 0秒6根、Hero/Structure/Controller/WaveMember来源；0/29/30/60/90/120/150逐对slot、465/284生命、顺序与最终3+3计数；默认出生兼容相机、恢复半径与兵槽间距。
- 3599/3600/7200边界、暂停不补发、prepare幂等、活局不能被替换、非法内容/出生无部分实体。
- 真实英雄能接近并攻击基地、终局后同显示帧多固定步与恢复paused均不继续、同时死亡唯一Draw、终局/波次同tick。
- 全波预检涵盖后段slot、中途失败无半对且保留早兵、终局优先于同tick待发对；死兵子树清理但已提交弹丸/区域继续命中、双方恢复同规则、部分出兵后reset保留静态内容和单调身份。

Mode测试不以改写私有HP或伪造Died制造结果；通过实际普攻/技能与配置公开入口完成。
最终执行make check/WASM Clippy及Web真实地图、相机、拾取、血条、结算/重开回归；浏览器不写World。

## 7. 错误与正确

错误：游戏结束只把DOM paused设true，下一次捕获成功又继续旧局。

正确：MatchState永久Finished，固定步门控独立检查阶段；客户端只合并有效暂停与展示结果。

错误：逐个出兵用显示帧/墙钟定时；先创建某slot我方兵，对方工厂失败后保留我方新兵并在下一帧重试。

正确：波次开始完整预检12个计划，每个到期slot的双方一对原子创建；失败回滚当前对且冻结错误，保留既有战斗事实。

临时墙覆盖出生点时工厂局部安全排离，原始home与formation_slot保留；计划可达性按当前实际Nav.path_static校验，不能因动态R圈临时无路触发配置错误。原定tick/成对原子构造/3+3均不变，静态非法及真实工厂失败仍按原错误契约处理。
