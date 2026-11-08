# 技能提交与结果事实

## 1. 范围

适用于`facts.rs`、SimulationEvent及Projectile/AreaEffect/BattlefieldObject/UnitMechanics公开来源视图。事实服务只读观察者，不规定外观、音效或客户端计时。

## 2. 提交接口

`AbilityCommitted`保留unit/ability/position，补充source_life、cast_source、origin、解析后的target、target_life和recast。中央helper仅在真实提交或重施成功处发布，取消、预检失败和未结束前摇不能发。Direction正规化用宽中间值防溢出；Point/Enemy的legacy position仍是解析点，Direction/Self仍为原点。

`AbilityCatalog::iter`只读实际已注册定义，不能开放运行态写入或把AbilityId复制到定义内部。

## 3. 结果接口

`ImpactResolved`携带source_life、team、origin、empower_origin、carrier提交组、position、direction及ImpactKind：

- Contact { target:UnitLifeRef }：真实payload先核对目标仍是活的同一生命、class可受效且effects非空，再按payload/target发布一次。位置是实际目标中心；纯控制和伤害被盾全吸收也成立，不依赖HP下降。
- AreaActivated { carrier_entity:Option<Entity>, radius }：实际延迟区域到期或即时区域/Leap落地，空场也发布一次；与每目标Contact分开。
- ProjectileEnded { carrier_entity:Entity, reason }：真实终止时一次，reason为Hit(UnitLifeRef)/Blocked/Expired/InvalidTarget；position取实际接触/终点分数，不用本步目标终点代替碰撞位置。

carrier组号只标一次提交，不标唯一箭；Volley同组多箭必须通过carrier_entity区分。新增事件不能改变payload排序、有序effects或提交/死亡/重生时序。

## 4. 独立来源与生命

Projectile和AreaEffect提供source_life()/origin()/carrier()/empower_origin()只读入口，来自提交快照。来源被删除或重生不取消已提交独立效果，目标生命仍按原规则复核。

强化普攻的DamageOrigin仍是BasicAttack；独立empower_origin只表示冻结的强化来源，不能把它改成技能伤害来规避/触发装备被动。snapshot_attack在提交时冻结，buff到期、死亡、来源移除后仍属于该在途投递。

## 5. 对象与附着视图

BattlefieldObject记录origin，旗/墙/Scout路径Reveal/到站Reveal/R揭示均透传；activates_at只读真实激活时间。盾记录每实例origin与expires_at，shields()返回不可写ShieldView { amount,origin,expires_at }。empower_origin()与真实强化状态同步设置、到期和死亡清理。

这些字段不含颜色/资源路径/皮肤ID；客户端不能从事件推回并修改模拟对象。reset_simulation继续清事件、载体、队列和对象，静态catalog与身份分配器保留。

## 6. 示例与验证

正常：同一W纯减速命中即使目标有全吸收盾也可观察Contact；空区域仍观察AreaActivated；七支W箭共享carrier组但有七个不同carrier_entity。
错误：只监听Damaged猜命中、取消前摇也发布Committed、用来源当前生命替换旧payload来源、把目标换生命后的接触当作合法命中。

`tests/skill_facts.rs`通过公开CommandQueue与生产内容覆盖提交、取消、纯控制/盾、空区域、单箭终止、障碍/射程/实际接触点、来源移除/新生命、强化冻结、对象来源、重施及整场清理。不要伪造SimulationEvent证明发布正确。

## 7. 扩展约束

增加新的真实投递结果时先明确产生阶段、单次身份、位置、受效资格与来源寿命；保持事实中立和既有规则顺序。表现层可丢弃旧事实或限流，但模拟不能为显示预算删命中、载体或目标。
