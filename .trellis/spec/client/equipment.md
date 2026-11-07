# 装备管理、输入与HUD

## 1. 适用范围

`src/client/equipment.rs`拥有管理适配和只读投影；`web/equipment.js`拥有装配界面及装备按钮DOM。
父级`hud.js`继续拥有暂停/捕获/会话和共用tooltip。核心规则见[装备契约](../simulation/equipment.md)。
首版为训练场直接装卸，不含商店或经济。

## 2. 接口

固定步前顺序为会话请求→装备管理→输入→演示AI；管理在暂停时仍消费。
`EquipmentAdapter { owner, ack }`仅在玩家改变或请求产生结果时变更，不逐帧重置导致HUD限频失效。

```text
data-equipment-requests = JSON array, maximum 32 entries
Equip { type:"Equip", player_id:String, slot:usize, equipment_id:u16, request_id:u32 }
Unequip { type:"Unequip", player_id:String, slot:usize, instance_id:String, request_id:u32 }
data-equipment-skill = JSON { slot:usize, instance_id:String }
```

请求类型封闭且`deny_unknown_fields`。玩家ID是当前Entity bits的字符串，实例是当前格实例Display字符串；
只比较后取回typed handle，不解析任意字符串创建模拟身份。

## 3. 请求、投影与交互契约

管理必须为当前Active玩家、模拟或DOM已暂停且实际未捕获；死亡仍可调整持久装备。
整批先解析结构，任何结构错误拒绝整批；结构有效后语义按FIFO逐条独立执行。
只调用核心装卸入口，不直接写属性、冷却或动作。

```text
equipment: {
  slots: [{slot, item:null | {id,instance_id,name,active:null | AbilityView}}], // 固定6格
  first_empty_slot: null | usize,
  can_manage: bool,
  ack: {request_id: null | u32, accepted: bool, error:null | String}
}
equipment_catalog: null | [{id,name,description,bonuses:[String],
  passives:[{name,description}], active:null | AbilityDescription}]
equipment_aim: null | usize
```

- 有效结构请求回显request_id；整批解析失败回`request_id:null, accepted:false, error`。
  UI确认请求队列已消费后接受这个通用错误并解除pending，允许用户重试。
  队列可含多个意图但投影仅保存最终回执，当前UI逐个等待，不依赖中间回执。
- 目录仅首次成功完整快照发送，此后为null；JS缓存只读目录（诊断`__kivaEquipmentCatalog`），
  不在每80ms快照重复静态内容。运行时格/CD/权限来自Rust。
- `AbilityDescription`提供无slot的共用说明；旧`AbilityMetadata/AbilityView`仍为扁平字段。
  装备active用实际格，原`abilities`仍是四技能，原`aim`仍只表示角色技能。
- `1–6`或真实游戏光标点击`[data-equipment-slot]`，共用`select_cast/submit_cast`；
  目标、范围和可用性按`CastSource`解析，瞄准携带实例，卸装后失效。
  自身目标立即提交，其余复用现有目标交互，Enemy点空地/友方保持瞄准。
- `data-equipment-skill`独立于原`data-skill`，HUD点击在同一个鼠标事件内消费。
  无玩家、暂停、just_captured边界排空旧意图，不能恢复后重放或释放新实例。
- P或暂停菜单打开装配界面并释放捕获；关闭/Esc回到暂停菜单，继续试炼才恢复捕获。
  面板按Rust首个空格装备；满格禁用、卸装带当前实例；pending期间阻止继续/重开与重复操作。
  Selecting隐藏面板、清本地pending/旧装备技能意图；普通恢复不清库存。
- HUD装备使用独立选择器，不复用`.ability`或`data-slot`；保留144px七属性、无单位和原四技能。
  小窗口调整布局/隐藏portrait并将地图移到HUD上方；装配目录可滚动、关闭和反馈保持可达。

## 4. 校验与错误矩阵

| 情形 | 处理 |
|---|---|
| 非数组、超过32条、缺字段、未知字段/类型、非法数值 | 整批拒绝，null回执解除UI等待 |
| 无玩家或旧玩家ID | 拒绝且消费，不影响下一局 |
| 仍运行或实际捕获中 | 拒绝管理，不信任仅DOM展示 |
| 越界/满格/空格/未知装备/旧实例 | 适配层或核心入口拒绝并回显可读错误，原装备和属性保留 |
| 空格/无主动/冷却/受控时选装备技能 | 只读可用性反馈；模拟仍二次校验 |
| 选择/暂停/恢复边界遗留主动意图 | 丢弃，不提交也不保留瞄准 |
| 目录已投影，后续快照catalog为null | 继续用只读缓存，不清空目录 |

## 5. 正常、默认和错误案例

- 默认：开局空六格；P打开装配，属性来自实际单位，关闭保持暂停。
- 正常：重复锋刃叠加，卸其中一格保留另一格；法珠6键/鼠标施法，Q冷却不变。
- 正常：用旧实例发Unequip，显示“装备已经变更”，当前装备保留；非法批次后按钮可重试。
- 错误：JS计算+20攻击、用旧slot点击释放刚换的新装备、恢复时重新执行暂停前装备点击。

## 6. 必须验证

`src/client/equipment/tests.rs`覆盖严格批次/门控/身份/回执/投影；controls测试覆盖键鼠选择、实例瞄准及捕获排空。
`scripts/browser-equipment.cjs`在1280×800、800×600、1000×720 DPR2验证实际装卸/满格/属性、被动命中与卸除、
鼠标/数字键主动、独立冷却、非法目标、冲刺/引导及卸装、暂停冻结、旧实例和整局清空。
原stats/cursor/skills/healthbars/characters/reselection/smoke/startup按影响回归，JS用`node --check`。
截图与像素断言用有窗口Chromium/独占临时显示，关闭自己的fixture，不接管开发服务。

## 7. 错误与正确

错误：Web维护一套装备属性/CD状态，主动发送任意技能ID；新面板接管捕获或再创建tooltip。

正确：Web仅提交封闭的格/装备ID/实例意图，Rust投影实际结果；共享父级暂停/捕获和tooltip所有者。
