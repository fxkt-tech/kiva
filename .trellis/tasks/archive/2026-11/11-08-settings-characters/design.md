# 分层与共享接口

## 只读角色目录

HudView新增character_catalog: null | CharacterPreview[]。只在首份成功完整快照发布，与equipment_catalog同属publisher的静态目录发送状态；把EquipmentProjection内sent_catalog移到publish的Local Publication{next_publish,sent_catalogs}，不让装备模块拥有角色目录状态。
CharacterPreview沿用现有JSON：id/name/description/portrait/max_health/max_mana/attributes/display_attributes/base_attack_speed/passives/abilities。提取CharacterPreview::new供Selecting预览与静态全目录共用。Active session.selection仍为null，不重复发送静态目录。
新增同步head脚本characters.js在WASM前缓存首个character_catalog，DOM工厂晚到也能读取；不复制属性/技能规则。前端可自定小型只读缓存工厂接口。

## 确认与权威重开

新增封闭会话请求ChangeCharacter { match_id:String, player_id:String, character_id:u16 }。必须当前Active身份/当前match、Settings打开、实际且事件确认的捕获、聚焦，且角色存在；允许Running或Finished，不允许构造错误。先校验再重开；未知角色不动旧局。
RestartMatch与ChangeCharacter校验分别保留，私有recreate_match共用reset/start/递增修订和失败Selecting回退，避免两份构造逻辑。新玩家替换后本批次尾部丢弃沿原process_batch边界。

## 前端整合

单一#character-select下原列表/预览保持ID；增加#entry-character-host与#settings-character-host，根据Selecting/Settings开关移动同一节点。原Selecting的SelectCharacter/Launch保留；Active角色页只改本地draft，预览来自只读目录。
角色页有#character-apply，点击冻结当前match/player/character并请求捕获，attempt.phase='ChangingCharacter'；确认捕获才提交ChangeCharacter，等待新player投影再关闭。失败保留设置和可见错误，不能在提交前ReturnToSelection。结算入口打开角色页，结果态只在刚进入Finished时关闭旧设置，后续允许查看角色页。
删除设置装备入口和equipment.js对应绑定，T保持。初始/运行/结算的确认按钮按状态区分文案即可，选角仍保留旧entry启动入口。

主会话负责Rust投影/会话及测试、规范和日志；实现子代理负责Web/HTML/CSS/Node及必要旧浏览器夹具入口同步，不并行改Rust。普通检查，不运行浏览器/GPU/服务。
