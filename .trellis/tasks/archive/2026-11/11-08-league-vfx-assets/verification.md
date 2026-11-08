# 资源子任务验证记录

实现边界：`assets/vfx/`、`scripts/import-league-vfx.py`、`src/client/skill_assets/`。
未改公共mod/scene/ui、renderer或模拟，未运行Cargo/Trunk/GPU、未启停服务或操作git。

## 已完成

- 实际取得固定16.22原PNG及官方16.22.8348910 WAD区间内SCB/BIN。
- 研究完整解码33张候选PNG、解析9个SCB至EOF，检查索引/有限UV、面/顶点颜色与原/转换后边界。
- 最终生产选24张PNG、9个GLB；catalog/parameters另计，共35产物、1,729,291 bytes。参数依据来自5个固定BIN。
- `python3 -B scripts/import-league-vfx.py`：35本地产物大小/哈希通过，无网络或可选依赖。
- `python3 -B scripts/import-league-vfx.py --source-dir /tmp/kiva-vfx-research-xz0f3gqf`：全部原始缓存再次精确重建并校验35产物。
- 从空的`/tmp/kiva-vfx-clean-download-verification`开始，按实际Cloudflare回应修正运输层PNG处理后，完整`--download --cache`已成功；官方bundle decoded片段、SCB/BIN均通过严格SHA，PNG可见RGBA/alpha及所有产物SHA通过。
- 针对实际原版/Polish重编码PNG：规范化结果字节相同；改动一个非透明像素必须被拒绝；损坏PNG CRC必须被拒绝。
- Python AST、所有Rust新增文件单文件rustfmt、`git diff --check`通过。

## 新增Rust回归（root已执行通过）

1. 全部24 TextureKey/9 MeshKey有唯一同源打包目录项，PNG实际尺寸对应spec。
2. 六类黑底公共指示器必须Add且均为单帧；图像前向明确。
3. Q符号2×1与岩石2×2的具体UV角点和帧上限，防止整张sprite sheet贴到一个实例。
4. 直接读取全部实际GLB的POSITION/NORMAL/UV/COLOR数组，验证有限值、边界、原alpha、R的-Z前向及0.01单位转换。
5. 未加载资源明确返回None与ready=false，保留Renderer回退。
6. 启动目录校验拒绝重复/缺失key、无效尺寸/帧格/来源/源blend、负scale、非正交轴及越过源转换边界的bounds。
7. `cfg(test) SkillAssets::fixture(world)`只替代渲染资源；无需AssetServer，实际Blend/Atlas材质与生产共用factory，不合成模拟事件。

## PNG运输层结论

固定补丁URL仍经过Cloudflare Polish。实际Q图容器从51822变45043字节而RGBA完全相同；
Q标志另有387个完全透明像素的RGB被清零，并存在RGBA→RGB格式优化。
导入记录原容器SHA、原RGBA SHA和规范化像素SHA，只统一完全透明像素下未使用的RGB为0，
拒绝任何alpha或非透明RGB变化，然后固定filter0/zlib9编码。没有图片缩放/换色。
这使CDN无损容器优化不再被误当资源补丁更新，未知内容改变仍失败。

## 集成待验

原生和WASM门禁、两个Renderer的真实材质/前向/尺度、技能生命周期/视野/暂停/重选，
以及有窗口WebGPU截图由父任务统一执行。结构校验与metadata不代替真实截图。

运行快照：`/tmp/kiva-vfx-final-files.json`（42文件），canonical SHA
`126abbaaf50375f92eec0989cd32b06afb9fea7f3f60364788b6b8922a1bb20b`。

需回写的已实现契约：公共typed资源API、AtlasSpec/default_frame、原/转换bounds及轴、
共享材质只读与实例克隆、局部ready/错误回退、PNG Polish语义指纹、原SCB颜色与UV保留。
父任务统一执行update-spec/索引与总架构同步，避免重复规范。

首次共享cargo check已通过但报告未用exports/metadata。已按实际使用裁剪导出、保留renderer使用的SpriteForward，并增加有意义的目录启动校验；没有allow整模块警告。root随后执行全部141项client focused回归均通过，包含本模块7项，日志`/tmp/kiva-skills-client-tests.log`无warning；完整Clippy/WASM/浏览器仍由父任务完成。

真实首轮GPU源核对与授权修正见research/gpu-source-review.md：Standard旧face VCP全0导致乘黑，已用显式且有守卫的策略修正；新旧position/normal/UV/alpha逐字节相同，其余8 mesh未变。W原UV90°及alpha峰值的证据交renderer修复，并补入参数来源。最终35产物已按当前manifest完整重建校验，再次冻结。

## 最终联合结果

父任务391Rust/Node40/原生与WASM/Clippy/Trunk通过；三窗口实际VFX、八技能深度交战、模型与输入回归通过。root实拍查看纹理、透明、UV/方向与比例，最终发布37文件逐字节一致，详父verification.md。
