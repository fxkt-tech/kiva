# 独立全范围检查

结论：未发现阻塞项，无需修改产品源码。检查覆盖simulation/client变更及其gameplay调用者、测试、脚本与规范，不仅是最后一处增量。

## 已核对
- 目标语义统一按Enemy：Direction载荷、Point严格射程/最大距离规则、数值与AI候选未改动；未添加英雄/技能ID分支。
- orders统一转入cast_orders，preview_cast仍严格且只读；客户端仅放行OutOfRange，错误类型/隐藏目标/结构等仍保留aim。
- CastTarget保存实际来源、技能和目标生命；请求先完成校验与可达预检才替换目的，非法新指令不破坏原路线/动作。
- 待放每tick复核视野/生命/来源，移动目标按节流重规划。到范围前没有费用/CD，清一次性目的后才转入原begin/commit，前摇失效和控制中断不回放。
- Root区分移动禁用与技能许可；Stun/Airborne、死亡/重生、失去视野、来源失效、暂停/reset和装备同实例卸装的清理路径齐全。
- 共享A*只增加到达区域模式，普通path/path_static的目标与回退政策保持；区域终点和通路共用当前静态/临时几何，敌人中心不可达也可从有效施法位置释放。
- 模拟唯一写规则和UnitPose，客户端只提交意图并投影新order/提示/颜色。新浏览器断言读取真实Approaching cast、无提前扣费/CD以及一次点击后的实际R围墙；E最大距离断言保留。
- 公共回归覆盖真实移动/目标、几何、取消/保留、来源、生命/视野和非指向性；客户端两条点击回归从红到绿。最终主代理报告make check 299项通过、WASM Clippy通过。

## 本次独立验证
- git diff --check通过。
- node --check scripts/browser-champions.cjs通过。
- 最终检查时88个运行时源码与/tmp/kiva-cast-approach-source-before.json一致；检查者未修改运行时或占用GPU。

主代理继续执行最终有窗口浏览器回归；本静态/跨层检查不替代浏览器实测记录。

## Web实测后两处增量复核

初次检查对应的是首版源码身份；Web实测进一步发现了两个需要定向反馈循环的时间/运动交互缺口，原初次通过不覆盖它们。

1. 进入范围后在advance_actions之后才begin，导致零前摇额外等一tick，逃跑目标可在这一步离开范围。公开`zero_windup_queued_cast_commits_on_boundary_entry_before_target_moves_again`已先RED（提交0而非1），修复后GREEN，日志`/tmp/kiva-cast-zero-windup-{red,green}.log`。
2. 导航终点恰停在旧位置的射程边缘，连续逃跑目标移动后，即使施法者更快也可能始终追不到。公开`faster_caster_can_close_range_on_a_continuously_fleeing_enemy`在修复调度后仍RED；日志`/tmp/kiva-cast-fleeing-red.log`。修复为保留原可达区域端点，并追加从该点朝目标中心的Nav.sweep安全延续；不缩小实际施法范围，不要求中心可达，也不替换旧端点造成切角。

独立增量复核确认：
- pending推进移到advance_actions之前，与直接指令共享开始时的来源/资源/CD校验及首个动作tick；零前摇当步提交，正前摇也没有额外tick。
- 资源、冷却和攻击间隔仍只在原actions阶段推进；待放在Ready下存续，开始前清成Idle，普通orders不会抢写路线或触发普攻，已开始前摇的中断/超距不重试。
- swept延续只在原安全区域点之后append，不抹去原路线，不跨静态/临时墙；实际范围仍逐tick按完整range校验，较快施法者可进入持续移动的范围边界。
- 现有移动/攻击/Stop/新技能替换、非法输入保留、生命/视野/装备、控制、暂停/reset与区域/NoPath契约保持。
- `/tmp/kiva-cast-approach-timing-final.log`的19项公开回归通过，包含两条新RED→GREEN及此前正前摇逃离、障碍与中心不可达场景。

增量检查无其它阻塞项，未修改产品源码。主代理将重新冻结构建身份、运行质量门及浏览器验证；首版88文件manifest不作为增量后构建身份。

## 真实控制与浏览器夹具增量复核

`/tmp/kiva-cast-browser-kit-trace.log`明确记录：tick474为超距Approaching cast，未扣R费用/CD；tick584出现真实Stun、Controlled反馈，目的转Idle；控制结束后也没有自动重放。日志未显示DeviceLost或应用退出。这次失败不能解释为新的施法提交缺陷，反而符合“控制打断取消且不重试”的契约。

独立复核最终browser-champions脚本调整：
- 在测试R之前停在范围外，先观察真实Stun及can_cast/can_dash恢复；此准备阶段没有提交玩家R，因此不存在取消后隐式重试。
- 随后通过正常捕获输入只提交一次新的敌方英雄R点击；无循环重发、重开绕过、AI/HP/World写入。
- 仍硬性断言实际Approaching cast距离大于完整R射程、零CD/未扣费用、退出aim、实际位移、真实Terrain及免费recast；保留小兵非法目标与E最大距离断言。
- 放宽准备位置最小间隙至range+0.1并未放宽核心超距断言，若点击时已经在范围内将无法产生pending记录，测试仍失败。
- 每状态转换记录仅为只读诊断；pageerror与任何console级别的Quitting立即失败，console.error/Rust WARN/ERROR均记录，不吞失败、不自动retry。
- node --check scripts/browser-champions.cjs通过。本次为脚本语义复核，最终实机通过情况由主代理单独记录。

此次脚本调整不弱化原断言，不改变生产技能或控制规则。检查者未改产品代码、未运行GPU/用户服务。

## 最终夹具：观察真实资源窗口

上一节“等待实际Stun”的准备方式随后被实测否定：`/tmp/kiva-cast-browser-kit-v2.log`在tick4487仍等待眩晕，玩家640HP、可见寒冰MP约61.78。真实大箭可以射失，不能把“必定中箭”作为测试开始条件。该超时不是通过记录。

最终脚本仅调整准备：
- 复用正常P装备管理穿6件ID4纯防御装备；通用helper断言最大生命保持装备前值、护甲恰增加150。寒冰基础26时仍得到176，不改变HP、伤害、AI、法力或World。
- 观察当前可见敌英雄MP<=75，以及玩家can_cast/can_dash、仍超出R完整射程的窗口，然后才提交一次新的R点击。此资源窗口用于避免短追近过程被100法力的大箭打断，不是假定隐藏冷却或修改资源；若仍被打断，原取消规则和测试失败仍生效。
- 核心断言完全保留：实际pending距离>完整射程、零CD/未扣费、退出aim、真实位移、真实围墙与免费recast；没有自动retry、重开、吞错误或放宽结果。
- trace和console诊断/退出硬失败继续保留；新脚本node --check通过。

本次只有浏览器夹具变化，v2产品源码不变；301项质量门、构建身份和最终浏览器pipeline由主代理记录。此准备方式替代上一节等待眩晕的夹具，不覆盖或抹去之前失败证据。

## 最终诊断监听器与分英雄执行复核

`/tmp/kiva-cast-browser-kit-final.log`已打印完整Jarvan阶段PASS；首次待放证据为tick3643、距离10.820806（R射程6.5）、mana300/CD0/aim=null，之后真实围墙与recast全部断言完成。该次整个进程随后因测试诊断监听器跨入Selecting而访问空abilities[3]报TypeError，退出1；因此只记录Jarvan阶段通过，不能标成整套通过。

最终脚本修复只在首次记录Approaching cast后注销recordApproach监听器，避免一次性诊断越过其所属阶段。未取消任何产品状态或功能断言，也未吞掉pageerror；后续失败仍会被原guard捕获。诊断trace只保留至首次pending，后续功能结果仍由原硬断言验证。

新增可选KIVA_CHAMPION_HERO=1/2仅为明确分段执行：默认仍两英雄全跑；非法值必报错，不存在零场景静默通过。仅寒冰时先正常进入该角色，再执行原fresh及完整Ashe断言，未删减层数/方向W/侦察/R等行为检查。

node --check通过。后续Ashe与smoke重跑结果由主代理另记，不能用上述退出1日志推断它们已通过。此次仍仅修改测试脚本，301项native及v2产品构建身份不变。
