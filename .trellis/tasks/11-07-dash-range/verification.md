# Dash最大范围验收

基线18e3589，已有pointer-response整项未提交变更保留。用户现已授权提交/推送，开发服务由用户管理。

## 回归与实现

旧模拟执行新公开用例dash_resolves_near_and_distant_points_before_committing_costs_and_event_position，
首个超距(30,40)被拒绝，未进入Casting，明确失败。修复后该场景及其它Dash行为通过。

内部resolve_target返回实际AbilityTarget；begin仅校验保留原点击，commit从当前Pose截取最大距离，
消耗/CD、AbilityCommitted.position及movement.start_dash统一用解析目标。角色/装备共用，不按ID分支。
只读scene用已有is_dash显示可释放色；共用描述说明超距规则。没有新增策略字段、依赖或客户端权威截取。

## 检查

- make check通过：格式、native all-target Clippy、162项Rust、WASM check。
- 162 = 45客户端 + 6角色 + 13装备 + 2reset + 59simulation + 37status，原158项保留。
- 新增3个simulation场景，覆盖近/远/斜向/自身点和事件/成本、f32 MAX有限点、非法目标不扣费且保留移动。
- 原sweep墙场景同时测试近点和超距点；新增装备超距实际端点/事件/独立CD/费用场景。
- 非Dash原射程拒绝、控制、取消、快照等原有场景保持。
- WASM Clippy deny warnings、临时Trunk构建/tmp/kiva-dash-site通过；JS语法和diffcheck通过。
- 独立代码/文档复核未发现需修正问题。

## 浏览器（最终构建）

| 入口 | 模式 | 结果 |
|---|---|---|
| skills | headless | 三尺寸全部实际通过（首轮1280/800及独立DPR2完整补验），真实E点击2倍range到最大端点/CD |
| equipment | headless | 三尺寸全部通过，深场景鼠标装备Dash点击2倍range到最大端点，仅装备CD变化；其它原断言保留 |
| smoke | headed/独占Xvfb | 全部通过：实际像素/指针、移动/技能/冲刺/普攻、UI/镜头/resize/高DPI、暂停失焦恢复；截图正常 |

不放宽旧控制/拾取/状态等断言，不写World；各脚本自有HTTP和显示fixture并在finally清理。
全部验收完成，独立复核无未处理问题；HTTP fixture、Chromium、Xvfb和临时认证均已退出。

## 慢软件GPU的执行预算

首轮skills前两尺寸通过，DPR2在原R预定位等待20秒宿主超时；尚未运行新E。
失败快照tick395/Airborne余0.6s且仍在位移，没有控制到期或新Dash拒绝；短动画帧时钟只推进少量虚拟tick。
仅在帧时钟启用后增加120秒宿主等待预算，模拟时长/数值/全部原断言保持。
增加可选KIVA_SKILLS_CASE选择（未知值报错、默认三尺寸）以单独补验DPR2、防整批软件渲染超过执行器总时限。
这两处fixture变更已独立复核。首次失败不记作通过；随后DPR2完整场景已实际补验通过，包括原控制/拾取/暂停断言及新E最大距离。

最终JS语法、JSON/JSONL、本地文档链接与diff检查通过；源码和临时构建资产匹配。
pointer-response原有变更与验收不撤销；按用户现有授权提交两项修复、归档任务并推送。
