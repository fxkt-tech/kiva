# 模拟层

## Scope / Trigger

`src/simulation/` 拥有权威运动和碰撞；变更位移、速度、占地或固定步时阅读本规范。
不依赖客户端、浏览器、窗口、相机、Transform 或具体关卡。

## Signatures

- `SimulationPlugin` 注册 60Hz `FixedUpdate`。
- `Body::new(position: Vec2, speed: f32, half_extent: f32)`；`position()`、`interpolated(alpha)` 只读。
- `MoveIntent::set(direction: Vec2)` / `stop()`。
- `CollisionWorld::new(half_size: f32, solids: Vec<SolidBox>)`；`is_clear(point, half_extent)`。
- `SolidBox::new(center: Vec2, half_size: Vec2)`。

## Contracts

Vec2 表示世界 `(x,z)`，速度单位 m/s。Body 字段私有；每固定步先存 previous 再进行扫掠与滑动。
MoveIntent 只传世界方向，长度不超过 1。模型和相机共用 previous / current 的显示插值。
碰撞把静态盒扩张人物半宽，最多处理四次接触；世界边界限制整个占地，滑动保留切向位移。

## Validation & Error Matrix

| 输入 | 行为 |
|---|---|
| 构造配置非有限 / 非正尺寸与速度 | assert，属于内容错误 |
| 运行时方向非有限 | 归零 |
| 方向长度大于 1 | 限制到 1 |
| 撞墙 / 到地图边缘 | 阻挡法线运动，保留可行切向运动 |

出生点必须由装配者保证可容纳整个占地；不支持把 Body 直接生成在实体内部。

## Good / Base / Bad Cases

Good：斜向扫掠很长距离仍停在墙前，继续沿墙滑行。
Base：1 秒直行或斜行都为 6m。
Bad：NaN 输入不移动，内容无效时构造失败。

## Tests Required / Quality Check

执行 `make check`。固定步测试直接推进实际 FixedUpdate；验证一秒位移、静止、无效输入、
长步防穿透、贴面离开 / 滑行、角落和地图边界，不能只复制实现公式。

## Wrong vs Correct

错误：在按键系统修改 Body / Transform 作为物理位置。
正确：提交 MoveIntent，由 FixedUpdate 唯一推进 Body，显示只读插值。
