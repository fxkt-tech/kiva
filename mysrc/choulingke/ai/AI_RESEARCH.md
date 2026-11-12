# 愁铃客 AI 静态恢复记录

调查与产物全部位于项目外。游戏 MPK 只读，不连接、启动或注入游戏进程；没有执行任何游戏 Lua。

## 已证实的资源与格式

- `lua_names.json` 的源路径、MPK pack/offset/size 同时匹配后才读取。
- 当前安装版本采用 `LUAL` 容器，而公开参考仓库记录的是较早的 LuaText/LZ4 包装；不能混用包装解码器。
- 已实现 `ai_export.py`：校验重复头长度、体积总和、Zstd 字典、每个代码/调试块范围、元数据消耗、全部块恰好连续覆盖正文；重新组装 Lua5.4 Proto 并用独立 walker 全量校验。
- 输出 `extracted/ai/*/{original.lual,custom.luac,remapped.luac,functions.json,frames/}`；完整来源与 SHA256 在 `export_manifest.json`。
- 初始 CLK/ylk/choulingke 范围 53 个脚本全部成功重建，无未知 opcode；随后扩展 AI 节点实现用于核对行为语义。

## 反编译的准确性边界

使用 [ArArgon/yysls-re](https://github.com/ArArgon/yysls-re) 的 opcode 次序参考（commit 见 manifest）；没有把该源码拷入 Kiva。参考仓库未见许可证文件，研究性导入保留原路径，不作为可再分发依赖。

标准 [unluac](https://sourceforge.net/projects/unluac/) 会在第一个 NEWDICTPROXY 处失败：该优化指令在 k=0 时没有 stock NEWTABLE54 的后继 EXTRAARG。隔离参考源码与许可证位于 `tools/ai/unluac-source`，最小补丁保存在 `tools/ai/unluac-messiah-newdict.patch`，只允许省略数组容量扩展，不修改原指令 PC / 跳转 / 常量 / 调试数据。OpenJDK、Lua5.4 也仅解包到外部 tools 目录，没有全局安装。

重要：`remapped.luac` 是供经过上述兼容补丁的静态反编译器使用的研究中间产物，不能声称是可执行的 stock Lua chunk。

发现公共参考还存在优化指令操作数语义未充分覆盖：GETTABUPFAST 的 C 在样本中固定为 1，而所需字段通常是 K0，直接套 stock GETTABUP 会产生 `Class.ctor.ctor` 之类错误。核心五棵行为数据树不使用这类指令，其常量表和节点构造可靠；复杂 common/client 实现的可读稿需要逐指令复核，语法通过不等于语义完全恢复。失败日志与完整反汇编全部保留。

## 核心行为树

| 源树 | 节点数 | 完整静态反编译 / 节点表 |
|---|---:|---|
| `51000025_CLK_fight_P1` | 192 | 是 |
| `51000025_CLK_fight_Action` | 192 | 是，60 路 `_ACTION_FLOW_` |
| `51000025_CLK_P1_ComboCondition` | 122 | 是 |
| `51000031_CLK_fight_P2` | 195 | 是 |
| `51000031_CLK_P2_ComboCondition` | 107 | 是 |

`ai_nodes.py` 只用 Lua AST 读取返回的字面量节点表，对常量 string.split / 非空过滤做有限静态折叠；不执行 Lua。每条节点保留反编译稿行号，父子引用完整。技能槽与动作流分派保存在 `extracted/ai/action_slots.json`。

## 可迁移的已恢复规则

- P1：一次性开场侧移约 2 秒（move_speed_rate=0.7）后选择 Action45 / slot90021641。
- P1 常规：距离 >12 选择 Action3（追到距离 6）；6–12 米选择 `[3,14,16,45]`、权重 `[10,50,50,50]`；近距离选择 `[1,10,12,11,19]`、权重 `[1,50,50,50,1]`。
- P2：一次性开场 Action21 / slot90021604。常规近距离池 `[50,25,53]` 等权；中距池 `[48,49,24,3,21]`、权重 `[50,50,50,10,50]`；远距依旧 Action3。
- Action1/2/3 为绕行/进退等移动动作，其余共 57 个 ApplySkillAction 映射到 9002xxxx 技能槽。20901006 是被检查的来袭玩家技能，不能误作 Boss 技能。
- ApplyActionByCondition 会先用 owner 的 action_flow_value 覆盖初始权重，再按累计权重抽取；不是简单均匀随机。
- ApplySkillAction 的 action_flow_modify / time / key / value 给出了将权重置零的动作流冷却；具体时间单位仍需 owner 实现确认。
- P1 树中有 `HP/HP_MAX <=65` 后加 buff450097；P2 树有 `<=75`、`<=30` 以及 CA_timer70、xihun_timer40。不能把这些任意一个宣称为 P1→P2 阶段转换阈值。
- 独立 `boss_clk_die_P1` 使用 slot90021659、同步死亡后销毁实体；尚未完整恢复关卡如何启动 P2。

## 移植方案与缺口

Kiva 使用专用 Rust Boss 状态机与精选源决策数据，不嵌入 Lua 运行时。玩法仅装配单一 Boss；模拟拥有决策、冷却、预警、命中、卸势打断、无敌、阶段与复活；客户端只读 action/slot/阶段/时间快照。玩家现有联动与被动大号靶子保留。

待协调/恢复：9002xxxx 槽到原动画 graph/skill id 的映射、原命中体积与伤害、攻击活跃帧、阶段跳转、buff/抓取/索魂系统以及 FAST 指令准确操作数语义。不能把尚缺的参数冒充恢复完整；Kiva 的平衡、伤害几何及精简机制须单独标成适配参数。

原骨架/动画由另一 worker 研究：`Char/c_77_boss_yulingke/yulingke.skeleton`、`animation_yulingke_clips.json`，确切别名为 yulingke。

## 检查

- 核心五树均经隔离 Lua5.4 `luac -p` 静态语法检查。
- `test_ai_export.py` 覆盖真实 274 函数 P1、全部元数据/块消耗、损坏长度拒绝、核心节点数与父子引用闭包。
- 完整反编译状态和语法结果见 `decompile_manifest.json` / `syntax_validation.json`；部分泛用节点与 common/client 尚有失败，不能把其截断稿当完整源码。

## Kiva 首版实现落地

`src/simulation/boss.rs` 保存唯一Boss阶段/动作/冷却与权限快照；实战模式改为单Boss，靶子仍为1.45倍被动目标。普通Enemy主动AI/复活显式跳过Boss；Avian仍是唯一移动与碰撞求解器。核心距离字段经 `CheckDisAndAngleDecorator.check_pass_by_dis` 核对为完整3D向量长度，Kiva用实际脚底3D快照保持6/12数值；不暗按模型归一高度缩放。

随包摘要 `assets/choulingke/boss_ai_manifest.json` 明确原始规则与Kiva适配边界。9项Boss真实FixedUpdate回归覆盖源开场/单次命中、正背面卸势、实际闪避无敌、玩家击飞+狙击仍有20+85伤害、墙体、暂停与HitStop、双生命阶段/过渡免疫/复活、玩家死亡取消攻击；170个库测试和9个林地物理集成回归、all-target Clippy通过。
