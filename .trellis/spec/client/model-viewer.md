# 模型、武器与动作验收页

## Scope / Trigger

修改 `client/viewer/`、`animation/preview.rs`、共享角色挂点、`web/{mode,viewer}.js` 或验收页入口时阅读。
本页用于检查真实游戏模型、武器位置与单体动作；未来方案仍写任务目录，不向此规范添加未实现功能。

## Signatures

- `web::viewer_mode() -> bool` 在App创建前读取 `window.kivaPage.isViewer()`；仅精确 `view=model` 启用，原生仍运行游戏。
- `viewer::configure(&mut App)` 组合独立场景，前置为DefaultPlugins与StartupPlugin；不安装SimulationPlugin、地图、战斗、游戏输入、跟随镜头、HUD或音频。
- `animation::preview_catalog(&AnimationLibrary, &SourceAnimationLibrary) -> Vec<PreviewClip>` 提供游戏单体场景、19 CC0、35源片段和资产参考姿态；片段时长读取实际库。
- `PreviewRig::seek(library, source, id, pair, blade, seconds, looping) -> bool` 重建游戏单体状态或按绝对时间采样片段；`pose()` 返回实际RigPose。
- `RigPose::trace()` 返回实际采样分支记录的主要片段／时间、源权重、过渡与IK信息；不由JS推导裁剪公式。
- `companion::local_frame(root, source, aim, grips) -> CompanionFrame`、`RigPose::sample_companion_hold(...)` 与真实妹妹显示共用局部武器frame和持握计算。
- `AvatarAssets::{evaluated_drivers,attachment_pose,body_mount,structural_bones}` 共用当前最终姿态；`stickman::attachment_transform` 共用近战挂接。
- `loading::{require_model(world, revision, meshes), model_ready(world, revision), model_failed(world, revision)}` 关联当前所选主体的实际渲染确认。
- Web桥：`window.kivaViewer.{takeCommands():string,render(json):void,ready():void,fail(reason):void}`；全局startup ready只发生一次，选择状态独立版本化。

## Contracts

同一WASM在 `?view=model` 下运行独立App；游戏／查看页使用普通相对链接整页切换，保留部署子路径。`mode.js` 在全部游戏脚本前同步安装不可变模式；游戏专属脚本必须在注册监听器和桥之前退出。查看页不安装假的kivaInput，不申请pointer lock。

默认李火旺＋伏浪锚，暂停待机。支持三个角色、四件独立武器、愁铃客原刀及无手持。实际配置包含哥哥两近战、妹妹两远程、愁铃客敌人原刀；其他组合明确为诊断，不能解释成游戏装备规则。

游戏单体检查复用游戏 `RigPose::frame`，保留动作通知与事件去重→observe→advance→advance_source顺序。固定60Hz检查场景以完整RigPose历史重放，向前增量计算、倒拖／更换场景参数从头重建；这是受控单体输入，不是任意真实游戏帧录制。游戏自身显示dt不改成60Hz。妹妹仅检查持握／瞄准，不能复制射击标记或冒称完整技能。

源／CC0对照使用实际绝对采样，不叠加游戏持握；源轨保留现有根运动／重定向规则，不是未加工原件。源模式隐藏手持，返回游戏模式保留已选装备。参考姿态读取角色自身导入rest，不能以inverse(IBM)替代AV03 sourceRestModel。

源层和CC0层最终显示、左右手／脊柱标记与实际皮肤共用求值；既有右手挂点与近战+0.055m偏移保持。远程使用实际枪口0/-Z及武器握点求双手，不把近战手腕根直接当枪口。禁止只在查看页增加纠偏、偏移或IK来掩盖游戏错误。

Rust独占播放时钟、配置、动作、相机和ready状态。换动作回零；换角色／武器保留时刻、暂停状态和观察方向。seek／步进暂停，游戏时间量化到1/60秒；循环末帧仍可显式检查，不把clip末帧意外包回零。倍速只改变累计的预览帧数量，每个游戏重放步仍为1/60秒。

命令为换行分批、制表符分字段；Web限制128条／16K字符，Rust再检查128条／16KiB、字段数量、有限数值与目录ID。支持preset/appearance/weapon/mode/action/play/seek/step/speed/loop/debug/part/orbit/pan/zoom/view/fit/reset。速度仅0.25/0.5/1/1.5/2；step仅±1；debug名为grid/axes/bones/grips/bounds。JS不计算姿态或本地播放进度。

快照version=1，包含revision、ready、配置、模式／动作、time/duration/frame、playing/speed/looping、trace/warning/status、camera、debug、parts、inspection；首次附catalog。字符串正确JSON转义，DOM只用textContent/createElement。旧revision不覆盖新状态，旧部件ID不作用于新选择。复制仅输出检查文本，不上传或编辑模型。

相机只消费画布上的环绕／平移／缩放；使用pointer capture，up/cancel/lostcapture/blur清理拖动，表单不接管镜头。暂停时镜头仍运行。全身取景在骨骼传播和动态skinned bounds更新后，联合身体、帽子、手持与背部附件，排除地板；首次或显式fit才取景，不能每帧抢镜头。

部件开关仅影响当前实例。显示子部件使用 `Visibility::Inherited`，隐藏使用Hidden；Visible会绕过Hidden祖先，不能用来恢复源模式中的武器子Mesh。选择版本变化时恢复真实可见状态并重建部件ID，防止隐藏主体使新选择永久等待render-ready。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 地板已渲染／旧模型迟到 | 所选主体没有可见、已特化Mesh或revision不符时不确认ready |
| 选择已ready后GPU／管线失败 | 显示当前选择失败和重载入口，保留全局ready一次性契约及原RenderErrorPolicy |
| 非有限时间／速度／相机参数、未知ID／字段 | 忽略命令，保持有效配置；Rust报告无效操作 |
| 快速换角色／武器／部件 | 最后有效选择生效；过期部件命令拒绝，资产缓存复用 |
| 暂停后仅转镜头 | 不重算动画、皮肤或联合bounds；相机和辅助显示仍更新 |
| 剪贴板不可用／拒绝 | 显示失败提示，不上传或静默保存到其他地方 |

## Good / Base / Bad Cases

Good：暂停攻击中段，换三角色并观察同一帧，打开真实骨骼／握点对比，复制配置复现问题。
Base：默认实际配置→选动作→播放／暂停→拖时间→自由镜头检查。
Bad：用独立查看器修正错误偏移后宣称游戏正确；用54片段冒称54个完整技能；只看地板ready或CPU测试就宣称模型画面通过。

## Tests Required

- `make check`、WASM Clippy；入口变更再执行固定临时Trunk构建，检查根路径／静态子路径。
- 真实game animate适配与PreviewRig比较动作、恢复、蹲伏、防御序列；三角色实际palette、源权重与近战武器变换一致，妹妹真实frame／双手握点回归。
- source权重0/0.5/1的左右手／脊柱语义与皮肤一致，右手既有行为保留；参考姿态不假定IBM抵消。
- 正放／seek／倒拖／切换／循环末帧结果一致，全部54片段有限；动态fit包含当前皮肤及附件，暂停相机帧复用计算，重复换装不增长实体／资产。
- 实际传播的子Mesh显隐、隐藏身体后重选、旧部件ID拒绝；加载选中主体、旧revision、后续错误可恢复。
- Node直接执行真实脚本，验证模式隔离、早期快照、队列边界、时间控件／画布／焦点／取消、复制失败和错误恢复；浏览器验证实际WASM操作及画面。软件GPU流程不替代真实GPU模型验收。

若无头Chromium截图空白但提交与资源已就绪，先用隔离有界面浏览器或实际输出surface读回定位。当前环境曾出现GPU输出有完整模型、无头截图却只含CSS背景的合成差异；不能据此修改游戏挂点／渲染，也不能只凭ready宣称画面通过。

## Wrong vs Correct

错误：Web维护动画时钟或复制clip裁剪；直接改RigPose私有clock来seek；给武器子Mesh设Visible绕过模式隐藏。
正确：Web发送有界命令，Rust重放共享显示计算；同一求值生成皮肤、挂点和来源信息；子Mesh显隐继承实际配置根。
