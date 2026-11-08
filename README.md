# Kiva · 俯视探索 Demo

Bevy 0.20.1 / Rust 单 crate 游戏，通过 Trunk / WASM / WebGPU 在浏览器运行，也支持原生桌面。
当前是最小操作原型：俯视透视镜头、WASD 移动、鼠标水平转向，以及带碰撞的箱子、石柱和石块庭院。
旧的 MOBA 游戏完整保存在 `lol` 分支，当前代码与素材已重新整理。

## 运行

需要 Rust 1.97.1 或更新版本：

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
make run
```

浏览器打开 <http://localhost:3221>，等待加载后点击进入。需要支持 WebGPU 的浏览器和 GPU 加速。
本地使用 localhost，发布使用 HTTPS。游戏运行不需要 Node.js。
自定义端口：`make run ARGS='--port 8080'`。开发服务由开发者自行启动与关闭。

原生桌面使用 `cargo run --locked`，需要可用的图形环境。Linux 构建依赖包括
`build-essential`、`pkg-config`、`libx11-dev` 和 `libxkbcommon-dev`。

## 操作

| 输入 | 行为 |
|---|---|
| 点击进入 / 点击游戏画面 | 捕获鼠标，开始或恢复操作 |
| W / S | 沿当前视角前进 / 后退 |
| A / D | 相对当前视角向左 / 向右横移 |
| 鼠标左右移动 | 同步旋转人物朝向和相机方位 |
| Esc / 切出窗口 | 释放鼠标，停止人物；重新点击继续 |

镜头保持固定俯视角，人物身体中心固定在屏幕横向中间、纵向约 65% 处。
鼠标上下移动不改变俯仰；斜向移动不会加速。50×50 米庭院内有 21 个障碍，支持滑墙和边界碰撞。
角色和环境由程序生成，不依赖外部美术资源。当前尚未加入战斗、技能或联网。

## 构建与检查

```sh
make check
cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings
trunk build --locked --release
```

`make check` 包含格式、原生 Clippy、Rust 行为测试、Node 输入生命周期测试和 WASM 编译检查，
测试需要 Node.js 22 或更新版本。普通测试覆盖控制方向、碰撞、步进、相机构图和浏览器输入桥。
它们不替代真实浏览器的手感和性能验收。

生产产物在 `dist/`，将该目录部署到 HTTPS 静态站点即可；反向代理需正确返回 `.wasm` 的
`application/wasm` 类型。部署到子路径时使用 `trunk build --locked --release --public-url /子路径/`。

## 代码结构

- `simulation/`：固定 60Hz 的权威运动与静态碰撞，无窗口可测试。
- `gameplay/`：关卡描述和玩家装配，场景与碰撞共用障碍数据。
- `client/`：设备输入、视角、插值、程序化模型和相机。
- `web/`：加载入口和浏览器鼠标捕获，游戏规则留在 Rust。

详细边界和执行顺序见 [架构文档](docs/architecture.md)，开发规范入口见
[.trellis/spec/index.md](.trellis/spec/index.md)。
