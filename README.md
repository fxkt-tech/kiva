# Kiva · 俯视探索 Demo

Bevy 0.20.1 / Rust 单 crate 游戏，通过 Trunk / WASM / WebGPU 在浏览器运行，也支持原生桌面。
当前是带动作和双人格武器联动的庭院战斗原型：固定 16:9 俯视镜头、骨骼动画火柴人，以及可反复练习的敌人。
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
| Shift 点按 / 长按 | 闪避 / 衔接疾跑；松 Shift 继续跑，松开所有方向键停止 |
| Space / 鼠标右键 | 跳跃 / 格挡 |
| 1 / 2 | 切换锤子+狙击枪 / 斧子+火炮套装 |
| 滚轮 | 在同套装内切换哥哥近战 / 妹妹远程 |
| 左键 / Q | 普攻 / 切换锁定目标 |
| E / Tab | 起手技能 / 有效窗口内释放联动 |
| 火炮模式鼠标上下 | 调整地面落点距离，不改变镜头俯角 |
| Esc / 切出窗口 | 释放鼠标并暂停；重新点击继续 |

镜头保持固定俯视角，人物身体中心固定在屏幕横向中间、纵向 2/3 处。
鼠标上下移动不改变俯仰；斜向移动不会加速。50×50 米庭院内有 21 个障碍，支持滑墙和边界碰撞。
锤子击飞后 Tab 衔接狙击，火炮跃进途中 Tab 衔接旋斧。角色和环境几何由程序生成，
身体动画复用 `act` 分支随包内嵌的 CC0 骨骼资源，包含待机、走跑、跳跃、翻滚、格挡和战斗反馈动作。
模型与武器跟随骨骼，位移、跳跃高度和伤害仍由模拟决定。资源来源及许可证见
[动画说明](assets/animations/README.md)。

## 构建与检查

```sh
make check
cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings
trunk build --release
```

`make check` 包含格式、原生 Clippy、Rust 行为测试、Node 输入生命周期测试和 WASM 编译检查，
测试需要 Node.js 22 或更新版本。普通测试覆盖控制方向、碰撞、步进、相机构图、骨骼采样、身体动作、武器挂点和浏览器输入桥。
它们不替代真实浏览器的手感和性能验收。

生产产物在 `dist/`，将该目录部署到 HTTPS 静态站点即可；反向代理需正确返回 `.wasm` 的
`application/wasm` 类型。部署到子路径时使用 `trunk build --release --public-url /子路径/`。

## 代码结构

- `simulation/`：固定 60Hz 的权威动作、碰撞、伤害和联动，无窗口可测试。
- `gameplay/`：关卡描述和玩家 / 练习敌人装配，场景与碰撞共用障碍数据。
- `client/`：设备输入、视角、插值、骨骼姿态、程序化模型和相机。
- `web/`：加载入口和浏览器鼠标捕获，游戏规则留在 Rust。

详细边界和执行顺序见 [架构文档](docs/architecture.md)，开发规范入口见
[.trellis/spec/index.md](.trellis/spec/index.md)。
