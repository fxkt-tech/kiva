# 联合验收

## 实现与边界

- 皇子原皮、源计划寒冰及双方近战/远程小兵使用本地骨骼 GLB 和源动作；就绪前保留程序模型，完整加载后原子替换。
- 八技能图标按实际 AbilityId 映射，头像按 CharacterId 映射；地图原纹理适配现有 MapDefinition，不改变导航、视角或玩法。
- 模拟只增加 death_tick 和现有复活计时的只读 getter；英雄尸体保留至真实复活，暂停冻结，安全落点阻塞时停在0秒。小兵仍按原规则回收。
- 独立音频适配按每个真实固定步捕获攻击/提交/生命事件及当时生命、位置和视野；单一 WebAudio 播放25个本地声音，音量/静音、解锁、暂停和重开清理独立于战斗规则。
- 皇子原皮/源计划寒冰 SFX 来自固定16.22银行；皇子银行没有独立复活事件，该 cue 明确使用另行记录的通用 League 复活音效。没有以旧2014样本或台词代替战斗声音。
- 只写只读诊断和封闭操作意图，没有增加写 World 的浏览器测试入口。

## 最终代码与构建

2026-11-08，主代理串行执行：

| 检查 | 结果 | 日志 |
| --- | --- | --- |
| `CARGO_BUILD_JOBS=2 make check` | 343 Rust通过，2个既有长局soak忽略；格式、native Clippy、WASM类型检查通过 | `/tmp/kiva-presentation-final-check.log` |
| WASM Clippy `-D warnings` | 通过 | `/tmp/kiva-presentation-final-wasm-clippy.log` |
| `node --test scripts/*-unit.test.cjs` | 40通过 | `/tmp/kiva-presentation-final-node.log` |
| Trunk临时构建 | 通过，包含最终比例/连续UV/暂停Root/低帧动作修复和头像尺寸修正 | `/tmp/kiva-presentation-final-build.log` |
| 模型/贴图/图标校验 | 17本地文件哈希通过 | `scripts/import-league-assets.py` |
| 音频校验 | 25产物哈希、完整解码和空缓存重建一致 | combat-audio/verification.md |
| 音库源复现 | 全新网络缓存重建7个固定bank/BIN，SHA均一致 | `scripts/fetch-league-audio.py` |

最终验收源快照191文件，记录在 `/tmp/kiva-presentation-final-source.json`，
排序JSON SHA-256：`3688ecac12e00d77a9439fa3ea0e7c15b0dd7ac7fb6749c5c0c131843d73f4bd`。
临时发布目录 `/tmp/kiva-presentation-site`；WASM SHA-256：
`9fddfc2bff35e254465cc605d30019e1676b47969cf169e363802644d13cd34b`。
临时构建的 hud.css 已逐字节比对最终源文件；不以旧构建代替最终验证。

## 有窗口浏览器

独占、自有、带认证 Xvfb 上的有窗口 Chromium + SwiftShader WebGPU；每个脚本创建并清理自己的临时HTTP fixture。
没有启动、关闭或接管用户开发服务。

| 脚本/场景 | 最终构建结果 |
| --- | --- |
| presentation 1280×800 DPR1 | 通过：实际GLB选中、原图标/头像、真实战斗死亡、尸体、暂停倒计时、复活和重选 |
| presentation 800×600 DPR1 | 通过：皇子原皮、实际模型选中、HUD图片边界和重选 |
| presentation 1000×720 DPR2 | 通过：源计划寒冰、实际模型选中、HUD图片边界和重选 |
| audio | 通过：25文件实际decode、可信解锁、皇子W/Q与源计划寒冰W的analyser输出、音量/静音/暂停/重选 |
| healthbars（原生浏览器时钟） | 通过：人物20/20帧更新，相机19/20帧更新；HUD仍节流。SwiftShader采样不代表硬件FPS |
| picking | 通过：1280、800、DPR2的模型命中/剑形/标签负例、A锁目标/空地自动索敌、超距追击伤害、暂停/重选 |
| skills | 通过：两英雄×三窗口的真实鼠标/键盘施法、取消、HUD隔离和暂停状态 |
| reselection | 通过：三窗口的整局清理、不同/相同角色重选、旧请求隔离和普通恢复 |
| startup | 通过：WASM先于HUD失败/成功、错误重试、迟到HUD和目录 |
| pointer | 通过：DPR1/1.5/2正常移动与DPR1.5物理单位校准、同事件显示、渲染暂停、按下坐标和恢复 |
| smoke | 通过：真实WebGPU、地面拾取/移动/Stop、HUD隔离、跟随/滚屏/缩放、失焦暂停/恢复和高DPI |

截图目录：`/tmp/kiva-browser-tests/artifacts/battle-presentation/`。
主代理已实际查看1280、800、DPR2截图及 `project-ashe-corpse.png`，不是仅据模型名称/ready字段验收。
首轮画面发现的硬接缝和头像溢出已修正，最终截图复核；英雄/小兵比例使用最终目录值。
交互脚本的虚拟帧钟只用于稳定输入和逐步规则，不作为FPS或跟随性能证据；healthbars另用浏览器原生时钟。
所有最终浏览器日志均为 `/tmp/kiva-presentation-final-browser-<场景>.log`，脚本逐个串行执行并以0退出，各自finally关闭fixture和浏览器，认证Xvfb及临时认证文件也均已清理。

## 独立审查

完整静态审查见 [review.md](review.md)，生命周期中途审查另见子任务。
已修复并通过回归：同显示帧跨生命后旧姿态残留、回退尸体不平整、暂停时源Root高度重写、跨完整高速普攻后旧技能记忆。
最终完整make check已包含这些回归与连续UV共享边测试。

## 交付限制

- 当前资源是模型/动作、原图标、地图纹理和选定SFX；没有扩成完整召唤师峡谷或重写现有技能粒子。
- 本地声音层导出不模拟整个Wwise RTPC/材质切换图，音频故障只影响表现。
- 无提交/推送授权；代码保持可审查的未提交工作区，Trellis任务保留到后续提交收尾，不自动归档或提交。
