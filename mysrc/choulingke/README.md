# 愁铃客镇守：原始动画与 AI 资料

本目录包含 129 条动画注册记录，其中 120 个动画文件已复制；另外 9 条没有已提取文件，在 `manifest.json` 中标为 `local_file: null`。动画包含攻击、受击、移动、处决等，文件数量不代表攻击招式数量。复制时检查了全部动画 SHA256，120 个文件均通过随附解析器读取。

## 文件用途

| 路径 | 用途 |
| --- | --- |
| `manifest.json` | 本目录入口：动画名、注册时长、相对文件路径、原 VFS 路径、资源包位置和 SHA256 |
| `animation/*.anim` | 已解包的原始 Messiah `CHAR::ANIM` v15 动画，不是 glTF/FBX |
| `metadata/animation_yulingke_clips.json` | 完整动画注册清单与时长（毫秒） |
| `metadata/animation_yulingke_cues.json` | 原动画事件 Cue、音效引用及资源来源 |
| `metadata/animation_yulingke_hierarchy.json` | 原主骨架 986 个节点、父子关系及局部/全局矩阵 |
| `metadata/yulingke.skeleton` | 原始主骨架文件 |
| `metadata/yulingke.skeleton.json` | 骨架解析树 |
| `metadata/yulingke_clips_extraction_manifest.json` | 原提取记录，包括资源定位与缺失项 |
| `metadata/animation_v15_decode_report.json` | 原解析研究报告，范围可能包括共用动画 |
| `tools/messiah_anim_v15.py` | 可独立使用的 Python 标准库动画解析器 |
| `ai/action_slots.json` | AI Action 编号到技能槽位及节点参数的静态提取表 |
| `ai/51000025_CLK_*` | 一阶段行为树、动作分派和连招条件 |
| `ai/51000031_CLK_*` | 二阶段行为树及连招条件 |
| `ai/boss_clk_die_P1/` | 一阶段死亡行为资料 |
| `ai/AI_RESEARCH.md` | 原研究记录及恢复边界，包含历史 Kiva 实现说明 |
| `ai/*manifest.json`、`ai/syntax_validation.json` | 原脚本来源、反编译及语法检查记录，覆盖范围大于本目录精选脚本 |

各行为树目录内，`decompiled.lua` 是静态反编译可读稿，`nodes.json` 是节点数据，`disassembly.txt` 用于逐指令核对。它们用于研究，不应直接执行。

原研究文件中的 `/home/asuka/kiva-asset-research/...` 绝对路径保留为来源记录；查找当前副本请用顶层 `manifest.json` 的 `local_file`，该字段相对于本目录。

## 先找攻击动画

原动画虚拟路径前缀为 `Char/c_77_boss_yulingke/`。以下是按命名整理的系列，尚未恢复对应中文招式名：

| 系列 | 动画名 |
| --- | --- |
| 一阶段主系列 | `skill_ylk_1` 至 `skill_ylk_12` |
| 一阶段分支 | `skill_ylk_1_2`、`skill_ylk_5_1` |
| 二阶段主系列 | `skill_ylk_p2_1`、`skill_ylk_P2_2` 至 `skill_ylk_P2_14`（注意大小写） |
| 化解相关 | `skill_ylk_huajie_6_1` |
| 多人相关 | `dr_Skill_01_*` 至 `dr_Skill_05_*`，原虚拟路径位于 `duoren/` |
| 其他相关 | `attack_02_01`、`skill_ylkzl_skill_1` 至 `skill_ylkzl_skill_4` |

`_hit`、`_behit`、`_executebehit` 等片段需要结合注册、事件和动作内容判断；不能全部算作 Boss 主动攻击。完整动画时长也不等于攻击前摇或命中窗口。

在 **Kiva 仓库根目录**执行以下命令，列出 `skill_ylk` 系列及当前文件位置：

```sh
python3 - <<'PY'
import json
from pathlib import Path
root = Path('mysrc/choulingke')
manifest = json.loads((root / 'manifest.json').read_text())
for clip in manifest['clips']:
    if clip['name'].startswith('skill_ylk'):
        path = root / clip['local_file'] if clip['local_file'] else '未提取'
        print(clip['name'], clip['duration_ms'], 'ms', path)
PY
```

## 读取骨骼动画

Python 3 即可读取，无需游戏运行或 Windows 安装目录：

```sh
python3 mysrc/choulingke/tools/messiah_anim_v15.py \
  mysrc/choulingke/animation/skill_ylk_1.anim
```

输出包含 `duration_ms`、骨骼数量、具名骨骼数量、编码名字引用数量和轨道数量。例如 `skill_ylk_1` 的解析时长是 6267 ms，有 88 根完全具名的骨骼。

需要关键帧时可以作为 Python 模块导入：

```sh
python3 - <<'PY'
import sys
from pathlib import Path
root = Path('mysrc/choulingke')
sys.path.insert(0, str(root / 'tools'))
from messiah_anim_v15 import parse_animation
anim = parse_animation((root / 'animation/skill_ylk_1.anim').read_bytes())
for bone in anim.bones[:5]:
    print(bone.name, 'parent:', bone.parent)
    print('平移时间(ms):', bone.translation.times_ms[:5])
    print('平移值:', bone.translation.values[:5])
    print('旋转值:', bone.rotation.values[:5])
PY
```

`translation`、`rotation`、`scale` 分别保存时间与值；`parent` 是当前动画骨骼列表中的父索引，`-1` 表示无父节点。无 scale 轨时不要把空列表解释为零缩放。

解析成功仅代表读取了轨道数据。部分动画的骨名使用尚未解开的共享名字引用，解析器保留在 `encoded_name_reference`，此时 `name` 可以为空。不要猜测骨名或直接按索引绑定到主骨架。

## 查看 AI 出招槽位

```sh
python3 - <<'PY'
import json
from pathlib import Path
actions = json.loads(Path('mysrc/choulingke/ai/action_slots.json').read_text())
for action in actions:
    for node in action['nodes']:
        if node.get('cls_name') == 'ApplySkillAction':
            print('Action', action['action_id'], '=> slot', node['skill_slot'])
PY
```

例如一阶段开场 Action45 对应 `90021641`，二阶段开场 Action21 对应 `90021604`。完整动作分派有 60 个 Action；其中 Action1/2/3 是移动相关，其余 57 个有技能应用映射，不能等同为 57 种不同攻击。

技能槽与动画文件编号的逐一对应仍未完整恢复。不能把 Action10 或槽位末尾数字直接对应到 `skill_ylk_10`。距离选择、权重、冷却等恢复情况见 `ai/AI_RESEARCH.md`，再到对应树的 `nodes.json`、`decompiled.lua` 核对。

## 在 Kiva / Bevy 中使用

本目录是资料源，放入这里不会自动加载进游戏。Bevy 不能直接把这些原生 `.anim` 当作 glTF 动画播放，需要先完成骨名解析、骨架绑定、坐标/单位转换、关键帧采样和运行时格式转换。

仓库已有转换入口 `scripts/import-choulingke-animations.py`，运行时动画位于 `assets/choulingke/animations.bin`，相关说明见 [现有资源说明](../../assets/choulingke/README.md)。该导入器使用原研究目录布局与其他依赖；这里是精选资料副本，不能直接用它替换完整研究目录。要直接消费这里的数据，应从 `manifest.json` 的本地路径开始读取，再按导入器的绑定和转换逻辑适配。

接入战斗时，模拟层负责出招时钟、命中窗口、伤害、冷却和阶段；客户端按模拟快照播放动画。原 Cue 可作为研究线索，其音效引用不代表本目录包含对应音频，也不能把普通 Cue 一律当作伤害事件。原伤害、命中体积、攻击活跃帧及阶段跳转尚未完整恢复，后续自行设定的参数应明确记录为 Kiva 适配值。

## 校验文件完整性

以下命令无需原研究目录或游戏安装目录：

```sh
python3 - <<'PY'
import hashlib
import json
from pathlib import Path
root = Path('mysrc/choulingke')
manifest = json.loads((root / 'manifest.json').read_text())
count = 0
for clip in manifest['clips']:
    if clip['local_file'] is None:
        continue
    path = root / clip['local_file']
    if hashlib.sha256(path.read_bytes()).hexdigest() != clip['sha256']:
        raise SystemExit(f'校验失败: {path}')
    count += 1
print(f'校验通过: {count} 个动画文件')
PY
```

这些原作资源及脚本不属于项目自制或 CC0 资产；随附工具的研究参考见其文件头。目录不包含原游戏运行环境、模型网格、贴图、音频或完整技能配置。
