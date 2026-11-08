# 独立检查：英雄技能表现

## 当前结论

全部已确认的代码问题均已修复并冻结；最后一轮跨层与规范复核未发现新的代码阻塞。最终391项Rust、Node40、WASM/Clippy/构建与相关GPU验收现已通过，详细结果见父verification.md。审查基线为本轮开始时的HEAD（上一轮功能e0fa8bc与归档5273207已经推送），不混入旧任务结论。

检查代理读取父任务PRD、design、implement、全部check.jsonl及client/simulation/gameplay入口和相关规范；由父代理串行执行Cargo、Trunk及GPU，检查代理不另起这些重型进程。

## 已发现并修复

1. **渲染职责混合**：原`render.rs`约860行同时持有资源分配、材质处置、素材几何、模拟查询及生命周期协调。按已授权方向拆为`drawing`（原子EffectGroup、临时材质所有权及清理）、`layers`（纯素材/几何配方）、`sampling`（只读真实载体/附着状态）、`render`（时钟与生命周期协调）。保留对外Plugin和诊断边界，不把规则移到客户端。
2. **Opaque临时特效不能淡出**：原落地破地网格复制的Opaque材质忽略alpha。临时淡出层显式转换为Blend，模板保持不变；持续岩墙/无人机和真正落地前的旗仍保持原Opaque。透明层统一排除阴影投射/接受，避免黑片阴影及多余shadow pass。
3. **拆墙视觉与落地混淆**：原WallBreak复用大块落地破地网格。改为复用持续墙的24个原岩块及相同图集/布局，在真实对象消失后短暂坍塌淡出。重施提交本身仍不产生新的施法/落地配方。
4. **附着闪光一帧旧视野**：原代码先按上一帧点判可见，再移动到来源的新位置。改为先确认当前来源生命、采样显示位置，再检查该位置可见性；来源失效时清理只属于旧生命的附着闪光。独立弹丸/区域不受此清理影响。
5. **普通箭因源删除而消失**：普通寒冰AA外观原本迟读CharacterIdentity；源删除后已替换通用几何的载体失去外观请求。成功渲染后按真实Projectile Entity保存纯外观身份，直到该载体结束/换会话；首次观察时已无来源且没有外观记录，保留通用回退。强化箭仍优先只读payload冻结的empower_origin，不从活源状态猜测。
6. **素材元数据重复**：网格默认纹理统一读`mesh_spec.texture`，方向贴片按`texture_spec.sprite_forward`校正原图前向，不再逐箭复制90°假设。原源网格相对pivot和图集UV保留。
7. 清除spawn返回值未使用的must_use警告；原子组创建失败回收已创建子层和材质，不删除旧通用几何。
8. **fallback盾根据资产猜归属**：原方案只要W材质可用就隐藏generic盾，未证明原盾组创建成功。新增`OriginalShield(owner)`只标记成功创建的真实原盾根，删除旧预测helper；父代理把fallback显隐放在Render后读取实际标记。加载迟到、上限或创建失败不再让回退与原盾重叠/一起缺席。
9. **父载体先清理后的重复despawn警告**：初轮GPU日志出现drawing disposer尝试再次销毁已随载体清理的根。改为`get_entity_mut`成功才despawn，材质仍无条件逐项回收；不把日志问题掩盖为忽略警告。
10. **原W环成为白楔**：初轮GPU和原BIN/网格UV共同证实：RingMesh的U沿圆周、V沿半径，源shockwave2却要求uvRotation=90°。Layer增加仅作用于实例的uv_override，将(u,v)映射为(1-v,u)，Nova源峰值alpha=.1569，原模板不改。第12项真实W回归按实际Nova Mesh句柄找到生产实例，检查UV样本、Add和初帧alpha以及模板不变。
11. **盾中心被billboard倾斜带走**：原平面局部Y=1.1随整组相机旋转，不能保持世界竖直偏移。改为平面局部中心0，组先定位到owner世界Y+.8（源EmitterPosition=(0,80,0)）再旋转平面；尺寸、图和tint保持。第13项真实W回归使用生产camera::setup斜视角和标准TransformPlugin，验证Global中心与平面normal。原生只证明几何位置正确。随后实际查看shield-only新图，左侧金弧可见且正面与Opaque躯干相交；经父代理决定进行局部表面前移A/B：世界Y+.8锚点保留，仅Quad沿camera-local +Z移.5，尺寸/图/tint不变、depth_bias仍0并保留普通深度测试。相机回归区分anchor与surface并检查法线/零bias。父代理完成13项原生回归、重建和同场景W-only实拍后决定保留；本代理也实际查看新旧两图，前移后头部附近金弧较清楚，改善幅度有限但可辨，不将普通深度测试取消或改全局材质。

12. **Scout阶段重建先删旧组**：原rebuild先dispose旧drone，再尝试ScanRing，真实Scout→Reveal时资源迟到会让已Managed载体暂时空白。改为同carrier/同source_life/同ability且根仍存在时先保留旧组，新组完整成功后insert并回收旧材质；失败保留旧request并下帧重试，不能误标新phase后停止重建。换生命或已丢根仍先清理，避免保留旧生命附着；已有slot的替换不受新增live预算误拦。

## 测试与证据边界

新增渲染测试使用内存图像/网格资源替身，只验证客户端实体与材质生命周期；真实PNG/SCB结构和最终实际渲染由资产专项与WebGPU负责。施法、前摇取消、普攻叠层、投射物、强化来源、计时、生命、团队观察均由生产模拟和公开CommandQueue驱动，没有伪造SimulationEvent或写战斗运行态。

新增用例覆盖：

- 真实W在暂停时Transform/材质/UV不变，到期后临时材质回到基线，换会话清队列/实体。
- 普通AA源删除后保留已观察载体的原素材，实际命中/载体结束后回收材质与外观记录。
- 实际4次普攻叠层再Q，强化普攻在第一次显示前删除来源，仍由被冻结的BasicAttack+Ability(21)来源渲染10层箭。
- 共享原材质保持Opaque/原值，临时破地层使用可淡出且无阴影的实例，落旗保持Opaque。
- 真实R初次落地后等待实际重施权限，拆墙只生成24原岩块坍塌，不产生新的AreaActivated或跳跃，结束后材质归基线。
- 110个真实W限制待表现队列为192、瞬时效果为96；110个持续盾重复显示帧不增加材质，重开清理回到共享模板基线。

已有捕获测试覆盖真实前摇取消、一次提交、隐藏施法与可见终点、Scout同Entity到站及暂停/重开。已独立读取`/tmp/kiva-skills-client-tests.log`，141项client原生测试全部通过，其中effects共9项（原捕获3项、新增渲染/生命周期6项）；资产7项、真实R重施音频、共享目标与当前相机指示器也通过。已独立读取`/tmp/kiva-skill-facts-tests.log`的16项真实模拟事实回归通过。单文件rustfmt和git diff --check通过；这些不替代最终完整质量门与实际GPU验收。

## 跨层已核对

- `facts.rs`在真实提交/命中/区域激活/弹丸终止点发布只读中性事实；Contact先校验受效生命/class且以目标实际中心记录，不依赖HP下降，不修改原effects顺序。
- AbilityCommitted保存来源生命、CastSource、原点和解析后目标；重施显式recast=true。普通攻击的DamageOrigin仍BasicAttack，强化外观来源独立冻结。
- 对象、盾、Scout路径/到站及R尾端Reveal透传实际origin；Volley组号不冒充单箭身份。
- controls与indicator共用ability_targeting，按实际装配ID/preview筛选目标，原点击点交回模拟；Point、Direction和Enemy接近语义保持。
- 指示器在当前相机/传播之后、可见性/边界检查之前更新独立显示根，不写UnitVisual或PointerState。视野、纯模型拾取、血条/名字不参与命中的边界保持。
- 角色与导入动作都跳过recast提交，避免拆墙再播R跃击；审查额外发现音频仍复用初次R cue，已由父代理同步修为recast不重播并新增真实R回归通过。真实固定步事实由独立cursor读取，普通暂停使用统一PresentationFrame。
- 已报告的首帧legacy gizmo重叠由父代理修复：先同步载体Transform，再执行effects RenderSet原子替换，最后scene::sync_effects→battlefield::draw_fallback→battlefield::sync_shields在同一Render后chain内读取实际归属；已读最终接线确认，不再靠下一帧移除旧圈或按素材可用性猜盾。
- 原Standard黑矛由资产所有者独立修复：只在已验证旧VCP面RGB全零的该网格忽略旧光照乘数，守卫非零变化；原POSITION/NORMAL/UV和alpha逐字节保持，其余八网格仍保留源面色。新GLB来源与产物哈希、正向RGB与非零VCP拒绝回归齐备。
- 源素材规范与architecture已覆盖本次模块边界、事实来源、预算、观察和暂停契约。指示器fan mesh在Update按catalog预热，PostUpdate只复用已准备资源，避免当前渲染准备错过首帧动态Mesh。
- browser共享pauseOnSkillVisual只监听可见只读快照并提交既有暂停/解锁意图，不写World；deep champions分别要求Q箭10层、R命中4层、拆墙24层，不能被普通箭或初次施法闪光假通过。浏览器等待指示器加入ready与新cursor_ground匹配，修复旧80ms快照造成NoTarget的测试竞态，没有放宽生产几何。
- 边缘滚屏保留合并方向/相反抵消/归一化，只在有效边缘输入选择1.75倍最终速度；键盘、缩放、地图边界及UI/暂停门控保留。

## 追加的时效修复（已落盘，等待最终门禁）

审查发现墙消失后的坍塌由render在当前显示tick临时生成，若墙在两次显示之间很早就结束，恢复绘制时仍会重播旧坍塌。经父代理确认、等待初轮Trunk和浏览器结束后，已修为FixedStep捕获真实Terrain缺席时刻：ObjectObservation只保存kind/ability/source/team/position/radius，移除时记录当步tick并进入统一有界队列，render只清根，旧短效按统一年龄丢弃，age等于duration也不重新分配资源。

新增第10项effects回归：真实R落地生成墙，等待实际重施权限并拆墙，之后跨30tick不执行显示更新；再显示时emitted不增长、无真实Terrain/闪光实体/材质残留。它验证实际施法与可观察资源结果，没有伪造模拟事件。此修复和测试已rustfmt/diff-check并重新冻结。已独立读取`/tmp/kiva-skills-effects-final.log`：加入时效与真实盾归属后的11项通过；当时唯一unused Request.team随后删除。父代理报告UV追加后的12项通过；世界Y/前表面回归后共13项。父代理已确认A/B修改后的13项、Trunk和真实W-only通过；当前再执行最终完整门禁与全套浏览器，不将W局部对照视为整任务验收完成。

追加第11项effects回归验证真实W成功/失败归属：临时移除渲染材质模板让原子组创建失败，实际模拟盾仍在、OriginalShield为零；恢复模板后成功归属player；真实carrier删除后下一Update仍把材质回收到共享基线并清所有Managed根。它同时覆盖父载体先清理再释放动态资源的边界。

修复前完整质量门已独立累计`/tmp/kiva-skills-check.log`：386项Rust通过、2项既有忽略；`/tmp/kiva-skills-node.log`40项Node通过。标准WASM Clippy及初轮Trunk完成，但这些不代替追加修复后的最终构建与验收。

## W 表面前移的实际对照证据

已实际查看`/tmp/kiva-browser-tests/artifacts/skill-visuals/jarvan-w-shield-before-surface.png`与覆盖后的`jarvan-w-shield-only.png`。前图只有左侧薄弧清楚，后图头部附近的金色轮廓较完整；两图使用相同真实场景夹具，演员位置/姿态/相机和W观察窗口相同，均已越过Nova瞬时层。

已独立读取`/tmp/kiva-skills-final-browser-shield-only.log`（前）和`/tmp/kiva-skills-ab-browser-shield-only.log`（后）：真实shield=60、Idle1、同位置`[-10.222971,18.001259]`、相同screen/body_screen，effects仅一个ability12/shield/layers1且pending0/errors空，排除Nova或不同场景造成的误判；两轮自有Xvfb与Xauthority清理均完成。A/B没有修改图、强度、人物规则或全局深度偏置。

## 最后一次原子重建回归

第14项effects测试通过真实E命令取得Scout Entity并先显示drone，再只移除渲染ScanRing材质模板。真实同Entity到站变Reveal后，连续4个显示更新仍保留旧drone根/材质且资源数量不增长；恢复模板后成功显示scan并回收旧根/材质；实际对象到期后全部动态资源回到共享基线。模拟事件、对象kind、位置与寿命均未伪造。该最小补丁已rustfmt/diff-check并冻结，待父代理统一运行focused与最终门禁；不再扩展静态审查范围。

## 原待验收项（现已完成，见末段）

- 完整native/WASM/Clippy/Node/离线资源门禁仍待父代理统一收敛。
- 最终本地资源与metadata/来源哈希、八技能实际纹理/方向/透明度/比例、混装与回退、当前相机指示器首帧、视野/重开/暂停、800/DPR2及原拾取/鼠标/血条/音频回归由父代理统一执行。浏览器脚本完成不等于已经实拍通过。
- 新实现规范和architecture已同步并复核一致；TODO、最终验证记录与最终源码/产物身份仍由父代理收敛。

本检查代理仅改独占的`src/client/skill_effects/`与本审查记录，未提交推送、未启动服务、未操作用户服务。

## root最终收敛

最终391项Rust、2项既有soak忽略、WASM Clippy、Node40、临时Trunk与资源门禁通过，生产源220文件自门禁未变。三窗口VFX/skills/equipment/picking/minimap，八技能深度战斗与Ashe展开帧补拍全部通过并实际目检；pointer四种DPR/物理单位场景通过，原生时钟healthbars人物20/20、相机19/20通过。最终证据见父verification.md。用户指出耗时后收敛相关检查，不重复未改变的前轮完整启动故障/音频解码/尸体套件。两处新增失败均为夹具采样/绝对定位残差，保持真实相机/指针来源与原阈值修复，无生产代码变更，细节在research/minimap-hud-sampling.md。
