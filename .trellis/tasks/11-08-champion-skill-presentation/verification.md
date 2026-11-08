# 最终验证与交付

## 提交边界

按用户要求先提交推送前轮：e0fa8bc功能、f14901a归档、5273207日志，均已推送origin/main。本轮TODO新增三项另行实施，当前保持未提交，待后续提交后归档。

## 最终生产门禁

`/tmp/kiva-skills-final-gates.log`全部PASS，构建前后250文件哈希一致。

- `CARGO_BUILD_JOBS=2 make check`：391项Rust通过、0失败、2项既有长局soak忽略，fmt/原生Clippy/WASM check通过。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`通过。
- Node40通过；夹具补强后再次Node40与四个修改脚本语法检查通过（`/tmp/kiva-skills-delivery-node.log`）。
- 原资源35产物离线大小/哈希校验通过。最终发布目录全部37个VFX文件与仓库逐字节一致。
- 临时`trunk build --dist /tmp/kiva-skill-site-final`成功；git diff --check通过。

新增行为包括16项公开模拟事实、14项effects生命周期/资源、7项素材、6项indicator、4项实际camera移动、共享目标及真实R重施音频等；通过生产命令驱动规则，不伪造SimulationEvent或写私有战斗字段。

## 实际浏览器验收

| 场景 | 结果 |
| --- | --- |
| VFX 1280×800 DPR1 | 通过：原素材请求/实际绘制，Q/W/E与寒冰W/E/R、当前相机回投影<1 CSS px、暂停4 RAF稳定、重选清零，无额外运行告警 |
| VFX 800×600 DPR1 | 通过，同上，窄窗口HUD边界保留 |
| VFX 1000×720 DPR2 | 通过，同上；补强后还明确核对同一Scout Entity到站变Reveal并实拍大扫描环 |
| 深度champions | 通过：真实皇子盾/E最大距离/光环/EQ途中W，R超距追近/实际伤害/原岩墙/重施拆墙；真实寒冰4次普攻→Q强化→10层箭、R英雄命中/伤害/眩晕 |
| Ashe补拍 | 通过：命中后延12tick捕获4层冲击，root目检蓝色冲击/辐射纹理，强化箭再次实拍 |
| skills | 两英雄×三窗口通过：鼠标/键盘槽位、取消、HUD按住隔离、暂停 |
| equipment | 三窗口通过；1280深度覆盖六格/属性、真实伤害/控制、角色与装备独立CD、普攻被动/卸装、最大距离位移/卸装打断引导、旧实例 |
| picking | 三窗口通过：模型悬停剑形、标签排除、A点模型锁定/空地攻击移动、超距追击/实际伤害、暂停与重选 |
| minimap | 1280/800/DPR2全部通过：全图点击/偏移拖框、组合键释放、UI隔离、边界/滚轮、逐帧框、暂停恢复/重选 |
| pointer | DPR1/1.5/2及DPR1.5物理单位全部通过：正常比例、同事件显示、引擎RAF停顿仍响应、按下点与捕获恢复；物理单位gain=2/3 |
| healthbars（原生时钟） | 通过：人物移动20/20帧更新，相机移动19/20帧更新，maxStale分别0/1；真实浏览器时钟，独立标签通道与HUD契约保持 |

八技能功能与关键原素材均已实际验收。root实际查看1280/800/DPR2指示/飞行图片，以及原盾、金色矛、原岩墙/坍塌、Q增强箭、真实E到站扫描和R冲击；没有只按ready/phase诊断字段宣称渲染正确。
截图目录为`/tmp/kiva-browser-tests/artifacts/skill-visuals/`与`/tmp/kiva-browser-tests/artifacts/champions/`。GUI使用独占认证Xvfb+Chromium/SwiftShader，虚拟帧钟只验证交互，不当作硬件FPS证据；血条单独用浏览器原生时钟。

主要成功日志：`/tmp/kiva-skills-final-browser-{vfx-1280,vfx-800,vfx-dpr2,champions,ashe-impact,skills,equipment,picking,pointer}.log`；小地图为`/tmp/kiva-skills-minimap-corrected.log`、`/tmp/kiva-skills-minimap-800-corrected.log`、`/tmp/kiva-skills-final-browser-minimap-dpr2.log`。

用户指出验证耗时三小时后，收敛到上述相关收尾。未再重复前轮已验收且本轮未改的启动故障注入、音频文件解码、尸体展示等完整套件；本轮实际冷启动/整场重选已在VFX、champions、skills、equipment、picking、minimap中多次覆盖。

## 已修正与独立审查

独立全范围代码检查见[research/review.md](research/review.md)，源素材证据见资源子任务research/gpu-source-review.md。

已修正：Nova源UV90度与alpha峰值、Standard旧VCP全零乘黑、billboard世界锚点及穿体遮挡、首帧旧圈重叠、已删载体再次清理、普通在途箭源删除外观、真实Terrain移除时刻/过期坍塌、成功盾归属、Scout外观失败原子替换、重施不重播初次动作/声音。规则、伤害、移动、冷却与视野不变。

一轮DPR2发生ERR_NETWORK_CHANGED、引擎未启动；保留失败日志后正常重试通过。小地图的两次失败均定位为夹具采样：80ms旧HUD误当点击瞬间镜头；绝对合成鼠标2.135635px残差超原2px阈值。纠正采样/有限绝对定位闭环后，同场景及三窗口回放通过。具体红绿证据见[research/minimap-hud-sampling.md](research/minimap-hud-sampling.md)。未改pointer.js或放宽阈值，moveRelative和响应测量仍单次输入；临时DEBUG/短超时全部删除。

## 源与构建身份

最后完整门禁后，生产代码/资产及Rust测试一直未变；只补强了四个浏览器夹具文件并清理了生成的Python缓存，已通过语法/Node及实际回放。

- 编译时250文件manifest：`/tmp/kiva-skills-final-source.json`，canonical SHA256 `4e2deee01d18c23625e71b89eeffe01ed15827dfc99d26d26fcd71612048b65b`。
- 其中220个非script文件不变：`/tmp/kiva-skills-production-source.json`，canonical SHA256 `0048d315d9f76e4c723bf960507063a0f6b3091950c623bf86b0ff1acde0364a`。
- 最终249文件交付manifest：`/tmp/kiva-skills-delivery-source.json`，canonical SHA256 `bfff9466ddccad1e4d229b85ff6a0cb297cb5145b6838d4567c9baa6ad55305b`。
- 实际WASM `kiva-12cdde3af63ff17f_bg.wasm`，SHA256 `c9d55647387fa14f3fa6cccbe3967f533f17e81b95b36e4aa74f3f96baaef0a7`。
- 资源专项42文件manifest：`/tmp/kiva-vfx-final-files.json`，canonical SHA256 `126abbaaf50375f92eec0989cd32b06afb9fea7f3f60364788b6b8922a1bb20b`。

重型构建与GPU由root串行；只启停自有临时fixture/浏览器/Xvfb，未启动或关闭用户开发服务。按收敛范围，pointer子进程exit0并清理后终止自有外层批量编排器，再独立运行原生时钟血条，避免继续排入无关完整套件。

## 表现边界

使用皇子原皮/源计划寒冰的原PNG、SCB网格和核对后的参数重建关键技能外观。噪声、扭曲与层级发射器有简化，未实现完整League粒子运行时，不宣称逐像素相同。新模块保持client→gameplay→simulation单向依赖，不开放写World桥。
