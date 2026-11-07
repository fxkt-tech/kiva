# 中路对局展示与生命周期

## 1. 范围

修改地图、基地、波次投影、结果界面、重开或英雄/小兵/基地表现时阅读。
模式规则与构造由`gameplay`拥有；客户端只调用`prepare_world/start_match/reset_match`并只读投影。
既有相机平移、滚轮、Y锁定、Space临时跟随与原有鼠标捕获保持。

## 2. 接口与模块

- `content::setup`准备目录及模式上下文；`SelectionSettings`只保存客户端默认预选。
- `world_scene::spawn_map`从同一`MapDefinition`读取边界、道路、基地和障碍，静态场景只装配一次。
- `unit_appearance::UnitAppearance`统一角色模型类型、拾取AABB与标签偏移；`scene`拥有唯一UnitVisual世界根。
- `match_view::MatchProjection`只读模式与基地状态；`web/match.js`单独管理比分、时间、结果与按钮等待。
- `presentation::PresentationFrame`为根、骨骼、附着和特效提供同一次显示采样及终局冻结边界。

HUD新增以下字段；所有实体/对局身份保持字符串，JS不分配或解析身份：

```text
match: {
  id: String|null, phase: Waiting|Running|Finished,
  outcome: Victory|Defeat|Draw|null, error: String|null,
  elapsed_seconds: number, wave_number: number, next_wave_seconds: number|null,
  bases: { ally: {id,health,max_health}|null, enemy: {id,health,max_health}|null }
}
map: { half_size, lane_width, ally_base:[x,z], enemy_base:[x,z] }
UnitView: role: Hero|MeleeMinion|RangedMinion|Base|null,
          wave: number|null, formation_slot: number|null
```

无模式的纯模拟测试演员可使用默认英雄表现；不可据此推断模式身份或建立生产训练模式。

## 3. 生命周期与表现契约

Web选角为Waiting、tick0、无对局单位。Launch成功才产生双方基地、双英雄和每队slot0近战兵（共6根）并绑定玩家。
模式固定步在30/60/90/120/150tick逐对补齐3近战+3远程，客户端不维护出兵计时器。
客户端安装SimulationPlugin、GameplayPlugin、AiPlugin；AI与波次/胜负规则不进入Web。
`MatchState.can_advance()`并入有效暂停和装备权限；Finished及波次构造错误不能通过继续恢复。
错误文本投影到notice，仍可回选角重新开始。结束时释放真实鼠标，结果界面没有继续旧局入口。

“再来一局”点击内申请真实捕获，本次pointerlockchange确认后提交：

```json
{"type":"RestartMatch","match_id":"1","player_id":"4294966763"}
```

Rust同时验证当前Active的玩家/对局身份、实际MatchState为相同Finished、焦点及已确认捕获。
通过后reset再start原角色，推进selection_revision并绑定新玩家；失败回到可重试Selecting，不能保留已清理玩家。
成功返回选角或重开是批次边界，丢弃全部旧局尾部意图。旧match/player请求不能重置新局。
普通恢复仍为相同实例；返回选角使用已有暂停/未捕获校验，保留递增的修订版本。
`camera::update`只观察当前player身份：首次启动或Active→Active重开绑定新实体时自由镜头居中一次并locked=false，
保留用户缩放；普通恢复不重复定位，用户平移/Y/Space继续按原规则运行。Selecting仍复位默认CameraRig。

英雄保留23骨骼及原动作；近战/远程兵使用共享轻量几何和不同武器；基地为低台与可隐藏核心。
所有类型共用UnitVisual根及Airborne高度、逐帧标签和实际根拾取；小兵/基地不创建英雄RigPose。
基地死亡后保留平台、隐藏核心，其他死单位由相应生命周期处理；表现不写血量或胜负。
Finished首次显示以fraction1.0同步最终权威根和骨骼一次，之后保持精确显示姿态；普通暂停直接保留原姿态。

## 4. 边界矩阵

| 情况 | 行为 |
| --- | --- |
| 开始构造失败 | 无半个队伍/玩家，保留有效角色与修订可重试 |
| Finished后按Esc/Y/Space/技能或伪造继续 | 保持固定步冻结，不重新捕获旧局 |
| 过期match_id/player_id、未结束、失焦、未确认捕获重开 | 拒绝且不改变当前对局 |
| 重开构造失败 | Waiting、无单位、Selecting有原角色和新修订；可再次Launch |
| 波次构造失败 | can_advance=false，显示错误、释放捕获，禁止继续/装备管理 |
| Enemy技能/装备点击基地或其他预检拒绝候选 | 公共preview_cast拒绝，保留瞄准，不清aim后才报错 |
| 表现帧间overstep变化而模式已结束 | 根/骨骼/附着不重采样；标签仍按实际根逐帧投影 |

## 5. 默认与错误示例

默认地图64×64，基地为(-8,24)/(8,-24)，道路宽7，方位为左下/右上；
开局镜头位于玩家，不要求全图/两基地同时可见，不可用全图相机替代自由操作。地图、小地图、导航及出生均来自同一MapDefinition。
开局即第1波首对，之后每0.5秒补一对，2.5秒齐3+3；近战720HP、远程480HP，下波仍60秒起；显示值由Rust给出，JS仅格式化分钟/秒与HP。
错误：浏览器独立计时出兵、按敌方HP直接判断胜利、改JS entered布尔量伪造新局或硬编码地图边界而不读MapDefinition。

## 6. 验证与稳定选择器

Rust保留原56项客户端回归，并增加真实对局重开/失败/终局输入门控、首次开局/终局重开的一次性居中、普通恢复保留自由镜头和缩放、出生Y/Space居中、
结构目标预检，以及模型类型/拾取/标签一致和最终根骨骼冻结测试。终局fixture通过公开基础攻击摧毁基地，不能写私有状态。
原Airborne、短后摇暂停、capture/event边界、有序输入、装备实例和选角修订断言继续运行。

稳定DOM为`#match-clock/#wave-count/#next-wave`、`#ally-base-health/#enemy-base-health`、
`#match-result/#restart/#result-reselect`；标签的`data-role`来自UnitView。
浏览器回归使用独立临时构建及HTTP fixture，覆盖800×600、DPR2、窗口缩放、终局与两种重开；
不启动或关闭用户的开发服务。浏览器驱动身份/捕获校准复用browser-support，不增加World写桥。
共享lane-support记录首个Running快照，要求tick<30、6根且双方slot0；完整阵容只在等待150tick之后断言。
250ms夹具帧步只节省宿主等待，模拟仍运行全部固定步。skills/equipment按两基地距离派生中线位置；
远处目标必须先通过真实箭头平移到可见区域，再使用当前屏幕投影点击，不能依赖画布外的伪点击。
`healthbars`保持原生performance/RAF，以长侧翼移动和可见我方基地相机平移验证每帧跟随、20帧/80%阈值，
并确认样本未卡在相机边界；不能用夹具时钟替代真实显示跟随证据。

## 7. 所有权示例

正确：gameplay更新MatchState，Rust生成只读快照，match.js格式化显示并提交封闭会话意图。
错误：HUD自行维护波次/胜负、session复制模式清理规则，或为基地创建英雄骨架再把四肢藏起来。
