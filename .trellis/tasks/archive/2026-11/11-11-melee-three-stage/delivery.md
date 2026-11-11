# 三段连击模拟交付

已实现MeleeStage、逐段不可变身份、单条后继/.45s窗口/重置与实际起手事件；旧Melee marker仍是唯一普通命中入口，每段保持锤24/2.8m/.12/.60与链斧18/2.6m/.08/.40。三段均普通攻击，没有第三段加伤、霸体或破防。

DefenseSnapshot保留当前阶段时钟与最后手势身份/经过时间，真实短弹反窗口即使被显示帧跨过也可重现；自然Idle保留手势、取消作废、复活保留单调ID。实际判定/耐力/冷却未延长。

新增MotionFeedback仅为受控Combatant记录真实接受Jump/Dodge和物理后Land/Footstep，独立128事件历史；初始接地、空中瞬移/复活不伪造落地，堵墙/无输入/空中/滑铲/闪避不踩步，源步距为明确Kiva适配。MotionRecorder共用固定时钟和流，不驱动物理。

公开API和完整生命周期见implementation-notes.md；结果见validation.json。源资产H0、Naraka姿态/实际手掌指节、音效播放和浏览器交互由父/表现任务继续，未称已验收。

## 文件

- src/simulation/abilities.rs、weapons.rs：MeleeStage/ActionSnapshot和纯数值MeleeProfile。
- combat.rs、combat_tests.rs、新melee_chain_tests.rs：MeleeChain/锁存取消/阶段事件/全部准入、重置与FixedUpdate测试。
- defense.rs：阶段与最后手势只读快照。
- 新motion_feedback.rs、actions.rs、physics.rs、mod.rs：真实运动事实/接地步距、门控、teleport清基线、exports。
- boss.rs：实际Damaged清未承诺后继、HitContext阶段None及真实Boss测试，不改Boss行为数值。
- 必要机械client适配：animation.rs忽略新MeleeStart，combat.rs跳过其默认粒子，audio.rs测试构造器stage=None；未做正式表现/音频接入。

未提交、推送或修改canonical specs；等待主会话独立check与集成。没有启动/停止用户服务。
