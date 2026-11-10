# 第一版动作与战斗契约

## Scope / Trigger

修改 simulation/{combat,abilities,defense,resources,weapons,boss}.rs、physics motor 的战斗约束或相关真实固定步测试时阅读。
client 只提交意图和读取快照；不拥有动作计时、伤害、耐力或强制位移。

## Signatures

- WeaponPair::{HammerMusket,ChainAxeBow}；WeaponMode 仅描述挂点握柄，不代表可控制人格。
- CombatIntent::{request_attack,request_melee_skill,request_ranged_skill,request_combo,request_toggle_target,request_pair,clear}。
- Combatant::{weapon_pair,target,attack_cooldown,melee_cooldown,ranged_cooldown,combo_window,combo_remaining,active_action,sequence,companion,axe_away,availability,clear_targets}。
- ActionAvailability 包含 attack、melee_skill、ranged_skill、combo、defense、equip、movement_actions。
- ActionSnapshot／SequenceSnapshot 保留 action_id、kind、pair、原目标、elapsed、duration、shot_count、aim_position、source_position、companion_position；SequenceSnapshot 另标 cancelled。
- Companion { owner } 是无 Body／Health 的逻辑施法实体；CompanionState 提供独立演出位置、朝向、射击源、目标点、动作时间与可见性。
- DefenseEdge::{Pressed,Released,Cancelled}；DefenseState::push_edge／clear 提交输入，其余窗口、kind、冷却只读。
- DefenseKind::{Idle,Deflect,Waiting,Guard,Parry}；Stamina::{current,maximum,quiet_elapsed}。
- HitContext／HitOutcome 供普通敌人与 Boss 共用来袭结算。事件含 sequence、actor、source_owner、pair: Option<WeaponPair>、action_id、hit_index、kind 和真实起止位置。

## Contracts

### 调度与权限

60Hz 同一门控：HitStop 步头 → 安装／准备动作与防御 → motor → Avian／当前空间查询 → publish → 玩家与普通敌人战斗 → Boss → 耐力恢复和妹妹快照 → 瞬移清理。
动作 elapsed 和冷却每步只推进一次。标记越界时有序执行，各标记只结算一次；不用动画结束回调制造命中。
动作准入在消费输入时验证；快照必须与本步完成后的动作／窗口／死亡状态一致。
默认无目标，中键锁定／解锁不自动补选。需要目标的技能不隐式选择；链斧 E 与普攻可以无锁定执行。
E／Q 冷却独立、跨武器组保留；动作中不接受新攻击、右键、切组或跳跃／闪避，不排队补放；当前技能匹配的 Tab 分支除外。
普通方向移动可保留；联动主动移动被禁用或由拉近驱动。普通空中近战保留；地面技能与防御不在空中起手。

### 两套技能与联动

锤 E 在原目标 4.5m 内、其周围 3m 内结算 20 伤害和约 3m 真实物理击飞；必须原目标实际被击飞才开放对应窗口。
鸟铳 Q 是妹妹单枪、40 伤害及 0.45s 僵直。锤鸟 Tab 消费本次 E 的原目标窗口，0.30／0.42／0.54s 三发分别结算 28 伤害，总序列 0.96s。
链斧 E 围绕自身造成范围伤害并施加单一 SlowEffect，减速进入真实 motor；同类效果不叠乘。
弓 Q 发射链斧、35 伤害和 0.90s 僵直，离手期间不能再次用该链斧近战；未接联动则按时间线归还。
链斧弓 Tab 必须原目标仍被本次 source_action_id 僵直；0.12／0.28／0.44s 三箭分别 18 伤害，总序列 0.93s。
拉近接受胶囊碰撞，最大平面速度 18m/s、最多 0.65s、接近约 1.4m 停止；高差、墙体或持续无有效前进时取消。不能改显示根／Position 伪造接近。
Tab 接受时要求浮空／本次僵直；承诺后自然落地或僵直到期不取消已经接受的三发。每发仍检查原目标、范围、视线和阶段无敌，不跳转新目标。
最后一发锤鸟成功命中沿用一次 3 固定步 HitStop；普通 Q 不触发这个专用停顿。

### 防御与耐力

拆招窗 0.20s，持续按住达到 0.20s 进入防御；从有效防御物理松开开启 0.18s 弹反窗。拆招／弹反各自再触发冷却 0.65s。
短按松开保留已开启的拆招直到结束，不额外弹反；取消、失焦与死亡都不能当 Released。
闪避无敌优先，其次正面拆招、弹反、有效格挡、普通受伤。正面点积阈值沿用 >0.25。
拆招免伤且不额外僵直；弹反免伤并让攻击者进入 0.80s 僵直；不附加反伤。
耐力上限 100，成功格挡扣 20 并承受原伤害的 50%；刚好扣到 0 的这一击仍成立，之后不足时正常受伤，无破防和命中次数上限。
拆招／弹反／闪避不扣耐力。实际到达命中判定的来袭重置安静期；进攻活动期间不恢复，安静 1.0s 后每秒恢复 15。
恢复排在所有来袭之后；SimulationActive／HitStop 冻结恢复与所有规则时间。复活补满，F2 不补满。
防御移动为常速 0.35 倍；跳跃／闪避可退出防御姿态，设备层不能直接改 MotionState。

### 归属与生命周期

玩家始终是哥哥，生命 1000。妹妹事件 actor=逻辑 Companion，source_owner=玩家；受击事件 actor=受害者。不能靠距离、中文状态或 source_owner 把妹妹射击动画播给哥哥。
ComboWindow 保存 source_action_id、pair、original_target 和 remaining；僵直来源由普通 Enemy 或 Boss 自己的唯一状态持有，不在 Enemy 里镜像 Boss 倒计时。
玩家死亡、目标死亡／移除、Boss 无敌阶段、模式替换取消未发射部分；镜头快照可有界恢复并保留最后瞄准点。F2 在暂停时也必须同步隐藏妹妹、清掉拉力／旧动作，不能等待恢复后的固定步。
暂停只清设备意图并冻结已承诺技能；恢复不补旧输入，不恢复跳跃次数。F3 不改动作、物理、武器组或妹妹。
玩家／靶子死亡 3s 后复活，清旧输入、目标、联动、临时控制效果和冷却；靶子保持被动。Boss 新遭遇清理旧减速等控制效果。
反馈最多 128 条、序号单调增加，各显示消费者各自游标；不能把显示历史当作命中去重账本。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 无锁定／超范围／隔墙／冷却中起手 | 拒绝并消费请求，不开启新冷却 |
| 起手后目标走开或遮挡 | 该次命中跳过，保留已经开始的冷却 |
| 重复 Tab／攻击／换组 | 活动动作不被替换，不排队，不重复伤害 |
| 耐力不足／背面攻击 | 正常受伤，无破防惩罚 |
| 右键队列取消／溢出 | 取消当前手势，重新按下才可重建 |
| 目标落地或僵直到期 | 尚未接受的窗口失效，已接受的三发按原目标继续复验 |
| Boss 转阶段／死亡／F2 替换 | 停止旧攻击与拉力，清妹妹／目标，镜头恢复 |
| 暂停／HitStop | 不补算规则时间、不推进耐力或冷却 |

## Good / Base / Bad Cases

Good：E 击飞原目标后改锁另一个敌人，Tab 三枪仍只向原目标分别结算。
Base：右键长按减伤，松开在有效窗内弹反；耐力见底只使后续格挡失效。
Bad：只播放三次闪光却扣一笔伤害；链斧拉近穿墙；F2 后在暂停画面留下旧妹妹。

## Tests Required

使用真实 FixedUpdate 测试三发时间点前后、独立 hit_index、原目标／stun_source、视线、阶段免疫、死亡和取消，不只断言最终总血量。
测量实际高台／平地击飞最高点、拉近碰撞／停距／高差及减速位移；保留原物理、二段跳、Shift／蹲伏、HitStop／暂停回归。
防御覆盖短按／长按／释放／取消、独立冷却边界、正背面、第五次有效格挡、低耐力恢复和同一步快照一致性。
执行 make check；Web Rust 另跑 WASM Clippy，打包另用临时目录 Trunk 构建。

## Wrong vs Correct

错误：客户端发射动画回调直接扣血，妹妹共享玩家 actor，Boss 再存一份 Enemy 僵直计时。
正确：固定步动作标记结算一次伤害，Companion 作为实际施法者，Boss 自己持有控制来源和期限。
