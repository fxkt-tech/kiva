# Kiva · 试炼之境

基于 Bevy 0.20.1 的浏览器俯视角游戏基础框架：固定角度三维镜头、鼠标点地移动、
绕障寻路、追击普攻、攻击移动、角色选择、组合技能、六格装备和骨骼角色动作。
当前可选择德玛西亚皇子或寒冰射手，进行有双方基地、定时兵线和敌方英雄 AI 的单中路对局。
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

浏览器打开 <http://localhost:3221>，选择角色后进入对局。需支持 WebGPU 的新版 Chrome / Edge，
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
| 左键小地图 / 拖动视野框 | 定位或平移镜头，切换为自由镜头 |
| Esc | 释放鼠标并暂停 |
| ? / 操作指南按钮 | 展开操作说明 |

浏览器中进入对局后会捕获鼠标，显示限制在游戏区域内的光标。
光标到达边缘仍可滚屏，按 Esc 可释放鼠标并暂停，点击继续按钮重新进入。
如果浏览器拒绝捕获，游戏保持暂停并显示重试提示。普通地点技能必须在射程内；方向技能沿鼠标方向发射。
地点冲刺与皇子军旗支持超距点击，沿该方向取最大距离；普通移动仍受障碍和临时地形约束。
点击底部技能/装备按钮同样可以释放；单位、地点、方向目标进入瞄准，自身目标立即释放。
前摇取消不消耗能量和冷却，出手后移动不会重置冷却。
小地图固定展示完整地图，点击或拖动只移动镜头。暂停菜单的“重新选择角色”会重开整场对局，
重置双方英雄、基地、兵线和装备；普通继续保留本局状态。

## 画面与HUD

底部操作栏集中显示属性、角色标记、技能、生命/能量与六格装备，小地图固定在右下角，目标信息位于左上。
默认镜头更近，英雄显示约为原来的两倍；滚轮仍可拉远，Y、空格与自由平移保持原操作。
小地图矩形框逐帧跟随实际相机，大小随缩放和窗口尺寸变化。透视视野的地面轮廓为梯形，矩形框取四角的外接范围；高DPI只影响绘制分辨率。

## 对局规则

64×64的正方形地图只有一条中路，我方基地位于左下、敌方基地位于右上。
地图大小不限制镜头操作，边缘滚屏、方向键、缩放、Y和空格跟随均可继续使用。
开局立即开始首波，此后每60秒开始下一波。每方每0.5秒出1个兵，先3近战、后3远程；
一波在2.5秒内出齐，暂停冻结出兵进度。近战兵465生命、远程兵284生命。
首次开局或整局重开时镜头定位到自己的英雄，随后可自由移动。
小兵沿中路推进，优先攻击敌兵、再攻击附近英雄和基地，短程追击后会回线。

基地初始3000生命，只有普攻本体可以伤害基地；技能、装备主动和普攻附属效果不会作用于基地。
任一基地生命归零立即结束对局，同一步双方基地归零则平局。结算后可重开同角色或重新选角。
英雄死亡约3秒后复活，己方基地附近脱战2秒后，每0.5秒恢复最大生命和最大法力的各10%。
敌方英雄使用实际装配、法力、冷却和控制规则，能够推线、施法、有限追击、撤退回补和守家。

目前为本地单人对局。单位没有硬碰撞，小兵普通行走时轻量避让；
已支持团队视野，敌军只有在友方英雄、小兵、基地或侦察视野内才显示和允许锁定；方向技能仍可盲打。
尚未加入防御塔、经济、等级、野区或联网。

## 基础属性

底部角色卡片左侧以紧凑布局显示七项当前生效的属性，文案依次为：攻击、法强、护甲、魔抗、移速、攻速、攻距。
移速和攻距显示端游距离数值（100距离单位对应1世界单位），面板不附单位；攻速为每秒次数。
显示值由Rust从模拟最终结果统一换算，装备和状态效果共用带来源的属性修正接口。

普攻造成物理伤害；技能伤害及法强加成由实际定义决定。护甲抵御物理伤害，魔抗抵御魔法伤害，
100 对应抗性将伤害减半，负抗性增加受到的对应伤害。真实伤害忽略两种抗性。
角色有各自基础属性，初始为空装备；具体值可在选角预览和HUD查看。
普攻具有前摇和后摇，出手后进入收招；移动、停止、换目标或施法可以取消后摇，已发射攻击继续生效。
改变攻速从下一次新普攻生效，取消后摇不会刷新已提交的普攻间隔。

双方英雄采用act分支的骨骼火柴人样式，具有关节和头眼；皇子持长枪，寒冰持弓，动作与武器同步。
待机、移动、普攻前后摇、施法、引导、冲刺及受控姿态跟随当前模拟，暂停冻结已显示动作。
动作使用内嵌CC0资源，不需要额外下载；来源、许可及重建方式见[动画资源](assets/animations/README.md)。

## 角色与技能

英雄固定一级、初始无装备，四技能均已开放并使用一级技能配置；不包含经验或加点。
皇子基础640生命、300法力、64攻击、36护甲、32魔抗、340移速、175攻距；
寒冰基础610生命、280法力、59攻击、26护甲、33魔抗、325移速、600攻距，双方基础攻速均为0.658。
皇子已学军旗被动会额外提供20%攻速，进入军旗光环再增加20%；选角预览与局内实际值一致。

| 角色 | 被动 | Q | W | E | R |
| --- | --- | --- | --- | --- | --- |
| 皇子 | 战争律动：按目标当前生命附伤，同目标独立间隔 | 巨龙撞击：线伤害、减甲，连到自己的军旗后突进击飞 | 黄金圣盾：护盾并减速附近敌人 | 德邦军旗：落点伤害、英雄攻速光环与视野 | 天崩地裂：跃击敌英雄并生成临时围墙，再按 R 可拆墙 |
| 寒冰 | 冰霜射击：普攻与伤害技能减速 | 射手的专注：先普攻叠满4层，激活后强化攻速与五箭普攻 | 万箭齐发：扇形箭，同敌单次伤害 | 鹰击长空：路径和终点侦察，最多2充能 | 魔法水晶箭：穿过小兵，击中敌英雄后范围伤害和距离眩晕 |

护盾吸收抗性结算后的伤害，临时墙真实阻挡移动。敌方英雄与玩家共用技能、资源、视野和冷却规则。
技能按钮显示当前层数、充能或重施状态；鼠标点击按钮与键盘操作共用同一施法入口。
指向敌人的技能（含装备主动）超距会先追近后释放，途中不扣费或启冷却；目标失效或新移动/攻击/停止会取消待放。
方向技能直接沿指定方向释放，插旗和冲刺在超距时仍按最大射程处理，不因鼠标点得远而走近。
数值来源与本项目的几何标定见[本轮数据说明](docs/champions.md)。

## 装备

对局中可直接装备/卸下9件示例装备，最多六件，允许重复。P打开装配后，装备进入首个空格；
满格时需先卸下一件。关闭面板仍保持暂停，点击继续按钮恢复。

装备支持七属性加成、普攻命中附伤/减速被动及普通施法/冲刺/引导主动。
主动有独立冷却，不占用角色四技能。卸装取消该实例未提交的施法和后续引导，已经发出的效果继续保留。
死亡保留装备及属性，装备冷却继续推进，重生不刷新；暂停冻结。每次穿戴生成新实例且冷却为0。
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
`tests/champion_content.rs`、`champion_mechanics.rs`验证真实套件与通用机制；
`tests/combat_foundation.rs`、`match_mode.rs`、`minions.rs`、`hero_ai.rs`验证结构受效边界、
完整对局生命周期、波次、兵线与英雄决策；玩法测试同样通过公开接口运行真实战斗。

浏览器集成测试需要额外的测试工具，可使用现有开发服务，或设置 `KIVA_SITE_DIR` 测试独立构建：

```bash
npm install --prefix /tmp/kiva-browser-tests playwright@1.65.0 pngjs@7.0.0
/tmp/kiva-browser-tests/node_modules/.bin/playwright install chromium
NODE_PATH=/tmp/kiva-browser-tests/node_modules node scripts/browser-smoke.cjs
NODE_PATH=/tmp/kiva-browser-tests/node_modules node scripts/browser-startup.cjs
node --test scripts/startup-unit.test.cjs
```

冒烟测试覆盖真实 WebGPU 渲染、点地移动、HUD 隔离、镜头操作、
窗口尺寸变化、高 DPI、暂停和失焦。启动测试验证早于HUD初始化的WASM下载失败仍显示提示与重试，
成功后迟到错误不会覆盖已就绪状态；启动单元测试不需要GPU。
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
不同目标和执行方式的键鼠技能、无效目标保留瞄准，以及真正失焦后的暂停恢复。

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

小地图回归覆盖全图投影、矩形框逐帧更新、点击定位、偏移拖动、边界、缩放和暂停/重选清理：

```bash
node --test scripts/minimap-unit.test.cjs
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-minimap.cjs
```

`browser-smoke.cjs` 和 `browser-startup.cjs` 也支持 `KIVA_SITE_DIR`，自行创建并关闭临时静态测试入口。

装备回归验证三种窗口尺寸（含高DPI）的实际装卸、属性叠加、被动命中、键鼠主动、独立冷却、
冲刺/引导卸装、暂停和整局重开：

```bash
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-equipment.cjs
```

完整对局回归验证首波阵容、基地位置、相机、暂停、真实终局、重开与旧请求隔离：

```bash
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-lane-match.cjs
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_SITE_DIR=/tmp/kiva-browser-site node scripts/browser-reselection.cjs
```

交互测试可用浏览器帧时钟 fixture 适配软件 GPU，不提供修改 World 的接口；血条频率测试保留原生显示时钟。

## 构建部署

```bash
trunk build --release
python3 -m http.server 3221 --bind 127.0.0.1 --directory dist
```

`dist/` 是完整静态站点，不需要后端。部署到子目录时指定
`trunk build --release --public-url /kiva/`。
