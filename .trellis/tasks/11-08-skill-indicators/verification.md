# 实施与验证记录

实施者：`check_battle_presentation`，2026-11-08。仅修改分配范围；不运行Cargo/Trunk/GPU，不操作服务或Git。

## 交付

- `client::ability_targeting::resolve_intent`统一实际槽/装备实例到AbilityId、原始意图、preview解析目标和Ready/ApproachingEnemy/Invalid/NoTarget。Enemy稳定扫描候选，跳过不适用的兵/结构后选择合法英雄；controls与指示器共用，不按英雄或QWER分支。Point始终提交原点击点，preview截断只供绘图。
- `CastInfo`增加实际ability字段；没有修改AbilityDefinition或任何模拟规则。
- `SkillIndicatorsPlugin`独立提供当前显示相机投影、实际执行几何、共享原reticle素材和有限平面实体池。Line/Arrow画完整range和真实width，Volley从count/总angle生成扇形和各箭道，Standard显示落地伤害半径，Scout显示路径和实际视野半径，Enemy/通用Impact/Channel/Dash及装备语义回退。
- 后处理位于camera更新、Transform传播、CameraUpdate之后，VisibilityPropagate/CalculateBounds/CheckVisibility之前。使用唯一CSS位置重投影，不改PointerState；仅自身无父平面根同时写Transform/GlobalTransform，避免相机pan后的落点/显隐晚一帧。
- 使用SkillAssets只读共享Add材质，不修改模板、不把青色PNG直接乘红色；非法/走近状态叠独立红/金边。图片未就绪时保留纯几何边界，不显示不透明方形fallback。扇形实际mesh由定义角度决定，UV将原约60度图案拟合进去，实际七射线仍与模拟对齐。
- 集成复核后，Leap的大落点圈按执行形状选择原`aoe_brand`/IndicatorTarget，Standard/Area仍用IndicatorAoe；没有按英雄或技能ID分支。
- Fan mesh在Update按实际AbilityCatalog首次/变更预热，角度相同复用；晚PostUpdate只选已有mesh，不新增渲染资产，避免错过AssetEventSystems后首次图案晚一帧。缺少预热项时仍保留真实箭道几何。
- 仅Enemy瞄准重查当前相机下的模型候选，Direction/Point不追加蒙皮picking遍历。暂停/失焦/失锁/UI/缺玩家/死亡/旧装备实例时清空只读view并隐藏池。没有新命令、战斗计时器或PickGeometry。

## 公共接线（root负责）

```rust
mod ability_targeting;
mod skill_indicators;
// Existing plugins/resources remain in their original owners.
.add_plugins(skill_indicators::SkillIndicatorsPlugin)
```

删除scene内旧技能aim分支，保留A攻击移动、单位/反馈圈；UI投影读取`Res<IndicatorView>`并克隆`.0`为`Option<IndicatorSnapshot>`。`ui::publish`须增加`.after(skill_indicators::SkillIndicatorsSet::Project)`以读取同帧结果。

IndicatorView是Resource/Clone/Serialize，serde transparent包装Option。诊断字段为ability_id、shape、status、rejection、origin、target、cursor_ground、target_id、range、width、radius、rays、angle、textured。None表示无有效瞄准，target_id只来自当前可见合法/明确拒绝候选；不接受浏览器反写。textured只证明所需本地材质已驻留，不能代替实际画面验收。

## 回归

新`skill_indicators/tests.rs`六项：

1. 真实W指令产生的7支Projectile方向与预览逐支一致；总角27.75°、范围12、直径0.2。
2. R全宽2.6/范围250，近鼠标点不是终点，不虚构落地爆炸圈。
3. Q线全宽1.36/范围7.85；混装Scout仍按实际ID、最大截断点250、视野半径10展示。
4. 寒冰混装皇子E：原始20单位点击保留，preview8.6点/伤害半径2与真实未激活旗位置一致。
5. 装备主动按实际实例/AbilityId3走Dash回退，卸装后旧实例解析为None。
6. 真实对局的平面指示器在不更新旧PointerState.ground时立即按新Camera重投影；同帧回投影回到CSS指针，自身GT与Transform一致；注册任意0.9弧度/4箭Volley后Update提前准备匹配角度的实际Mesh，重复Update与50次PostUpdate不增加Mesh/Material；UI/暂停即时清空。

现有`controls::real_hero_target_leap_keeps_aim_on_minions_and_recasts_without_enemy_or_second_cost`增强为有序候选`[minion, hero]`，先断言共享helper选同一英雄，再用真实控制/模拟验证实际R和重施，没有只验证metadata。

## root联合验证结果

最终391项Rust完整门禁通过，包含本模块6项及增强后的controls真实R行为；WASM Clippy、Node40及临时Trunk构建通过。Native几何、混装、实例失效与当前相机回投影均通过，没有伪造SimulationEvent或写模拟状态。

有窗口WebGPU的1280×800/DPR1、800×600/DPR1、1000×720/DPR2真实素材指示器流程均通过。root实际查看原Add纹理、七箭扇形、Q线、E落点和R全距箭；当前相机回投影<1 CSS px，暂停4 RAF稳定、重选清空、无额外运行告警。深度champions已通过真实皇子R超距追近/围墙/重施。

原模型剑光标、HUD/装备点击、原生时钟血条和小地图等父任务相关回归已由root统一验收通过，最终源/构建身份及日志以父任务verification.md为准。虚拟帧钟不作为性能证据。
