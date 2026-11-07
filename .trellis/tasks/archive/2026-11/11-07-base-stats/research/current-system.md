# 现有系统调查

调查基线：`1745b74`，2026-11-07。以下均为源码事实，供设计与检查定位，
不代表本提案已实现。文件行号为调查时位置，实施后以符号搜索为准。

## 属性与公开接口

- `src/simulation/model.rs:11`：`UnitStats` 同时包含生命/法力参数、碰撞半径、移动和普攻参数。
  默认攻击力 40、移速 5.5、射程 5.5、周期 0.8 秒、前摇 0.22 秒。
- `src/simulation/model.rs:39`：`UnitStats::valid` 已拒绝非有限数值，正数/非负数分开校验。
- `src/simulation/model.rs:69`：`Unit` 持有私有 stats，通过 `stats()` 返回只读引用。
- `src/simulation/mod.rs:55`：`spawn_unit` 统一校验、初始化，客户端和测试均使用它。

## 普攻、移动与结算

- `src/simulation/orders.rs:185`：`resolve_orders` 读取基础射程，用于获取敌人、追击和开始前摇。
- `src/simulation/orders.rs:253`：有效攻击距离为攻击距离加双方碰撞半径，射线不可穿障碍。
- `src/simulation/actions.rs:119`：`release_attack` 重新检查射程，按周期减前摇设置独立冷却，
  发出追踪弹丸，伤害取 `stats.attack_damage`。
- `src/simulation/movement.rs:36`：移动预算为 `move_speed * TICK_SECONDS`；冲刺另有独立速度。
- `src/simulation/model.rs:313`：`PendingDamage` 是 `(source, target, f32)` 队列，没有伤害类型或来源说明。
- `src/simulation/combat.rs:108`：`resolve_damage` 直接从血量扣除 amount，并在同一路径处理死亡。
- `src/simulation/combat.rs:141`：重生按静态定义恢复生命/法力，清理冷却，并重置位置。

## 技能与快照

- `src/simulation/abilities.rs:15`：`AbilityEffect` 有 Projectile、Area、Dash、Channel，伤害是 `f32`。
- `src/simulation/abilities.rs:48`：注册时校验目标与效果组合、数值、时长和消耗。
- `src/simulation/actions.rs:155`：`commit_ability` 在前摇结束时复核并统一扣费、进入冷却、生成效果。
- `src/simulation/actions.rs:74`：引导动作保存 damage，后续脉冲生成范围效果。
- `src/client/content.rs:14`：地图、示例技能和单位在 setup 中注册装配，适合修改内容定义和加成系数。

## 调度与测试

- `src/simulation/mod.rs:33`：固定步顺序为 tick、重生、指令、动作、持续指令、移动、弹丸、范围、伤害。
- `tests/simulation.rs:14`：`Arena` 使用无窗口公开接口测试，已有 29 个模拟场景。
- `tests/simulation.rs:383`：验证取消攻击和走砍不能重置普攻冷却。
- `tests/simulation.rs:412`：验证死亡清理与重生；`:439` 验证暂停冻结状态。
- `tests/simulation.rs:585`：验证已提交范围效果在施法者死亡后继续存在。
- `tests/simulation.rs:685`：验证非法定义/出生被拒绝；`:714` 验证显示帧与固定步输入消费。

## HUD 与浏览器

- `index.html:51`：`hero-hud` 是头像、生命/法力、四技能组成的底部卡片，没有属性展示区。
- `src/client/ui.rs:11`：`UnitView` 目前没有七项战斗属性。
- `src/client/ui.rs` 的 `publish` 约每 80 ms 推送快照；`publish_labels` 每显示帧投影头顶血条。
- `web/hud.js` 的 `renderHero` 更新生命/法力/动作，适合增加只读属性展示。
- `src/client/controls.rs` 的 `update_pointer` 使用 `[data-ui]` 隔离 HUD 输入。
- `web/style.css` 在 1150px 和 800px 调整布局；新增左侧属性区必须检查卡片、小地图和相机提示。
- `scripts/browser-support.cjs` 已提供 Pointer Lock 光标、世界坐标点击和临时静态服务器辅助函数。
- `scripts/browser-skills.cjs` / `browser-cursor.cjs` 包含训练哨兵 400→325 等数值断言；
  默认新增法强/抗性为 0 可保留该行为，不应靠删除断言通过迁移。

## Trellis 初始化

- 使用本机已有 CLI 0.6.17：`trellis init --codex --no-monorepo --yes --user asuka`。
- 首次模板将 Rust 游戏识别为 backend；已按真实结构改为 `spec/simulation`、`spec/client` 和共享入口。
- 采用 `trellis-brainstorm` 写方案、`trellis-spec-bootstrap` 写现有规范；不启动功能实施。
- `session_auto_commit: false`，避免归档/日志触发未请求的提交；全局 Codex hook 配置未修改。
- 项目生成 hook 文件；自动注入是否启用由用户已有全局设置决定，当前可手动读取规范和任务，
  不把“已生成文件”等同于“全局自动 hook 已启用”。
