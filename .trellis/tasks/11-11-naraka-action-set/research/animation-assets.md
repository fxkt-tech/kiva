# Research: 永劫无间横刀与通用动作的原始身份和解析路径

- Query: 本机横刀三段横击、振刀、成功反击、移动/跳跃/闪避及妹妹远程动作分别是什么；是否能从原始资源恢复真实身体与手指运动？
- Scope: internal；只读用户指定客户端和既有项目外提取工具，只在本任务 research 写研究件。
- Date: 2026-11-11

## Findings

### 1. 客户端与索引确实可读

输入根目录是 `/mnt/c/Program Files (x86)/Steam/steamapps/common/NARAKA BLADEPOINT/NarakaBladepoint_Data/StreamingAssets`。没有启动/注入游戏或改写安装。

现场重算 `AppRes.info`：5,881,612 字节，SHA256 `ec96c074370bff349b58f75bcf23f335f3ba8ed53a731f7c3763d5c1a06a48ce`，与之前建立的 `/home/asuka/kiva-asset-research/naraka/identity/appres.sqlite` 来源完全一致。不能把 AppRes bundle index 当 Unity PathID。

使用原 `containers/naraka_sparse.py::SparseBundle` 按需解压 CAB 的对象表和指定对象，`read_typetree(check_read=True)` 验证完整对象消费。29 个具体 AnimationClip（25主体动作与4个默认指层）已现场读取，路径、CAB、原 PathID、对象 offset/size、原对象 SHA256、30Hz 和 duration 见 [animation-core-inventory.json](animation-core-inventory.json)。各 `*.anim.probe.json` 另留实际读取块哈希与预算；没有把这些部分读取宣称为整包 SHA。

### 2. 三段横击有状态和原配置交叉证据

原 protobuf TextAsset `playable_runtime_override_mapping_archive.bytes`，bundle `e/5/e50ef81ba77ae51a`，PathID `4241003489660708546`，对象 17,542,124 字节。其中 `male_hd_weapon_player` 的 archive 范围 `7854318..7908321`，message 53,999 字节。

它的 field5 保存状态 hash 和两个原始 clip 名字符串（子字段2/3）。对原 `AutoAnimatorStates` **全部 23,132 条** string-key index/record 完整 framing，并检查每条记录自身 key 与 index key 相同；`CRC32(原状态名)` 精确等于 override key 的 low32。没有靠文件编号排序猜阶段。**子字段2/3最终如何选择未恢复；两串相同时素材身份无此歧义，两串不同时只能记录两个原候选，不能把第一串当作实际播放结果。**

| 原状态 | CRC32 | 原动作 | 原时长约秒 | 证据与用途 |
| --- | ---: | --- | ---: | --- |
| `attack_101` | 770101480 | `male_hengdao_attack_light_01` | 0.766667 | 左键第1段 |
| `attack_102` | 3035595090 | `male_hengdao_attack_light_02` | 1.066667 | 左键第2段 |
| `attack_103` | 3286806980 | `male_hengdao_attack_light_03` | 1.033333 | 左键第3段 |
| `attack_104` | 1569465447 | `male_hengdao_attack_light_04` | 0.933333 | 另一个原状态，不能凭同为 light 就塞进三段 |
| `attack_105` | 713757937 | `male_hengdao_attack_light_05` | 1.000000 | 同上 |
| `attack_141` | 1233786348 | `male_hengdao_attack_shock_light_01` | 1.833333 | 原振刀后反击片段；用于用户弹反表现须注明 Kiva 适配 |

另一原 TextAsset `combo_graph_archive.bytes`，PathID `4148642664715223420`，包含 **`male_hd_combo_graph` 的 101→102→103 边**；103 节点的原中文备注是“终结技，有霸体，伤害高”。此处只证明原序列和素材身份，**不要求 Kiva 复制原霸体、伤害、输入窗或追加命中**。compact 证据见 [combo-graph-hd-compact.json](combo-graph-hd-compact.json)。未知数字字段保持 field 编号。

完整 85 条定向状态桥、61 个不同 clip、每条源表 record 范围/SHA、override entry 范围/SHA、原路径/包与时长见 [animation-state-mapping.json](animation-state-mapping.json)。[animation-info-selected.json](animation-info-selected.json) 从原 `animation_info_archive` 独立核对 64 位 clip hash/名字/时长。[AutoAnimationClip.hengdao.json](AutoAnimationClip.hengdao.json) 是同 61 个 hash 的原 65-byte metadata record，已消除早期误存的全库 75,889 条。

### 3. 右键必须区分振刀姿态、反击片段和实际成功结算

| 原配置事实 | 确认程度 | Kiva 接入建议 |
| --- | --- | --- |
| `dinner_shock_01`→`male_hengdao_shock_01`，2.0s | override 和真实 Clip 对象均已验证 | 拆招起手的素材候选；防御保持可截取末端/合适姿态，属于 Kiva 播放适配 |
| `dinner_shock_04`→`male_hengdao_shock_04`，2.0s | 同上；weapon_battle_config 有“飞刀被振打击(横刀正振)”并引用对应 shock04 FX | 可证明正振/反弹相关，**不能据此把 shock04 标为“振刀成功”** |
| `attack_141`→`male_hengdao_attack_shock_light_01`，1.833333s | 原状态/素材/攻击轨/同名音频轨均有依据 | 是振刀反击素材。若用于用户要求的“右键弹反/振刀成功动作”，标签写“原振刀反击片段，Kiva 弹反表现适配” |
| 原成功反馈独立 Clip、shock01..08 各分支精确条件 | 未完整恢复 | 不发明一个 success-only Clip；不把播放反击片段当作结算成功证据 |

`AutoAnimTrackAnalysis_AttackAnimInfo` 里 attack_shock_light_01 有三个原 hitId；**不能把这些原打击轨自动变成 Kiva 弹反伤害**。现有防御成功由模拟反馈决定；成功碰撞声音只消费实际 Parry/Deflect 等结算。音频代理已在原 proto_track 证明 shock01/04 条件声→`gman_all_weapon_touch_shock_katana`，attack_shock_light_01 挥击声→同名原 Event，详见本任务音频研究。

### 4. 通用动作对应与远程来源

以下是原状态→clip 的真实桥；是否适合当前 Kiva 时序仍要实际采样与画面验收。

| Kiva 状态 | 原状态/动作依据 | 状态 |
| --- | --- | --- |
| 近战待机 | `dinner_idle_battle`→`male_hengdao_idle_01`，4.0s | 对象已验证 |
| 行走 | `dinner_promenade_battle`→`male_hengdao_promenade_01`，1.2s | 对象已验证；不是不存在的 `walk_01` |
| 跑步/疾跑 | `dinner_run_battle`→`male_hengdao_run_01`，0.8s | 对象已验证；疾跑播放速率/步幅映射为 Kiva 适配 |
| 蹲伏/蹲行 | `dinner_crouch_idle_battle`→`male_hengdao_crouch_idle_01`，2.0s；`dinner_crouch_promenade_battle`→`...crouch_promenade_01`，1.066667s | 对象已验证 |
| 快速蹲行 | `dinner_crouch_run_battle`→`male_hengdao_crouch_run_01`，0.8s | 原状态桥已验证，非本轮25对象之一 |
| 一/二段原地起跳 | `dinner_jump_inplace_up_01/02_battle`→`male_hengdao_jump_inplace_up_01/02`，0.266667/0.233333s | 对象已验证；Kiva 已用跳数决定何时重启 |
| 前向起跳 | `dinner_jump_forward_up_01/02_battle`→对应 `male_hengdao_jump_forward_up_01/02` | 原状态桥已验证，未单独采样对象 |
| 空中下落 | `dinner_jump_down_loop_01_battle`→`male_hengdao_jump_down_loop_01`，0.8s | 对象已验证 |
| 落地 | `dinner_down_to_land_01_battle`→`male_hengdao_jump_down_to_land_01`，0.733333s | 对象已验证；02..06 也在原状态桥，不猜速度阈值 |
| 四方向闪避 | `dinner_dodge_{front,back,left,right}_battle`→`male_hengdao_dodge_{方向}_01`；前2.366667s，其它2.0s | 四个对象均已验证；截取/重定时不是原无敌窗 |
| 普通正面受击 | `common_hurt_front_1`→`male_hurt_stand_light_front_01`，1.333333s | 原共享受击组与对象均已验证 |
| 正面死亡 | `common_die_hurt_front_1`→`male_die_hurt_stand_light_front_01`，5.433333s | 原共享受击组与对象均已验证；不叫横刀专属 |
| 滑铲 | 尚未选定可靠的基础移动 Clip | 不能拿 `slide_attack_*` 当普通滑铲；继续保留明确回退或在实现阶段补原运动组 |
| 妹妹鸟铳瞄准/开火 | 原 **`dinner_female_movement_player`** 子字段2/3分别列出 `male_sniper_ads_idle_01`/`female_sniper_ads_idle_01`，以及 `male_sniper_attack_01`/`female_sniper_attack_01` | **两个不同候选，实际选择字段未证明**。本轮只实读 male 候选（2.0s/2.1s），不能证明女性原角色播放 male 片段。实现可补读/验证 female 候选，或明确选择男性片段作 Kiva 适配 |
| 妹妹弓持弓/射击 | 原 **`female_bow_weapon_player`** 子字段2/3同为 `male_bow_hold_idle_01`，2.0s；同为 `male_bow_attack_01`，1.133333s | 两候选字符串相同且两个对象已验证，**共享素材身份证据成立**；仍不声称原运行时层权重/时钟已恢复，不能伪称女性专属动画 |

原 `attack_*` 的命中窗/耐力/倍率和角色根位移不自动移植；模拟仍是唯一权威。妹妹以现有权威枪口/目标与真握点 IK 适配，不能用源 root motion 移动枪口或物理实体。E/Q/Tab 若采用未验证的源技能槽映射，必须单列候选，不按名称补造。

### 5. 已读出的是 Humanoid muscle，尚不是完整骨骼动作

横刀首段原 Clip：`assets/res/animclipadapter/01/male_hengdao_attack_light_01.anim`，bundle `d/6/d653f1688755a556`，PathID `5670820053815679417`，20,588B，SHA256 `90cb9647d45f0d975749d973393d660be324cba0ee6f4efcd3ce977711790f96`。

曲线层已经实测：

- 7 个 streamed scalar + 113 个 dense scalar ×25帧 +59个 constant scalar =179个标量，与 binding 展开宽度179和 `m_ValueArrayDelta`179项完全一致。
- `typeID=95/customType=8` 的 attribute42..136 是95个 Humanoid muscle bindings。身体/手指不在普通 `m_RotationCurves`，后者为空；7个 generic Transform bindings 主要是辅助/附件，不是全身骨架。
- 独立重放 streamed 三次多项式、按原30Hz dense 时间线采样，0时全部179标量与源 `m_ValueArrayDelta.m_Start` **误差0**；末时与 `m_Stop` 最大差约 `3.13e-13`。中间帧全部有限。
- Stream 包含 `-FLT_MAX` 初值和 `+Infinity` 结束哨兵，哨兵不当作有效动画时刻；JSON 证据用字符串表示 Infinity。
- `m_Events=[]`，原音效/命中事件位于独立 proto_track，不能据空 Unity events 说源片段没有事件。

可重跑 [check-animation-curves.py](check-animation-curves.py)，结果 [animation-curves-validation.json](animation-curves-validation.json)。**这只验证原始标量读取及源端点，不是 muscle→骨骼姿态的正确性 oracle。**

**默认手指层必须单独恢复，缺轨不能填零。** 原 `male_hd_weapon_player` 明确映射 `male_hengdao_finger_left_01/right_01`；原 `female_bow_weapon_player` 明确映射 `male_bow_finger_default_left_01/right_01`。另外原 `dinner_male/female_movement_finger` 提供 unarmed 左右默认层（男女源名不完全相同，两个精选 JSON 保留原 field2/field3 字符串）。

| 已读对象 | 主动作左手指属性数 | 主动作右手指属性数 | 事实 |
| --- | ---: | ---: | --- |
| 横刀 idle（75肌） | 0 | 20 | 左侧缺轨，右侧有作者值 |
| 横刀 jump_down_loop（55肌） | 0 | 0 | 两手缺轨 |
| 横刀 attack01（95肌） | 20 | 20 | 两手都有作者轨 |
| 横刀 default 左/右 | 20/0 | 0/20 | 各20 dense、0.333333s，有轻微原变化 |
| 弓 default 左/右 | 20/0 | 0/20 | 各20 constant、0.333333s |

上述左右集合由原 left/right 命名 Clip 的实际 bindings 证明：左 attributes97..116，右117..136。全部值、原对象 SHA 和稀疏块证据保存在 [finger-layer-values.json](finger-layer-values.json)，可重跑 [probe-finger-layers.py](probe-finger-layers.py)。真实默认值可超出[-1,1]，例如弓左首值约-2.21947，横刀右首值约-1.59504；不能在读入时夹断。必须保留 **缺失属性与显式0作者值的区别**。其它实读片段含55/59/75/95不同mask，不能假设每个Clip都有95肌或完整手指。

原默认层存在已证，**原层混合顺序、权重、加法模式、AvatarMask尚未恢复**。将默认层提供缺失手指、主动作覆盖其实际属性，是可选Kiva适配，不能称为已经恢复原controller规则。H0至少用idle75、jumpLoop55、attack95三个样本验证，不以全95攻击片段通过替代缺轨用例。

原男性 Avatar 已由父线程现场导出：`assets/art/characters/_common/avatar/l22_male_skeleton.fbx`，bundle `5/8/589302f25ec7b3dd`；现有标准 bundle 在 `/home/asuka/kiva-asset-research/naraka/export/rig/589302f25ec7b3dd/converted.unityfs`。`humanoid-avatar.probe.json` 和父/运行时研究说明原 Human axes、pre/post/limits/sign、30手指以及缺少独立原生采样对照的限制。

AppRes 对三个关键名称 `male_hengdao_attack_light_01`、`...02`、`male_hengdao_shock_01` 的完整精确路径查询，只找到 animclipadapter 主片段及 prepare/recover/charge/FX，没有同名 generic/FBX 身体动画副本。`.../animation/hengdao/` 所见 FBX 只有特殊 PVE FX 和武器水下动画，不可据此绕过 muscle。未把每个无名 CAB 对象全部扫描，负面结论限定为索引和已定向读取对象。

### 6. 代码边界

- `src/client/source_animation.rs:139` 仅认 KIVASR01；`:172` 根硬编码 `Bip001`。不能把新 gMan 库强塞现有索引。
- `src/client/avatar.rs:633` 的 source_driver_pose 与 `avatar/hand.rs:91/136` 的手掌/指链按燕云 Bip001 命名绑定；新增来源需明确 rig 元数据和 semantic bindings，保留近期真实掌轴/指链修复。
- `src/client/animation/preview.rs:14/29` 是现有共享查看场景/目录，需暴露三段与新源片段，不能另造查看专用动作计算。
- `.trellis/spec/client/model-assets.md:18` 规定臂段解剖方向、实际掌框、保自身骨长和独立指轨。原 Avatar 参考/IBM 不得为了套曲线而改写。

### 7. 可重跑入口与大件归档

研究脚本均在本目录，使用原只读工具；默认输出本目录。Python环境为 `/home/asuka/kiva-asset-research/naraka/.venv/bin/python`（UnityPy1.25.4），设置 `PYTHONDONTWRITEBYTECODE=1`，避免修改既有工具目录。

1. `probe-animation-assets.py <精确文件名...>` 从 AppRes 精确定位，按需读对象。首段保留完整树；其它片段用 `--metadata-only`。读取已确认10.39MB animation_info/17.54MB override 对象时只为对应 caller 显式提高单次读取上限。
2. 对原 protobuf archives，保存原 TextAsset payload；`probe-animation-proto.py` 严格消费 wire，未知字段保持 numeric。不要把无 schema 的 protobuf推断写成原类字段名。
3. `map-animation-states.py` 重建 string-index表、CRC32→state桥、原override与combo graph compact证据；`probe-animation-tables.py` 完整消费三种已验证key-layout的表，只交付选定61个clip metadata。
4. `check-animation-curves.py` 重放首段原标量，不执行运行时/蒙皮求解。

本轮大临时件可由父线程统一移到 `/home/asuka/kiva-asset-research/naraka/action-set-2026-11-11/`，不加入仓库：

- `*.bytes.payload.bin`：原 archive payload（含同伴共用 `proto_track_archive`，先等其完成）；重跑可由 probe script 从客户端再生成。
- `combo-graph-male-hd.json`：早期完整图展开1.49MB；交付使用 compact JSON。
- `BattleConfig.selected.json`、`override-male-hd-selected.json` 等阶段性试探件可一并外部归档；最终报告只依赖紧凑身份件、原source哈希与可复现脚本。
- 本研究不删除音频同伴的 debug；由拥有者标注归档。

## Related specs / References

- `.trellis/spec/client/model-assets.md`：真实掌轴/指轨、自己的骨长、参考资产和运行时格式边界。
- `.trellis/spec/client/model-viewer.md`：游戏与验收页共用采样、挂点和反向seek。
- `.trellis/spec/client/combat-presentation.md`、`.trellis/spec/simulation/combat-v1.md`：权威动作、事件去重与展示隔离。
- `docs/architecture.md`：client→gameplay→simulation 单向依赖。
- 格式实现依据为本机原 TypeTree、protobuf wire、DDBL 完整 framing，以及只读 `/home/asuka/kiva-asset-research/naraka/containers/README.md` 所记工具版本。本主题未依赖网上示例冒充客户端事实。

## Caveats / Not Found

1. 身份、原曲线、源 Avatar 数据已读出，**完整原生 Humanoid 求解/原始渲染姿态尚未验证**。预/后旋转的数学候选不能取代身体 COM/root、twist、IK 与指骨的原生对照。此项是接入前技术门槛。
2. “振刀成功”独立反馈 Clip 未证明；attack141是真实振刀反击素材，使用时必须明确 Kiva 动作适配。不能把素材播放误报为权威成功。
3. 滑铲和E/Tab原技能对应未收敛；明确回退，或在实现中补证。不能假称已经接入全部对应动作。
4. 29对象之外的表内候选只通过原状态/资源桥，尚未逐个采样/渲染。源时长是事实，Kiva时间窗和截取是产品适配。默认指层的存在不能当作原混合顺序/权重已恢复。
5. 鸟铳女性组的male/female两个字符串不同，原运行时选择字段未恢复。此前研究把第一串解释为女性共用男性片段，属于过度推断，现已纠正。JSON的hash/时长/资源路径仅指其明确标注的首个候选，不可转用于female候选。
6. 所有结果属于 planning 研究，未改生产代码、运行时资产、模拟规则或 task status；未进行提交/推送。
