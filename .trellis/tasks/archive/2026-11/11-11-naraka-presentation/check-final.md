# 最终独立审核

本轮继续同一 trellis-check 审核，不创建其它代理。已读取父／四子任务的 PRD、design、implement、check manifests 与交付证据，以及新规范 `client/naraka-motion.md`。规则／输入／音频的前轮细节与修复证据见 `check-stable.md`；本文件扩展到离线动画、双来源重定向、共享查看页与最终质量门。

## Findings (fixed)

1. **本步冷却到期的闪避被自动后继吞掉**：`simulation/actions.rs` 的私有优先级预测由本步前 `cooldown==0` 改为包含 dt 的准入，与 motor 同步。实际长闪→链斧攻击＋缓冲在最后半步冷却复现失败后修复，连击16项通过。
2. **最终active的取消样本遗留未来音效**：经主会话批准，`controls::CaptureState` 保留取消epoch，`audio::AudioState` 在变化帧清pending、丢该帧事实并前移两游标。真实sample＋零固定步＋非零offset及恢复不重播回归通过；不修改已承诺动作。

## Findings (fixed by motion owner, independently rechecked)

- **弓射击漏声明局部身体范围**：对全部44段的原 authored mask 与6534个native输出时刻做逐项量化，`male_bow_attack_01` 是39维身体／右指20、骨盆Y −0.120..−0.106m、原生脚Y −0.803..−0.774m。原scope只覆盖FK/sniper，导致raw弓射击对照仍将局部层当完整身体。游戏妹妹已有lower-body组合，但raw路径需要同一个scope修复。原始量化见 `review-source-scopes.json`，其余FullBody未发现同类低骨盆漏项。
- **弹反尾势可能在打断后重新出现**：只按当前移动／受击／动作抢占时，短走后停下或短受击结束，仍保留的同一个DefenseSnapshot.last_gesture可以重新选中尚未到0.60s的旧尾势。需要按gesture_id永久锁存该次打断，新gesture可正常播放。

以上两项由仍拥有该区域的motion实现者修复，已独立复查最终代码：弓射击加入共享UpperBody范围，真实三角色raw路径的骨盆／腿／下身LBS回归扩展到该片；RigPose按gesture_id锁存尾势打断，短移动／受击结束不恢复同ID，新ID可正常播放，viewer直接seek／顺播／倒拖重建同一锁存状态。

另复查出长显示帧同帧观察并结束Hit（0.24s）／HeavyHit（0.30s）时，仅看advance后的状态会漏掉打断。实现者已在真实新受击事件观察后、advance退休前保存这一事实，新增dt=.32s、gesture_elapsed=.4s两种命中回归；没有延长判定或改模拟状态。

## 已检查的证据与数据流

- 原资源serialized2019.4.41f2，实际官方独立求值器2019.4.41f1；差异在环境／runtime／准备包清单披露。NativeSampler加载原Avatar和Clip，通过原生AnimationClipPlayable输出locals/world；没有把肌肉数组当Euler，没有用有损HumanPose Get→Set回写。FootIK/playable IK和root-motion的明确设置，以及未恢复原controller权重，均有边界说明。
- 原scalar streamed三次多项式、dense/constant宽度／端点、95/75/55与39维遮罩以及整指链缺失与显式零分开。默认仅补完整缺失四DoF指链的三节原生locals，原作者指链优先；部分缺DoF硬失败，循环权重1明确为Kiva组合。
- native自适应采样检查区间1/4、1/2、3/4，保留65个邻接f32切点；Rust与portable采样都精确命中键直接取值，正dt不钳EPSILON。20项Python测试包含真实均匀60Hz包必须被独立采样门槛拒绝的负例，不是只验证端点或非零动画。
- H0的独立137Hz＋0.00123s参考未参与选键；portable验证从真正runtime重新采样8670时刻、全63骨FK与每帧1116个真实权重手部点，对照原生world，另有Native BakeMesh与独立LBS交叉校验。报告实测最大角0.078316°、全骨位置0.283705mm、手部LBS 0.278271mm；不是连续函数的误差数学上界。
- KIVASR02显式独立rig／父序／rest／f32时间，v1兼容，数量／字节范围／finite／quat／父序／重复／尾字节有界拒绝。两来源各自索引／inverse rest／指链／掌框；跨rig通过目标姿态过渡，裸数组不zip。目标自身rest/IBM不改，原生全palette只用于同一Chou＋Yanyun来源。
- 原Naraka末指−X有真实子骨偏移和distal权重点证据；目标gMan自身＋X和Chou真实Nub分开。三节局部quat混合后FK保持自身指长，腕／MCP掌框统一驱动皮肤和正向握柄，+0.055m握柄适配保留。共享live／viewer经过同一个RigPose、avatar和挂点求值，没有viewer额外修正。
- 游戏只读权威stage/elapsed、短防御快照与真实运动；空中剔除Pelvis额外Y避免二次跳高；FK/狙击候选／指层及妹妹上身组合保CC0下身，真实双手武器IK随后求解。玩家F3两外观仍用Naraka，Boss保留Yanyun。
- 起手、真实命中／防御／实际运动与妹妹Shot按immutable身份路由；audio_start/marker不重复，枪弓family与可选通用男声没有角色／皮肤专属声明。音频完整离线重建、四source预算／优先级／两层延迟、解锁／cache／取消／无恢复补播已在稳定审核确认。

## 已明确的产品限制

- 首版移动和静止起跳均采用已证inplace01/02＋权威物理移动；forward01/02仍为未落地候选，主会话已确认不新增这两项。
- E／Tab／普通滑铲保持已批准的原编排／明确回退；未恢复永劫完整控制器、状态机、原层权重、Wwise RTPC／总线／材质switch名称／原proto时间单位。
- 原f2／求值器f1补丁差异不能称逐位等效；标准离线verify与需要外部原生环境的rebake是两个流程。
- 浏览器真输入、原AudioContext、硬件GPU与截图由主会话验收；本审核不以CPU／Node fake代替画面或主观听感。

## Verification

稳定阶段亲自执行并通过：native lib Clippy、连击16、audio13、controls17、Node音频25、Python音效10、48媒体完整portable verify、owned rustfmt／diff check。详见 `check-stable.md`。

冻结动画的20项测试、完整44段portable verify／repackage通过记录复用动画交付；独立阅读核查其输入身份、真正偏移参考、FK/LBS和负例。未因无代码变化重复运行原生烘焙。

最终runtime冻结后亲自执行一次完整质量门，全部通过：

- `make check`：fmt、native all-target Clippy、297项Rust单元测试、9项真实地形测试、105项Node测试、WASM check均通过。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`：通过。
- `git diff --check`：通过。

完整日志与机器可读结果保存在 `qa/make-check.log`、`qa/wasm-clippy.log`、`qa/quality-gate.json`。没有为通过检查新增抑制项；本轮两项motion发现、漏帧受击和共享viewer回归均纳入最终297项测试。

## Findings (not fixed)

冻结代码范围内没有剩余已确认缺陷。上述版本／源控制器／原音轨语义与first-version映射限制是已批准并披露的边界，没有当作已恢复能力。

主会话正在同一冻结树执行增量Trunk及Windows NVIDIA真实浏览器验收；最终GPU画面、实际输入、48媒体AudioContext与主观听感证据由主会话单独落盘。本审核的完整代码质量门已经通过，不声称替代该浏览器／听感验收。
