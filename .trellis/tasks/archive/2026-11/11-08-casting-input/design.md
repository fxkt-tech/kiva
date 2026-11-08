# 设计与改动边界

当前差距：键盘只设置Aim，且begin_cast只清Movement.route，UnitOrder仍可在技能结束后重新寻路。自主位移也保留此前追击目的。

## 客户端

- controls/casting.rs保存客户端CastMode与按事件顺序更新的Shift状态，不在模拟中增加输入模式。
- web/casting.js只拥有本地偏好与菜单选择，data-cast-mode作为封闭客户端设置；web.rs读取设置，controls镜像到客户端状态。
- web/pointer.js仍是唯一CSS坐标所有者，为QWER/1–6的可信非重复keydown保存坐标和UI手势归属；独立FIFO与鼠标快照共用清理边界。web.rs仅解码，controls按原WindowEvent顺序更新镜像并解析实际候选。
- 普通入口select_cast保留图标操作。键盘入口根据模式直接调用共享的try_cast_at_pointer，和普通左键复用ability_targeting::resolve_intent与submit_cast。智能技能失败不遗留瞄准；不会自动选择附近其它目标。
- index.html增加两项模式选择与快捷键说明；style.css只处理设置布局。默认智能，Shift临时普通；不实现按住指示器/松键释放模式。
- 既有浏览器fixture通过真实菜单明确选择普通模式，保留原左键确认场景；本轮只调整fixture并做语法检查，不运行浏览器。

## 模拟

- actions::begin_cast仍只清路线暂停走路，commit_cast在全部校验/快照成功后（重施除外）终止旧地面目的。这样提交期InvalidDamage/失效目标等拒绝仍保留原有效指令；成功后的AttackTarget/AutoAttack继续保留普通技能与持续普攻的组合。CastTarget仍由cast_orders管理。
- movement::start_dash/start_compound为自主运动的唯一开始入口，在提交时替换旧UnitOrder并清普通路线。Dash/Leap用Idle，Link按用户追加需求建立新的AutoAttack，从落点重新索敌，普通Q不会误取消普攻。
- 不用客户端Stop作为补丁，不按角色ID分支；Enemy走近和装备统一受益。强制位移start_forced保持既有规则。

不扩展技能队列、智能选敌/瞄准辅助、单技能配置、键位编辑器或战斗系统重构。

快速EQ追加方案见[eq-followup.md](eq-followup.md)。瞬发后的输入复用原CommandQueue FIFO，不引入独立技能缓冲资源。
