# 实现边界

单crate保持client→gameplay→simulation。simulation只拥有Body位置和固定60Hz运动/静态碰撞，gameplay装配一名Player和Level，client拥有按键/鼠标、ViewYaw、相机与程序化表现。旧活跃规范在落地后替换为Demo事实，Trellis运行器和历史保留。

## 共享接口

`simulation::{SimulationPlugin, Body, MoveIntent, CollisionWorld, SolidBox}`。
Body::new(position:Vec2,speed:f32,half_extent:f32)构造可信配置；position()/interpolated(alpha)只读。MoveIntent::set(direction:Vec2)/stop()提交世界平面方向，模拟归一化并按秒推进。CollisionWorld::new(half_size:f32,solids:Vec<SolidBox>)，SolidBox::new(center:Vec2,half_size:Vec2)，is_clear(point,half_extent)只读校验。人物占地保守方形，扫掠扩张AABB并投影剩余位移滑动，不加物理库。

`gameplay::{Player, Level, Prop, PropKind, setup}`：Player为marker；Level{half_size:f32,props:Vec<Prop>}；Prop{center:Vec2,size:Vec3,kind:PropKind}；PropKind为Crate/Pillar/Stone。gameplay::setup(&mut World)由同一props生成碰撞并创建Player+Body+MoveIntent。出生Vec2::ZERO，速度6，半宽0.35。关卡生成确定性，不依赖窗口/材质/Transform。

`client::controls::ViewYaw`资源提供yaw()->f32；鼠标右移让Bevy Y旋角减小，forward=Quat(Y,yaw)*NEG_Z；键盘平面轴x=D-A、y=W-S按该基向量转换。Web只消费input.js的CSS校准水平位移及捕获期间WASD状态；Native只消费AccumulatedMouseMotion，不重复累计。相机与模型共享同一ViewYaw。

`client::scene::{setup(&mut World),sync_player}`：setup建立地面/光/props并给已创建Player附Transform/Visibility与纯几何子网格。sync_player读取Body插值位置和ViewYaw，唯一写玩家显示根，不写模拟位置。

`client::camera::{setup,follow}`：透视FOV50°，相机=body_center-forward*12+Y*18，look_at=body_center+forward*4，body_center为显示根+(0,0.9,0)。由此身体中心投影约65%屏幕高度，不靠HUD/模型挪动补偿。follow在scene同步后执行，鼠标角度不平滑延迟。

Web保留原通用startup.js早期加载状态观察；新input.js在WASM前安装只读输入桥，DOM就绪后连接进入按钮。window.kivaInput.isActive()/takeLook()/keys()/ready()供wasm-bindgen使用；仅confirmed capture+focus启用，失锁清累计，启动失败有重试。相对位移按正常未捕获CSS运动学习单位，尤其系统150%缩放不盲目除DPR。

## 分工

主会话先清理旧运行时，然后负责模拟/碰撞、控制/相机、Web桥/入口、测试、规范和整合。实现子代理拥有gameplay/mod.rs与client/scene.rs的关卡/几何表现，可添加对应纯数据回归，不启动服务/浏览器或构建。共享接口按上文固定；不引入框架、物理库或外部美术依赖。
