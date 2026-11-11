# 左键三段连击权威规则设计

独立私有MeleeChain管理后继，与技能ComboWindow分开。MeleeStage是域类型，ActionSnapshot/HitContext/CombatEvent复制接受时身份。prepare接受/推进，advance完成末marker后retire，下一固定步衔接。防御阶段时钟留在DefenseState；运动cue从接受/物理publish生成只读单调有界反馈。

## 边界与兼容

src/simulation/{abilities,combat,weapons,defense,actions,physics,boss,mod}.rs及相应测试/exports；boss.rs仅接实际受伤清未承诺后继，physics.rs仅接publish运动cue；controls由表现任务拥有，本任务只交付取消锁存API。

依赖父design固定的Kiva规则与runtime-combat-integration研究；可以与离线资产并行，禁止import客户端clip/WEM名。表现任务依赖本任务公开只读接口。

完整数据流、三段/防御/音效及回滚约定以父任务design.md为准。离线源路径不得进入运行时，当前请求不包含提交与推送。未知原始语义不能补造。
