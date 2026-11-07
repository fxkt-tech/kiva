# Kiva · 试炼之境

基于 Bevy 0.20.1 的浏览器俯视角游戏基础框架：固定角度三维镜头、鼠标点地移动、
绕障寻路、追击普攻、攻击移动、角色选择、组合技能和六格装备。
通过 Trunk / WASM / WebGPU 运行，支持原生桌面调试。

游戏模拟独立于输入、相机、模型和浏览器。模块职责、执行顺序、扩展约定见
[架构文档](docs/architecture.md)。旧长剑训练原型保留在 `act` 分支。

## 启动

需要 Rust 1.97.1 或更新版本。安装工具后，在仓库根目录运行：

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
make run
```

浏览器打开 <http://localhost:3221>，选择角色后点击“进入试炼”。需支持 WebGPU 的新版 Chrome / Edge，
开启 GPU 加速。本地使用 localhost，部署使用 HTTPS。不需要 Node.js 运行游戏。

开发服务监听代码改动并刷新页面。自定义 Trunk 参数可以使用：

```bash
make run ARGS='--port 8080'
```

WSL 中把仓库放在 Linux 文件系统，安装 Linux 版 Rust。原生工具缺失时先安装：

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libx11-dev libxkbcommon-dev
```

使用 Windows 浏览器访问 localhost 即可，网页渲染不依赖 WSLg。`cargo run` 可调试原生版，
需要可用的桌面图形环境；原生版提供基本文字 HUD。

## 操作

| 操作 | 行为 |
| --- | --- |
| 右键地面 | 移动到指定位置，自动绕过静态障碍 |
| 右键敌人 | 追击并持续普攻 |
| 左键单位 | 选择单位并查看信息 |
| A → 左键 | 攻击移动，途中获取附近敌人 |
| S | 停止并取消当前可中断动作 |
| Q / W / E / R | 选择或释放当前角色对应技能，目标方式见按钮说明 |
| 1–6 / 点击装备格 | 选择或释放该格装备主动，自身目标直接释放 |
| P / 暂停菜单“调整装备” | 打开装配界面，暂停并释放鼠标 |
| Y | 切换自由镜头和跟随镜头 |
| 空格 | 按住时镜头跟随角色；松开保留位置 |
| 鼠标靠近画布边缘 / 方向键 | 自由模式下平移镜头 |
| 滚轮 | 有限范围内缩放 |
| Esc | 释放鼠标并暂停 |
| ? / 操作指南按钮 | 展开操作说明 |

浏览器中点击“进入试炼”后会捕获鼠标，显示限制在游戏区域内的光标。
光标到达边缘仍可滚屏，按 Esc 可释放鼠标并暂停，点击“继续试炼”重新进入。
如果浏览器拒绝捕获，游戏保持暂停并显示重试提示。普通地点技能必须在射程内；方向技能沿鼠标方向发射。
流风步等地点冲刺在点击超出射程时沿该方向冲刺到最大距离，范围内则到点击点；障碍物会提前截断位移。
点击底部技能/装备按钮同样可以释放；单位、地点、方向目标进入瞄准，自身目标立即释放。
前摇取消不消耗能量和冷却，出手后移动不会重置冷却。
训练目标死亡后约 3 秒重生。地图预览只读，不下达移动指令。
暂停菜单的“重新选择角色”会重开整场试炼，重置玩家、NPC和装备；普通继续保留本局状态。

首版为本地单人测试场景，包含固定靶和巡逻靶。单位暂不互相阻挡；
尚未加入经济、兵线、战争迷雾或联网。

## 基础属性

底部角色卡片左侧以紧凑布局显示七项当前生效的属性，文案依次为：攻击、法强、护甲、魔抗、移速、攻速、攻距。
移动速度以世界单位/秒表示，攻击距离表示双方碰撞边缘之间的距离，攻击速度以次/秒表示。
显示值直接来自模拟结果，装备和状态效果共用带来源的属性修正接口。

普攻造成物理伤害；技能伤害及法强加成由实际定义决定。护甲抵御物理伤害，魔抗抵御魔法伤害，
100 对应抗性将伤害减半，负抗性增加受到的对应伤害。真实伤害忽略两种抗性。
角色有各自基础属性，初始为空装备；具体值可在选角预览和HUD查看。
普攻具有前摇和后摇，出手后进入收招；移动、停止、换目标或施法可以取消后摇，已发射攻击继续生效。
改变攻速从下一次新普攻生效，取消后摇不会刷新已提交的普攻间隔。

## 装备

训练场可直接装备/卸下9件示例装备，最多六件，允许重复。P打开装配后，装备进入首个空格；
满格时需先卸下一件。关闭面板仍保持暂停，点击“继续试炼”恢复。

装备支持七属性加成、普攻命中附伤/减速被动及普通施法/冲刺/引导主动。
主动有独立冷却，不占用角色四技能。卸装取消该实例未提交的施法和后续引导，已经发出的效果继续保留。
死亡保留装备及属性，装备冷却继续推进，重生不刷新；暂停冻结。训练装配每次穿戴生成新实例且冷却为0。
金币、商店、合成、唯一装备和共享冷却尚未实现。

## 检查

```bash
rustup component add rustfmt clippy
make check
```

`make check` 依次运行格式检查、Clippy、Rust 测试和 WASM 类型检查，GitHub Actions 使用相同入口。
也可单独执行 `make fmt`、`make lint`、`make test`、`make wasm`。

`tests/simulation.rs` 使用相同的公开指令和属性变更接口，验证来源修正、伤害类型与抗性、
攻击时序、寻路、施法提交、冷却、中断、死亡重生等规则，不创建窗口、相机或 GPU。

浏览器集成测试需要额外的测试工具，可使用现有开发服务，或设置 `KIVA_SITE_DIR` 测试独立构建：

```bash
npm install --prefix /tmp/kiva-browser-tests playwright@1.65.0 pngjs@7.0.0
/tmp/kiva-browser-tests/node_modules/.bin/playwright install chromium
NODE_PATH=/tmp/kiva-browser-tests/node_modules node scripts/browser-smoke.cjs
NODE_PATH=/tmp/kiva-browser-tests/node_modules node scripts/browser-startup.cjs
```

冒烟测试覆盖真实 WebGPU 渲染、点地移动、普攻、示例技能、HUD 隔离、镜头操作、
窗口尺寸变化、高 DPI、暂停和失焦。启动测试验证 WASM 下载失败提示和重试。
截图默认保存到 `/tmp/kiva-browser-tests/artifacts`，可通过 `KIVA_SCREENSHOT_DIR` 覆盖。

可选环境变量：`KIVA_TEST_URL` 指定服务地址，`KIVA_BROWSER_HEADED=1` 显示窗口，
`KIVA_BROWSER_CHANNEL=chrome` 使用本机 Chrome。
没有硬件 GPU 的 Linux 自动化环境可设置 `KIVA_BROWSER_SOFTWARE=1` 使用 SwiftShader / Vulkan；
Windows 正常游玩不需要这些参数。

Linux 截图验收使用 `KIVA_BROWSER_HEADED=1`，可在桌面或 `xvfb-run -a` 下运行。
某些无头 Chromium 会正常运行 WebGPU 模拟，却把画布截成黑色，相关环境说明见
[WebGPU 浏览器测试记录](https://github.com/pmndrs/react-three-examples/blob/main/research/webgpu-ci-github.md)。
如果浏览器缺少系统运行库，可运行 Playwright 的 `install-deps chromium` 安装依赖。

单独验证鼠标点击技能的完整流程，不需要启动开发服务：

```bash
trunk build --dist /tmp/kiva-browser-site
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-skills.cjs
```

此测试自行创建临时静态文件测试入口并在结束时关闭，覆盖跨帧按下按钮、
Q/W/E 选目标释放、R 直接释放以及真正失焦后的暂停恢复。

血条跟随的浏览器回归测试使用同一份构建，无需启动开发服务：

```bash
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-healthbars.cjs
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-cursor.cjs
```

它按浏览器显示帧采样实际血条 DOM 坐标，验证人物移动、镜头移动时的更新频率，
同时检查完整 HUD 状态仍按较低频率发布。
光标测试覆盖捕获、越界移动、边缘滚屏与回移停止、Esc 释放与重新进入、
技能按钮、点地移动、高 DPI 以及捕获失败提示。

属性面板回归使用同一份构建，验证七项显示值与模拟一致、大小窗口和高 DPI 布局、
捕获鼠标下属性区的输入隔离，以及暂停和恢复：

```bash
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-stats.cjs
```

`browser-smoke.cjs` 和 `browser-startup.cjs` 也支持 `KIVA_SITE_DIR`，自行创建并关闭临时静态测试入口。

装备回归验证三种窗口尺寸（含高DPI）的实际装卸、属性叠加、被动命中、键鼠主动、独立冷却、
冲刺/引导卸装、暂停和整局重开：

```bash
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-equipment.cjs
```

## 构建部署

```bash
trunk build --release
python3 -m http.server 3221 --bind 127.0.0.1 --directory dist
```

`dist/` 是完整静态站点，不需要后端。部署到子目录时指定
`trunk build --release --public-url /kiva/`。
