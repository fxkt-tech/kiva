# 首次进入检查记录

实施基线：main／fe6042f，用户确认问题主要发生于首次加载后进入。

## 已建立的反馈循环

Web 实现者在生产代码改动前添加并运行实际桥回归：

```sh
node --test --test-name-pattern='first entry is visible|capture pending' scripts/input-unit.test.cjs
```

首次运行 exit1，0通过／2失败：加载中按钮预期可见但实际隐藏；捕获请求期间预期 disabled，但实际仍可点且无进行中状态。测试直接 VM 执行 startup.js／input.js，能锁住这两个实际 UI 缺口。
修复后同组及原输入回归40项通过；完整门槛在最终实现交接后执行。

## 首次点击时序观察与边界

研究者运行 `node /tmp/kiva-startup-evidence/baseline.cjs --assert`，从当前用户服务读取冷启动并记录ready／实际点击／pointerlock和长任务。headed环境发生 root document invalid 的捕获拒绝，不能据此断定用户的实际原因。
headless再次运行首击成功，但用户Trunk自动刷新已载入正在修改的UI，属于混合版本的中间集成观察，不能当作原始基线性能对比。具体数据与本机Trunk接口见 startup-evidence.md。
最终验收使用固定打包产物的隔离HTTPfixture，不通过用户服务HMR，且ready后立即点击，不沿用此前等待5秒的脚本。

## 可证伪假设

1. Startup后的ready早于必要场景／渲染准备：预测ready后仍存在准备工作，使用真正的渲染门槛后按钮不会在这些工作前放行。
2. 点击后鼠标捕获等待或拒绝：预测click已触发而pointerlockchange迟到／报错；捕获状态和事件序列能识别。
3. 初始布局或遮挡使点击未命中：预测实际click落点／目标不是按钮；固定产物下的真实命中与请求事件可排除。

当前已证明UI反馈缺口和源代码中的过早就绪时序；用户具体设备上的首次点击延迟尚不能只凭上述观察唯一归因。

## 实现与独立检查

新增专用WASM流加载器，首次按钮显示真实进度／阶段并保持disabled，鼠标捕获进行中即时反馈；Rust移除Startup末尾ready，改为实际主视图输出、可见网格／材质specialization和管线准备后、渲染提交／present之后，由主线程通知一次。
引擎6项聚焦回归和原生／WASM Clippy通过；独立make check通过217个Rust单元＋9个集成、82个Node（最后JS／模板局部修复后重跑Node），格式／原生Clippy／WASM check通过。未再修改Rust，沿用其最终WASM Clippy结果。
独立检查额外复现并修复：init拒绝后未取消WASM未读流；Trunk将事件对象简写`{wasm}`误替换为文件名；CORS隐藏编码头时错误信任长度。对应红／绿回归已加入。
最终固定Trunk产物 `/tmp/kiva-entry-loading-dist.BBkvqj` 构建通过，生成初始化脚本及JS／WASM预加载SRI校验通过；游戏资源／规则／相机／Cargo与基线未变。
最终浏览器流程验证另行补充，不能沿用用户开发服务HMR中的中间结果。

## 最终固定产物浏览器结果

`node /tmp/kiva-entry-loading-browser/qa.cjs` 最终运行exit0；成功场景无page／console错误，report.failure为空。使用真实206.8MiB打包WASM与独立HTTPfixture，ready后立即单击，没有额外等待5秒。

- 慢速已知总量：按钮可见且disabled，真实百分比／进度条／字节显示；下载中强制点击不申请捕获。
- 引擎实际完成主视图输出，记录61可见网格／84已准备管线后才最终ready；首击一次捕获，点击→确认45.8ms。
- Esc后恢复不重新下载；模拟一次捕获拒绝后可重试，再次真点击成功；迟到进度／错误不能降级ready。
- 完整页面再次加载首击58.5ms通过；该大文件实际重新传输，不能称为缓存命中。
- 无总量完整WASM冷加载：不显示百分比，显示已收数据／准备阶段；首击34.1ms通过。
- 故意HTTP503：显示加载失败与重载，入口disabled，无Started／ready。预期错误不计为成功场景错误。

`node /tmp/kiva-entry-loading-browser/cache-probe.cjs` 另行验证真实HTTP缓存：最终loader与原生最小WASM经两个页面加载，仅一次网络WASM请求，第二次transferSize=0，两次各一次原生流式实例化，准确8／8字节；exit0。
该缓存probe覆盖相同loader的实际缓存Response，不声称完整206.8MiB游戏必然被浏览器缓存。

首次fixture曾在实际已进入后因监听器顺序过早读取内部active失败；调整观测为实际pointerlock元素＋最终isActive后重跑全部通过，产品代码未因该观测问题改动。
完整报告、时序、脚本、截图保留 `/tmp/kiva-entry-loading-browser/QA.md`、report.json、cache-report.json。两个fixture和浏览器均在finally关闭，临时Trunk目录随后删除，用户服务未接管。
本次只确认入口反馈、真实渲染准备／提交门槛和点击流程；不宣称复现用户设备唯一原因或完成此前模型真实GPU外观验收。

## 当前开发服务的配置应用

最终检查本地 `dist/index.html`，现有用户Trunk服务生成的产物仍没有 `kivaLoader.load`，说明它仍使用启动时读取的旧模块模板；固定临时Trunk产物已验证新模板。
本次修改了 Trunk.toml，需要用户重启一次自己的开发服务再刷新，以应用真实下载进度加载器。遵守项目服务所有权，不自行停止或重启用户进程。
