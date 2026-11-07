# 技术设计

## 变化边界
差距是角色只有胶囊和简单后摇倾斜，目标是旧act样式的骨骼火柴人与连贯动作。真实行为归属client角色表现。
预期新增src/client/character/模块（骨骼数据采样、分段模型附着、模拟状态到姿态适配及测试），修改client/mod.rs注册系统，scene.rs接入模型生成并去掉胶囊和旧倾斜。
恢复assets/animations/{stickman.anim,LICENSE.txt,README.md}与scripts/import-animations.py，更新文档与client规范。不修改simulation或输入行为。

## 数据与职责
- 资源层从act只导入内嵌骨骼二进制和可重建导入脚本。可信资源可以启动时校验失败即报错；边界验证应覆盖父序、时长/帧数、有限TRS、根平移、完整消费及所需骨骼/动作。
- 采样层维持局部TRS与父序变换，四元数插值；统一+Z源到-Z游戏朝向与1.1比例，禁止二次root motion。
- 模型层每个角色一个表现容器，所有分段、头眼、剑socket都在该容器下，按骨骼位置/旋转更新。模型与全局根分离，底座留在scene。
- 动作适配只读UnitAction、UnitPose、UnitState、UnitStatuses、ControlState、AbilityCatalog及已有只读配置。移动来自实际相邻模拟位置差，不能仅凭MoveTo命令断言在走；冲刺/强制位移采用真实运动和执行状态区分（无须泄露私有Movement）。
- 普攻准备缓存本次首次观察的remaining，后摇直接读取权威duration，不按当前攻速重算已有攻击；前摇末段连续挥到CONTACT，后摇跨clip接缝保持骨骼/socket连续。Casting读取实际ability定义，不按技能ID。
- 独立MessageReader按顺序读取AbilityCommitted、移动、死亡和重生通知，识别未经过可见Casting帧的瞬发/装备冲刺；记忆只用于选择姿态，不驱动或延长运动。
- 可缓存本次生命的当前局部/模型姿态与过渡起点；过渡平滑不能延迟或触发伤害，零/短前摇安全处理。取消/状态改变过渡回待机/移动，连续攻击不能倒播。
- 暂停直接保留最后显示骨骼与播放缓存；life_ref变化清除时钟/动作记忆。隐藏死亡根即可，不新增尸体规则。

## 兼容性
root继续由scene::sync_units插值和旋转，击飞高度只在根应用一次；Pickable及标签依旧读同一个显示根。根据实际骨架高度验证现有2.2标签锚点和2高拾取范围，无必要不改公共投影。
不恢复act::Fighter/Attack/Controls，不引入角色/装备特例或第二套动作规则。旧后摇模型测试迁移到新骨骼实际变换并保留暂停断言。

## 验证与回退
单元测试验证真实骨骼、socket、状态时序、暂停、死亡重生和清理。项目全量检查守住现有玩法；临时Trunk构建、自有HTTP fixture与独立Xvfb验证实际WebGPU截图和回归，结束清理自有进程。
如出现表现回归，可以只撤回本任务client/assets/docs改动；模拟数据与存档无迁移。
