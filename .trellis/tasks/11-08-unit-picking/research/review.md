# 独立检查：模型命中与剑形光标

## 范围与当前状态

以用户最新要求和本任务PRD/design为准：只检测敌方模型，悬停显示剑形，血条/名字不参与；几何候选与操作资格分开，保留未来友方技能复用入口。

最终独立审查通过，无阻塞项。模型命中、剑形光标及追加的A+左键目标修复均完成；追加后的完整质量门、三视口真实WebGPU专项与既有浏览器回归通过，当前源码与最终被测构建身份一致。

## 追加：A+左键目标

已确认原`Aim::AttackMove`分支完全忽略公共enemy候选，只提交指针射线的地面交点为AttackMove，导致点击敌方模型仍由模拟重新索敌。应与右键复用同一候选：命中合法敌人提交AttackTarget并更新selected；空地才提交AttackMove，不使用旧selected作为目标。

已实现上述分流。新增3项原生回归先取得真实红灯：点击远敌仍自动攻击近敌；修复后验证实际`UnitOrder::AttackTarget`持续12步保持点击者，而不只断言HUD选择。空地自动索敌不受旧selected控制，友方/失去视野候选不能成为显式攻击对象。追加后94项client原生测试全部通过。

追加前流水线已由root主动终止并清理，旧构建不作为最终验收。最终重新构建后六阶段流水线全部退出0；三个视口均验证A点模型进入`Attacking`且selected为该单位，A点空地保持`Attack move`，区别于仅检查最终伤害或持久选中信息。

## 已核对实现

- `picking.rs`单独拥有部件几何、候选聚合与稳定排序，没有敌友/射程/技能ID分支。可见、存活与团队观察在查询时复核。
- 直接读取固定步输入前保留的最后显示`GlobalTransform/Aabb/Camera`，无第二份位置缓存；逐部件凸投影与6 CSS px容错，避免整根斜武器形成大空白点击矩形。
- `PickGeometry`沿模型祖先继承，普通脚下标记/技能特效不算身体；真实Base平台显式作为基地模型参与。模型有边界后不退回root profile，隐藏模型也不能重新激活fallback。
- `controls`先得到阵营无关候选，再按当前操作选择合法敌人；右键、A+左键与Enemy技能使用相同候选，既有preview与模拟复核保留。原有测试夹具明确填入候选，没有生产测试专用回退。
- 完整HUD只增加只读`hovered`，`selected`仍是点击后持久选择；原`kiva-labels`数组保持。逐帧`kiva-camera.enemy_hovered`复核目标存活、观察和相对玩家队伍。
- `pointer.js`只读取Rust发布的敌方布尔反馈并门控真实捕获、焦点、暂停、HUD和UI手势；不推导几何/阵营、不改位置或按下三元组。暂停/失锁/失焦清掉旧剑形状态，UI命中在鼠标事件内立即恢复箭头。
- 双SVG继续复用原光标尺寸、热点和边缘翻转，`style.css`保留原光标样式所有权。没有新增标签DOM命中桥、`data-unit-id`、HUD标签点击或Cargo依赖。

## 测试有效性

- 已协调并复核实际皇子/寒冰构造集成：生产`spawn_character`与场景/骨骼模型创建真实Mesh资产，通过`MeshAabb::get_aabb`和标准Transform传播取得显示边界；只有原生产CharacterModel的`PickGeometry`祖先标记，没有给武器手工打标。身体和实际武器子孙端点都可命中，且至少一个端点在旧root投影矩形外超过8 CSS px，不能由6px fallback假通过。
- `browser-picking.cjs`走真实侧翼进入敌基地团队视野，使用真实模型悬停、光标SVG、鼠标选中和右键追近/攻击结果，没有写World或瞬移接口。血条/名字只在远离躯干的实际DOM边缘做负向检查。
- 悬停与持久选择分别断言，HUD输入隔离、暂停/重选、800及DPR2覆盖；帧时钟用于稳定交互，不作性能证据。
- 已复核新的unit-picking规范和browser/character/pointer/index/README/architecture同步：原固定root AABB主判定和“骨骼手脚不能移动拾取框”旧表述已改为实际显示部件主判定、root profile仅明确fallback；标签负向与逐帧剑形契约一致。

## 已执行与审阅证据

- 本代理执行`git diff --check`、变更JS语法检查均通过。
- HTMLParser对比HEAD：76个稳定ID唯一且完整保留，四技能/六装备槽顺序及脚本安装顺序不变。
- 已审阅`/tmp/kiva-picking-red.log`：原始重叠友军挡敌人场景准确失败，右键选中为None；不是编译错误假红灯。
- 已审阅`/tmp/kiva-picking-native.log`：当前6项几何专项通过，包含真实显示变换、远近/DPR容错、斜武器空白、显式命中优先、友方候选复用、隐藏/死亡/销毁/退化投影。
- 已审阅`/tmp/kiva-picking-client-native-final.log`：初版91项client原生测试通过，含真实皇子/寒冰身体与武器接线集成、重叠友军挡敌人、既有技能/装备/视野/击飞/暂停/重开回归。追加后的`/tmp/kiva-picking-attack-click-red.log`为真实行为红灯，`/tmp/kiva-picking-attack-click-native-final.log`的94项client测试全部通过。
- 已审阅`/tmp/kiva-picking-node.log`：32项Node通过，包含光标形状反馈不移动指针、UI手势与生命周期清理。
- 已独立读取追加后的`/tmp/kiva-picking-check.log`并累计各组：322项Rust通过、0失败、2项既有长跑默认忽略；fmt/native全target Clippy/WASM check均通过。`/tmp/kiva-picking-wasm-clippy.log`的标准WASM Clippy也通过。
- 已独立逐项核对before/final清单的92文件当前源码hash，二者相同，清单SHA256为`49ea754576fe31bb79799fdb4cd38749aa469e7a12ba53d605315f51382a47a9`。最终构建成功；WASM `kiva-6d83f58da2cd5750_bg.wasm` SHA256为`944d6fc8574a1a60a4a6aa1185e694b07c7718e049c628e939a7d0e4726e6ef7`。
- 已独立读取`/tmp/kiva-picking-regressions.log`：browser-1280、browser-800、browser-dpr2、smoke、pointer-response、healthbars六阶段全部退出0，并完成自有Xvfb退出及临时Xauthority清理。
- 三份最终browser日志均通过：模型剑形与血条/名字负向命中、友方/地面箭头、HUD隔离、A两种分流、超距右键真实追近及攻击伤害；1280另验暂停恢复及重选寒冰无旧状态。已目检800与DPR2模型上的剑形截图；最终800截图再次复核正常。1280截图位于重选之后，显示普通箭头符合当时状态。
- `/tmp/kiva-picking-smoke.log`覆盖真实渲染、移动/停止、HUD、跟随/拖边/缩放/resize、焦点暂停恢复及DPR2。`/tmp/kiva-picking-pointer-response.log`四个场景通过：DPR1/1.5/2原生位移增益1，DPR1.5物理位移增益2/3，同事件光标更新、渲染停顿响应、按下坐标快照及恢复均正常。
- `/tmp/kiva-picking-healthbars.log`使用原生RAF：单位移动20帧更新20次、可见20帧、最大滞后0；相机移动20帧更新19次、可见20帧、最大滞后1，实际移动80.595、最小视口余量2.452。既有逐帧血条与80ms HUD节流契约保持。

本代理只写审查记录，未修改产品源码、未另起Cargo/GPU、未提交推送或操作用户开发服务。
