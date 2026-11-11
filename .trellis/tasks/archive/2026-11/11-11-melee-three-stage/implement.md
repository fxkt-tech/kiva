# 左键三段连击权威规则实施

## 前置

依赖父design固定的Kiva规则与runtime-combat-integration研究；可以与离线资产并行，禁止import客户端clip/WEM名。表现任务依赖本任务公开只读接口。

## 顺序

1. 先明确并加MeleeStage、快照stage、防御snapshot（保留last_gesture_kind/gesture_elapsed）、运动cue公开类型及取消后继锁存API。
2. 实现阶段profile与单条后继/窗口，加入普攻起手事件，保留marker结算和原准入优先级。
3. 锁存失锁清后继请求，处理全部重置点；普通伤害只清未承诺后继。
4. 在combat.rs和boss.rs两条实际受伤路径清未承诺后继；接入防御gesture与真实运动cue，physics.rs仅提供实际publish/teleport边界，执行FixedUpdate边界与既有战斗/物理回归。

## 校验

真实FixedUpdate三段/末帧/总伤害；低显示帧率无关；狂点、早晚输入、窗口、死亡/暂停/F2/换组；无目标/遮挡/无敌；防御短按和成功结算；实际二跳/第三跳拒绝、落地、堵墙与传送。

适用Rust/Web变更执行make check和WASM Clippy；资产变更执行明确的离线verify，不能用语法检查代替解码/重建。所有切片保留可回滚边界；不覆盖同伴文件、不改用户客户端、不启动常驻服务。
