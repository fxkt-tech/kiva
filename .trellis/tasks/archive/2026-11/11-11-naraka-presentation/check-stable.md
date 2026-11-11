# 稳定切片独立审核

本轮为 trellis-check 独立检查。范围是 simulation 三段／防御／真实运动反馈，client controls/audio，Web 音频桥及离线音效工具／48 个媒体；动作采样、重定向、查看页与离线动画仍有同伴修改，尚未作为最终版本审核。没有提交／推送或启动服务。

## Findings (fixed)

### 1. 自动连击吞掉本步刚恢复冷却的闪避

- 文件：`src/simulation/actions.rs:67`、`combat.rs:929`、`melee_chain_tests.rs:277`。
- 原因：后继普攻的优先级预测检查 `dodge_cooldown == 0`；同一步的 motor 会先减 dt，再接受闪避。尚余小于一个固定步的冷却因此被预测为不合法，自动普攻先承诺并清掉 Shift 请求。
- 证据：实际 Shift 长闪→链斧第1段＋缓冲→自然退休，使用半步时钟相位让冷却尚余约0.00833秒；原实现接受第2段，`active_action.is_none()` 断言失败。
- 修复：私有预测方法接收该步 dt，使用 `cooldown <= dt` 与 motor 本步衰减后的准入一致。没有改闪避冷却／段时长／伤害或显式攻击的原优先级。回归同时验证没有第2段起手、存在第二次真实 Dodge 反馈。

### 2. 最终仍 active 的取消样本不能撤销未来音效

- 文件：`src/client/controls.rs:65/318`、`audio.rs:75/94/227`。
- 原因：AudioState 只观察最终 SimulationActive 和当前 action 身份。blur→recapture 合并在同次采样后仍为 active，旧 pending offset cue 可继续发出。Web blur 清节点无法撤销 Rust 尚未到点的 cue。当前目录 offset 全0，主要是承诺的非零偏移合同缺口。
- 主会话明确批准私有客户端接口方案后修复：CaptureState 在真实采到 Cancelled 时统一递增 cancellation_epoch；AudioState 保存上次序号，变化帧按 inactive 消费音频。清 pending、前移 combat/motion 游标并丢该帧所有旧事实，下一帧正常发新声音，不修改已承诺动作。
- 回归：非零offset=.2，最终active=true的取消帧同时有 Hit／Jump，确认待播与该帧事实一起丢弃、恢复不补播、新序号正常发声。另经真实 controls::sample 的有序鼠标队列溢出产生相同 Cancelled，零 FixedUpdate 下验证 epoch 变化、动作ID/elapsed不变、pending清空且普通 active 样本不继续增长 epoch。

## 已核对的实际路径

- MeleeChain 独立于技能 ComboWindow；第1/2段各一条缓冲、第三段不存第四段；最后 marker 经物理后结算再退休。阶段／pair／action_id 复制到快照和命中事实，旧事件不读取当前“下一段”。有效技能／换组／防御／运动、普通受伤与死亡／模式清理不会重启旧后继。
- DefenseSnapshot 自然 Idle 保留最后短手势及年龄；取消作废，重置保留单调ID。实际判定窗口不随表现延长。
- Jump/Dodge 由 motor 真接受发布；Land/Footstep 在真实物理 publish 后形成独立128条流；支撑相对步距、不动／顶墙／空中／滑铲／闪避／传送清基线、死亡和暂停门控均检查。
- 音频区分 actor 防守者与 source_owner 发起者；MeleeStart 播三段挥击，Melee marker不重复；Shot 按不可变pair选 naraka_gun/bow，SniperShot不重播。死亡只用drop，通用男声不自动绑定角色。
- JS 有界16 cue／64媒体白名单、每context唯一decode、未解锁／未就绪丢弃；每层独立权重与gain，保持自然音高。落地两个实际source分别delay .1/0，未来节点计入四声部且暂停stop，整组先预留再淘汰；BGM独立单循环、旧context回调隔离。
- 音效 importer 普通verify仅用冻结原WEM／PCM／HIRC身份及ffmpeg重建，不调用客户端或vgmstream；prepare/redecode分开。原proto time未知与bank真实100ms delay区分；FK/bow为原家族共享事件，没有皮肤或角色专属声明。

## Verification

- `cargo test --locked simulation::combat::tests::melee_chain_tests -- --quiet`：16 passed。
- `cargo test --locked client::audio::tests -- --quiet`：13 passed。
- `cargo test --locked client::controls::tests -- --quiet`：17 passed。
- `node --test scripts/audio-unit.test.cjs`：25 passed。
- `python3 scripts/test-naraka-audio.py`：10 passed。
- `python3 scripts/import-naraka-audio.py --verify assets/prepared/naraka/audio`：15原事件／16cue／48媒体逐字节重建通过，945182B Ogg，最大分层2，effect上界0.612。
- 本轮修改5个Rust文件的 rustfmt check、稳定文件 diff check：passed。
- native `cargo clippy --locked --lib -- -D warnings`：passed。针对测试的真实Rust编译通过；全量all-target/WASM Clippy与make check待动作同伴冻结，由主会话通知后统一执行。

## Findings (not fixed)／后续

本稳定范围没有剩余已确认产品缺陷。以下尚未完成，不冒称全任务通过：

- 同伴还在修动作／重定向；完整 source format／rig/scope／手掌手指／下身与viewer独立审核、最终全量质量门待冻结后跟进。
- 实际WebAudio／硬件GPU游戏和查看页验收由主会话执行；本报告不把Node fake和CPU测试当浏览器证明。
- canonical specs 由主会话更新：三段缓冲是旧“不排队”条款的有界例外；新增MeleeStage/MeleeStart/DefenseSnapshot/MotionFeedback，双来源动作/映射另待核心验收；音频来源从旧程序枪声改为16原cue/48媒体、落地双层、取消epoch和未来cue丢弃策略需要同步到输入／战斗表现规范。
