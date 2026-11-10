# 玩法层

## Scope / Trigger

`src/gameplay/mod.rs`、`terrain.rs` 定义确定性的 50m 竹林、地形与实体装配，修改场景、坡面或出生配置时阅读。
依赖模拟，不导入客户端、窗口、材质和网格。

## Signatures

`setup(&mut World)` 插入 `Level`，通过模拟的 `spawn_box` / `spawn_terrain` 装配三维道具、地面、坡道和有限高度围墙，再装配玩家与所选模式的敌人。
`EnemyMode::{Target,Combat}` 为模式资源，默认 Target；`set_enemy_mode(&mut World, EnemyMode)` 在模式改变时替换敌人，并清理玩家旧目标、联动和战斗意图，保留玩家姿态、生命、耐力、武器与冷却；相同模式不重置敌人。
所有战斗实体含 Body / MoveIntent / ActionIntent / MotionState / Facing / Health / SpawnPoint；
玩家另含 Player / Combatant / CombatIntent / DefenseState / Stamina，敌人另含 Enemy::default() 或 Enemy::target_dummy()。
`Level { half_size, props, ground, ramp, bamboo, forest_trunks, forest_rocks }`；
`Prop { center: Vec2, size: Vec3, kind: PropKind }`；kind 为 Crate / Pillar / Stone。
`TerrainGeometry { vertices: Vec<Vec3>, triangles: Vec<[u32;3]> }` 是碰撞和客户端 Mesh 的同源几何。
`Level::boundaries() -> [Boundary;4]`；`Boundary { center: Vec3, size: Vec3 }` 与 Prop 都作为组件挂在相应物理实体上。

## Contracts

Prop.size 是完整三维尺寸。Crate 为可转动动态刚体；Pillar / Stone / Boundary 为固定实体。
客户端通过 owner 关联物理实体，读取 PhysicsPose 的中心位置和旋转，不把初始描述坐标当作动态箱子的当前位置。
地面使用一份包含小山的连续三角网格，不在其平面区域叠加第二份碰撞地板；坡道与高台共用闭合几何，顶面接缝共用精确位置。
西侧坡道约 20°，升至 1.5m 高台；东侧小山顶约在 `(16.5,2.6,8.5)`，具有缓坡和超过行走上限的陡坡。
小山用确定性连续高度函数采样；不能分别在模拟和客户端计算两套山形，也不能用堆叠长方体替代连续山面。
玩家出生在原点，速度 6m/s，半宽 0.35m；玩家 Facing 由客户端输入表达，敌人 Facing 由模拟决策。
玩家生命 1000、默认 WeaponPair::HammerMusket（锤子＋鸟铳）；实战模式仅装配一个 `(0,-6)` 愁铃客，Body 速度5.2m/s、半宽0.3m、标准身高，另含 `Boss::default()`，生命上限 `Boss::PHASE_HEALTH`（当前650）。两条生命阶段和伤害等是Kiva适配；已恢复的原作决策由simulation管理。
靶子模式只装配一个 `(0,-4)` 的被动敌人、生命 1,000,000、Body::scaled(1.45)，半宽0.435m和同比胶囊高度；主动移动和攻击由模拟禁用，仍有完整物理与生命状态。
出生点由玩法指定，复活规则由模拟执行；客户端不能另造目标或生命数据。
玩家装配时建立逻辑 Companion／CompanionState，无独立 Body 或 Health；妹妹的显示资产属于客户端。模式替换时同步清旧联动、拉力和妹妹可见性，暂停也不能残留。

## Validation & Error Matrix

| 情况 | 行为 |
|---|---|
| 非有限 / 非正尺寸、无效索引或退化三角形 | 内容错误，由构造或几何回归拒绝 |
| 地形与出生点 / 既有道具重叠 | 关卡回归失败，不由客户端移动对象补救 |
| 小山接平地或坡道接高台 | 共用几何精确衔接，用实际控制器测试接缝 |
| 原点附近 | 保持 2m 出生净空和原有战斗练习区域 |
| 装饰线、绑带、地面涂装 | 保持视觉用途，不生成碎小绊脚碰撞体 |

## Good / Base / Bad Cases

Good：移动或倾倒箱子后，模型、承托和遮挡随同一物理实体变化。
Base：从出生区走到西侧坡道 / 东侧小山，使用真实坡面接触上下行走。
Bad：复制初始坐标渲染动态箱子，或为同一块地面创建重叠的两份碰撞网格。

## Tests Required / Quality Check

`make check`。纯几何检查有限性、索引、向上法线、闭合坡道、山脚平接地面、坡度范围和安全放置。
真实固定步装配检查默认玩家与一个高血量被动靶子、动态 / 固定道具和有限高度围墙，确认出生净空与接地。
模式回归检查暂停时替换、单个高血量靶子不追击 / 不攻击且可以受伤击飞、切回单Boss及旧目标清理 / 冷却保留。
`tests/physics_terrain.rs` 覆盖真实竹林坡道上下、平台边缘、小山登顶 / 下山、陡坡限制和落体暂停。
平台二段跳、箱顶支撑和移走支撑使用真实物理解算；画面与手感另在浏览器验证。

## Wrong vs Correct

错误：客户端决定地图碰撞，玩法创建 Mesh。
正确：玩法提供共享原始几何并装配物理实体，客户端只据同源几何和实体姿态生成表现。

默认 Level.props 为空，移除旧庭院箱子/石柱；ground/ramp/forest_trunks/forest_rocks 四份共享三角几何驱动显示与碰撞。竹林保持中央椭圆战斗空地和曲径；确定性竹干位置、半径、倾斜、高度同源，装饰不在玩法层制造Mesh。地图为主题重建，不是原版实例导出。靶子的Body尺寸在物理安装前配置，客户端只读scale；模式替换、传送、蹲起和复活不得丢失该尺寸。
