# 客户端与真实浏览器验证

## 构建身份

- 最终临时构建：`/tmp/kiva-champions-site`，干净构建日志 `/tmp/kiva-champions-build-v3-clean.log`。
- WASM：`kiva-16c737afcded6a50_bg.wasm`，SHA256 `ba77afc2f0c0fee2723007185a7a3c42bafe4f960300726e850497c2be849f58`。
- 运行时源码清单：`/tmp/kiva-champions-source-v3-final.json`，SHA256 `27af5b0e431bbcce18f5556cd8834afb316c435e64bf4dd251194120bb514ace`。
- 清单包含 `src/`、`web/`、`assets/` 与构建入口/依赖文件；浏览器脚本和规范不影响该WASM身份。构建前后清单一致。
- 用户Trunk服务未启动、关闭或重配；临时构建在其自动编译空闲时执行。所有GPU套件串行。

## 已通过

| 验证 | 构建与证据 |
| --- | --- |
| 19项Node启动/指针单测 | `node --test scripts/startup-unit.test.cjs scripts/pointer-unit.test.cjs`，全部通过 |
| browser/Web语法 | 20个browser/Web脚本 `node --check` 通过；后续改动的装备/中路脚本单独复查 |
| 属性面板，3视口 | v2 `/tmp/kiva-champions-browser-stats.log`；端游显示口径、无单位、精确属性与UI隔离 |
| 角色选择，3视口 | v2 `/tmp/kiva-champions-browser-characters.log`；版本/旧请求、拒绝捕获、旧Promise、焦点、DPR均保留 |
| 重新选角，3视口 | v3 `/tmp/kiva-champions-browser-reselection-v3.log`；真实W提交后重开、Waiting零根/零标签、新旧实例隔离、同/换角色和普通恢复 |
| 新套件800与DPR2 | v3 `/tmp/kiva-champions-browser-kit-{800,dpr2}-v3.log`；盾、E截断/光环、EQ途中W、寒冰零层Q/W/充能/侦察 |
| 新套件有窗口1280 | v3 `/tmp/kiva-champions-browser-kit-headed-v3.log`；另覆盖皇子R英雄限定/围墙/无目标重施、寒冰真实4次普攻→强化Q→R眩晕 |
| 技能输入，3视口×2英雄 | v3 `/tmp/kiva-champions-browser-skills-v3.log`；键鼠、长按UI隔离、取消、真实计数、暂停精确冻结与同实体恢复 |
| 启动屏障/迟到HUD | v3 `/tmp/kiva-champions-browser-startup-v3.log`；WASM早失败保留/retry及正常首ready/catalog先于HUD仍有9件目录 |
| 原生时钟血条、指针、地点截断 | root串行v2 `/tmp/kiva-champions-browser-{healthbars,pointer,cursor}.log`；移动90/90帧、相机45/45帧更新，maxStale0，HUD保留80ms节流 |
| 最终有窗口smoke | root串行v3 `/tmp/kiva-champions-browser-smoke-v3-final.log`；实际WebGPU像素、点地/替换/Stop、HUD隔离、镜头、resize、原生失焦与DPR2 |

有窗口截图在 `/tmp/kiva-browser-tests/artifacts/champions-v3/`：已目检枪和盾、真实24段石墙、弓与强化态。
最终smoke截图在 `/tmp/kiva-browser-tests/artifacts/champions-final/single-lane-smoke.png`。

## 实战夹具与环境失败

1. 寒冰裸装站桩叠4层会被真实皇子AI击杀。夹具经正常P装备UI穿6件守护披甲，只改变真实防御，保持610HP/59AD/基础攻速；实战4次普攻、强化与R命中全部通过。没有修改英雄、AI、模拟伤害或增加World后门。
2. 旧装备夹具从敌方身边直接徒步回 `nearHome(4,-5)`，此点距基地6.4，仍在追击/获取范围且不是无敌泉水。失败日志 `/tmp/kiva-champions-browser-equipment-v3{,-retreat,-diagnostics,-fountain}.log` 中有真实死亡；新夹具用W和装备Dash横向出线，走到侧翼9.5，再朝外验证独立最大距离Dash与引导，保留原装备实例及所有核心断言。移动/下一次施法显式检查alive，冲刺等待实际终点。
3. 全级console捕获确认 `/tmp/kiva-champions-browser-equipment-v3-flank.log:93`：`Caught DeviceLost error: Unknown A valid external Instance reference no longer exists`，随后Bevy因DeviceLost退出。宿主RAF仍在，应用投影停止。Rust tracing-wasm的ERROR通过console.log输出，不能只监听pageerror/console.error。此前缺全级日志的超时不能唯一反推为同一原因。
4. 装备1280与800在该轮完整通过，DPR2受上述软件GPU设备退出影响而失败；并未跳过功能断言。装备main现每尺寸独立浏览器进程，应用退出立即硬失败，无自动retry/吞错误。隔离重验结果见下节。
5. 中路终局原host180秒超时，实际仍推进到tick7623、第3波、基地854/3000；RAF1651/state524/labels601。fixture performance时间不代表Bevy实际推进（软件GPU与max_delta仍生效）。仅将host预算放至360秒，并增加10分钟真实模拟上限，不改生产时钟/波次/伤害。
6. 独立真实死亡→重生页面通过，普通右键走向敌基地，看到同实体死亡并在tick2072重生；`/tmp/kiva-death-repro.log` 无panic/设备丢失。该证据排除“死亡必然使应用退出”的推断。

## 最后两项收口

- 装备侧翼方案：1280与800已完整通过，`/tmp/kiva-champions-browser-equipment-v3-flank.log`；独立进程DPR2 `/tmp/kiva-champions-browser-equipment-dpr2-v3-isolated.log` 与收紧真实终点守卫后的深1280 `/tmp/kiva-champions-browser-equipment-1280-v3-isolated.log` 均完整通过，无DeviceLost/断言跳过。
- 中路终局：v3独立进程全3视口完整通过，`/tmp/kiva-champions-browser-lane-v3-final.log`。实际Defeat、终局tick/波次/单位/高度冻结、伪恢复拒绝、真实可信Restart、首3→8可见根及旧match请求隔离均保留；过程中真实tick与基地血量持续推进，无设备退出或断言跳过。

本记录客户端/输入与浏览器套件已收口。root随后单独处理R墙与排期/重生落点的后端边界；其后构建身份/定向验证由总verification记录补充，不能把本v3浏览器证据标成后续构建。

原generic Root/Stun/Dash/channel精确规则保留在native规则/控制与装备测试；新browser-skills测试真实新套件。
没有删除旧输入排序、捕获、暂停、选角、装备实例断言；生产只存在新英雄套件。
