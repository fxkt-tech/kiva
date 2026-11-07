# 角色系统实施切片与验收

用户已审阅并明确要求开始，运行task.py start后按以下切片实现。
按实际落地与验收更新规范/TODO；不启动常驻服务，不自动提交或推送。

## P1：角色定义、装配与统一创建

- [x] 新增characters模块，显式导出CharacterId/Definition/Catalog/Identity、LoadoutSelection、CharacterSpawnSpec及错误。
- [x] 在abilities模块定义同形四槽AbilityLoadout别名，保留原数组调用兼容性。
- [x] 注册先校验技能引用/基础定义，创建解析完整装配后复用spawn_unit；失败不留实例。
- [x] 初始属性纯计算由attributes唯一实现，构造与角色预览共用；不更改公式。
- [x] 目录、默认/Explicit（空槽/重复ID）、多实例独立、失败原子性、施法和重生装配保持测试。
- [x] 先过全部既有109项，再做客户端生命周期迁移，避免把核心与UI失败混在一起定位。

## P2：内容迁移与会话生命周期

- [x] content拆成地图/技能/角色/NPC准备，增加DemoSettings，不再创建Web玩家。
- [x] 注册旅者和秘术师；旅者完全保留当前默认值/技能作为回归路径。
- [x] 新增client/session.rs拥有Selecting/Active和唯一活动玩家Entity；PlayerController仅保留交互状态。
- [x] 初始化合法无玩家阶段，不使用占位Entity或隐藏默认英雄；模拟插件默认暂停值保持。
- [x] 启动请求处理先于collect_input/巡游/固定步，暂停期间也能消费。
- [x] controls/camera/scene/反馈/HUD逐个迁移无玩家读取，排空菜单输入且不向空玩家提交Stop。
- [x] Native通过相同创建接口自动默认旅者，保持编译与现有调试玩法，无额外选角UI。

## P3：预览投影与选角交互

- [x] 抽出已有技能静态展示构造，选角预览和实际AbilityView共享；旧扁平JSON字段保持。
- [x] HudView增加session、可空player_id、UnitView可空character_id；选择阶段不伪造UnitView。
- [x] 选择和启动带明确回执/修订号，选择变更和错误强制发布，Active继续80ms限频。
- [x] entry增加角色卡片/当前预览；初始旅者预选，实际英雄HUD在无玩家时隐藏。
- [x] 卡片可键盘操作、data-ui隔离；新预览不复用实际HUD的data-stat选择器。
- [x] 完成1280×800、800×600及DPR2/缩放布局，开始按钮始终可达。

## P4：捕获成功后的原子开局

- [x] 选角只提交ID，不允许DOM传属性或任意技能数组。
- [x] 最新选择回执确认后才能开始；本次角色/修订号在点击时冻结，捕获申请单次进行。
- [x] 保持click内直接requestPointerLock；成功pointerlockchange同回调先发Launch，不依赖Promise返回时序。
- [x] Rust复核阶段/最新选择可开始资格/角色/修订/实际捕获/焦点，再唯一创建绑定；捕获/创建失败保留有效选择资格允许重试，选择失败须重新接受选择。
- [x] Active恢复仅捕获，不发初次Launch；重复捕获/启动不重置Entity、生命、装配、冷却或NPC。
- [x] 保留GamePointer居中、清旧WindowEvent和暂停Stop规则；选择期间游戏输入不积压。

## P5：完整验证与规范同步

- [x] 对照PRD的AC1–AC16写实际验收矩阵，完成独立全范围复核。
- [x] make check：格式、原生Clippy、完整Rust回归、WASM check；加WASM Clippy -D warnings。
- [x] node --check所有修改JS，最终Trunk构建到临时目录。
- [x] 新browser-characters检查初始无玩家/冻结、切卡预览、两角色开局、快速点击/捕获失败/恢复、重复启动拒绝。
- [x] 更新browser-support区分enter/resume；测试默认旅者时显式确认预选，恢复测试不偷偷重新选角。
- [x] 运行既有skills/smoke/stats/cursor/healthbars；保留旧数值、DPR、血条和空中暂停断言。
- [x] 落地后回写架构/模拟/客户端Trellis规范并勾选TODO角色项；装备项继续未完成。
- [x] 记录实际结果；提交/推送以届时用户授权为准，不启停用户开发服务。

## 行为测试矩阵

| 场景 | 断言 |
|---|---|
| 角色注册 | 重复ID、空名、NaN/非法stats、未知默认技能拒绝，旧目录保持；稳定ID排序 |
| 创建失败 | 未知角色/Explicit技能、非法出生点不增加Unit/Identity或消耗状态，不绑定玩家 |
| 默认/显式 | Explicit替换完整四槽，None保持空；重复技能ID按槽冷却；角色目录未改 |
| 实例隔离 | 同角色两个单位HP/冷却/状态/来源独立，任何一方不回写模板 |
| 预览 | 七项初始值与实际构造结果一致，包含0/极限攻速归一化；无隐藏单位/来源分配 |
| 重生 | 身份和Explicit装配保持，epoch推进、状态/运动清理与既有冷却复位保持 |
| 初始选角 | player_id=null、abilities=[]、tick不推进、NPC不巡游；菜单按键/点击不积压 |
| 选择回执 | A→B→A快速点击只展示最新接受结果，旧回执不能启用开始；[非法SelectB,旧LaunchA]拒绝；重新接受SelectA递增修订，旧启动仍失效 |
| 开局提交 | 已接受选择+匹配版本+真实捕获/焦点才有唯一玩家；失败不部分绑定 |
| 捕获时序 | 捕获成功在Launch之前/之后通知的适配分支不会单独解暂停；Promise/失败回调不重放旧请求 |
| 重复事件 | 多次Launch、双击、多个FixedUpdate不增加玩家，不刷新角色/技能/冷却 |
| 暂停/恢复 | 同Entity/角色/实际装配、生命、冷却、NPC场景保持；死亡不回到选角 |
| 第二角色 | 预览匹配30攻击/40法强；Q方向瞄准、R自身引导；同一控制/效果执行管线 |
| 无玩家/平台 | 所有consumer不panic、不用假Entity；Native默认路径不等待DOM |
| 显示与布局 | 三分辨率/缩放，键盘选择/开始可达，144px无单位HUD、逐帧血条和击飞拾取不回退 |

## 风险文件与回退

PlayerController读取者较多，必须以rg列出的controls/camera/scene/ui/content/测试为一组迁移，
不能边留强制Entity字段边加入第二份会话玩家引用。捕获/Launch/暂停/入口UI必须一起验收。

P1可独立验证，不改变旧spawn_unit语义；P2–P4以阶段一致版本集成，不交付“卡片能点但仍启动旅者”的中间态。
若失败，回退对应完整切片并保留原109项行为对照，不清工作区、不修改备份分支。
规划只做文档、链接和JSONL检查；尚未实现时不声称游戏测试通过新角色功能。

## P6：用户补充的重新选择与整场重开

- [x] 模拟统一清理入口与真实行为测试，保留来源/序号分配器单调性。
- [x] NPC内容创建复用，暂停重选回到无玩家Selecting，重置tick/镜头/输入反馈。
- [x] 选择版本跨局单调，旧Launch和带旧player_id的返回意图隔离；错误/重复请求不破坏当前局。
- [x] 暂停菜单按钮与说明，Active→Selecting重置DOM选择关联/捕获尝试，不刷新页面。
- [x] 无GPU会话/清理回归，浏览器两个角色反复重开及三尺寸验证。
- [x] make check、WASM Clippy、JS语法和最终临时构建；独立复核与文档/验收同步。
