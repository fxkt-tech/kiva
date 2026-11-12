# 永劫动作、来源与音效

## Scope / Trigger

修改 `source_animation`、`animation/naraka`、源姿态重定向、`audio`、`web/audio.js` 或 `scripts/*naraka*` 时阅读。权威连段与防御见 [战斗规则](../simulation/combat-v1.md)，实际皮肤／掌框见 [模型资产](model-assets.md)。

## Signatures

- `SourceAnimationLibrary::for_rig(SourceRig::{Yanyun,Naraka})` 选择独立骨架；`SourcePose` 显式携带 rig、scope、model、weight。旧库默认 API 保留燕云语义。
- `SourceScope::{FullBody,UpperBody,Fingers}` 限定目标组合范围，游戏与源对照共用；不得用不同拓扑的局部数组直接混合。
- `RigPose::frame` 读取 `ActionSnapshot.melee_stage/elapsed/duration`、`DefenseSnapshot` 和实际运动状态；伤害 marker 不重启动作。
- Web 保留 `playEffect(name)`、`playSniperShot()`、`setActive(bool)`，新增只读 `cueOffset(name) -> number`。原生音频为空操作。
- `python3 scripts/prepare-naraka-animations.py --verify` 校验冻结源件、原生证据并逐帧重算 runtime 插值／FK／真实手部 LBS；`--repackage` 验证后再生成运行时文件。普通 verify 不依赖游戏目录、Unity 或第三方 Python 包。
- `python3 scripts/import-naraka-audio.py --verify assets/prepared/naraka/audio` 从冻结 WEM／无损 PCM 重建并核对 Ogg、cue、manifest；需要 ffmpeg/ffprobe，普通 verify 不读取游戏或调用 vgmstream。

## Contracts

`assets/naraka/animations.bin` 为 KIVASR02，包含独立 `naraka/l22_male_skeleton`、63 骨、44 原片段。显式小端：magic8；u16 长度 UTF8 rig ID；u16 joints/clips/motion-root/flags；每骨 name、i16 parent、rest T3/Q4/S3；每片段 name、f32 duration、u16 tracks；每轨 u16 joint，T/Q/S 各 u32 key count 与 f32 seconds＋分量。KIVASR01 原毫秒键兼容。源 rest、目标自身 rest／IBM 分开保存。

原 Humanoid clip 必须通过真实 Avatar 的原生 PlayableGraph 求局部 TRS；不得把肌肉数组直接当局部角度，也不得经有损 GetHumanPose→SetHumanPose 回写作为烘焙路径。来源版本、求值器版本及差异写 manifest。缺失整条三节手指链时，才以原 default finger clip 的独立原生局部姿态补层；主片段已写出的指链优先。这是明确的 Kiva 组合，不能称恢复了未知原控制器权重。

原生旋转存在近瞬时分支切换。自适应采样须覆盖四分点／中点／四分之三点，并用未参与选键的偏移时刻独立验证。切换两侧用相邻 f32 时间键保存；运行时按真实正 dt 插值，不能用 `max(EPSILON)` 改写微小间隔，精确命中键直接取该键。完整验证必须检查独立帧的全骨 FK 与真实权重手部点，不能只对导出键自证。

哥哥横刀三段分别取 `male_hengdao_attack_light_01/02/03`，由权威段内比例映射到源时长。拆招／持续防御取 `male_hengdao_shock_01`，有效防御释放后的表现取 `male_hengdao_attack_shock_light_01`，0.60s 可打断表现不延长 0.18s 弹反判定；RigPose锁存已打断gesture_id，同ID短暂移动／受击后不恢复，连显示帧跨过整段受击时也按本帧真实Hit事实打断；该源片段已证反击用途，未证“成功专用”。成功声音只取真实结算事件。

待机、走跑、蹲伏、二段跳、四方向闪避、落地、受击和倒地使用明确对应片段。蹲伏普攻／防御只组合来源上身，保留目标蹲伏过渡和下肢；移动中的实际防御同样采用振刀上身＋真实步行下身，移动只抢占已结束判定的弹反尾势。E／Tab／滑铲保留原编排／回退，trace 必须说明。玩家 F3 两外观都可用永劫来源，Boss 仍用燕云。空气中剔除永劫 Pelvis 额外 Y（MotionRoot 的 T 为零），避免重复物理跳高；源对照保留原骨盆曲线。

妹妹枪姿使用原 `male_fk_hold_idle_01`／`male_fk_attack_01`，female/male FK override 实际共用这些片段；不能把通用 sniper 候选说成鸟铳专属。枪弓只组合来源上身／手指，保留目标下肢／骨盆，再执行真实武器双手 IK。源对照也遵守声明 scope，隐藏手持且不叠加武器 IK；独立 Fingers 片段保留参考身体。实际臂向、掌框和每指局部 FK 仍遵守模型资产规范。

音频 `naraka-cues.json` version=1：每 cue 的 priority(0..3)、offset(0..2s)、1..4 layers；每层 delay(0..2s)、1..16 个 variant(file/weight/gain)。仅允许固定 16 名 cue、`naraka-[0-9]{1,10}.ogg` 同源文件、正且有限权重、gain∈[0,1]，最多 64 媒体。当前 48 Ogg；16 名中包括可选通用死亡语音，不自动叠加。

原 Wwise Event→PlayAction→容器→Sound→DIDX/WEM 身份及变体保留。落地两条 PlayAction 是两个声音层，原 bank 的 100ms 延迟与零延迟分别保留；未证明的原 track 时间单位不猜。cue offset=0 是 Kiva 起手适配。波形不变调，variant gain 对应解码峰值预算，乘全局音效增益一次（`web/audio.js` 的 `EFFECT_VOLUME`，当前 0.45；BGM 为 `MUSIC_VOLUME` 0.18）。实际成功闪避结算播放用户选定的 `move_soul_dodge_1p` 三个等权变体（cue `naraka_dodge_success`），普通闪避起手声独立保留；来源调查见 [成功闪避音效](../../../docs/plans/naraka-success-dodge-audio.md)。

`MeleeStart` 以 actor/action_id/pair/stage 和当前权威 elapsed 排起手声，超过预定点 0.12s、身份失效或动作结束即丢弃；旧 Melee marker 只管伤害／FX，不再重复挥声。实际 Hit／Deflect／Block／Parry 和 MotionFeedback 分别产生反馈；空右键没有成功声。Shot（actor=Companion） 按不可变 pair 播 `naraka_gun`／`naraka_bow`，伴随 SniperShot 不重复。

最多四个短音效声部；整组层原子预留，低优先级脚步不能抢成功声。CaptureState在实际Cancelled采样时递增cancellation_epoch；AudioState发现epoch改变，将该显示帧按inactive消费，清待播并前移两游标，涵盖同帧失焦重捕获。声音未解锁／未解码／失锁／失败即丢弃，不能排队恢复后重放；BGM 单循环另计。可信进入／画布点击才解锁 Web Audio，异步缓存复用。

## Validation & Error Matrix

| 输入／状态 | 处理 |
|---|---|
| 错 rig ID、坏父索引／时间、非有限 TRS、非法 quat 或尾部数据 | 有界解码拒绝；不能静默标为新来源成功 |
| 两来源拓扑不同 | 通过目标语义 binding／CC0 过渡，不 zip 源数组 |
| 原片段缺整指链 | 显式原 default finger 补层，记录来源；不填零旋转 |
| 原 FK 上身含低骨盆 | 按 scope 保留目标下身，不能靠查看页偏移掩盖 |
| 暂停／同帧失焦后重捕获 | 清待播 cue、前移事件游标，已承诺模拟动作仍冻结／恢复 |
| cue JSON 越界／媒体加载失败 | 拒绝配置或跳过声音，不阻塞游戏启动、不回放积压 |

## Good / Base / Bad Cases

Good：真实第 2 段在高速翻腕边界两侧与原生参考一致；换三角色仍保留各自骨长和实际皮肤掌框。
Base：三次有效左键产生三段和三次起手声，每段各一次 marker 命中。
Bad：用均匀低频采样插出不存在的 90° 转腕；把仅上身片段的骨盆覆盖到全身；播放动作就触发防御成功声。

## Tests Required

- 离线源解析／坏输入／mask／native 切点测试、两条 portable verify；独立偏移帧原生局部姿态、全骨 FK、真实手部 LBS 验证。
- Rust 格式 v1/v2、相邻 f32 键与真实中点、双来源、3 段时钟、防御尾势抢占、跳高、三角色独立指链／掌轴／实际权重点和远程下肢保护。
- Rust 音频起手身份／过期／取消、真实运动事件、不可变枪弓 pair；Node 实际脚本的可信解锁、配置边界、声部优先级／层预留／暂停／异步竞争。
- make check、WASM Clippy、临时 Trunk；真实 GPU 查看页三角色／各段／握点和游戏真实点击／右键／跳跃／枪弓，录入独立报告。CPU 数值通过不等于画面验收。

## Wrong vs Correct

错误：`delta.max(f32::EPSILON)` 平滑掉相邻键；把缺轨当零；直接将不同 rig 的 source.model 混合。
正确：精确键直接取值、正 dt 用实际差；缺层按明确来源组合；通过实际目标 binding／局部指旋转 FK 后统一求皮肤与握柄。
