# 最终集成验收

状态：实现、独立质量检查与硬件浏览器验收通过。用户已追加“提交&推送”授权，按流程提交main并归档本次已验收任务，提交身份见task记录与工作日志。没有启动或关闭用户常驻服务。

## 自动检查

- 动画20项Python测试、完整44段portable verify／repackage通过；8670个未参与自适应选键的原生参考时刻、全63骨及每帧1116原手部权重点重新核对。真实均匀60Hz负例被拒绝，65个邻接f32切点保留。
- 音频10项Python测试、48原媒体完整离线重建通过；48个最终Ogg浏览器解码通过。
- 最终make check：297项Rust单测、9项地形、105项Node；格式、native all-target Clippy、WASM check通过。另跑WASM Clippy -D warnings通过。
- 最终临时Trunk构建通过，Cargo.lock不变。WASM 223,710,944B，SHA256 `5fde231b1cd2ebc980a9427c31b72cafde9d3281a826d6e4d778474277b87ce3`；构建后没有src/web运行时改动。动作嵌入包新增6,281,901B，随包永劫Ogg945,182B；prepared原生参考仅用于离线验证，不复制进网页。

机器记录：[质量门](qa/quality-gate.json)、[构建](qa/trunk-build.json)、[48音频解码](qa/audio-codec.json)。独立复核及实修根因见[check-final.md](check-final.md)。

## 硬件浏览器

使用自建临时HTTP fixture、独立临时Windows Edge配置和本机RTX 4070 Ti SUPER／WebGPU；浏览器与fixture测试结束后关闭，没有接管用户服务／浏览器。与Linux无头合成异常分开，实际模型画面来自Windows GPU。45个完成的检查场景，无意外页面／控制台错误：

- 查看页31项：根入口与隔离、三角色、五实际配置、各三段攻击与防御近景、蹲伏攻击／防御／行走防御、跳跃／闪避、四种永劫源scope对照、源／参考切换、部件显隐、暂停／正倒拖／播放、自由相机、复制、静态子路径。
- 游戏11项：单击只一段；续段宽限过期后真实连续点击1→2→3且无第四段；空防御释放没有成功声；实际二跳且第三跳拒绝、真实落地；枪／弓Q；锤Tab三枪与链斧Tab三箭；真实格挡与弹反，各成功只发对应声音，Parry与riposte一一对应；首点进入／Esc；503失败恢复。
- 新遭遇3项：以实际敌人出招和接近距离为依据短按拆招成功；首点进入／Esc；503失败恢复。前一组合已通过的场景不依赖此新遭遇的状态。

真实trusted输入经现有桥与固定模拟产生事件；音频走实际AudioContext、playEffect与AudioBufferSource.start，52项运行时解码包含48新媒体、3旧兼容与BGM。输出静音以免打扰用户，未宣称人耳主观试听通过。四声部／优先级／延迟层生命周期另由实际JS的Node测试验证。

完整矩阵：[浏览器总表](qa/browser-summary.json)、[查看页](qa/viewer-gpu.json)、[游戏](qa/game-gpu.json)、[新遭遇拆招](qa/deflect-gpu.json)。开发测试中曾修正fixture对60Hz时间轴、模式过滤、续段宽限、真实技能范围和Boss接近命中时刻的假设；这些没有靠修改产品代码通过。各最终场景使用同一冻结WASM。

## 画面证据

[默认手部](qa/screenshots/idle-hands.png)、[第2段](qa/screenshots/attack-stage-2.png)、[妹妹鸟铳](qa/screenshots/sister-musket.png)、[弓源上身范围](qa/screenshots/bow-source-scope.png)、[蹲伏攻击](qa/screenshots/crouch-attack.png)、[实际弹反](qa/screenshots/actual-parry.png)、[实际拆招](qa/screenshots/actual-deflect.png)。已观察真实主体／武器／网格／姿态，ready和CPU断言不作为画面替代证据。

## 保留边界

E／Tab／滑铲保留原编排／明确回退；移动起跳首版统一使用inplace一／二跳＋真实物理。原反击clip适配弹反表现，但success-only原状态未证；原默认指层混合权重／顺序、完整控制器／Wwise总线与原音轨时间单位未恢复。源f2／官方评估器f1补丁差异如实保留。相关trace／manifest不会把这些适配称为完整原作复刻。
