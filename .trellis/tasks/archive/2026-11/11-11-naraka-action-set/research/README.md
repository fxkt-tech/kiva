# 永劫动作与音效研究入口

本目录是规划/解析证据，不是已接入的运行时动作包。H0原生Humanoid求值与指层混合尚未通过；游戏代码未改，产品测试/浏览器验收未执行。

优先读：

- animation-assets.md：原状态/override/三段链、通用素材与明确缺口。
- audio-assets.md：17个原事件媒体链、可解码样本和原轨关联。
- humanoid-conversion.md：95标量正确读取与候选骨骼求值的区别，缺native twist/bodyCOM/IK oracle。
- runtime-combat-integration.md：权威三段、具体防御阶段、双rig与实际掌/指、音效生命周期接缝。
- plan-review.md：方案独立复核与主会话处理记录。
- planning-validation.json：本轮实际核查范围，不把规划/数学检查当产品验收。

精简原件证据为animation-state-mapping.json（85桥/61首候选身份；12个鸟铳双名的运行时选择未知）、animation-core-inventory.json（29真实对象）、combo-graph-hd-compact.json、animation-curves-validation.json、finger-layer-values.json、audio_sample_trace.json、audio-tracks-selected.json和audio-measurements.json。

约82MB原配置payload/早期debug已移至/home/asuka/kiva-asset-research/naraka/action-set-2026-11-11，逐件大小/哈希/外部路径见cache-index.json。未删除原客户端或既有研究成果。

## 重跑

使用/home/asuka/kiva-asset-research/naraka/.venv/bin/python，并设置PYTHONDONTWRITEBYTECODE=1。读取原客户端需要该安装仍可访问；原DBBL/AppRes辅助环境位于既有外部研究目录。本目录脚本默认只写本目录，不能把它当未来portable verify工具。

从当前外部缓存复核：

    python map-animation-states.py --source-root /home/asuka/kiva-asset-research/naraka/action-set-2026-11-11
    python audio_tracks.py --source-root /home/asuka/kiva-asset-research/naraka/action-set-2026-11-11

两者已真实运行，统一脚本输出后重复生成的紧凑证据SHA稳定。原proto通用探针probe-animation-proto.py也支持相同--source-root；需要重新提取payload时用probe-animation-assets.py指定精确原.bytes文件名，再传新的缓存目录。

独立原曲线/默认指层/Avatar探针分别为check-animation-curves.py、probe-finger-layers.py、probe-humanoid-avatar.py；候选肌肉数学probe-humanoid-muscle-math.py只能证明研究模型属性，不是完整Mecanim导出器。audio_sample_trace.py可从PCK重提17事件首变体并调用现有vgmstream/ffmpeg；默认只保留两个小Ogg样本，其余临时音频测量后清理。

普通产品--verify与原生--rebake尚待实施；前者未来重打包冻结的已验证locals，后者必须记录实际求值环境。两者未实现或执行，不得混称本目录的研究复跑。
