# Research: 最终方案可执行性复核

- Query: 复核父/四子任务是否与已取得源证据一致，是否有跨任务契约或验收遗漏。
- Scope: internal；未重新调查片段或媒体身份，未修改产品、任务状态或其它代理产物。
- Date: 2026-11-11

## Findings

### P1 — 原生转换依赖已经披露，但子任务仍缺可判断的首关口

锚点：`.trellis/tasks/11-11-naraka-action-set/design.md:73`；`.trellis/tasks/11-11-naraka-animation-assets/implement.md:10`；同子任务 `prd.md:16`。

父设计正确声明先验证Humanoid转换，不会用Generic附件替身；环境研究也正确声明当前未发现可用Unity求值环境。但子任务第2步只要求“记录原生求值尚未对照的部分”，第3步即开始扩展全部核心动作；PRD的“身体/30指动态”条件会被现有不含twist/body solve/IK的数学spike满足。若执行代理把非零动态、有限quat和骨长不变当验证成功，后续默认动作仍可能错误。

建议在子任务步骤2与3之间加入明确Gate H0：**取得同版本native/独立可信完整解码器对照，验证body/30指/root（与所用片段相关的twist/IK）后才扩展/交付默认动作。** 对照未取得只允许交解析研究，不宣称动画资产子任务通过。建议对照阈值及样本见 `humanoid-conversion.md`，主计划至少固定“逐帧local角差、腕/指位置、真实LBS误差均有实测报告”这些验收类别。该依赖是整个核心动作交付的blocking技术未知，应作为首个可验证交付暴露，不能藏到全面接入后。

### P1 — 延迟声音调度承诺了丢过期，但尚无足够的身份/时钟接口

锚点：父 `design.md:49`、`:77`、`:83`；`.trellis/tasks/11-11-melee-three-stage/implement.md:10`；`.trellis/tasks/11-11-naraka-presentation/implement.md:12`。

现在仅约定“普攻起手事件”和不可变stage；现有 `CombatEvent` 没有发生tick/时间，历史可跨多个固定步后一次被显示消费。若客户端以收到起手事件的浏览器时刻开始调度，低帧率/HitStop/F2/死亡或新段接替后会播放延迟旧cue，违反父设计的“历史过期cue丢弃”。仅靠sequence防重不能判断年龄或撤销未来cue。

建议锁定一种最小接口：

- 普攻起手事件必须带 actor/source_owner/action_id/pair/stage；以当前同action_id的权威ActionSnapshot.elapsed为调度时钟，已经结束的旧action起手直接丢弃；或事件另带冻结规则时间的模拟tick，并明确年龄门槛。
- pending cue以 `(actor,action_id,cue_index)` 去重和绑定；F2/死亡/取消/换段时撤销旧**尚未发出**cue，暂停/HitStop不让浏览器wallclock推进它们。已经自然播放的尾音是否保留与未来cue取消分开。
- 把原轨偏移映射成同一Kiva阶段clock；若time单位未证明而选Kiva适配偏移，明确该表/默认值，不能等待未知单位时默用“秒”。

这不要求将源event名移进simulation；只需把跨层生命周期契约明确交给两子任务。

### P1 — 弹反判定窗与成功片段的完整可视时长没有分开

锚点：父 `design.md:53`、`:60`、`:63`、`:87`；`.trellis/tasks/11-11-naraka-presentation/prd.md:15`。

原Parry有效窗只有0.18s，父设计只说按权威clock播成功动作和Idle收势。若直接把成功反击全片压到0.18s，会严重加速；若kind变Idle就清掉source，长片又会只显示起手。动作展示恢复不应延长免伤/冷却，也不能借此额外制造反击伤害。

建议精确定义有限的纯表现手势：

- 新Parry gesture_id首次观察时只启动一次 `ParryRiposte` 展示；初版**总表现时长0.60s**（从Parry接受算起，含原0.18s判定窗，最长0.42s可视收势），这明确是Kiva适配值。
- source clip按其真实duration/已验证阶段单调映射到0.60s；不能仅用Parry.remaining归一。若最终片段有必须保留的起手/定格/收势分界，在映射表逐项列出；原duration来自资源库，0.60来自表现策略。
- authoritative kind变Idle后，只允许该手势剩余的视觉恢复继续。死亡、真实受击、新承诺攻击/技能、新防御gesture、有效闪避/跳跃/滑铲立即抢占；进入实际行走/跑步也按明确规则结束恢复，不能让旧片压住新运动。
- 该表现时钟使用共享RigPose已有gesture machinery，受SimulationActive/HitStop冻结；game与viewer同一frame推进。它不写DefenseState、不改变耐力/窗口，也不追加Hit。
- CombatEffect::Parry只触发真实成功反馈/声音，不重新把同一gesture启动到0。
- 若显示帧跨过整个0.18s窗口，当前kind可能已Idle。应保留 `last_gesture_kind + gesture_elapsed` 或发不可变防御起手cue，使共享driver仍能识别未过期Parry手势；仅有“当前kind/phase_elapsed/gesture_id”不足以还原这种短相位。过期手势丢弃，不补播。

精确展示时长可随源片段验收调校，但必须先固定“判定窗与视觉恢复分开、0.60s有界默认、抢占/冻结/去重/漏帧”合同，再做两子任务适配。

### P2 — 运动反馈与受伤清后继的必要文件所有权未完整列出

锚点：`.trellis/tasks/11-11-melee-three-stage/prd.md:24`、`design.md:7`；父 `design.md:44`、`:79`。

三段子任务仅列 abilities/combat/weapons/defense/actions/mod，但新增HitContext.stage至少需要更新 `src/simulation/boss.rs:493` 的实际构造；普通受伤清后继也必须覆盖 `boss.rs:492` 和普通敌人 `combat.rs:1626` 两条真实来袭路径。MotionFeedback的实际接触/位移采集必须说明落在 `physics.rs:490` publish，还是新增在其后的明确反馈系统；当前scope既没有physics，也没有单独该系统的所有权。

建议将必要 `boss.rs` 适配与运动反馈具体落点纳入三段子任务；不改Boss决策/伤害参数。若选择不改physics而新增publish后采集系统，写明调度位置与死亡/teleport步距清理。controls的清后继适配也应直接指定由三段任务拥有，表现任务只消费，避免“协调单独所有权”留到两方并行修改时。

### P2 — 离线verify与原生烘焙复现的含义需要区分

锚点：父 `design.md:91`；`.trellis/tasks/11-11-naraka-animation-assets/prd.md:18`、`implement.md:12`。

计划承诺verify不依赖Windows安装，同时Humanoid完整求值可能必须依赖外部同版本Unity。两者可兼容，但目前没说verify是“冻结姿态→runtime binary重建比字节”，还是“原曲线/Avatar→完整原生重烘焙”。执行者可能只核缓存哈希后把前者表述成完整转换可复现。

建议明确两级：冻结包包含原曲线/Avatar证据和已通过oracle验证的baked locals；普通 `--verify` 无客户端安装即可检查这些SHA、结构和二进制打包重建；另一个 `--rebake`/原生工具命令用于从曲线再次求值，需要记录求值工具版本和环境。若最终纯离线完整converter不需要Unity则可合并，但必须实际证明，不预先声称。

## 已检查且没有发现矛盾的部分

- 父设计保留旧每段时长0.60/0.40与marker0.12/0.08，当前冷却同样在起手设置为段时长；按现有prepare推进、advance最后marker后retire、下一步衔接，不必绕过冷却。缓冲不能在“命中marker已经过”就启动下一段，必须等**整个duration**完成。
- 原输入优先级Tab→E→Q→LMB、只有实际有效其它动作重置后继的表述可行；建议测试明确“buffer存在时有效新E/Q/防御优先，不能被自动后继抢占”。
- 父设计已明确原音轨time单位未恢复、不把反击挥刀当成功金属碰撞、落地并行PlayAction不压成一层、不做变体笛卡尔积，未发现对应假claim。
- Parry阶段按用户要求播振刀成功**动作**，只有结算Parry事件发成功声，与当前DefenseKind/HitOutcome契约一致。
- SourceRig身份、异拓扑不zip、同名也核rest、哥哥F3仍走玩家动作、Boss保留燕云和妹妹IK优先，以及查看页继续静音，均与现代码和要求一致。
- 提交/推送未重新授权、当前保持planning的状态与说明一致。

## Caveats / Not Found

- 音效资源最终报告仍在生成，本复核只检查计划已明示的时间单位/并行层/生命周期约定，不重复核源媒体身份。
- 本文件针对读取时版本的行号；主会话修改后行号可能变化。没有执行产品测试，因为目前只有规划/研究改动。

## 主会话处理记录

本轮复核后的父/子计划已调整：H0改为硬性逐帧原生/可信转换对照，覆盖95/75/55通道和指层；PendingActionCue固定身份并以同ActionSnapshot.elapsed调度、at+0.12s过期；防御snapshot保留last_gesture_kind/gesture_elapsed，Parry判定0.18s与0.60s纯显示恢复分开；boss/physics必要适配归三段任务，controls归表现任务并消费前者取消锁存API。verify冻结locals重打包与原生rebake已分开。

本处理记录由主会话写入，表示计划已具体回应问题，不表示产品实现或H0已经通过。动作/音效最终报告已到齐，女鸟铳双候选误推断也已纠正，原指层混合参数未恢复继续显式列为H0研究项。
