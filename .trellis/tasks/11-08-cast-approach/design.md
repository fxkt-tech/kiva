# 边界与契约

问题在通用模拟的持续意图与客户端预检，不在英雄内容。新增UnitOrder::CastTarget，保存CastSource、实际AbilityId和UnitLifeRef；表示尚未开始的一次施法目的。指令层先完成所有校验与可达预检才替换动作/意图。

- 严格preview_cast继续表示当前能否立即开始，供AI/提交复核，不改为带副作用或全局忽略射程。
- 通用施法请求入口仅把Enemy的OutOfRange转为走近目的；其它错误仍拒绝且保留原意图。范围内走原begin_cast；新合法技能同样替换旧待施法目的。
- 独立cast_orders模块拥有走近目的的验证、重新规划与转入begin_cast；位置仍只由movement写，动作/CD/资源仍由actions写。
- 每tick验证来源/生命/可见性并在范围内开始；待放推进位于advance_actions之前，保持直接命令与排队施法共用首个动作tick，零前摇同tick提交；目标移动按既有约0.2秒节流重规划。普通resolve_orders跳过此目的，避免抢写路线或误触发普攻。
- 到范围开始前清掉一次性目的，结束后不自动普攻/重新追放。已开始的前摇仍按原提交复核，目标出范围可拒绝，不恢复旧目的。
- 新导航到达区域查询复用当前几何/A*：终点在施法距离内且角色半径可达，不要求敌方中心可达，不能穿过静态或临时墙。原普通path/path_static保持语义。
- 失效取消清目的与其普通路线；死亡直接清UnitOrder，reset随实体删除。装备卸装只取消同实例待施法，不能影响其它来源。
- Stun/Airborne或技能控制权限失效取消待放；只禁止普通移动但允许该技能时暂停走近，敌人进范围仍可施法。尚未开始的意图与已被打断的前摇区别明确。
- 客户端Enemy点击仅放行严格预检的OutOfRange到统一命令入口，其它非法点击继续保留瞄准；UI同步新order只读标签，无DOM规则补丁。

预期文件：simulation/model、orders、actions、mod、navigation及新cast_orders；equipment通过原取消入口、client/controls、只读order标签和通用目标提示/预览颜色；公开测试和架构/规范。
非目标：改数值、改方向/地点目标载荷、接管服务或移动状态写入客户端。
