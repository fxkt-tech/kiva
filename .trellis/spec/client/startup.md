# 首次加载、渲染就绪与进入

## Scope / Trigger

修改 Trunk.toml、web/{startup,loader,input}.js、入口 DOM／样式、client/{loading,web,mod}.rs 时阅读。
加载只决定客户端入口准入；模拟仍由既有输入捕获状态门控，不新增玩法时钟或改动资源。

## Signatures

- `kivaLoader.load(init,wasmUrl)`：只 fetch 指定 WASM，将计量后的 Response 交给 `init({module_or_path:response})`，返回真实实例结果并传播失败。
- `kivaStartup.progress(phase,loaded,total)`：phase=`download|initialize|render`；真实字节／计数，0/0 表示不可计量。Rust 桥为 `(&str,u32,u32)`。
- `kivaStartup.fail(reason)`：失败原因限制长度并以纯文字呈现。
- `kivaStartup.subscribe(callback)`：立即重放 `(state,detail)`，state=`loading|failed|ready`，detail 含 phase／loaded／total／可选reason；`__kivaStartup` 保留原字符串兼容。
- `kivaInput.ready()` 仍为最终放行入口并触发 kiva-ready，只有 Rust 首次必要渲染就绪后调用；下载或 TrunkApplicationStarted 不能代替。
- `loading::StartupPlugin` 共享 StartupSignal，Startup 使用 begin／scene_ready；主世界 First 的独占系统通知 JS，渲染世界在 RenderSystems::Render 后、Cleanup 前观察。

## Contracts

Trunk 0.22.1 通过字面量 pattern_script 的 `{base}{js}`／`{base}{wasm}` 导入实际哈希产物，调用专用 loader；成功后保留 wasmBindings 和 TrunkApplicationStarted 的 `{detail:{wasm}}`。
不得把内置占位符移进外部参数展开文件，或用 worker-only data-loader-shim 替代主模块加载。Trunk会在任意位置替换字面量 `{wasm}`，事件对象简写必须保留为 `{ wasm }` 或显式键值，不能与文件占位符碰撞；回归执行替换后的实际初始化脚本。自定义加载失败不能继续派发 Started。
专用 loader 只包装自身 WASM Response，保留 streaming／背压、HTTP状态、MIME和缓存路径，不全局修改 fetch。
已知且与解码流可比的 Content-Length 才能作为分母；Content-Encoding、缺失／非法长度或累计超过声称总量时使用未知总量，显示阶段／已收字节。CORS可暴露长度而隐藏编码头，因此cors响应缺少编码信息也按未知总量处理。EOF 不伪造计数；已知长度不足、reader／fetch／init失败必须终止并报告错误，提前失败取消未读流，取消失败不覆盖原错误。

阶段单向推进为下载→初始化→画面准备；最终 ready 忽略迟到进度／错误。failed 保留重试，真实 ready 可以恢复。订阅前／DOM前的状态可以重放，不能靠计时器模拟百分比。
按钮加载时可见、原生 disabled，测量下载时显示百分比及 progress，未知阶段显示忙碌状态；下载100%仍不可点击。敌人／外观选择继续可用。
捕获请求开始立即显示正在进入并禁重入；仅 pointerlockchange＋焦点确认成功，Promise完成不代替确认。拒绝／blur／Escape清理后可重试；恢复保留已完成ready，不重走下载或自动申请捕获。

Rust门槛同时要求目标GameCamera已提取、有真实输出附件写入、非空可见Mesh3d均具有材质／网格准备后的specialization、PipelineCache存在且已就绪，并位于真实渲染提交／present之后。
SpecializedMaterialPipelineCache为本机Bevy共享缓存，覆盖Standard与Forest；初始空队列／设备空提交／只装配实体均不是ready，隐藏妹妹／后续技能材质不成为首次可见场景门槛。
ShaderNotLoaded／ShaderImportNotYetAvailable按Bevy机制等待；处理shader／创建module错误报告失败。渲染错误包装器先通知Web，再委托原RenderErrorHandler保持AppExit／恢复策略。跨世界信号有界且最终通知一次，DOM桥仅主线程调用。
原生不依赖JS，不改变原生激活；无渲染测试上下文不能凭空放行，WASM缺渲染器可报告失败。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 下载100%但场景／管线未就绪 | 保持disabled并进入准备阶段 |
| 无总量／编码长度／不可信分母 | 不显示虚假百分比，保留已收数据与阶段 |
| 流截断／网络／初始化拒绝 | 取消未读流、报告失败／重载，原错误保留 |
| empty pipeline／无可见主视图／附件未写 | 继续准备，不能按帧数或等待时长放行 |
| genuine ready后迟到loader或错误 | 保持ready，按钮与捕获状态一致 |
| 捕获进行中／失败／失焦 | 进行中可见且禁重入，清理后可重试，无幽灵输入 |
| Esc后恢复 | 已ready直接可点击，无重复下载 |

## Good / Base / Bad Cases

Good：下载82%时按钮禁用，接着准备游戏／画面，必要渲染提交后才允许用户首击进入。
Base：缓存命中仍走同一数据／就绪链，Esc暂停后重新点击即可恢复。
Bad：Startup末尾直接ready，或100%计时条放行；读流出错悬挂、init失败仍派发Started、在工作线程访问DOM。

## Tests Required

实际桥测试覆盖加载可见／禁用、100%仍禁用、阶段／早期DOM／迟到状态、捕获进行中／拒绝／blur／恢复及原输入契约。
实际ReadableStream和最小WASM测试覆盖已知／未知／编码／错误长度、缓存式响应、原生instantiateStreaming、HTTP／读流／init失败及取消原错误保留。
Rust检查空渲染状态、目标视图与Standard／Forest实体准备、真实瞬态／终态shader状态、线程通知一次、错误策略委托和无GPU上下文。
执行make check、WASM Clippy及固定临时Trunk产物；浏览器在独立fixture验证冷加载禁用／真实进度、ready后立即首击、缓存／无长度／失败与Esc恢复，不用额外等待5秒掩盖初次时序。

## Wrong vs Correct

错误：把下载成功／Startup装配／空GPU队列当成“游戏已就绪”。
正确：真实流进度负责反馈，初始可见场景的准备与提交负责最终ready，用户点击负责申请鼠标捕获。
