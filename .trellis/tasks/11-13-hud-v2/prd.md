# HUD v2

## Goal

按定稿设计稿（仓库根目录 `ui-hud-v2.html`，未提交）重做战斗 HUD，减少重复信息、压低底部高度、改用游戏界面语言。

## Requirements

| ID | 需求 |
|---|---|
| R1 | 底部三段：左妹妹（当前组远程武器、月光、当前组耐久、Q），中生命与联动，右哥哥（当前组近战武器、烈阳、暴怒、E），左右镜像 |
| R2 | 生命每 100 一块，Boss 两阶段并排；超过 20 块的目标（演武靶 100 万）画成一整条 |
| R3 | Boss 读招：蓄势倒计时、连招第几式、本招各刀与已结算的刀 |
| R4 | 锁定目标：画面内锁定框与距离，画面外边缘箭头（侧方／身后） |
| R5 | 能量、暴怒、耐久用斜切分格；血条、耐力平直；不显示备用武器组 |
| R6 | 页面不持有计时器，伤害残影用 CSS 过渡 |

## 数据

- 新增 `Boss::{telegraph, swings, chain}` 只读接口。
- 新增桥接 `renderBossCue`、`renderLock`；`renderTarget` 增加 `phase, phases`。
- `hud::sync` 移到 `camera::follow` 之后。

## Acceptance Criteria

- [x] Node HUD 测试覆盖分块、两阶段、读招、锁定、按键、分格、联动、越界与死亡清理
- [x] `make check`、WASM Clippy、临时目录 Trunk 构建通过，Cargo.lock 未变
- [ ] 用户在运行中的游戏里验收画面（字体用本机楷体／宋体回退，和设计稿的网络字体会有差别）
