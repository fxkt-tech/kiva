# 第一版双角色表现契约

## Scope / Trigger

修改 client/{companion,weapons,animation,combat,camera,impact,hud,audio,appearance}.rs、Web HUD 或相关 World 测试时阅读。
具体输入见 [input.md](input.md)，权威动作见 [../simulation/combat-v1.md](../simulation/combat-v1.md)。

## Signatures

- CompanionView { owner, performer }：owner 为哥哥，performer 为逻辑 Companion；不要求 performer 有 Body。
- companion::setup／sync 读取 CompanionState，维护独立白灵淼蒙皮／RigPose 显示根。
- weapons::spawn／setup_tether／sync_tether 建立共享武器、链斧离手实体和绳索；四件源模型由 weapon_models 按 pair＋mode 缓存，格式与挂点见 [model-assets.md](model-assets.md)。
- Combatant::sequence() 返回 SequenceSnapshot；kind 区分 HammerCombo 与 ChainCombo，cancelled 及 action_id 用于恢复和冲击归属。
- kivaHud.renderLoadout(pair,maximum,attackReady,meleeReady,rangedReady,guardReady,comboReady)，pair=1 锤鸟／2 链斧弓。
- kivaHud.renderResources(stamina,maximum,meleeCooldown,rangedCooldown,axeAway,defense)，defense=0 Idle／1 Deflect／2 Waiting／3 Guard／4 Parry。
- renderTarget(name,current,maximum,status) 与 render(persona,weapon,fraction,health,combo,cooldown,context) 保留只读方向。
- CombatEvent.pair 固定记录产生事件时的武器组；声音或弹道不能用当前套装猜历史事件的武器。

## Contracts

哥哥默认李火旺，沿用近战动作；妹妹使用白灵淼衣装／莲母发型／女性基脸和自己的结构参考与 RigPose，不贴 PlayerView／ActorView 来重放哥哥所有动作。
妹妹出场位置、射击源、瞄准点和 elapsed 都来自模拟；显示使用相同 fixed overstep，暂停和 HitStop 取当前快照。
引冰落以局部原点为枪口，羿之影以局部原点为箭源，均朝 -Z；武器、枪口火花、弹道与模拟 source_position 对齐。弓箭类型按事件 pair 判断，不能因玩家随后切组而改变历史表现。
锤使用伏浪锚，链斧使用谪星摧山，鸟铳／弓使用引冰落／羿之影；均使用源几何和基础色纹理，源蒙皮先求真实当前 LBS。谪星互补部件采用有记录的固定静态装配，未恢复随机星格／完整形变／FX。握点／尺寸／附件挂接为明确的 Kiva 适配；枪弓上身原动作和目标下肢／双手IK组合见[naraka-motion.md](naraka-motion.md)，不冒称恢复完整原武器控制器；重复技能不增长网格／材质。
链斧离手时哥哥手中武器隐藏，绳索／飞斧读取实际动作时间线与物理位置；收招／取消归还，不能同时出现两把同一链斧。
飞斧和手持使用同一份谪星网格／材质，绳索起点在本帧骨骼附件同步后读取实际握柄世界坐标，不用固定身体高度或滞后的 GlobalTransform；终点与飞斧根原点一致。
F3 只替换哥哥身体和服饰，保留 RigPose／显示根／物理状态；不替换新武器、不影响妹妹与敌人。敌人原刀外观保留。

锤鸟镜头从正常跟随进入低位仰视：约 0.20s 入场，三发为 0.30／0.42／0.54s，0.68–0.96s 回切。
链斧弓使用约人物胸口高度的平视镜头，0.65–0.93s 回切。两种相机都跟同一个哥哥显示位置，复用球体扫掠净空。
普通镜头 FOV 50°、高度 6m、后距 4.2m 和下三分点构图保持。
镜头和后坐必须关联当前动作，取消、死亡、F2 替换不能让冻结的旧后坐永远留在普通相机。
目标死亡或取消时使用序列最后瞄准点恢复；相机不另建会推进技能的时钟。
普通特效与专用冲击各持一个 `client::FeedbackCursor`（`take(feedback, active)` 批量消费，`advance(sequence)` 逐条消费），SniperShot／SniperImpact 只追加表现，不重复伤害或播放第二声。

HUD 显示玩家生命 1000、耐力、当前组、E／Q 独立冷却、右键阶段、Tab 条件与链斧离手。就绪快照必须跟随模拟本步最终状态。
目标血条来自当前 Combatant.target，联动原目标不能强行覆盖用户当前锁定。
Web 只更新安全文字、progress、SVG href、显隐、样式类和无障碍属性；无独立冷却时钟或演示点击控制。

音频按真实 Shot（actor=Companion） 一次消费，按不可变pair选择已追溯的永劫鸟铳／弓原媒体；SniperShot只追加视觉不重复发声。MeleeStart逐段起手声与实际Hit／Block／Deflect／Parry反馈、真实MotionFeedback分别消费，空挥和空弹反无成功声。cue格式、来源边界、迟到丢弃、四声部层预留与暂停规则见[naraka-motion.md](naraka-motion.md)。
source_owner 用于玩家归属，actor 决定真正演出者；妹妹开枪不在哥哥身上播放射击动作。
失锁跳过新事件并前移游标，不恢复后补播；暂停／音乐／声部契约继续沿用客户端索引。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| performer 无 Body／Health | 正常读取 CompanionState，不尝试物理查询 |
| 模式替换／逻辑妹妹失效 | 显示隐藏或回收，旧绳索／后坐不残留 |
| 事件跨套装变化后才消费 | 仍使用事件原始 pair，不看最新 CompanionState |
| 暂停／HitStop | 共同取 current；骨骼时间冻结，专用冲击可在 HitStop 继续 |
| F3 在暂停／空中切身体 | 不重置物理、动作、武器组或妹妹 |
| DOM 未就绪 | 缓存最新状态，挂载后重放 |
| 重复消费／反馈历史跨多步 | 每序号一次，无重复声音／伤害／资产分配 |

## Good / Base / Bad Cases

Good：Q 由妹妹开枪，哥哥保持原武器与动作；随后切组，尚未消费的事件仍播放原武器声音。
Base：一轮联动结束，妹妹退场、链斧回手、镜头回普通构图。
Bad：F2 暂停换敌人后还保留上次后坐；同一个 PlayerView 同时承接哥哥和妹妹的射击。

## Tests Required

保留皮肤／源动画／IK／外观资源复用／真实相机投影与净空回归。
新增实际 Q／三发事件的妹妹归属、独立模型／枪口、暂停／失效回收、绳索与离手、双冷却／耐力和 immutable pair 音频测试。
Node 执行实际 HUD／appearance／audio 桥；make check、WASM Clippy 和临时 Trunk 构建后，单独记录浏览器画面与交互观察。

## Wrong vs Correct

错误：用 source_owner 匹配玩家后把妹妹事件播给哥哥，或依据最新武器组重放历史声音。
正确：actor 路由动作、source_owner 归属玩家、事件 pair 决定武器，模拟快照唯一推进时序。
