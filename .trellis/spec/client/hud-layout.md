# 对局HUD布局与镜头取景

## 适用范围与事实所有者

适用于index.html、web/hud.css、style.css、match.css、hud.js，以及client/camera.rs与ui.rs。
用户参考docs/hud_lol_game.png，保留Kiva深青/暖金风格；只采用构图和比例，不引入训练侧栏或尚无的经济/经验系统。

- hud.css唯一拥有游戏内布局、技能/资源/装备/头像、浮层和单位标签样式；style.css拥有基础主题、菜单与选角/装备对话框；match.css拥有比赛事实、结果及兵种样式。
- HTML允许移动节点，稳定ID、data-ui/data-slot/data-equipment-slot及原暂停/捕获/会话协议不得改变；不能依赖transform缩放旧HUD来处理响应式。
- camera.rs拥有CameraRig取景参数和相机变换，ui.rs只读投影；CSS不裁切或移动游戏canvas，不另算一套世界拾取偏移。
- HUD/属性/资源/冷却仍由kiva-state约80ms更新，标签坐标只由逐帧kiva-labels更新；不能在UI重建战斗计时。

## 构图与响应式

底部卡片依次为七属性、角色标记/被动、四技能与下方资源、2行3列装备。七个中文短标签及无单位显示保留，常规属性区120px，小窗口可112px，始终不超过144px。
卡片约110px高、距底8px；小地图固定右下8px，尺寸由同一个--minimap-size控制，HUD中心/宽度同时为地图留空间。800×600仍保留全部必要区域，不再将地图抬高或隐藏。
目标在左上8px；比赛状态与操作集中右上，Active隐藏大品牌。没有新增金币、经验或召唤师槽。
帮助、技能/装备/被动说明、护盾、状态、暂停/选角/结果入口保持可达，新增UI节点必须继承data-ui隔离，不能穿透为世界指令。

## 取景与投影

CameraRig默认zoom=0.5、最近0.4、最远1.5，OFFSET/FOV/视角方向保留。DEFAULT/MIN/MAX及变换构造集中在camera模块，setup与update复用，避免首帧跳变。
键盘平移18*zoom，有效画布内边缘平移31.5*zoom（原1.75倍），按dt推进；方向合并/相反抵消/归一化、边界、滚轮、Y/Space、首次开局/重开居中、普通恢复保留自由中心/缩放的语义不变。
通过镜头获得约2倍英雄屏幕比例；模型、UnitPose、碰撞半径及战斗数值保持，拾取与血条继续取同一个真实显示根。
默认2世界单位参考体的垂直投影约占视口8–12%，用实际投影和实机截图验证，不能只看zoom数值。

HudView新增camera_footprint: null | [[x,z];4]，顺序为视口左上、右上、右下、左下。由实际Camera的逻辑viewport角点射线与y=0平面相交，包含viewport偏移/DPR契约；无有效结果返回null。
JS按MapDefinition将真实四角的轴对齐外接范围显示为矩形，裁到地图边界；透视地面轮廓本身仍为梯形，不能声称矩形内每个点都在屏幕内。不得改主镜头投影或使用固定大小假框。现有view_projection与viewport保留，世界/UI坐标仍走web.rs转换。
地图2D画布后备分辨率跟随实际CSS尺寸与DPR，不能继续假设180px，也不能把DPR再次乘到世界输入位置。

## 验证

- 原生camera测试覆盖初始/更新同变换、缩放边界、UI/暂停时滚轮拒绝、英雄投影比例、真实四角/缩放/宽高比/DPR/viewport偏移及非法投影；会话测试保留开局/重开/恢复。
- browser-stats覆盖1920×1080、1280×800、800×600、1000×720 DPR2：实际七属性、技能/资源/装备布局、地图角落、目标左上、人体参考投影和四角roundtrip、map后备DPR及输入隔离。
- DOM-only检查用于快速布局迭代，不能当作WebGPU场景或真实游戏输入通过。
- healthbars保持原生RAF/时钟和20帧/80%阈值；可先用真实滚轮扩大视野容纳长路径，默认近景另由投影/截图验证。慢软件GPU下以只读标签边界驱动真实左右方向键往返，保持目标可见；验证累计实际平移及整段地图边距，不能写相机/World、改墙钟或降低跟随阈值。输入控制器在finally停止采样、等待排队输入并松开方向键。
- smoke、技能/装备/重选等按变更回归；临时构建和独占浏览器串行运行，不接管用户服务。完整质量门使用CARGO_BUILD_JOBS=2避免本机并发链接过量，仍执行全部原检查。

## 小地图导航契约

### 1. 范围与所有者
minimap.js拥有固定全图绘制、双向坐标转换与拖动手势；camera.rs唯一写CameraRig。地图不跟随主镜头裁切，单位仍来自经过视野过滤的只读快照。

### 2. 接口
`kiva-camera`每显示帧在相机变换/投影更新后发布JSON字符串`{center:[x,z],footprint:[[x,z];4]|null,enemy_hovered:boolean}`；enemy_hovered来自模型命中/当前生命视野复核，供pointer所有者显示剑形，地图只消费前两项。既有完整快照camera_center/camera_footprint及kiva-labels数组保持。
Web在root覆盖最新`data-camera-request` JSON `{player_id:string,center:[x,z]}`，通过`web::take_camera_request()`消费并移除；`data-minimap-dragging=true`由`web::minimap_dragging()`读取。不排队、不提供写World或英雄指令入口。

### 3. 坐标与行为
`#minimap-stage`与canvas同CSS尺寸，8 CSS px内边距将完整[-half_size,+half_size]映射至[8,width-8]×[8,height-8]；canvas `data-map-inset=8`为只读提示。DPR仅用于后备画布，不影响输入。
`#minimap-viewport`相对stage绘制实际四角外接矩形，逐帧相机事件只更新框；完整state更新地图/单位。空白左键点击把地面目标放到屏幕中心；框内按下不跳动，拖动保留CameraRig.center与按下世界点的偏移。松开保留最后有效目标待消费，继续移动不得继续拖动。
事件位置只取kivaPointer已校准CSS坐标，不积分raw movement。真实可信鼠标、Active、未暂停、实际且确认的canvas捕获、document焦点全部满足才接受手势。minimap以setUiCapture启停手势，指针所有者在每次按下快照记录capturedByUi，controls消费时跳过该次已处理按下；同帧释放后仍保留旧事件归属。

### 4. 门控与错误矩阵
- Rust每帧消费请求；严格字段、当前player_id、未暂停且有限坐标才接受。拒绝请求不会留到下局/恢复时重放。
- 合法请求统一clamp到既有half_size-5边界，退出Y锁定、保持zoom；拖动hold期间不执行跟随、方向键/边缘平移或滚轮，结束后恢复原键位语义。
- 暂停、失锁、blur、pointercancel、buttons丢失、resize及玩家/会话变化清拖动和旧请求。document级pointerup覆盖移出地图后的释放，明确chord左键释放同样保留末目标；普通移动发现buttons丢失才丢弃。
- null/非法footprint隐藏框，不画默认框；隐藏或零尺寸地图不进行坐标换算。小地图/矩形保持data-ui隔离，左/右键不改变移动/攻击/技能瞄准。

### 5. 示例
正常：Y跟随时点击地图空白切自由镜头，按住框内偏左位置拖动不跳到框中心。默认：未开始/暂停只有只读显示，不接收导航。错误：旧玩家请求、NaN、缺字段或额外字段直接消费拒绝。

### 6. 验证
native相机专项覆盖定位/边界/身份/非法输入/暂停/连续hold及保留zoom。Node加载真实minimap模块验证完整地图边界、共用投影、DPR、点击/偏移拖动/拒绝与清理；原指针/启动专项保持。
browser-minimap在1280×800、800×600、1000×720 DPR2用真实捕获鼠标验证主镜头实际投影、框内按下不跳动、点击/右键隔离、移动/松开、clamp、滚轮、独立逐帧框及暂停/重选和resize；使用帧时钟仅验证交互，性能仍由原生RAF血条专项负责。
手势的前后镜头断言同样读取独立kiva-camera逐帧通道（fixture中的__minimapCamera），不能拿约80ms完整HUD的camera_center当点击瞬间位置。方向键平移后完整HUD可能落后一帧，曾产生约0.3的假跳动；改取正确采样层次，保留原误差阈值和真实按住/等待/拖动流程。

### 7. 错误与正确
错误：根据鼠标在JS预测框位置、直接改CameraRig/英雄，或捕获模式读取原生clientX/clientY。正确：只发送带当前玩家的相机目标，框读取Rust真实逐帧结果；UI和世界继续共享唯一指针坐标所有者。
