# 数据与架构调查

## 已核对数据（2026-11-08）

| 基础值 | 皇子 | 寒冰 | 近战兵 | 远程兵 |
|---|---:|---:|---:|---:|
| 生命 | 640 | 610 | 465 | 284 |
| 法力 | 300 | 280 | 0 | 0 |
| 攻击 | 64 | 59 | 11 | 21 |
| 法强 | 0 | 0 | 0 | 0 |
| 护甲 | 36 | 26 | 0 | 0 |
| 魔抗 | 32 | 33 | 0 | 0 |
| 基础攻速 | 0.658 | 0.658 | 1.25 | 0.667 |
| 移速（端游单位） | 340 | 325 | 350 | 350 |
| 攻距（端游单位） | 175 | 600 | 110 | 550 |
| 每5秒回血 | 8 | 3.5 | 0 | 0 |
| 每5秒回蓝 | 6.5 | 7 | 0 | 0 |

英雄基础攻速不含皇子E被动/军旗等技能额外加成。小兵为基础未成长口径，当前端游另有成长、对不同目标的伤害倍率与对小兵附伤。
统一100端游距离单位=1世界单位，速度/射程/弹速/半径一并换算，生命与伤害不缩放。面板展示用反向换算，还原340/325移速、175/600攻距，无单位。技能固定一级且全部开放，不实现等级/技能升级。

## 来源
- [英雄基础统计](https://wiki.leagueoflegends.com/en-us/List_of_champions/Base_statistics)
- [近战兵](https://wiki.leagueoflegends.com/en-us/Melee_minion)
- [远程兵](https://wiki.leagueoflegends.com/en-us/Caster_minion)
- [兵线规则](https://wiki.leagueoflegends.com/en-us/Minion)
- [25.S1.1官方数值调整](https://www.leagueoflegends.com/en-gb/news/game-updates/patch-25-s1-1-notes/)

## 关键缺口
完整皇子技能涉及基于当前生命的普攻附伤、护盾、减甲、持续旗帜/攻速光环、Q-E路径联动、指向跳跃和动态地形。
完整寒冰技能涉及被动减速（当前装备无暴击，零暴击没有固定115%增伤）、普攻叠层与强化、扇形多弹丸及同次命中去重、侦察视野、仅英雄拦截的大箭与按飞行距离变化的眩晕。
这些应通过通用定义/执行和效果能力组合，避免以CharacterId在动作、移动和命中中编写特殊分支。

## 技能数据与版本辨析
截至2026-11-08核查端游当前数值，非手游/竞技场模式。官方版本列表可访问但英雄CDN JSON返回403；不得因此声称已从完整DataDragon读取。以官方当前英雄页面、官方补丁和Wiki单项模板交叉核实。

- [皇子官方](https://www.leagueoflegends.com/en-us/champions/jarvaniv/)、[寒冰官方](https://www.leagueoflegends.com/en-us/champions/ashe/)：技能语义。
- [皇子Q数据](https://wiki.leagueoflegends.com/en-us/Template%3AData_Jarvan_IV/Dragon_Strike)：一级90+145%额外AD，45法力/10秒/0.4秒前摇，伤害后10%减甲3秒；线宽136；旗联动范围850，沿途击飞0.75秒。不是旧80+140%额外AD。
- [皇子E数据](https://wiki.leagueoflegends.com/en-us/Template%3AData_Jarvan_IV/Demacian_Standard)：80+80%AP，55法力/12秒，860地点射程，半径200伤害；旗8秒、1000光环、700视野、20%攻速只对友方英雄；自身叠已学被动至40%，多旗光环不重复叠加。超距投到最大范围。
- [皇子完整数据](https://wiki.leagueoflegends.com/en-us/Jarvan_IV)：被动8%命中前当前HP、最低20、非英雄最高400、每目标一级6秒；W护盾60+70%额外AD+每敌英雄1.3%最大HP，4秒，600范围15%减速2秒，30法力/9秒；R指向敌英雄，650射程，0.35秒跳跃，200+180%额外AD，3.5秒地形，0.75秒可二次释放，100法力/120秒。
- [寒冰被动](https://wiki.leagueoflegends.com/en-us/Template%3AData_Ashe/Frost_Shot)：普攻和伤害技能附20%减速2秒；暴击强化减速，暴击率本项目固定0。[25.S1.1官方](https://www.leagueoflegends.com/en-us/news/game-updates/patch-25-s1-1-notes/)明确移除旧115%基础伤害，不照旧攻略保留该增伤。
- [寒冰Q](https://wiki.leagueoflegends.com/en-us/Ashe)：4层普攻生成，4秒后逐层每秒衰减；30法力、6秒、20%攻速，5箭合计110%AD，重置普攻间隔，一次装备on-hit。[26.10官方](https://www.leagueoflegends.com/en-ph/news/game-updates/league-of-legends-patch-26-10-notes/)核实当前110%，[25.23官方](https://www.leagueoflegends.com/en-ph/news/game-updates/patch-25-23-notes/)核实20%攻速；不复刻第一轮多一箭等已知端游bug。
- [寒冰W](https://wiki.leagueoflegends.com/en-us/Template%3AData_Ashe/Volley)：一级7箭，60+100%额外AD，75法力/18秒/0.25秒前摇，1200距离、2000弹速、20宽、27.75°扇面；同敌仅一次伤害且仍挡后续箭，英雄40%强化减速（第一秒衰减到20%）。
- [寒冰E](https://wiki.leagueoflegends.com/en-us/Ashe)：全图地点，1400弹速，1000视野，路径短暂侦察、终点5秒；初始1充能，上限2，90秒恢复，施放间隔5秒，0法力/0.25秒前摇。
- [寒冰R](https://wiki.leagueoflegends.com/en-us/Template%3AData_Ashe/Enchanted_Crystal_Arrow)：首个敌英雄触发，不被兵挡，200+120%AP（附近同额伤害），100法力/100秒/0.25前摇，1500初速随飞行增至2100，1至3.5秒按距离眩晕，400范围伤害/350视野，终点额外1秒视野。28世界单位达到最大眩晕属于本实现显式距离标定（Wiki当前模板没有给出插值端点），不当作已精确复刻端游内部曲线。
- [普攻数据](https://wiki.leagueoflegends.com/en-us/List_of_champions/Basic_attacks)：寒冰2500弹速、21.93%前摇。皇子近战无弹丸，前摇约17.544%。

## 本轮不移植的周边系统
等级/经验/金币/暴击装备和小兵随时间成长不在当前对局规则中；基础值不假装包含这些。小兵对不同目标的特殊倍率和对小兵百分比附伤不属于本次属性替换，保持现有统一伤害规则。
