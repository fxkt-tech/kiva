# HUD 桥接层：规则与武器数据只由 Rust 提供

## 背景

`web/hud.js` 保存了两份本该只在 Rust 里的数据：

- `const COOLDOWN = { melee: 3, ranged: 4 }`：技能冷却总时长，注释指向已不存在的 `combat.rs`。真实值在
  `src/simulation/combat/skills.rs` 的 `SkillSpec.cooldown`，调整平衡后冷却水位会显示错误。
- `const sets = {1: {...}, 2: {...}}`：每组武器的名称、类别字、E / Q / Tab 文案和联动条件。武器组表现本应只在
  `client/loadout.rs` 的 `Loadout` 行（规范：“HUD 通过 `loadout::of(pair)` 读取，不得按具体武器组写分支”），
  新增武器组时还要改 JS 和 `set === 2 ? 2 : 1` 的截断。

这是桥接层整改的第 1 步；字段化快照（`HudFrame`）、契约 fixture、变化检测属于后续任务。

## 需求

1. 冷却总时长来自技能表：模拟层在 `ActionKind` 上公开只读的 `cooldown()`（与 `shot_times` 等展示事实同处），
   客户端按当前武器组的 E / Q 技能取值，随 `renderResources` 发送；JS 删除 `COOLDOWN`。
2. 武器组文案进入 `Loadout` 行：近战 / 远程名称和类别字、E / Q / Tab 描述、联动条件。客户端在装配时把全部行
   以 JSON 一次发给页面（`kivaHud.defineLoadouts(json)`，以 `hud_index` 为键）；JS 删除 `sets`，
   `renderLoadout` 接受任何已定义的组号。
3. 页面在收到定义前不绘制依赖文案的部分，不抛异常；未知组号不绘制。
4. JSON 字符串转义复用现有的无依赖实现（从 viewer 提出为客户端共享模块），不新增依赖。

## 不做

- 不改其余 `render*` 的位置参数和每帧调用方式，不删除 JS 未使用的字段（后续任务）。
- 不改原生窗口标题文案和 `melee_name`。

## 验收

- `rg "COOLDOWN|const sets" web/hud.js` 无结果；HUD 文案和水位与改动前一致。
- Rust 测试：每组 `hud_index` 唯一、JSON 可被解析且包含全部字段；`ActionKind::cooldown()` 与技能表一致。
- Node 测试：定义前 / 未知组号不报错；冷却水位使用传入的总时长；文案来自定义。
- `make check`、WASM Clippy、临时目录 Trunk 构建通过。浏览器画面需用户验收。
