# 属性 HUD 契约

## 1. 适用范围

`src/client/ui.rs` 的 `AttributesView`、`UnitView.attributes`、`web/hud.js` 和 `#hero-stats`。
模拟规则见 [../simulation/attributes-and-damage.md](../simulation/attributes-and-damage.md)。

## 2. 接口与选择器

- `UnitView.attributes` 是七项模拟世界实际数值；`display_attributes` 是同一结果的显示投影，移速和攻距经统一100:1反向换算。
- 选角预览同样提供两份，DOM只读取`display_attributes`，不能在JS里补算。
- `#hero-stats` 是 `#hero-hud` 的第一个内容区域，继承 `data-ui` 隔离。
- `[data-stat="<field>"]` 的 textContent 仅包含数字文本，面板不显示单位。
- `player_id` 对应单位决定玩家属性，不能从固定角色常量取值。
- 属性区常规宽度为 144px，缩窄窗口可进一步收紧；保留七项可读数值和正确配对顺序。

## 3. 字段与显示契约

| 字段 | 标签 | 小数位上限 | 数值含义（不展示单位） |
|---|---|---|---|
| attack_damage | 攻击 | 1 | 数值 |
| ability_power | 法强 | 1 | 数值 |
| armor | 护甲 | 1 | 数值，可负 |
| magic_resistance | 魔抗 | 1 | 数值，可负 |
| move_speed | 移速 | 2 | 端游距离单位/秒 |
| attack_speed | 攻速 | 2 | 次/秒 |
| attack_range | 攻距 | 2 | 端游距离单位 |

Rust 从 `EffectiveAttributes` getter 投影；只在AttributesView::display调用gameplay::units统一转换距离，不在JS重算属性。
格式化移除无意义尾零；非零值四舍五入会变 0，或绝对值达到 1e6 时使用简短科学计数法。
死亡保留当前属性显示。完整快照约 80 ms 一次，血条位置仍走独立逐帧通道。

## 4. 缺失与错误处理

| 情况 | 行为 |
|---|---|
| 缺少玩家或某项值非有限/未提供 | 对应位置显示 `—` |
| 负护甲/魔抗 | 保留负号 |
| 零攻速/移速 | 显示 0，不根据动画猜测实际值 |
| Pointer Lock 下点击属性 | 不移动、不攻击、不释放/取消瞄准中的技能 |
| 小窗口 | 重排卡片、移动小地图，不隐藏七项属性 |

## 5. 正常、默认和错误示例

- 正常：模拟返回 attack_damage 75，数值元素显示 `75`。
- 默认：寒冰移速 `325`、攻速 `0.658`，仅显示标签和数字。
- 错误：显示值从 `BaseAttributes` 硬编码，或在 JS 中重新叠加装备属性。

## 6. 验证

`scripts/browser-stats.cjs` 验证真实 WASM 快照与七个 DOM 数值，检查指定标签顺序、字段配对、
属性区不宽于145px且不展示单位，以及技能、卡片和小地图的边界。
必测 1280×800、800×600、1000×720 DPR=2，及属性区输入隔离、暂停/恢复。
同时运行既有技能、光标与血条回归。浏览器脚本可以从 `KIVA_SITE_DIR` 自建临时静态 fixture。

## 7. 错误与正确用法

错误：每次 `kiva-labels` 都更新整个属性面板，或把卡片内容放在没有 `data-ui` 的外部浮层。

正确：属性随 `kiva-state` 更新，坐标随 `kiva-labels` 更新；属性放在角色卡片左侧，并复用既有 UI 输入隔离。

属性DOM与预览使用Rust的display_attributes；world attributes仍用于实际几何/规则诊断。移速与攻距经统一league_units显示端游数值，
其余五项不变；装备flat距离同口径。初始攻速读取已学技能贡献，不在JS为皇子另加E被动。
