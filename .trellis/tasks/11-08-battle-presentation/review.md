# 全范围独立质量审查

审查者：`check_battle_presentation`，2026-11-08。范围为相对 HEAD 的全部本次修改，包含未跟踪的客户端模块、模型/声音/图标/地形、离线导入器和浏览器脚本。已读取父任务 PRD、design、implement、JSONL，上述三个子任务的方案/研究/验证记录，以及 simulation、gameplay、client 的规范入口和 Quality Check、相关动作/拾取/浏览器规范与总架构。

## 发现并修复

1. **暂停跃击时导入模型的源 Root 高度会突然回弹。** Bevy 0.20.1 的 `animate_targets` 在 ActiveAnimation 暂停时仍逐帧重写曲线；原 `keep_committed_motion_root` 却随 `PresentationFrame.should_sync=false` 跳过。运行帧已去除的第二份源跳跃高度，会在暂停首帧重新出现。现在只冻结显式采样和 MotionView，仍在每次 AnimationSystems 后以同一冻结高度重施局部 Root 校正。新增 `paused_animation_evaluation_keeps_the_committed_leap_root_correction`：使用公开皇子 R 指令产生真实 leap/MotionView，再由实际 AnimationPlugin 重写一条带高度的曲线，检查暂停前后局部/全局 Transform 与模拟 tick 精确不变。测试同时检查曲线的 X/Z 确实被引擎评估，避免后处理单元夹具空跑。

2. **低显示频率下完整普攻之后可能继续显示先前技能。** 若两显示帧之间已经完成高速普攻，最后 UnitAction=Ready，原 sample 中的 AttackWindup/Backswing 清理无法观察到该攻击，`last_cast` 可继续命中 0.45 秒技能记忆。现在独立事件读者在实际 `AttackReleased` 清除旧 cast；不补播已跳过的普攻。新增 `completed_attack_between_display_frames_clears_the_older_cast_pose`，通过真实黄金圣盾提交、30 攻速普攻击杀和完整后摇，在所有攻击显示帧均跳过后检查 Ready/Idle，确保旧技能仍在时间窗内也不会重现。

运行码修改在 `src/client/imported_models/runtime.rs`。将原有两个内联采样测试与上述两项一起移入私有 `runtime/tests.rs`，避免运行文件继续堆积测试夹具。主代理已同意并串行负责 Cargo 验证。

前一轮 `hero-lifecycle/review.md` 已修复旧生命跑步姿态继承和回退尸体四肢朝上问题；本轮复核了其实际落点、读时钟、放松姿态和回归，未重复改动。

## 架构与联动结论

- 模拟只新增私有死亡 tick 和既有复活秒数的只读 getter，死亡/复活仍由 combat 单写；真实出生受阻时保持同一死亡生命和零倒计时。没有音频、模型、DOM、相机依赖进入 simulation/gameplay，也没有修改已提交事件结构。
- 导入场景与回退模型挂在原 UnitVisual 下；就绪前保持回退，普通暂停不进行异步替换，就绪后只移除旧 PickGeometry 模型子树，保留地面标记和护盾。重选依靠原 linked children 清理，目录/共享资源保留；独立模型/播放器缓存不改变模拟实体或计时。
- 源动画依模拟 tick/统一 PresentationFrame 显式设置暂停播放器，生命变化重置遗漏轨道；离线固定 Root 的 XZ，Leap 的 Y 校正位于动画之后、变换传播之前。尸体以实际死亡位置与 death_tick 采样并保持到真实复活；输入仍由 alive 门控排除尸体。
- 蒙皮命中采用每关节 JointAabb 与 `joint GlobalTransform * inverse_bindpose`，矩阵与本地 Bevy 官方 skinning 实现一致。离线删除不可见 alternate head/drone 并按 primitive 实际 indices 压缩所有属性，避免共享顶点污染可见几何边界。零比例附件不产生可点的退化区域，刚性模型和 CSS 容差规则继续保留。
- 图标 DTO 从实际装配调用者接收 AbilityId，未把 ID 复制进 AbilityDefinition；按真实技能映射，不按槽位。头像/model 元数据仍位于已观察单位的投影内；模型、拾取、HUD 和声音均复用团队观察，没有新隐藏敌军信息旁路。JS 倒计时只格式化模拟秒数。
- 音频在每个完整固定步后、Outcome/清理前捕获位置和生命身份，缓存寿命/队列/并发有界，分离于 80ms HUD。WebAudio 唯一拥有设备，在可信手势内同步 resume；未解码或不可播放的 cue 直接丢弃，暂停/失焦/捕获丢失/新会话停止并断开节点，没有补播队列。未修改既有指针增益或按下坐标所有权。
- 最终声音是已固定哈希的 Riot 16.22.8348910 源计划寒冰/原皮皇子真实事件 SFX；2014 旧音效和角色台词不在最终目录。皇子银行没有独立复活事件，单独使用已明确记载的 shared respawn。导入器验证段、源 bank 和输出哈希，完整游戏构建不依赖 Python、ffmpeg、vgmstream 或外部 CDN。
- 地图规则仍取 MapDefinition；地形只更换表现。首轮截图发现的平铺网格由主代理交资源代理改连续 UV，属于尚待最终截图确认的表现修正。

## 已执行与已取得证据

本审查者实际执行并通过：

- 两个修改 Rust 文件的 rustfmt；`git diff --check`。
- `python3 -B scripts/import-league-assets.py`：17 个模型/贴图/图标哈希通过。
- `python3 -B scripts/import-audio.py --verify`：25 个声音产物哈希通过。
- audio.js、hud.js、browser-presentation.cjs、browser-audio.cjs 的 `node --check`。

主代理串行执行/报告的前一快照证据：原生与 WASM Clippy、40 项 Node 回归、完整 make check 通过。实际有窗口浏览器首个1280场景已覆盖资源、尸体、暂停倒计时、复活和重选；真实 browser-audio 已验证25文件解码、可信解锁、皇子 W/Q 与源计划寒冰 W、analyser 非零输出、音量/静音/暂停和重选。上述前一快照尚不包含本轮两项修复及最后地形/比例调整，不能代替最终门禁。

随后已读取主代理串行运行的 `/tmp/kiva-presentation-runtime-review-tests.log`：四项 runtime 测试全部通过（包含上述两项新增回归），0 失败。本审查者没有另起 Cargo 构建。

## 必须由主代理收尾的检查

- [x] 对 `client::imported_models::runtime::tests` 的四项 focused Rust 测试确认结果；主代理串行执行，4 通过、0 失败。
- [x] 最终源快照的格式、native/WASM Clippy 与完整 Rust/WASM 门禁，记录实际结果及原有忽略项。
- [x] 连续 UV/最终比例及本轮修复之后重新 Trunk 构建，再验证真实 WebGPU 画面；800×600、1280×800、DPR2 及死亡/复活/重选。
- [x] 最终构建上的真实模型拾取/剑形光标/标签负例、HUD 隔离和已有 skills、reselection、startup、smoke 回归；按影响保留原生帧钟 healthbars 验证，虚拟帧钟不作为性能证据。
- [x] 清除 `scripts/__pycache__/` 研究副产物；最终检查 TODO 勾选、任务/验证记录与规范路径一致，不自动提交/推送。

本次静态审查未发现其余尚未处理的架构或规则阻断项。最终接受结论必须等待上述实际验证；此文件不是所有浏览器检查已通过的声明。

## 主代理最终闭环

最终源快照已通过343 Rust、40 Node、native/WASM Clippy及临时Trunk构建。四项focused采样测试通过；连续UV、模型最终比例及头像边界已在1280/800/DPR2的实际截图复核。最终构建的presentation、audio、healthbars、picking、skills、reselection、startup、pointer、smoke均串行通过，fixture及认证Xvfb清理完成，源文件指纹无漂移。证据/限制见[verification.md](verification.md)。全部审查发现已解决，无剩余实现阻断项；未提交/推送。
