# 初版单中路对局验收记录

本记录对应地图边长32、开局批量出兵的初版。用户追加调整后的验收另见tuning-verification.md。

## 实现范围

四个子切片已落地：通用战斗基础、地图/对局、兵线、英雄AI。
依赖方向为`client → gameplay → simulation`；模拟仍唯一拥有战斗运行态。
具体接口见[集成契约](research/integration-contract.md)与[玩法规范](../../spec/gameplay/index.md)。

## 已通过的规则与构建检查

- `make check`：格式、原生Clippy、223项Rust测试、WASM检查通过。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`通过。
- `trunk build --locked --dist /tmp/kiva-lane-site`通过；包含新弹丸首帧位置修复。
- `node --test scripts/pointer-unit.test.cjs`：12项通过。
- 30分钟真实波次长跑：108000tick，31波，每波12新兵；峰值186活兵、299实体、91独立效果。
  无死兵残根，reset后回到开局前30个非对局实体，Unit/Projectile/Area清空；没有活兵上限或跳波。

Rust组成：client62、characters6、combat_foundation10、equipment13、hero_ai12、match_mode14、
minions8、reset2、simulation59、status_effects37。长跑是另行显式执行的ignored测试。

## 浏览器验收进度

- `browser-lane-match.cjs`已通过1280×800、800×600、1000×720 DPR2。
  实际首波16根、双方3+3、基地方位、结构目标保留瞄准、暂停/镜头、真实Defeat、终局冻结、原生捕获重开和旧请求隔离均通过。
- `browser-healthbars.cjs`原生钟通过：人物80帧/1516.6ms、镜头48帧/800ms，更新率均100%、maxStale=0，完整HUD保持节流。
- `browser-stats.cjs`三尺寸属性/布局、HUD隔离、移动与暂停通过。
- `browser-cursor.cjs`捕获、边缘滚屏/越界回移、Esc恢复、技能按钮、resize/DPR2与捕获拒绝通过。
- `browser-characters.cjs`三尺寸角色预览/实际实例、严格与过期意图、延迟真实捕获、拒绝重试、失焦和旧Promise隔离通过。
- `browser-reselection.cjs`三尺寸真实延迟技能提交后整场重选、Waiting无单位、全16根替换、干净新局、旧请求隔离与同实例恢复通过。
- `browser-pointer-response.cjs`四种DPR/位移单位配置通过，保留正常鼠标比例、同事件显示、停引擎帧响应及点击位置身份。
- `browser-startup.cjs`下载失败与重试通过。
- `browser-skills.cjs`三尺寸QWER键盘/HUD、非法目标保留瞄准、真实Q/W控制与恢复、精确4距离击退、空中拾取与姿态冻结、超距E最大端点通过。
- `browser-equipment.cjs`三尺寸六格/属性、真实键鼠主动、独立CD、暂停/旧实例/重选通过；1280深场景另验证被动Slow卸除、最大物品冲刺、自身引导与卸装中断。
- `browser-smoke.cjs`在独占Xvfb有窗口Chromium通过：真实WebGPU像素几何、单中路/HUD、行走/替换/Stop、UI隔离、Y/Space/边缘/缩放、resize、高DPI与原生失焦恢复。
  截图：`/tmp/kiva-browser-tests/artifacts/lane-final/single-lane-smoke.png`；自有Xvfb、认证和HTTP fixture均已关闭。

浏览器只发生产输入或封闭DOM意图、读取投影；没有World写接口、测试地图或隐藏训练模式。
交互夹具的帧时钟仅适配软件GPU，血条频率仍使用原生显示时钟。
原固定训练目标HP断言替换为真实目标ID、生命变化、控制、动作和CD证据；精确伤害公式继续由无窗口规则测试验证。

## 复核发现及修复

1. 小兵侧避让撞墙可能反复清路线：回退原始前进sweep，增加窄走廊真实行进回归。
2. 英雄普攻与进攻冲刺可能使用不同追击范围：共用Pursuit资格、6距离绳、240tick期限和180tick回线窗口。
3. 新弹丸首次插入显示组件时默认在原点：用同一PresentationFrame立即初始化位置和缩放，避免首帧闪烁。

## 维护与交付

已同步三层spec、README和架构文档；用户开发服务保持原样。
本轮未授权提交/推送，代码与任务保留在工作区，全部验收已完成，任务标记可提交；不提前归档。

## 最终复核与构建身份

独立复核两轮均通过，最后夹具增量未弱化真实输入、控制窗口或冻结断言。
最终源文件摘要与构建记录`/tmp/kiva-lane-build-manifest.json`一致，Web资产match/hud/equipment/CSS与源文件相同。
本轮日志位于`/tmp/kiva-lane-final-*.log`与`/tmp/kiva-lane-browser-*.log`。
首轮技能夹具曾在无阶段输出时主动终止，不计通过/失败；补齐诊断与长走位时钟后完整三尺寸从头重跑通过。
