# 验证记录（最新版本全部通过）

## 最终范围
按用户最新要求只检测实际模型，不增加血条/名字命中；敌方模型用剑形光标。公共模型候选与操作资格分离，保留未来友方谓词入口。用户追加A+左键点模型必须锁定该目标，点空地才自动索敌，一并处理。

## 实现与红绿回归
- picking.rs直接读最后已显示的Camera/模型Aabb/GlobalTransform，PickGeometry子树逐部件凸投影，6 CSS px容错，不另加帧缓存。UnitGroundBase装饰排除，基地实际平台保留；根profile仅在无模型边界时fallback。
- controls从公共候选找合法敌人，右键/Enemy技能/A点模型复用；A点模型发AttackTarget并更新selected，空地保留原AttackMove，不黏旧selected。未改simulation/gameplay或战斗数值。
- 原友军重叠红灯selected=None，/tmp/kiva-picking-red.log退出101。几何6项、控制重叠及真实皇子/寒冰身体/武器集成通过；真实武器端点至少超旧根框8px，不能靠6px root fallback假通过。
- A分流红灯/tmp/kiva-picking-attack-click-red.log：3新用例中明确远敌那项失败，实际AttackMove自动锁另一近敌，另外2项旧语义原本正确。修复后全部client94/94，/tmp/kiva-picking-attack-click-native-final.log；验证真实UnitOrder/目标并持续12步，另覆盖空地与友方/失视候选。
- 逐帧kiva-camera.enemy_hovered驱动pointer唯一所有者的剑/箭头，80ms完整HUD不掌管形状；HUD/小地图、UI手势和按下快照隔离保留。hud标签逻辑、web.rs与Cargo无改动，hud仅同步A分流提示文案。

## 最终质量门
- CARGO_BUILD_JOBS=2 make check退出0：322 Rust通过、2项既有长跑默认ignored，含fmt/native全target Clippy及WASM check；/tmp/kiva-picking-check.log。
- 标准WASM Clippy退出0，/tmp/kiva-picking-wasm-clippy.log；Node32通过，/tmp/kiva-picking-node.log，pointer/minimap/startup源码未因A分流更改。新增浏览器与hud文案JS语法、diff检查通过。
- 独立检查见[review](research/review.md)。

## 最终构建身份（含A分流）
临时构建/tmp/kiva-picking-site成功，/tmp/kiva-picking-build.log退出0。
WASM kiva-6d83f58da2cd5750_bg.wasm，SHA256 944d6fc8574a1a60a4a6aa1185e694b07c7718e049c628e939a7d0e4726e6ef7。
92运行时文件before/final相同，/tmp/kiva-picking-source-before.json与/tmp/kiva-picking-source-final.json；清单SHA256 49ea754576fe31bb79799fdb4cd38749aa469e7a12ba53d605315f51382a47a9。

## 实际浏览器
- 首轮纯模型/剑形版本三尺寸1280、800、DPR2及smoke已通过；用户追加A分流后主动终止自有旧流水线，退出143，Xvfb/auth已清理，GPU进程数确认0。旧日志保存为/tmp/kiva-picking-initial-*，不能把旧的后续回归算作新版本通过。
- 当前含A分流版本六stage全部退出0，/tmp/kiva-picking-regressions.log：1280×800、800×600、1000×720 DPR2的picking，随后smoke、pointer-response及原生healthbars。最终整体退出0，Owned Xvfb及临时Xauthority已清理。
- 三个/tmp/kiva-picking-browser-{1280,800,dpr2}.log分别通过：模型剑形、标签空白负向、HUD隔离、A点模型order=Attacking/selected=id、A点空地order=Attack move，以及真实右键追近和实际普攻伤害；1280另含暂停恢复/重选寒冰，无旧身份或剑形。
- /tmp/kiva-picking-smoke.log退出0：真实renderer/拾取/走位与停止、HUD、Y/Space、边缘/滚轮/resize、失焦冻结与同实例恢复、DPR2全部通过。
- /tmp/kiva-picking-pointer-response.log退出0：原生movement单位DPR1/1.5/2与物理单位DPR1.5（gain=2/3），即时事件响应、持有真实RAF仍能移动、早先按下坐标及暂停/再进入全部通过。
- /tmp/kiva-picking-healthbars.log退出0：原生RAF/时钟人物20/20帧更新且20/20可见，相机19/20帧更新且20/20可见，最大停滞1帧、累计真实平移80.595、全程地图边距至少2.452；既有20帧/80%与完整HUD节流阈值未改。
- 实机截图在/tmp/kiva-browser-tests/artifacts/unit-picking/；root已目检800的敌模型剑形场景，checker已目检800/DPR2并在最终收尾再次确认；1280截图位于重选寒冰后，显示友军箭头，不把它当敌方剑形截图。
- 模型实机目标用静态敌方基地，避免AI走位与采样迟滞混为命中错误；英雄/动作武器另由原生真实构造集成覆盖，不宣称浏览器已点到敌方英雄。
- 帧时钟仅规范交互时序，不充当性能证据；血条专项保持原生时钟/RAF。浏览器只发真实输入，不写World或传送单位，临时HTTP/显示归fixture清理。

## 收尾
用户自行管理的开发服务未操作。实现、独立审查与全部最新验收完成，AC已勾选，规范与架构已同步。当前保持未提交，后续按用户提交指令归档。
