# 技能特效与瞄准指示器

## 1. 范围

适用于`skill_assets/`、`skill_effects/`、`skill_indicators/`、`ability_targeting.rs`及源特效离线导入。
技能规则由simulation拥有；客户端只读真实提交、命中、载体、场上对象和附着状态。具体AbilityId只选择外观，不决定伤害、碰撞或施法资格。

## 2. 模块和接口

- `skill_assets`集中typed TextureKey/MeshKey、来源元数据、共享Image/Mesh/StandardMaterial和ready/errors；renderer不重复加载或修改共享模板。
- `skill_effects/capture`在完整SimulationSet::Step后、GameplaySet::Outcome前，用独立MessageCursor捕获真实事实；render协调统一时间与生命周期，sampling只读实际载体，layers生成外观配方，drawing原子创建/回收显示组及临时材质。
- `ability_targeting::resolve_intent`是controls与indicator共用的只读目标资格入口，返回实际装配AbilityId、原意图、preview解析结果及Ready/ApproachingEnemy/Invalid/NoTarget。
- `skill_indicators`按实际execution推导geometry，独立render更新少量显示根。IndicatorView和SkillEffectsView仅发布可见表现诊断，没有写World桥。

未知技能或资源未就绪时保留通用回退；原素材组完整创建后才替换旧几何。先同步载体Transform，再执行SkillEffectsSet::Render，最后绘制scene/battlefield的未Managed回退，不能同帧叠画两套范围圈。

同一载体外观过渡（如Scout→Reveal）也必须先创建新组，成功才回收旧材质；新资源暂不可用时保留当前可用外观及旧request以便重试。丢失根或来源生命/技能变化则先清理，不能借回退保留旧生命附着。替换复用原live槽位，不被总预算误拒；最后仍按真实载体寿命统一回收。

盾回退在Render之后由battlefield::sync_shields按真实成功组的OriginalShield(owner)标记决定显隐，不能按“素材已加载”预测创建成功。W原锚点置于来源世界Y+0.8，再独立旋转平面朝向相机；不能将局部Y偏移随billboard一起旋转。显示表面从锚点沿相机法线前移0.5，避免穿过躯干；保持普通深度测试和depth_bias=0，不全局压过墙/角色。

## 3. 资源契约

资源位于`assets/vfx/`，离线入口为`scripts/import-league-vfx.py`。使用固定16.22原PNG、官方WAD中的SCB和核对后的发射器参数；来源、版本、源与产物哈希、轴变换、bounds和图集记录在清单中。运行时只加载本地标准PNG/GLB，不下载或解析League私有格式。

固定URL的PNG可能经CDN Polish无损重压缩。importer验证CRC、尺寸及可见像素/全部alpha指纹，清零完全透明像素无意义RGB后规范编码，再验证固定产物SHA；不能只放宽原SHA，或接受可见像素改变。实际SCB3.2面记录为100字节，解析须校验EOF、有限坐标和索引；保留UV/顶点色，源单位与前向只在已记录变换中处理。

E Weapon源SCB的全部旧VCP面RGB为0，不能直接乘入粒子基色。仅对已核对的JarvanStandard显式使用ignore_zero_legacy_vcp策略并验证仍为全零；保留其UV、顶点色/alpha。其它网格继续保留实际面色，不能全局忽略。W nova源UV旋转90度，以实例变换(u,v)→(1-v,u)让灰度沿半径而非圆周采样，源alpha峰值为0.1569；遗漏会显示亮楔。

黑底全alpha的原指示图使用Add，不能贴成不透明黑方块。texture_spec提供atlas和sprite_forward，mesh_spec提供texture/bounds/forward；不可散落重复90度猜测。模板已使用default_frame，逐帧UV覆盖使用绝对atlas变换，不能再乘一次格子缩放。R头与尾保留共享pivot；不能分别居中。

短效需要淡出时克隆材质并转换为Blend，透明层不投射/接受阴影；持续岩墙、无人机和落地旗保持适当Opaque。材质实例属于EffectGroup，销毁组时显式回收；共享模板不随单次技能删除。

## 4. 时间、观察与清理

瞬时表现年龄只由SimulationTick与统一PresentationFrame计算。普通暂停冻结时钟/Transform/UV/材质，不在暂停中换装迟到素材；终局允许最终同步一次。迟到且已过寿命的事实丢弃，不能在恢复显示时重播。

Terrain消失和Scout→Reveal在FixedUpdate捕获实际发生tick；render不能以当前显示tick补造旧拆墙时间。独立效果按真实carrier Entity跟随；同一carrier组号不能合并Volley的不同箭。已观察普通箭可按真实Entity缓存纯外观身份，强化箭使用payload冻结的empower_origin；来源删除/换生命不取消已提交独立投递。附着闪光/符号必须核对当前source_life并先更新实际位置再复核可见性。

Contact必须按命中点实际团队视野过滤，友方施法也不能泄露隐藏受害者；其它友方已知独立载体可显示。源模型是否还活着不是独立载体的可见性条件。显示实体不加PickGeometry，不参与鼠标选人。

会话键为(MatchId, player Entity)，变化时清所有队列、显示组、来源外观缓存和材质。pending最多192、瞬时组最多96、live组最多256，超过预算只影响表现；不限制模拟载体/命中或修改规则。持续组复用资源，逐帧不能不断创建Mesh/Material。

## 5. 指示器几何与当帧相机

指示器在PostUpdate中位于camera更新、Transform传播和CameraUpdate之后，可见性传播/CalculateBounds/CheckVisibility之前。用唯一CSS指针与当前相机重投影；起点读实际显示UnitVisual，资格读模拟事实。只更新自己的Transform/GlobalTransform，不改PointerState、UnitPose或角色根。

Line按真实range/width，Volley按完整angle/count/radius显示（寒冰W七箭总角27.75度），Arrow按全range及2*radius整宽显示（寒冰R宽2.6），不能把近处点击误画成短箭/光标落点AoE。Standard/Scout用preview真实截断点和效果/视野半径；Leap用实际合法英雄与落点半径。图集扇形mesh在Update按实际catalog变化预热，不能在PostUpdate创建尚未进入当前渲染准备的新Mesh。

Enemy候选保持模型命中顺序，跳过preview不合资格的候选，只有OutOfRange表示可追近；原Unit命令交模拟追踪。Point仍提交原点击点，resolved只用于绘图；Direction保持方向并显示完整真实射程。非法颜色用独立红边/标记，不能用红色乘青色原图而变黑。退出aim、UI覆盖、暂停、玩家或装备实例失效即隐藏。

## 6. 示例与验证

正常：皇子R跳过前排不合法小兵选中后方英雄，超距预览显示ApproachingEnemy并交模拟走近；拆墙只播放真实岩块坍塌，不再次跃击或播初次R声音。
正常：4次真实普攻后寒冰Q激活，来源随后删除，已提交强化箭仍从冻结来源显示；载体结束后回收。
错误：用血量下降猜命中、以渲染帧补算伤害、暂停使用墙钟继续发光、源死亡就删掉独立箭、读取隐藏敌人后仅在DOM隐藏。

`tests/skill_facts.rs`验证公开真实规则事实；`client::skill_assets`验证清单/材料，`client::skill_effects`通过真实命令验证观察、暂停、来源、重施、跨显示帧时效和资源预算，`client::skill_indicators`验证真实几何/当前相机/任意装配。内存显示资源替身只覆盖客户端资源生命周期，不能替代原PNG/GLB或GPU验收。

运行make check、WASM Clippy、Node；重建后串行执行browser-skill-visuals及深度browser-champions，覆盖八技能和实际R命中/拆墙、Q强化，再按影响回归skills/picking/equipment/minimap/pointer/audio/reselection/presentation/smoke与原生时钟healthbars。必须实际查看有窗口截图的方向、图集、透明度和尺度，不能仅用ready/phase字段验收。虚拟帧钟不能当作FPS证据。

## 7. 表现边界

当前实现用原PNG、SCB网格及已核对参数重建关键箭/尾迹/扫描/盾/旗/岩墙，简化了噪声、扭曲和层级发射器，不是完整League粒子运行时或逐像素还原。新外观继续扩展局部素材配方，不能向simulation引入皮肤、材质或模型依赖。
