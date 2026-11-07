# act 参考与边界

固定参考提交：bd3e3443cfb2922c5cbf6a682cb75bb1091fea12。

- src/game/skeleton.rs：KIVAAN01解码、插值、model_pose与原生/WASM内嵌；可以复用独立计算，不照搬宽松读取和未检查输入。
- src/game/player.rs：17骨段+14关节、头眼、右手socket与剑；只复用几何附着，不引入Fighter/EnemyAi/SwordTip碰撞。
- src/game/animation.rs：旧状态机不适用于当前MOBA模拟，只参考姿态过渡。
- src/game/animation/attacks.rs：旧专有连招编舞不全量搬入；如源手腕朝向不足，可以提取必要的简单guard/手附着算法，避免恢复旧攻击规则。
- assets/animations/stickman.anim：578598字节，23骨骼19clip，CC0；保留LICENSE与原作者信息，README改为当前路径及当前功能。
- scripts/import-animations.py：标准库烘焙，方便重建，不能从网络拉取或付费资源。

当前公开模拟信息：UnitPose::interpolated(0/1)可得实际步位移；UnitAction带普攻tick与施法秒剩余；UnitState::life_ref(entity)能识别重生；ControlState::forced_motion、UnitStatuses::airborne_height与状态种类可用于受控姿态；AbilityCatalog可识别执行类型。
不得为动画公开私有Movement或可写模拟组件。若确实缺少必要只读信息，应先向主代理说明而非加旁路。
