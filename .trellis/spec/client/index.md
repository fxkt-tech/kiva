# 客户端

## Scope / Trigger

`src/client/`、`web/`、`index.html` 拥有输入、程序化表现和镜头，修改控制 / 构图 / Web 启动时阅读。

## Signatures

- `controls::ViewYaw::yaw()` 供 scene 和 camera 读取；鼠标样本只能由 controls 消费一次。
- `scene::sync_player` → `camera::follow` 在 Update 链式执行。
- Web 桥 `window.kivaInput.isActive(): boolean` / `takeLook(): number` / `keys(): number` / `ready(): void`。
- wasm-bindgen 绑定在 `client/web.rs`，原生不需要 JS；加载观察器为 `window.kivaStartup.subscribe(callback)`。

## Contracts

输入在 `BeforeFixedMainLoop` 采样，W/S 沿视角前后、A/D 横移；斜向归一化。
鼠标右移令 yaw 减小；0.0025 rad/CSSpx，不乘 dt，不重复除 DPR，忽略垂直移动。
scene 根据 Body.previous/current 和 fixed overstep 插值，camera 读取同一显示位置，朝向不延迟平滑。
相机：FOV 50°，身体中心为根位置 + Y*0.9，相机在后方 12 / 上方 18，望向前方 4；投影约 65% 屏高。
网格 / 材质复用，无外部资源；移动权威在模拟。

Web 两个 head 脚本先于 Trunk 模块同步安装，DOM 就绪后才绑定元素。
捕获必须收到 pointerlockchange 且文档有焦点，requestPointerLock Promise 完成不算确认。
失锁清水平累计；失焦失效整个请求，重新点击才能恢复。takeLook 返回并清空，合并事件和父事件不能重复计数。
Web keys() 位掩码为 W=1 / S=2 / A=4 / D=8，仅捕获期间可信非重复 keydown 建立按住状态；
失锁清零，keyup 释放。原生消费 WindowFocused 的中间失焦事件，避免同帧失焦恢复遗漏清理。
自由鼠标用 clientX / movementX 校准 CSS 增益，丢弃边缘截断样本；DPR 改变清校准，不盲目用它换算。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 未就绪 / 未捕获 / 失焦 | 清按键状态并停止 MoveIntent，水平累计返回 0 |
| 捕获失败 | 显示重试入口，不激活控制 |
| 旧捕获请求在失焦后成功 | 立即释放，不恢复控制 |
| 已确认捕获后旧 Promise 报错 | 忽略 |
| 非有限 / 非可信 / 非鼠标样本 | 不产生转向 |
| WASM 失败 / 无 WebGPU | 加载失败提示与重载按钮；真正 ready 可覆盖早期失败 |

## Good / Base / Bad Cases

Good：边走边转向，人物及镜头读取相同 yaw / 显示位置，无跟随拖尾。
Base：点击进入、WASD 行走、Esc 释放、点击继续。
Bad：先 await 捕获再假定锁定、把 movementX 乘 dt、把垂直位移当俯仰。

## Tests Required / Quality Check

`make check`、WASM Clippy；入口改变再执行临时目录 Trunk 构建。
Rust 测试验证相对方向、斜向速度、分帧转角一致和真实透视投影；Node 直接执行桥脚本，验证加载重放、
捕获确认 / 失败 / 失焦、旧请求、一次消费、合并事件和缩放单位。真实浏览器手感及画面验证需单独报告。

## Wrong vs Correct

错误：独立平滑相机、用模型位置反写 Body，或在 JS 计算角色位移。
正确：JS 只管理浏览器输入边界，Rust 提交 MoveIntent，模拟推进，模型和相机共用显示位置。
