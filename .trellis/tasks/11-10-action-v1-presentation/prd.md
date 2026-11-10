# 完成独立临时妹妹、四件武器、双角色动画归属、两类联动镜头、双技能HUD与音效。

状态：继承父任务已获用户明确授权的第一版范围。

## 目标与范围

完成独立临时妹妹、四件武器、双角色动画归属、两类联动镜头、双技能HUD与音效。

唯一写入范围：src/client/**（除 controls.rs、web.rs）、web/hud.js、web/style.css、index.html、scripts/hud-unit.test.cjs、scripts/appearance-unit.test.cjs、scripts/audio-unit.test.cjs。不修改其他实现者的文件，不撤销用户或其他代理的修改。

## 需求与验收

需求、排除项和具体验收继承 [父任务 PRD](../11-10-action-system-v1/prd.md)。本子任务负责 AC01、AC05–AC09、AC11–AC13 的只读表现与完整可玩闭环；全部第一版行为必须在父任务集成后共同通过。

## 依赖

依赖模拟子任务发布 runtime-contract.md；不得另造规则时钟。与输入子任务文件不重叠，集成遵循父任务A→B→C检查点。

## 授权

用户先授权实施，后续明确授权提交、合并 main、删除本次临时分支并推送；不启动常驻开发服务。
