# 验证记录

## 最终构建身份

- v4临时站点：`/tmp/kiva-champions-final-site`，构建日志 `/tmp/kiva-champions-build-v4.log`，退出0。
- WASM：`kiva-f196e1cb33cf5548_bg.wasm`，SHA256 `ff3f564c8809df090799a4c00e0bad2a7211644a91a9987422ee62a191f06d0a`。
- 87个运行时文件清单：`/tmp/kiva-champions-source-v4-final.json`，SHA256 `74219672c9e63fe48c32304bf23b7a9fa1b6acd150ad95e1e4d49e366ea0801a`。构建前后及最终检查完全相同。
- v3完整浏览器证据的构建身份保留在 [客户端记录](research/client-verification.md)。v4相对v3只更改spawn、combat、simulation入口和navigation四个后端文件，用于临时围墙下的出生/复活；客户端与Web源完全相同。

## 完整质量门

| 验证 | 最终结果与证据 |
| --- | --- |
| `make check` | v4退出0：fmt、native Clippy、280项Rust测试通过、WASM目标检查；2项默认ignored长跑已分别显式通过。`/tmp/kiva-champions-check-v4.log` |
| WASM Clippy | v4退出0，`/tmp/kiva-champions-wasm-clippy-v4.log` |
| Node启动/指针 | 19项全部通过，最终文档收口时复查 |
| JS语法 | 20个相关脚本曾全部通过；最终15个变更/新建JS再检查通过 |
| 规范/任务上下文 | implement/check各15条路径均验证通过；变更JSON/JSONL全部可解析 |
| 文档/差异 | 48个本地Markdown链接通过，`git diff --check`通过 |

## 规则、内容与AI

- `champion_content` 11项：真实默认内容的一级数值、被动、Q减甲/EQ击飞、盾、军旗被动/光环、寒冰扇箭去重/4层强化、侦察与丢视野停止追踪、透兵大箭/终点视野、英雄限定R与免费重施。
- `champion_mechanics` 9项：盾全吸收仍on-hit/战斗时钟、溢出原子拒绝、精确充能tick/暂停、组合拒绝、衰减Slow、墙与弹丸区别、死亡与已提交效果、EQ越墙/途中施法/Root/暂停。
- `definition_validation` 5项：被动时长tick溢出、非有限箭速、派生飞行寿命、盾聚合与被动攻速聚合溢出等。
- `terrain_placement` 5项：受静态墙阻挡的排离、双圈重叠/独立拆墙、封闭腔室整体拒绝与事件、f64解析扫掠覆盖薄圈/擦边；不以固定采样跳过碰撞。
- `placement_lifecycle` 4项：临时圈覆盖工厂原点/复活home时共享局部落点解析，原home保留，墙消失后回原home；静态非法仍拒绝；无安全复活点不发布新life/恢复资源，等待下一tick。
- `match_mode` 22项：包含真实R覆盖下一波出生点，仍在tick3600出第一对、3750出齐3+3；保留原编队/身份，不漏发、不把临时无路当wave_error；静态非法与工厂原子回滚保留。
- `hero_ai` 18项：双方真实技能、先旗后Q、盾/资源/叠层/侦察与视野；Dash/EQ/Leap共用可见支持下的净风险检查，没有满血绕过。
- `minions` 8项、原有simulation 59项、status 37项、reset 2项等均纳入最终280项。基地获取测试走真实移动接近与视野，不关闭可见性。
- 伤害精度：盾后伤害保持f64扣HP后再转f32，Damaged.amount使用真实HP差；原负抗性精确估算回归通过。

## 显式长局验证

两个方向均真实推进31波、108150tick、372个唯一出兵，无单位上限/跳tick/残留死兵根：

- 玩家皇子/AI寒冰：峰值175活兵、269实体、67旧口径独立效果；reset回32静态实体。
- 玩家寒冰/AI皇子：峰值180活兵、280实体、81效果（包含BattlefieldObject）；实际军旗与围墙都出现，reset回32静态实体。

长跑在最后出生排离修复前完成；v4增量另由placement_lifecycle与真实二波围墙公开回归覆盖，不能把旧长跑标成v4执行。

## 真实Web浏览器

全部使用正常输入/装备UI与只读投影，没有修改World、英雄HP/AI、生产时钟或伤害：

- v2原生时钟血条：移动90/90与镜头45/45帧更新、maxStale0，HUD保留80ms节流；指针DPR1/1.5/2及系统150%物理单位校准、技能最大距离截断均通过。
- v2属性/选角各3视口：精确显示数值、无单位、UI隔离、捕获/失焦/旧请求协议均通过。
- v3完整回归：重新选角3视口、技能3视口×2英雄、启动屏障/迟到HUD、装备3视口与同实例深流程、中路3视口及真实Defeat/冻结/可信Restart/旧match隔离、有窗口smoke全部通过。
- v3新套件800/DPR2与有窗口1280深流程均通过；枪、弓、盾、旗、真实围墙、五箭表现已截图检查。
- v4有窗口1280新套件已通过：真实最大E、军旗光环、EQ途中W、英雄R/拆墙、寒冰零层Q拒绝、W、充能/侦察、真实普攻4层强化Q与大箭眩晕。`/tmp/kiva-champions-browser-kit-v4.log`。
- v4最终有窗口smoke退出0：WebGPU像素/中路几何、点地与替换/Stop、HUD隔离/取消瞄准/Y与Space/边缘平移/缩放和resize、暂停与原生失焦/同实例恢复、DPR2投影与拾取全部通过。日志 `/tmp/kiva-champions-browser-smoke-v4.log`；截图 `/tmp/kiva-browser-tests/artifacts/champions-v4/single-lane-smoke.png`。

v3分项日志、截图与失败环境分析见 [客户端记录](research/client-verification.md)。软件GPU多context触发DeviceLost时明确硬失败，后续改每视口独立进程验证，无自动retry/吞错/跳过断言。中路host预算按实际模拟推进放宽，仍保留10分钟模拟上限。早期失败日志不计入最终通过结论。

## 独立检查与收尾

Trellis Phase2.2独立全范围检查及玩法/客户端增量复核均已完成，无未修实质阻塞。检查发现的围墙排离、解析扫掠、投射物首帧、视野半径、输入验证/溢出、AI风险与出生/复活问题全部已修，契约已写入架构和分层规范。

实现阶段未提交；用户随后明确授权提交与推送，按该授权执行工作提交及任务归档/会话记录。Trellis归档/日志自动提交配置保持关闭，由本次显式提交收口。用户管理的Trunk服务未被启动、关闭或重配；临时构建/浏览器/Xvfb由测试自身清理。
