# 设计与改动边界

## 公共几何与操作资格
新建client/picking.rs，拥有模型部件归属、屏幕几何、候选聚合/排序。几何层只判断指针覆盖哪些可见存活单位，不判断敌友、攻击射程或技能资格。
controls使用同一候选解析普通选择、默认敌方悬停、右键攻击与Enemy指向技能；最终资格仍由现有preview/模拟验证。默认优先合法敌方模型，对友方仍保留公共候选；未来友方技能接入自己的filter，不复制几何。hovered只读诊断与现有selected持久点击语义分离。
A+左键明确命中合法敌人时同样复用该候选，发AttackTarget并更新selected；无合法敌人时才沿用原AttackMove地面指令。修复只在controls，不改模拟的自动索敌规则或把旧selected当作新点击目标。

## 真实已显示模型
固定步输入收集前，GlobalTransform/Aabb/Camera本来就是最后实际显示帧，直接读取，不额外维护第二份显示位置缓存。
实际可选几何以PickGeometry显式归属模型/底座，读取Mesh3d部件Aabb和GlobalTransform，逐部件投影为凸区域，加6 CSS px边缘容错。不能合成一根斜武器的大空白矩形，不包含技能范围、弹丸或特效。Pickable迁至picking所有者并更新使用者；无Mesh边界的原生夹具/模型首帧可有明确根profile fallback，但实际模型存在时优先真实部件。
UnitVisual仍唯一拥有插值、朝向与击飞高度，不写战斗位置或扩大模拟半径。候选按明确模型命中、容错距离、可见深度及稳定实体ID排序，每单位一次；死亡/无视野/旧实体在消费时继续拒绝。

## 剑形光标与Web边界
现有逐帧kiva-camera只读payload增加enemy_hovered:boolean，Rust对悬停敌人复核存活/可见/阵营。完整HudView增加hovered:string|null用于实机核对，既有kiva-labels数组和selected保持。
web/pointer.js继续唯一拥有光标位置、显示与形状，监听enemy_hovered切换#game-cursor[data-target=enemy]，实时UI命中或UI手势捕获时用箭头；stop清掉旧目标状态。index提供箭头/剑两个SVG，已有style.css的光标样式负责切换。不会在JS推导敌友、模型命中或战斗规则。
标签不改、不加data-unit-id或命中桥，不更改按下三元组、原始位移、DPR校准。DPR只通过既有viewport/CSS转换；每显示帧敌方反馈不改变80ms完整HUD节流。

## 文件边界
implement代理：picking.rs、controls及必要测试、mod/scene/ui/web、必要character/model与unit_appearance归属接线及相关断言。root：index.html、web/style.css、web/pointer.js、pointer-unit测试、browser-picking、文档和集成。hud.js标签和Cargo依赖无需改动。
