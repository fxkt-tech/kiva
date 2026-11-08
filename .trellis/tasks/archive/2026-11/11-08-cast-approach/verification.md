# 验证记录

## 已复现并修复的缺口
- `cargo test --locked --test cast_approach out_of_range_enemy_cast_walks_then_commits_once_without_early_cost`先失败于未移动，再通过；/tmp/kiva-cast-approach-{red,green}.log。真实公开命令路径，模拟运行约0.05秒。
- `cargo test --locked --lib client::controls::tests::distant_enemy_click`先2项失败于有效超距点击仍保留aim；/tmp/kiva-cast-client-red.log。
- 修复后`cargo test --locked --lib client::controls` 24项全过，包含英雄槽与装备实例两条实际点击路径；/tmp/kiva-cast-client-green.log。

## 范围与规则回归
用户最后确认通用目标语义：只处理Enemy，Direction只需要方向，Point保留原最大距离/严格射程。根因是客户端提前拦截OutOfRange及模拟缺少尚未开始的走近施法目的。没有按技能ID分支，不改变冷却、费用或原提交校验。

新增cast_approach19项公开回归覆盖：不提前扣费/CD与单次提交、移动目标、绕障和中心不可达但施法区域可达、无可达区域的原子拒绝、新移动/攻击/Stop/技能替换与非法输入保留、装备来源及卸装/旧实例、丢视野、旧生命/施法者死亡、暂停/reset、前摇目标逃离后不重试、Root暂停但可施法、Stun取消、换目标、范围内施法继续原移动，以及Direction/Point Dash不变。

首轮定向135项通过（17新增、simulation59/equipment13/status37/placement4/terrain5），/tmp/kiva-cast-approach-native.log。

后续两个公开场景额外先RED再GREEN：
- 零前摇入范围后多等一tick，目标再移动便提交失败；/tmp/kiva-cast-zero-windup-{red,green}.log。待放推进移到动作时钟之前，与直接命令同tick开始/提交。
- 较快施法者仍无法追上持续逃跑敌人，因路径每次停在旧射程边缘；/tmp/kiva-cast-fleeing-red.log。保留安全区域终点，再追加朝目标中心的扫掠段，每tick继续复核实际射程。
最终19项全过，/tmp/kiva-cast-approach-timing-final.log；正前摇失效不重试、中心不可达/区域可达与NoPath原子拒绝继续保留。

## 完整质量门
- `CARGO_BUILD_JOBS=2 make check`退出0，fmt/native全target Clippy/301项Rust测试/WASM目标检查通过。2项原长跑默认ignored；本次不改兵波/AI策略，未重复长跑。/tmp/kiva-cast-approach-check-v2-bounded.log。最初v2检查因并发链接内存占用较高，只中断自有检查进程后按两路重启；没有跳过测试，旧中断日志不计通过。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`退出0，/tmp/kiva-cast-approach-wasm-clippy-v2.log（最终v2退出0）。
- 曾额外尝试WASM `--all-targets`，因现有native专用cfg(test)夹具start_native被条件编译排除而失败，不算通过；没有为此更改产品或扩大测试平台范围。native全target由make check覆盖。
- 独立Trellis全范围检查已完成，无阻塞项/无产品代码修改，详见[检查记录](research/review.md)。

## 临时Web构建身份
- `/tmp/kiva-cast-approach-final-site`，最终v2构建退出0，日志/tmp/kiva-cast-approach-build-v2.log。
- WASM：kiva-8dc3a9c9d76141ab_bg.wasm，SHA256 `e60cd07d7b3492fc4dd55428f60006cfd601896a565e9423e0f402935acb1ff7`。
- 88文件源码清单/tmp/kiva-cast-approach-source-v2-final.json，SHA256 `2106653aa8fbf9122af4a030807cf046bd41db85ce3be9ba8a20b738cc3ccd30`；构建前后、格式化和独立检查后完全相同。

## 浏览器（已完成）
- 最终v2有窗口1280皇子阶段所有断言已PASS：实际距离10.8208的R单次敌人点击，pending法力300/CD0，追近后扣100、真实跃击/围墙/免费重施；原方向Q、E最大距离、EQ途中W、枪盾旗也通过。日志/tmp/kiva-cast-browser-kit-final.log，截图/tmp/kiva-browser-tests/artifacts/cast-approach-final/jarvan-terrain-1280x800.png。
- 该次完整套件随后在重新选角阶段因测试诊断监听访问空abilities报TypeError，整体退出1，不能当完整套件通过。监听已改为首次pending后立即注销，避免跨选角残留；产品/WASM无修改。新增显式KIVA_CHAMPION_HERO=1/2定向入口，默认双英雄、未知参数报错，核心断言未删除；只重跑实际失败/后续的寒冰阶段，日志/tmp/kiva-cast-browser-ashe-final.log（退出0，全部寒冰断言通过）。
- 第一轮R实战超时，/tmp/kiva-cast-browser-kit-trace.log确认tick474待放无扣费/CD，tick584受到真实Stun后转Idle，随后未重放；没有DeviceLost/Quitting。第二轮等待Stun的准备流程因真实大箭可以射偏而超时（我方640HP、对手61法力，/tmp/kiva-cast-browser-kit-v2.log），没有技能提交或重试。最终先通过正常P界面穿纯防御装备，在当前可见对手法力<=75、我方控制许可且超距时只点击一次R；对手无法支付100法力大箭，保留超距/pending/费用/CD/移动/围墙/重施全部断言。脚本增加ERROR/Quitting日志和退出硬失败，独立检查已复核，没有隐式重试或World/AI/HP改写。
- 串行最终smoke退出0：真实WebGPU像素、点地移动/替换/Stop、HUD隔离/取消瞄准/Y/Space/边缘平移/缩放与resize、暂停/原生失焦/同实例恢复、DPR2投影与世界拾取全部通过。日志/tmp/kiva-cast-browser-smoke-final.log，截图/tmp/kiva-browser-tests/artifacts/cast-approach-final/single-lane-smoke.png。

用户管理的开发服务不操作；测试独占临时构建/HTTP/Xvfb并自行清理，实现阶段未提交，用户随后已明确授权提交与推送。

## 最终收口

- 最终检查：git diff --check、27个本地Markdown链接、JSON/JSONL和任务上下文均通过。最终88文件源码再次与v2构建清单完全一致。
- 浏览器按阶段保留证据：皇子全部PASS；修复测试诊断监听后定向寒冰退出0；最终smoke退出0。没有把中间整套退出1或准备超时记成整套通过。
- 实现、独立全范围及增量检查已完成，全部验收达成；用户已明确授权提交与推送，本次工作提交完成后按Trellis归档。

收口时发现额外的docs/TODO.md和docs/hud_lol_game.png变更，均非本任务编辑，已原样保留，未纳入本任务验证或提交范围。
