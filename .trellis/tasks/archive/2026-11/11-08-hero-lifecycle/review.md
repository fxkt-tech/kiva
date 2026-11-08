# 生命周期与 HUD 中途审查

审查者：`check_lifecycle_ui`。本次为 Phase 2.2 中途静态检查，音频和导入模型仍在实现；不代表最终集成验收。

## 已修复

- `src/client/character/animation.rs`：显示中的跑步/攻击生命可能在两次显示帧之间死亡、复活、再次死亡。原死亡分支仅在旧 gesture 已经是 Death 时换用中立姿态，旧 gesture 为 Run/Attack 时会把旧生命的局部骨骼当成新生命尸体起点。现在 Died 只清施法记忆，Respawned/生命变化才标记新生命重置；死亡采样遇到待重置新生命时使用其初始姿态，普通首次死亡仍从当前真实可见姿态倒地。
- `src/client/character/tests.rs`：新增 `unseen_respawned_life_does_not_inherit_previous_running_pose_when_it_dies`。通过公开移动与致死技能指令制造实际死亡→复活→再死亡，期间不运行显示更新，检查新生命的首个尸体姿态不继承旧 Run。

## 已交主代理修复

- 私有技能 DTO 曾使用不存在的 `AbilityDefinition.id`。目录拥有 ID，定义本身没有该字段；已建议从实际装配和装备调用者传入 AbilityId，主代理回复已完成，不复制 ID 到模拟定义。
- 新图标逻辑曾通过 `svg.hidden` 隐藏 SVG。SVGSVGElement 不实现 HTMLElement.hidden，需操作 `hidden` attribute 以使用既有 `[hidden]` CSS。UI 文件由主代理拥有，已发送定位与建议，本代理不越权修改。

## 静态审查结论

- HP、死亡 tick、复活剩余秒数仍由模拟唯一写入，客户端 getter 只读；出生落点不可用时保持原生命且剩余为0，实际重生才清除。
- 死亡显示根读取最终死亡 UnitPose.position，避免用移动前一采样回退；尸体没有新的模拟实体，依然被 alive 拾取门控排除，不提供视野。
- 程序骨架只在局部骨骼倒地；普通暂停经 PresentationFrame.should_sync 保持精确姿态，终局沿用最后权威步同步一次。
- 新模型、头像、技能图标 metadata 位于客户端投影；UnitView 仍先经过 VisibleWorld 过滤，未建立可见性旁路。
- HUD 倒计时只格式化 Rust 剩余时间，没有 JS 独立时钟。活着和选角均隐藏。
- 现有公开行为测试覆盖移动中死亡、尸体可见但不可选、暂停冻结、同模型重生和递归清理；新增本次跨生命死亡边界补齐旧测试结束为活着时覆盖不到的情况。

## 验证状态

已执行：仅拥有文件的 rustfmt、`node --check web/hud.js`、`git diff --check`，均通过。

按主代理串行构建安排，本代理没有执行 Cargo、Trunk 或浏览器。以下必须由主代理运行，当前状态为 **pending**：

```sh
cargo test --locked --test simulation death_clock_freezes_on_pause_and_waits_at_zero_until_respawn_is_safe
cargo test --locked --lib client::character
cargo test --locked --lib client::scene
make check
```

最终还需实际 GLB 的尸体/动画/拾取、Web 倒计时、暂停恢复与整局重选回归。完成后回写仍描述“死亡隐藏”的客户端动画、浏览器与对局规范，不把本中途审查当作完整质量门。

## 首次联合测试后的倒地修复

主代理首次运行组合 lib tests，报告106通过、2失败；本次新增跨生命死亡回归已通过，原 `hero_corpse_stays_at_death_position_is_unpickable_and_freezes_until_respawn` 的倒地高度断言失败。另一个 imported_models 测试缺少 NavigationMap 的问题由资源代理处理。

独立解析实际 `stickman.anim` 骨骼/首帧数据并按库中的层级与坐标变换计算：Sword_Idle 整体倒转90度仍有约0.894世界单位的垂直跨度，其中原举起的左手和弯曲左腿变成朝上伸出的肢体。问题在死亡目标姿态，不能调宽测试阈值。中立 rest 骨骼倒转后跨度约0.202。

修复 `poses.rs`：Death 使用中立 rest 骨骼并跳过活体持武器的手臂 IK，之后仅在骨骼局部旋转倒地；落地高度修正从模型空间换回根局部单位，避免把包含1.1展示缩放的距离直接加进源数据。原尸体高度 `<0.6` 断言保持不变。

新增 `fallback_death_pose_is_flat_and_grounded_for_every_weapon_style`：剑、枪、弓分别验证倒地模型实际最低点为0.12、上下跨度低于0.35。rustfmt 与 diff 检查通过，已请求主代理串行重跑整个 `client::character` 测试组，当前重跑结果 **pending**。
