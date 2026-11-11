# 音频／输入切片交付

## 范围与接口

只改 `src/client/audio.rs`、`src/client/controls.rs`、`web/audio.js`、`scripts/audio-unit.test.cjs`。冻结的音频配置／42 个 Ogg、动作 worker 的客户端文件、Web 输入桥均未改。无提交／推送，无服务启动。

`window.kivaAudio.playEffect(name)` 签名保留，新增只读 `cueOffset(name): number`，Rust 绑定仍位于 audio.rs，不需改 web.rs。原程序枪声、三种旧 effect、单条 BGM、妹妹 Shot/source_owner 归属和 SniperShot 去重保留；查看页仍提前退出整个音频桥。

## 行为

- 普通挥击从不可变 `MeleeStart(actor, source_owner, action_id, pair, stage, sequence)` 选择 attack_1／2／3；当前同动作快照验证身份和 elapsed。旧普通攻击伤害 marker 不重复 whoosh，敌人的未分段 Melee 保留旧声。
- 声音偏移读取经过验证的 cue 元数据，目前全部为 0。将来的非零偏移只按同一 `ActionSnapshot.elapsed` 等待，容量全局 8，单动作至多 2；只有当前一个 cue。重复事件／动作只消费一次，结束／切动作／切组／死亡／取消／失锁丢弃未播部分，超过时点 0.12 秒丢弃，不用墙钟补播。
- Deflect／Block／Parry 按真正防御者 `event.actor` 选择原金属声，不能用攻击方 source_owner 或武器组误判。空右键没有伪成功声。选择为：真实结算 Parry 同时尝试成功金属声和 riposte whoosh；不在空弹反手势发声。
- 接受的跳跃／闪避、真实着陆及物理距离脚步只消费独立 `MotionFeedback` 游标；只为受控活角色发声。免疫 Dodge 不是物理闪避开始。倒地只播放原落物声；通用男声作为可用 cue 保留，不自动绑定女性／角色死亡。
- JS 只读有界 version=1 元数据，14 cue 名与纯数字源媒体文件白名单，校验有限 priority／offset／delay／weight／gain、数量和 64KiB 上限。每个 context 媒体 fetch／decode 缓存；失败／未就绪丢弃，不积压。
- 每层独立权重选择，播放 gain=.18×variant.gain，只应用一次，不变速。落地保留两个独立 PlayAction（delay .1／0），用 AudioBufferSource.start 排程；预定节点立即计入 4 个 effect 声部并在暂停清理。
- 整个分层 cue 先预留足够声部，优先淘汰较低等级，再淘汰同级最旧声部；脚步不挤占命中／弹反。BGM 独立不计入 effect 四声部。
- controls 原失活 clear 已锁存取消；增加最终仍 active 的 `DefenseEdge::Cancelled` 边沿锁存，覆盖同帧失焦／恢复与队列取消。普通 active 帧不重复清理后继。

## 验证

- `cargo test --locked client::audio::tests -- --quiet`：10 passed。包含真实 FixedUpdate 三段启动／旧伤害 marker 去重、两次接受跳跃／第三次拒绝和一次真实落地；身份／时点／未来偏移／固定钟冻结／取消／死亡／暂停／防御归属／运动独立游标／妹妹三发与 immutable pair。
- `cargo test --locked client::controls::tests -- --quiet`：17 passed。新增真实 sample→FixedUpdate 三路：正常零固定步保留后继、失焦暂停多固定步／重新捕获丢弃后继、最终 active 的取消边沿丢弃后继。
- `node --test scripts/audio-unit.test.cjs`：23 passed。包含 42 媒体唯一加载／14 cue、独立加权与 gain、双层真实 start 参数、优先级／整组预留、第二层启动失败清理、坏元数据／缺文件、关闭 context 的旧加载竞争、未就绪丢弃和 viewer 禁音。
- `node --test scripts/*-unit.test.cjs`：103 passed。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`：passed。
- `cargo clippy --locked --all-targets -- -D warnings`：最终结果见 audio-input-validation.json；首次仅发现另一 worker 的 source_animation 测试初始化 lint，已通知其修复。
- 本切片 `git diff --check` passed；owned Rust 文件已 cargo fmt。全局 fmt 在动作 worker 格式化后复跑通过。

## 合并验收

主会话执行最终 make check、全局 fmt、真实浏览器音频与交互验收和规范更新。真实 Chromium 对 42 个 Ogg 的 decode／实际 AudioContext、三连和右键／跳跃场景仍由主会话记录；Node fake 生命周期和 CPU 固定步测试不替代这些结果。原源音频实测峰值／哈希等沿用音效资产任务的冻结报告。

## 远程声音定向补齐（最终消费者更新）

资产任务后续已交付 16 cue／48 媒体；本切片只消费新增 `naraka_gun`／`naraka_bow`，未改资产或离线工具。真实 Companion `Shot` 根据不可变 event.pair 选原鸟铳族／弓族声音，伴随 SniperShot 继续忽略；缺失 pair 不猜声音，未解码不回退或排队。公共 JS `playSniperShot()` 与缓存程序枪声仍保留给兼容调用，实际新远程事件使用原 Naraka 声音。移除 Rust 中不再被真实事件使用的程序枪声绑定／旧弓回退变体，普通旧攻击、BGM、四声部和落地分层行为未改。

来源是原 male_fk_attack_01→gman_fk_attack_01 与 male_bow_attack_01→gman_bow_attack_01。male/female FK 原组 attack_501 指向同一 male_fk 片段／字段名称；采用的是原武器家族共享声音，不能称为引冰落、羿之影、李火旺或白灵淼专属声音。每族三个变体，单层 delay=0、priority=2、offset=0，沿用原 variant gain。

定向验证：

- `cargo test --locked client::audio::tests::shot -- --quiet`：3 passed；真实 Q 在两组分别只出一个妹妹 Shot，之后实际切组再消费仍用事件原组；锤狙三发各一次，伴随 SniperShot 不重播，缺 pair／非玩家归属跳过。
- `cargo test --locked client::audio::tests::sounds_follow_actual_feedback_once_for_both_sides_of_combat -- --quiet`：1 passed。
- `node --test --test-name-pattern='bundled effects load|unverified music|all sixteen|metadata arriving|original ranged' scripts/audio-unit.test.cjs`：6 passed；覆盖目录 16／48、六个新变体的独立媒体／gain／零延迟、未解码无程序回退或补播、旧 gun 公共入口，以及原 BGM／关闭 context 竞争回归。
- owned Rust rustfmt check 与本切片 diff check passed。按主会话要求不重复全量 Rust／Node；最终 make check 统一由主会话执行。

浏览器仅需另验 6 个新增 Ogg（原 42 媒体未变）和真实枪／弓 Shot，不能把先前 42 件浏览器验收冒称全部 48 件已通过。
