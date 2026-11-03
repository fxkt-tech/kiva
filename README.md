# Kiva · Sword Training

Bevy 0.20.1 的第三人称 3D 近战原型：一个持剑主角、一个训练木人和一座练武场。在 WSL2 中开发，通过 Windows Chrome / Edge 的 WebGPU 运行；也支持原生桌面构建。

## 操作

点击 **Enter training** 进入操作，鼠标直接旋转视角，无需按住鼠标键。

| 操作 | 行为 |
| --- | --- |
| WASD / 方向键 | 相对相机方向移动 |
| 鼠标移动 | 旋转视角 |
| 滚轮 | 调整相机距离 |
| 空格 | 跳跃；空中再按一次可二段跳 |
| 左 Shift | 按移动方向闪避，无方向输入时向后闪避 |
| M | 开关音效 |
| 左键 | 挥剑；在收招窗口继续点击，衔接下一段 |
| 按住右键 | 正面格挡，降低移动速度 |
| T | 让木人原地挥击一次，用于验证格挡 |
| R | 重置双方位置、血量和战斗状态 |
| Esc | 释放鼠标并暂停训练，再次点击进入可继续 |

角色移动时朝移动方向转身，攻击起手时朝镜头水平方向出剑；攻击期间锁定朝向，格挡时持续朝镜头方向转身。右键菜单在游戏画布内禁用。

## 战斗规则

- 三段轻攻击：横斩、反斩、下劈，伤害依次为 **10 / 15 / 25**。
- 每段分为前摇、有效攻击时间和后摇；同一段对同一目标只结算一次伤害。
- 前摇结束后的点击可短暂缓存 **0.22 秒**，在收招窗口最多消费一段后续攻击；界面显示 **CHAIN NOW** 时可直接续招。前摇点击不排队，第三段结束后连招归零。
- 命中按前方扇形、距离与目标半径计算，和模型动画分开。跨过整个伤害窗口的慢帧也不会漏掉判定。
- 格挡覆盖正面 **120°**，减免 **80%** 伤害；侧后方攻击仍造成完整伤害。
- 格挡和攻击互斥，攻击收招前不能切换格挡。暂不包含精力、破防或精准弹反。
- 双方血量均为 100。血量归零倒下，约两秒后复位复活；木人不追击、不自动反击。
- 命中包含闪白、后仰、击退、伤害数字、短暂停顿、火花、剑轨与轻微镜头震动。
- Esc / 失焦暂停模拟，释放格挡并清理按键状态，避免重新进入后持续误操作。

## 跳跃、闪避与音效

- 空格从地面起跳，空中可再按一次进行二段跳，每段向上速度重置为 6.4。最多两段，落地或重置后恢复次数；单段最高约 1.14 米，落地后恢复普攻。
- 左 Shift 按当前 WASD 方向闪避；没有方向输入时向后闪避。每次约 2.4 米、持续 0.42 秒，冷却 0.72 秒。
- 闪避中段（0.06～0.28 秒）可避开伤害。可从待机、格挡或攻击收招进入，前摇和出刀阶段仍需完成动作。
- 人物增加层叠衣袍、肩甲、束发、分段手臂与手腕关节；三段攻击包含平滑蓄力、腰身转动、手腕跟随和下劈重心变化。
- 浏览器 WebAudio 实时合成三种挥剑、命中、金属格挡、抬剑、脚步、跳跃、落地、闪避、倒地、复活等音效，无需下载音频素材。
- 点击 Enter training 时解锁音频，Esc / 失焦时暂停声音。按 M 或点击右上角按钮静音，设置保存在当前浏览器。
- 音效在网页端输出；原生窗口保留画面和战斗逻辑，方便本地调试。

## WSL 环境

把项目放在 WSL 的 Linux 文件系统中。使用 Linux 版 Rust，最低版本为 1.97.1：

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source "$HOME/.cargo/env"
rustup update stable
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

如果已经运行开发服务，保持现有服务即可，Trunk 会监听改动并刷新页面。需要手动启动时：

```bash
./scripts/web-dev.sh
```

在 Windows 浏览器打开 <http://localhost:8080>。网页端需要 WebGPU 和可用的 GPU 适配器；本地使用 localhost，正式部署使用 HTTPS。当前构建不自动回退到 WebGL2。WSL 不负责网页渲染，不需要 WSLg 或 Node.js 运行游戏。

## 构建与预览

```bash
trunk build --release
python3 -m http.server 8080 --bind 127.0.0.1 --directory dist
```

`dist/` 是完整静态网站，不需要后端。子目录部署可使用 `trunk build --release --public-url /kiva/`。首次开发构建包含调试信息，体积较大；发布时使用 release 构建。

## 代码结构

```text
src/lib.rs               引擎、窗口和网页适配
src/game/mod.rs          系统注册与执行顺序
src/game/input.rs        输入、鼠标捕获状态与暂停
src/game/camera.rs       第三人称相机、自由视角与震动
src/game/arena.rs        地面、灯光与练武场
src/game/player.rs       主角和木人的简易人形模型、关节与武器
src/game/combat/core.rs  不依赖渲染的攻击时序、连招、范围和格挡规则
src/game/combat/mod.rs   双方状态、移动、命中、伤害、击退与复活
src/game/animation.rs    程序化挥剑、格挡、行走、后仰与倒地动作
src/game/effects.rs      剑轨、火花、闪白、伤害数字与打击反馈
src/game/hud.rs          血条、动作状态、命中与格挡提示
src/game/sound.rs        动作音效事件
web/audio.js             浏览器合成音效、音量与暂停
assets/                 后续模型、贴图、音频等资源
index.html              网页入口、加载提示与鼠标捕获交互
```

攻击数值集中在 `combat/core.rs` 的 `LIGHT_ATTACKS`、`TRAINING_ATTACK` 和 `COMBO_WINDOW`。当前人物由基础几何体组成，通过关节变换驱动动作，后续可替换为正式模型和骨骼动画。

## 检查

```bash
cargo fmt --check
cargo check --locked --target wasm32-unknown-unknown
cargo check --locked
./scripts/test-combat.sh
```

战斗规则测试直接使用 Rust 测试框架运行，不初始化窗口或 GPU。

`cargo run` 可打开桌面版，鼠标左键进入操作、Esc 释放鼠标。原生运行需要系统支持 X11 与图形渲染；WSL 中通常通过 WSLg。

浏览器集成测试可选，需要另外安装测试工具；它连接已有服务，不启动服务：

```bash
npm install --prefix /tmp/kiva-browser-tests playwright pngjs
/tmp/kiva-browser-tests/node_modules/.bin/playwright install chromium
NODE_PATH=/tmp/kiva-browser-tests/node_modules KIVA_BROWSER_SOFTWARE=1 node scripts/browser-smoke.cjs
```

该测试验证真实 WebGPU 渲染、鼠标捕获、单次伤害、完整连招、倒地复活、格挡减伤、跳跃落地、闪避、音频解锁、静音和暂停。`KIVA_BROWSER_SOFTWARE=1` 使用 Linux 无头 Chromium 的 Vulkan / SwiftShader 测试配置，用于没有硬件 GPU 的自动化环境；Windows 正常游玩不需要这些参数。可通过 `KIVA_TEST_URL` 指定已有服务地址。
