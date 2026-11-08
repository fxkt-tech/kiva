# 验证记录

## 回走根因与回归

`begin_cast`与自主运动开始只清Movement.route，旧UnitOrder.MoveTo/AttackMove/AttackTarget仍存在，动作结束后resolve_orders重新寻路。

先运行：`CARGO_BUILD_JOBS=2 cargo test --locked --test simulation do_not_resume -- --nocapture`。
真实公开指令场景失败：普通技能后预期位置(-0.09166667,0)，实际(-8.983331,0)，正是回走旧目的地。

最终规则：前摇仍只清路线；commit_cast完成全部校验/快照后清旧地面目的；真实start_dash/start_compound清旧追击。不能在begin提前清目的，否则装备InvalidDamage等提交期拒绝会破坏原指令，现有原子性回归已验证。

新增公开场景验证普通技能与Dash在MoveTo/AttackMove后不回走、新移动仍可执行、Dash清旧追击、普通技能保留明确普攻。实际内容的皇子EQ/R覆盖真实自主运动，AI原先预期一次旧躲避路线NoPath的场景改为断言零拒绝并继续攻击。Enemy走近一次性规则、强制位移及拒绝资源/动作回归保留。

## 输入与偏好

默认智能、普通模式、Shift同帧按放、按键后鼠标继续移动、重复键、空目标、UI覆盖、英雄/装备Point截断、Enemy追近及图标普通瞄准通过无窗口真实WindowEvent/模拟命令验证。
Node执行真实pointer.js验证独立键快照的坐标/Shift/FIFO/可信事件/UI手势/暂停捕获清理；真实casting.js验证早期偏好、本地持久化、无效值和存储不可用。

## 最终门禁

- `CARGO_BUILD_JOBS=2 make check`：通过。405 Rust passed、0 failed、2既有长时soak ignored；含格式、原生Clippy、完整Rust、WASM check。
- `CARGO_BUILD_JOBS=2 cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`：通过。
- `node --test scripts/*-unit.test.cjs`：46 passed、0 failed。
- `node --check web/casting.js`、`node --check web/pointer.js`、`node --check scripts/browser-support.cjs`：通过。
- `git diff --check`：通过。

遵从用户要求，没有运行浏览器测试、没有启动构建fixture或常驻服务，也未启停用户服务。既有browser-support仅通过真实菜单明确Normal模式，未执行浏览器场景，不声称视觉/真实浏览器验收。
上一任务与本轮均保持未提交，等待用户后续提交推送指令。

## 快速EQ追加验证

通过真实builtin套件/CommandQueue复现了三个失败场景：同tick E后Q（E正常提交但Q被Busy丢弃）、间隔1tick的未落地旗无法连接，以及EQ后Idle无自动普攻。初次champion_content场景11通过、3失败；没有浏览器或客户端重试。

修复后，同tick和1/3/6/12/24/30tick间隔均能连接；旗落地前只有Q伤害且无军旗光环，原0.5秒后E伤害/光环仍正常生效。新AutoAttack只在运动结束后获取落点附近敌人，旧远处目标不会恢复，运动期间Stop能取消后续普攻。瞬发FIFO另覆盖其它单位仍当tick响应、后续Stop取消Q前摇且不扣Q费用/CD。

最终`CARGO_BUILD_JOBS=2 make check`通过：410 Rust passed、0 failed、2既有soak ignored；包含格式、原生Clippy、全量Rust与WASM check。WASM Clippy及git diff --check通过。没有Web代码变动，不重复Node/浏览器测试；本轮继续未提交推送。
