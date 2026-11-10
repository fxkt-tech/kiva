# 固定产物验收记录

日期：2026-11-11。代码及软件GPU验证完成；用户已授权提交并推送main。真实硬件GPU验收仍待完成，任务保持in_progress。

## 质量门槛

- `make check`：格式、原生all-target Clippy、241项Rust单元＋9项集成、94项Node、WASM check全部通过。
- WASM Clippy `-D warnings`通过。独立审查见[independent-review.md](independent-review.md)，已修复子部件Visible绕过父节点隐藏，并用实际VisibilityPlugin传播测试锁定。
- 真实游戏animate适配与预览、三角色实际palette／左右手／脊柱、两近战挂接、妹妹枪口／握点共用计算通过；全部54片段有限，seek／倒拖／末帧和重复换装资产复用通过。
- 资产、Cargo.toml、Cargo.lock、simulation、gameplay和既有导入器没有差异；未启动／关闭用户开发服务。

## 固定Trunk产物

使用本机缓存wasm-bindgen 0.2.130加入当前命令PATH，Cargo及Trunk离线、locked构建，public-url=`./`，输出到唯一临时目录。初次Trunk离线工具搜索未找到缓存；显式加入同版本缓存PATH后成功，无依赖／锁文件变更。

- WASM：217,233,984字节，SHA256 `292dd3c454035191067977a0392d1870bfcd08814d307fcf8d149e2b6da02b91`。
- 检查复制的9个同步脚本、两份CSS、实际初始化模板及生成JS／WASM的SRI；模式脚本在游戏监听器前安装。
- 相同固定目录在独立临时HTTP fixture的根路径和`/kiva/`下运行，未借用开发服务热更新或做SPA回退。
- 构建元数据：`/tmp/kiva-viewer-browser/build.json`。验收后删除临时dist，保留报告／脚本／图片。

## 浏览器与画面

Chromium1254，最终完整检查使用独立认证Xvfb上的有界面浏览器、1500×980、DPR1。WebGPU为SwiftShader回退CPU适配器；宿主RTX可被nvidia-smi看到，不代表浏览器使用硬件GPU。

13条记录覆盖：专用页直接加载且不安装游戏桥、三角色切换保持帧／镜头、五种实际配置、攻击往返seek、隐藏部件后换装恢复、源片段隐藏武器／参考姿态、暂停转镜头／辅助标记／播放／复制、静态子路径、正常游戏首击进入及Esc、故意HTTP503后的重载入口。成功路径没有JS异常或console error。

最终截图确认三角色、四指定武器／敌人原刀、中性地面与阴影、来源对照和诊断标记可见。它证明此软件GPU环境下页面能显示和操作，不是原模型全部美术细节正确或真实硬件画面／性能通过。

原生180步连续源层重放：10次重置中位3.49ms、最大5.12ms。浏览器4次seek从Rust取命令至确认快照为0.7–2.9ms，中位2.05ms；这是本次样本，非任意输入的P95或硬件性能承诺。暂停相机原生60帧约2.61ms，动画／皮肤／bounds缓存回归通过。

## 无头截图问题定位

原无头fixture的UI与真实提交门槛正常，但中央区域只有背景色。正常DPR、正确canvas尺寸／可见性仍复现。临时GPUCanvasContext增加COPY_SRC后，在提交后读取同一输出surface，得到23,111种颜色的完整模型／武器／地面图；页面截图仍空白。切换隔离有界面浏览器后，同一产物正常显示；随后完整套件在没有GPU读回改动的有界面环境中通过，中央像素断言得到30,409种颜色。

因此未修改游戏渲染或模型偏移来规避测试环境。以后遇到“无头截图空白但渲染已提交”时，先核对有界面截图／GPU输出，不应仅靠ready判断画面通过，也不应直接认定角色没有绘制。

## 留存证据与未完成项

- 完整最终报告：`/tmp/kiva-viewer-browser/headed/report.json`；截图同目录，包含`02-initial.png`、五个`preset-*.png`、`04-source-comparison.png`、`06-diagnostics.png`。
- 实际GPU读回：`/tmp/kiva-viewer-browser/gpu-headless.png`；对应配置、读回与无头差异报告在同一临时目录。
- 再运行：用新的固定dist，执行 `KIVA_XVFB_SIZE=1800x1200x24 python3 /tmp/kiva-browser-tests/with-isolated-xvfb.py -- node /tmp/kiva-viewer-browser/qa.cjs <dist> --headed`。fixture自动关闭自身HTTP／浏览器／Xvfb。
- 真实硬件GPU最终验收未完成；先前角色资源自身的脸型／材质／布料和固定组装限制没有在本任务修复。
