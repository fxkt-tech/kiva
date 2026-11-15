# 现有角色动画兼容研究

## 已证实事实
- `src/client/appearance.rs` 的默认玩家外观为 `Appearance::Lihuowang`。
- `provenance/characters/lihuowang/README.md` 的“坐标与动作”规定本体高 1.73m、脚底为零、前向游戏 -Z；运行资源为 KIVAAV03。
- `.trellis/spec/client/model-assets.md` 要求运行时只读离线二进制，不引入 GLTF/JSON 解析器；导入参考姿态与 inverse bind 独立保存。
- 同规范规定真实腕/MCP掌框及指链局部 FK 驱动挂点；仅骨名相同并不足以保证兼容，骨轴、参考臂向、骨长与手掌坐标必须验证。
- `.trellis/spec/client/naraka-motion.md` 定义 Naraka 63 骨/44 源片段，另有燕云来源及 CC0 回退。兼容要走共享游戏求值，不能只在 Blender 预览。
- `src/client/hit_bake_tests.rs:40` 当前玩家命中轨迹以李火旺外观烘焙。改变肢体比例可能改变实际显示武器轨迹，必须验证与权威扫掠的一致性。
- `scripts/import-prepared-models.py` 针对冻结 prepared 包与哈希，不可直接覆盖旧源件以导入新原创角色；需要独立离线转换入口或明确复用底层格式工具。

## 建议及待决策
- 推荐新角色以当前默认男主的实际身体骨长、参考姿态、掌框及手指约定为兼容基准，另建原创外观网格；不依赖通用自动绑定直接成功。
- 用户已确认采用现有1.73m及骨架比例，撤销原约1.85m概念尺寸。
- 制作控制骨与运行变形骨分离；第一版无面部表情，不新增面部控制系统。
- Windows Blender 5.3 已连接官方 MCP 插件，独立工作窗口的标准 MCP 握手与工具调用通过。参考骨架已用后台独立文件生成并往返验证，用户原未保存场景保留；详见 blender-connection.md 和 art/wasteland/rig-reference-provenance.md。
