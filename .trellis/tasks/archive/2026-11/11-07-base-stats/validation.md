# 实现与验收记录

日期：2026-11-07。基线：`1745b74`。用户在方案总结后批准“开始实现”。
首轮及用户追加实现已完成，工作提交为 `f2176b0`；任务已归档。最后外观调整依用户指令未测试，详见文末。
本文前半部分保留首轮验收记录，最新追加结果见文末。

## 实现结果

- 新增 `attributes.rs` 和 `damage.rs`，保持单 crate、模拟权威状态及 `client → simulation` 依赖。
- 七项基础/最终属性、按来源修正、原子拒绝、FIFO 更新、死亡清理和暂停行为已实现。
- 普攻、弹丸、延迟范围及引导共用类型化伤害；支持攻击力/法强系数与命中时抗性。
- 普攻使用整数 tick 时序快照，移速/距离/攻速使用最终属性。
- 卡片左侧七项属性来自 Rust 快照，保留逐帧血条与 HUD 节流，覆盖普通/小窗口及高 DPI。
- 新增 20 个无窗口行为场景，原有 29 个模拟场景及 8 个输入/光标单元测试保留。
- 没有实现装备、完整状态效果或角色选择，没有新增游戏运行依赖。

## 验收映射

| 验收项 | 证据 | 结果 |
|---|---|---|
| AC1 | default_attributes_are_observed_separately_from_static_definitions；browser-stats | 通过 |
| AC2 | modifiers_replace_by_source_and_recompute_without_residuals | 通过 |
| AC3 | modifier_updates_are_fifo_atomic_and_wait_while_paused、invalid_modifiers_preserve_casts_and_missing_targets_are_rejected、finite_extremes_saturate_and_source_removal_still_restores_values | 通过 |
| AC4/AC5 | physical_magical_true_and_negative_resistance_use_one_damage_contract、spell_power_and_attack_damage_coefficients_scale_at_commit | 通过 |
| AC6 | area/projectile 来源移除与命中抗性场景、channel_reuses_committed_damage_and_reads_defenses_for_each_pulse | 通过 |
| AC7 | effective_movement_speed_changes_immediately_and_zero_preserves_intent、changed_attack_range_is_used_for_release_and_chasing | 通过 |
| AC8 | attack_speed_produces_exact_fixed_step_cadence、changing_attack_speed_during_windup_keeps_the_current_attack_snapshot、既有走砍/重复指令场景 | 通过 |
| AC9 | 零移速、零攻速、冲刺独立速度场景 | 通过 |
| AC10 | death_removes_temporary_modifiers_and_respawn_keeps_persistent_modifiers_once、暂停队列场景 | 通过 |
| AC11 | browser-stats：1280×800、800×600、1000×720 DPR=2；实际截图目视检查 | 通过 |
| AC12 | browser-stats、skills、cursor、healthbars、smoke | 通过 |
| AC13 | make check、WASM Clippy、Trunk 构建、六类浏览器回归与规范回写 | 通过 |

## 命令与结果

| 检查 | 结果 |
|---|---|
| `make check` | 格式、原生所有目标 Clippy、57 项 Rust 测试、WASM 检查通过 |
| `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings` | 通过 |
| `trunk build --dist /tmp/kiva-stats-site` | 最终构建成功，日志时间 2026-11-07T01:33:14Z |
| `node --check`：hud.js、stats/support/smoke/startup 浏览器脚本 | 通过 |
| `browser-stats.cjs` | 无头与有窗口均通过，七项值/布局、UI 隔离、移动、暂停/恢复 |
| `browser-skills.cjs` | 有窗口通过，Q/W/E/R 鼠标释放、伤害、冷却、移动和真实失焦 |
| `browser-cursor.cjs` | 有窗口通过，边界、越界恢复、居中、Esc、DPR=2、缩放、拒绝捕获 |
| `browser-healthbars.cjs` | 无头通过，人物/镜头移动更新比率均 1.0，HUD 节流保持 |
| `browser-smoke.cjs` | 有窗口通过，实际 WebGPU 画面、完整操作、尺寸变化、DPR=2、失焦与恢复 |
| `browser-startup.cjs` | 有窗口通过，WASM 下载失败提示和重试 |
| Trellis 两份上下文清单 | 各 9 条规范/调查引用，通过路径校验 |
| `git diff --check`、文档/JSON/JSONL 校验 | 通过 |

浏览器测试使用已安装的 Playwright 和 SwiftShader，`KIVA_SITE_DIR=/tmp/kiva-stats-site`。
每个脚本自建临时 HTTP fixture 并在 finally 中关闭，未启停用户的开发服务。

血条的有窗口软件渲染尝试只在 10 秒内采到 16 帧，更新比率仍为 1.0，但未达到原测试的 20 帧样本要求。
保持该断言不变，改在既有无头采样环境执行：人物 90/90 帧更新、镜头 48/48 帧更新，均通过。
实际画面由有窗口属性/综合测试另行验收，没有用降低更新比例或样本量的方式通过检查。

## 复核中解决的问题

1. f32 秒长导致最大攻速周期误取整为 3 tick；改用 f64 `TICK_RATE` 推导，公开行为测试验证 2 tick。
2. 极小正攻速可能先转 f32 下溢为 0；先在 f64 中应用技术下限，极值来源移除测试覆盖。
3. 有窗口/无头 Chromium 捕获后合成鼠标锚点不同；测试辅助函数统一锚点并恢复中心，
   已验证两种模式 × DPR1/2 × 首次/恢复共 8 个真实技能点击场景。
4. 校准会主动居中，因此增加校准前的生产光标居中断言，避免测试自行修正后掩盖缺陷。

独立 `trellis-check` 复核全部数据流、49 个模拟场景及最终辅助函数，无剩余确认缺陷。

## 人工查看的产物

- `/tmp/kiva-browser-tests/artifacts/stats-1280x800-dpr1.png`
- `/tmp/kiva-browser-tests/artifacts/stats-800x600-dpr1.png`
- `/tmp/kiva-browser-tests/artifacts/stats-1000x720-dpr2.png`
- `/tmp/kiva-browser-tests/artifacts/proving-grounds.png`

## 后续步骤

本期功能已完成，`docs/TODO.md` 仅勾选第一项。
用户已授权提交与推送；后续可按既定顺序规划技能/状态、角色装配和装备系统。


## 追加验收：普攻后摇与紧凑属性区（AC14/AC15）

用户直接要求新增普攻后摇、收窄属性区，并指定七项短标签及顺序。

实现结果：

- 成功出手后进入 `AttackBackswing { remaining, duration }`，使用本次时序快照。
- 移动、停止、换目标和施法可以取消后摇，已发射攻击及独立攻击间隔保持。
- 相同目标重复指令、无效请求不重启后摇；自然完成不增加攻击周期，暂停冻结、死亡清理、攻速变化不改本次快照。
- HUD 区分“攻击前摇/攻击后摇”，模型子层显示收招，角色根位置、底座、拾取及血条锚点不受姿态变换影响。
- 属性区由248px收窄至184px（约25.8%），顺序及文案为攻击、法强、护甲、魔抗、移速、攻速、攻距。

验证结果：

| 检查 | 结果 |
|---|---|
| make check | 65项Rust测试通过（9单元+56模拟），格式、所有原生目标Clippy及WASM检查通过 |
| WASM Clippy | `-D warnings`通过 |
| Trunk最终构建 | 2026-11-07T02:02:01Z成功，`/tmp/kiva-stats-site` |
| 新增规则测试 | 7项后摇场景：自然结束/间隔、取消、重复和非法请求、暂停、死亡、攻速变化、失败出手 |
| 新增显示测试 | 真实1-tick后摇与0.9 tick插值，暂停保持Transform、恢复更新、Stop归零 |
| browser-stats（有窗口） | 三种视口、DPR2、精确短标签顺序/字段配对、宽度≤185px、UI隔离、暂停恢复通过 |
| browser-healthbars（无头） | 人物91/91帧、镜头48/48帧更新，比率均1.0，HUD节流保持 |
| browser-skills（有窗口） | Q/W/E/R鼠标释放、伤害、冷却、移动与真实失焦通过 |
| browser-smoke（有窗口） | 实际画面、后摇HUD显示与Stop取消、攻击移动、镜头、窗口变化、DPR2、暂停/焦点通过 |
| 静态布局 | 客户端检查1440至360共12种宽度；主会话查看1280×800和800×600实际截图，无遮挡 |
| 独立复核 | 暂停姿态问题修正后，无剩余确认缺陷 |

复核修正：暂停原本将插值比例置0，短后摇可能回到出手姿态；现在暂停时保留已显示模型Transform，
通过实际模拟状态的无窗口呈现测试回归。Clippy发现测试模块顺序问题后已移至文件末尾，无警告抑制。

最新截图沿用本记录上方的stats及proving-grounds文件，内容已由追加验收覆盖。
本轮按改动范围执行四类浏览器回归；启动重试和完整光标捕获套件的首轮通过记录保持在前文。


## 追加外观调整：去除单位并进一步收窄

用户要求属性不显示单位，并明确“直接改，不用测试”。
已移除移速/攻速/攻距旁的单位，面板184px→144px，小窗口最大宽度同步调整。
同步更新既有浏览器断言与HUD规范；本次未运行测试、构建或浏览器检查。前述65项测试结果属于此前版本。
