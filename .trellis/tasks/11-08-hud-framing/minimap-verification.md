# 小地图导航验证（全部通过）

## 已完成实现与静态/原生检查
- 完整地图固定8 CSS px内边距，绘制边界/网格/中路/静态基地/当前可见单位；矩形为真实透视地面四角的外接范围，按地图边界裁切。
- 点击定位、偏移拖动、地图外拖动与chord左键使用同一转换；先加抓取offset再夹最终目标，避免反向边界不可达。
- data-camera-request只保留最新带player_id目标，Rust严格解析、有限数/当前身份/暂停门控，并统一clamp、退出锁定、保持zoom。
- kiva-camera逐显示帧更新框，底图保留完整state频率，原kiva-labels数组与完整快照字段保持。
- 按下三元组保存事件时UI捕获归属，拖出地图chord后即使同帧全释放，controls仍跳过该次已由UI拥有的按下；stop与手势finish复位capture，不回写已排队元数据。
- 初始minimap专项准确红灯：原模块7项中6失败，/tmp/kiva-minimap-unit-red.log；最终9项小地图+指针/启动合计30通过，/tmp/kiva-minimap-node-final.log。
- CARGO_BUILD_JOBS=2 make check退出0，311通过/2项既有长跑默认ignored；包含fmt/native全target Clippy与WASM目标检查，/tmp/kiva-minimap-check-final.log。相机11项（新增4）、Web桥1项新增。
- 标准WASM Clippy退出0，/tmp/kiva-minimap-wasm-clippy-final.log。JS语法、diff及task上下文路径检查通过。
- 独立检查见[review](research/minimap-review.md)，包含稳定选择器、旧HUD兼容、数据流、输入归属与测试有效性。

## 构建身份
构建/tmp/kiva-minimap-site成功，/tmp/kiva-minimap-build.log。
WASM kiva-fd397e469e2c3600_bg.wasm，SHA256 e79bfa591a83f28ba642c72a81d6d120d0c27725ccfde3b20e0562e01c8016d8。
运行时90文件before/final清单相同，/tmp/kiva-minimap-source-before.json及/tmp/kiva-minimap-source-final.json，SHA256 5241fcc9f8ae04d7da46425809ea510f1b6c9e32db1cc2d6e15678c0d18ea322。

## 真实浏览器
独占临时有窗口Xvfb、真实WebGPU、原生Pointer Lock与可信鼠标；复用帧时钟仅规范交互时序，不当作性能证明。
- 最终browser-minimap三尺寸全部通过：1280×800日志/tmp/kiva-minimap-browser-1280-final.log，800×600日志/tmp/kiva-minimap-browser-800.log，1000×720 DPR2日志/tmp/kiva-minimap-browser-dpr2.log。真实点击/偏移拖动/松开、右键隔离、边界、滚轮、独立相机帧、暂停恢复均通过，1280另含resize与重选。
- 地图外chord专项持有真实游戏RAF，在同一Rust帧前完成右键按下与左右全释放，并断言camera事件未推进；恢复后aim/order/position保持，能抓住只读取当前DOM拖动态的错误。
- 首次pipeline的1280通过、800拖动超时，日志保留/tmp/kiva-minimap-browser.log（退出1）。根因是测试用理想小数CSS端点推算位移，而真实Pointer Lock合成鼠标会取整；800小地图每CSS像素约0.57世界单位，起止偏差叠加超过旧0.9容差。改为读取真实按下/移动端点，再按独立地图比例算预期，未改产品源码；镜头容差收紧至0.04世界单位、实际屏幕中心投影误差收紧至2px。
- DPR2最终脚本逐个核对8个camera事件的矩形left/top/width/height与该次真实footprint，而非只靠完整HUD之间的几次样式变化。Node也验证独立camera事件只刷新框、不重绘底图。
- root目检三种尺寸实机截图：/tmp/kiva-browser-tests/artifacts/minimap-navigation/minimap-{1280x800-dpr1,800x600-dpr1,1000x720-dpr2}.png；全图边界、矩形和原HUD均清楚且不交叠。
- /tmp/kiva-minimap-smoke.log退出0：真实renderer/地面拾取/走位与停止、HUD隔离、Y/Space、捕获边缘平移、实际缩放、resize拾取、DPR2、原生失焦冻结和同实例恢复全部通过。
- /tmp/kiva-minimap-pointer-response.log退出0：原生单位DPR1/1.5/2，以及模拟物理单位DPR1.5（实际校准gain=2/3）全部通过；同事件立即显示、真实游戏帧被持有时仍响应、早先按下位置保留与重新进入居中均通过。
- /tmp/kiva-minimap-healthbars.log退出0：不安装虚拟帧时钟，原生RAF人物20/20帧更新、20/20可见；相机19/20帧更新、20/20可见，最大停滞1帧、累计行程80.595、整段最小地图边距2.452。20帧/80%更新与可见阈值及完整HUD每80ms上限保留。
- /tmp/kiva-minimap-regressions.log整体退出0，1280-final、DPR2、smoke、pointer-response与healthbars五stage均0；800单独退出0。所有HTTP/浏览器由各脚本清理，最终Owned Xvfb stopped与临时Xauthority清理已确认。

## 完成状态
全部追加验收已勾选；README、architecture、HUD及pointer规范已同步，任务JSON/JSONL路径、JS语法、文档链接与diff检查通过，最终90文件源码身份保持冻结构建一致。
未启动、关闭或重配用户开发服务；全部HUD与本次小地图导航仍未提交，等待后续提交指令再归档。
