# 小地图交互追加设计

## 行为缺口与边界
原minimap.js只绘制全图背景、中路、标记和透视四角多边形，无相机意图入口；全图轮廓不清楚、取景框不可操作。
修改仅在client/Web及其测试：不改主镜头投影、模型、simulation/gameplay或战斗指令。

## 唯一所有者
- web/minimap.js拥有完整地图到CSS坐标的双向转换、绘制和局部拖动状态；读取kivaPointer.position获取已校准CSS坐标，不另算movement或DPR输入。
- index.html/web/hud.css提供小地图stage和独立矩形overlay。地图覆盖[-half_size,+half_size]完整世界范围，固定绘制边界，中路/障碍/基地/可见单位共用转换。
- 现有透视镜头的地面视野为梯形；矩形采用真实camera_footprint四角的轴对齐外接范围，可在地图边缘裁切。保持真实footprint协议不变，不改相机投影来凑矩形。点击点对应相机地面目标（屏幕中心），拖动保留该目标和按下地图点的差，避免框内按下跳动。
- ui.rs每显示帧发布独立kiva-camera只读{center:[x,z],footprint:[[x,z];4]|null}；完整kiva-state既有camera_center/camera_footprint保持。地图底图/单位按完整快照，框用逐帧相机数据，不改kiva-labels数组协议。
- Web仅提交最新data-camera-request JSON {player_id:string,center:[x,z]}，每次新目标覆盖未消费目标；data-minimap-dragging=true表示鼠标左键拖动中。所有位置是世界x/z，player_id沿用字符串，不提供写World接口。
- web.rs消费/移除意图并桥接拖动状态；camera.rs复核当前Active玩家、未暂停、有限坐标、当前player_id，统一夹紧到既有地图边界，退出Y锁定但保持zoom。拖动中不执行边缘/方向键平移，Space/Y在拖动结束后按原路径使用。
- 边界审查发现拖出地图后的多键按下可穿透，且仅在pointer_over_ui读取当前拖动态不能覆盖同一Bevy帧内全部释放。因此最小扩展现有pointer按下快照为[x,y,capturedByUi 0/1]，由pointer.js记录事件时UI手势归属，minimap用setUiCapture启停；web.rs解码，controls在消费该次快照时跳过已被UI拥有的按下。结束手势不修改已排队标记，暂停/失锁清理沿用唯一指针所有者。

## 生命周期与输入
只有Active、真实且已确认Pointer Lock、未暂停、document有焦点且kivaPointer有效时接受原生可信鼠标输入；按下位置读取指针所有者，同一事件先更新pointer再处理小地图。
小地图及矩形全部data-ui，左键操作不穿透世界；右键不改走位/瞄准。地图空白点击立即定位并可继续按住拖动，框内按下保持位置、移动才平移；离开地图继续夹紧到地图坐标，document级松开结束。
松开、buttons丢失、pointercancel、blur、失锁、暂停、会话/玩家变化、resize均结束拖动并清旧请求；普通左键松开（包括chord释放）可保留最后有效目标等待Rust消费，暂停/会话边界必须丢弃。Rust每帧消费意图，拒绝态不能留至未来重放。偏移拖动必须先做未夹紧的CSS→world、加抓取offset，再夹最终目标，不能先夹鼠标点导致一侧边界不可达。

## 实施与验证
1. Rust代理只负责camera.rs/web.rs/ui.rs/mod.rs及native tests；Web代理只负责minimap.js/index.html/hud.css及Node专项，双方遵循本协议。
2. root负责真实浏览器专项与集成、文档；先看专项红灯再实现到绿灯。Node验证实际地图绘制/坐标、点击/偏移拖动、边界和清理；native验证门控/有限值/身份/边界/锁定与zoom。
3. CARGO_BUILD_JOBS=2 make check、标准WASM Clippy、Node指针/启动/小地图测试；冻结源码后重建临时站点，真实Pointer Lock下验证1280、800、DPR2点击/拖动、框逐帧、缩放、UI隔离及暂停恢复，并串行运行smoke、pointer-response和原生healthbars。
4. 独立检查、同步规范/README与verification；保持全部HUD改动未提交，不操作用户开发服务。
