# 角色与创建时技能装配

## 1. 适用范围

新增角色、改变默认或显式装配、预览初始属性、修改角色创建/重生时，读取本规范。
实现为`simulation/characters.rs`；低层运行态仍由已有单位/属性/技能/战斗模块拥有。

## 2. 接口

```text
CharacterId(u16)
AbilityLoadout = [Option<AbilityId>; 4]
CharacterDefinition { name, description, stats: UnitStats, default_loadout: AbilityLoadout }
CharacterDefinition::initial_attributes(&self) -> Option<EffectiveAttributes>
CharacterCatalog::register(&mut self, id, definition, &AbilityCatalog) -> Result<(), CharacterError>
CharacterCatalog::get(id) -> Option<&CharacterDefinition>
CharacterCatalog::iter() -> ordered (CharacterId, &CharacterDefinition)
CharacterSpawnSpec { character, team, position, loadout: LoadoutSelection, respawn_delay }
LoadoutSelection = CharacterDefault | Explicit(AbilityLoadout)
spawn_character(&mut World, CharacterSpawnSpec) -> Result<Entity, CharacterError>
CharacterIdentity::id() -> CharacterId
```

角色ID可多次实例化；不是Entity、玩家引用或某个单位的唯一实例键。

## 3. 数据与生命周期契约

先注册技能，再注册角色。目录只读get/iter，按ID稳定排序，重复ID不覆盖。
角色只保存技能ID，技能定义在AbilityCatalog共享，不内嵌技能规则或角色ID条件分支。
`spawn_character`解析装配并调用spawn_unit，之后同一次World操作附加身份才返回；不复制运行态初始化。
技能引用校验由abilities内部helper集中实现，目录/角色创建/raw spawn复用。

CharacterIdentity仅保存出处ID。Unit已有stats为本次构造快照，AbilitySlots唯一保存实际ID及每槽冷却。
默认装配与本局实际装配不双向同步；Explicit完整替换四槽，None为空，重复ID按槽独立冷却。
NPC无需CharacterIdentity，旧UnitSpec/数组调用和SimulationPlugin默认暂停行为保留。

角色初始属性与AttributeState::new复用attributes纯计算；零/极值攻速等归一化不能另写在client/JS。
预览不spawn隐藏角色、不分配ModifierSourceId。无效stats的纯投影为None。
死亡重生保持角色身份、stats快照和实际装配；只沿用原生命代次/状态/来源/冷却规则，不重新查默认技能。

## 4. 校验与错误

| 输入 | CharacterError/结果 |
|---|---|
| 重复ID | DuplicateId(id)，原定义保持 |
| 空/全空白名称，非法UnitStats | InvalidDefinition |
| 未知默认/实际技能 | UnknownAbility{slot,ability} |
| 未知角色 | UnknownCharacter(id) |
| 地图出生点/原构造失败 | InvalidSpawn{reason} |
| 部分或全部空槽，重复已知技能ID | 合法 |

正常返回的创建失败不增加Unit/CharacterIdentity、不分配半成品运行态、不修改目录或绑定玩家。

## 5. 示例

默认：旅者使用定义的四槽。显式：`[Some(other),None,Some(other),None]`保留两个空槽，
两个other分别计冷却，角色默认列表不被修改。重生继续该显式列表。
错误：重生时反查目录把实际技能换回默认，或用CharacterIdentity保存另一份技能/属性缓存。

## 6. 必测

`tests/characters.rs`通过公开注册、创建、命令、属性队列和战斗验证：
注册/排序/旧定义保持、失败不留实体、空槽/重复/显式施法、预览归一化、同角色实例隔离、死亡重生保持。
保留既有simulation/status行为场景；运行make check，跨客户端时加WASM和浏览器验证。

## 7. 错误与正确

错误：每个角色复制spawn、施法和属性代码，UI按角色ID猜技能，或同时维护“角色当前技能”和AbilitySlots。

正确：内容只注册角色与技能ID组合，创建统一解析，运行时读取实际组件，显示只投影已确认定义/实例。
