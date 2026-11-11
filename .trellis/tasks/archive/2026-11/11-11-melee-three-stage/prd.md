# 左键三段连击权威规则

## 目标

将现有单段左键改为可核对的1→2→3普通攻击，并给表现提供不可变阶段、防御手势与实际运动事件。

## 来源和依赖

父任务11-11-naraka-action-set/prd.md是需求源；具体产品取舍见父design.md。

依赖父design固定的Kiva规则与runtime-combat-integration研究；可以与离线资产并行，禁止import客户端clip/WEM名。表现任务依赖本任务公开只读接口。

## 验收标准

- 每次实体点击仅承诺一段，第1/2段最多一条后继缓冲，第3段不堆第4段；0.45s窗口正确重置。
- 每段独立action_id，stage存入快照和事件；marker恰好一次，总伤害/范围/冷却按父design纯数值profile。
- 实际有效技能/防御/闪避/跳跃/换组/受伤/模式替换/死亡与暂停输入取消的后继重置符合父design；当前已承诺攻击保留原取消规则。
- DefenseSnapshot明确kind/阶段时钟/gesture_id；短按不生成弹反，真实防御结算保持不变。
- MotionFeedback只在真实接受起跳/闪避及接触落地/实际接地步距发生；第三跳拒绝、堵墙和传送不发假事件。
- 既有E/Q/Tab/妹妹/HitStop/物理与事件归属回归通过，不增加强化/暴怒/耐久/破防。

## 范围

src/simulation/{abilities,combat,weapons,defense,actions,physics,boss,mod}.rs及相应测试/exports；boss.rs只接实际受伤清后继；physics.rs只接真实publish后的运动cue，不改求解器。controls适配由表现任务拥有，本任务交付锁存取消API。

父任务整体方案及四子任务已获用户明确批准，按依赖启动实施，不重复请求规划批准。
