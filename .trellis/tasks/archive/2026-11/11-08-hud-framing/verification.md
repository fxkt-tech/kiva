# 首轮HUD验证（全部通过）

后续用户追加完整小地图与点击/拖动导航，见[追加验证](minimap-verification.md)。下文构建身份与记录属于首轮HUD，不代替追加任务验收。

## 前置提交
按用户顺序要求，先将旧施法修复提交推送到origin/main：49b34fa工作提交、ec7b578归档、e00d861会话记录。TODO与参考图留为本HUD任务输入，HUD本轮未授权自动提交。

## 实施与静态/原生检查
- web/hud.css统一游戏内布局；style.css仅基础/菜单，match.css事实/结算；minimap.js独立处理只读投影与DPR画布，稳定ID和UI协议保留。
- CameraRig默认0.5/最近0.4/最远1.5，统一setup/update变换；新增camera_footprint真实角点求交。未改simulation/gameplay/model/radius。
- DOM-only布局通过1920/1280/800/1000@2（另含640/480）；/tmp/kiva-hud-layout-check.log。root已目检1280与800图片，证明布局，不当作游戏证据。
- 实际HUD模块DOM检查通过：头顶MP仅在有完整数据时显示、值不覆盖逐帧位置、null footprint无假框、DPR2画布与被动tooltip；/tmp/kiva-hud-render-check.log。
- 相机7项（5新增）、UI2项与会话镜头生命周期1项定向通过。
- CARGO_BUILD_JOBS=2 make check退出0：306项Rust测试通过，2项既有长跑默认ignored，本次不改玩法/AI未重复；fmt/native全target Clippy与WASM目标检查通过。/tmp/kiva-hud-check.log。
- 标准WASM Clippy退出0，/tmp/kiva-hud-wasm-clippy.log；Node启动/指针19项通过，/tmp/kiva-hud-node.log；变更JS语法通过。
- 独立Trellis全范围检查无阻塞/无产品修改，包含74个旧ID唯一/保留、4技能6装备选择器顺序、规范/数据流和测试有效性；见[review](research/review.md)。

## 最终构建身份
/tmp/kiva-hud-site，/tmp/kiva-hud-build.log退出0。
WASM kiva-37b10f69e9a18716_bg.wasm，SHA256 35d8353e641a88544b60b864cf8c845c74aeaf3741a3586720b2196b05fbcfc0。
运行时90文件清单/tmp/kiva-hud-source-final.json，SHA256 0b5defc3874e9c7089f3d716423fbb1f37456e2e1636e6ff08035318285cd1c7；构建前后及最终检查保持一致。

## 实际Bevy浏览器
- /tmp/kiva-hud-browser-stats.log退出0：1920×1080、1280×800、800×600、1000×720 DPR2全部通过；保留原属性数值/输入隔离，新增底部/地图/技能资源装备几何、模型参考投影、真实相机四角roundtrip、DPR画布、被动hover、实际装备/主动/重选。1280场景实际装备并施放主动，验证独立冷却，再重选寒冰。
- /tmp/kiva-hud-browser-smoke.log退出0：真实renderer、拾取/移动/停止、HUD隔离、取消瞄准、Y/Space、边缘平移、滚轮缩放、resize、DPR2拾取、失焦冻结及同局恢复通过。
- /tmp/kiva-hud-browser-healthbars-final.log退出0：人物20/20帧更新、20/20可见；相机19/20帧更新、20/20可见，最大连续停滞1帧，累计真实平移80.595，整个采样中距地图夹紧边界至少2.452。原生RAF/时钟、20帧/80%跟随及HUD每80ms节流上限均保留。
- 首次血条fixture单向持键在慢软件GPU的20帧（约8秒）中移走目标，可见率6/20而更新20/20，未通过；原日志保留在/tmp/kiva-hud-browser-healthbars.log。最终fixture用真实滚轮恢复长路径视野，以只读标签边界驱动实际左右键往返，完整采样保持目标可见，并检查累计行程与全程边距；未写World或改时钟。独立检查再次核对输入串行及finally释放。
- 第一轮串行pipeline因上述fixture失败退出1，不能当作整套通过；stats、smoke各自退出0，修正后的独立healthbars退出0，三部分最终全部通过。
- 实际截图位于/tmp/kiva-browser-tests/artifacts/hud-framing/，包含single-lane-smoke.png与四个stats视口；root已目检1440×900、1280×800和800×600游戏截图，主要区域清晰可见。
- 临时HTTP/Xvfb/浏览器均由各fixture清理；未启动、关闭或重配用户服务。

## 完成状态
- docs/TODO.md与任务验收项已勾选；README、architecture及client规范已同步。
- 首轮HUD验收时保持未提交，仅前置施法修复已推送；随后按用户追加完成小地图导航。用户现已授权本轮提交与推送，工作提交见task.json，本任务完成归档。
