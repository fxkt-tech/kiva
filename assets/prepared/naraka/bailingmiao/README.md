# 白灵淼衣装与莲母：仅资源准备

本目录没有接入 Kiva 外观、WASM、include_bytes 或武器玩法。保留原坐标/单位的三个原蒙皮组件：白灵淼 s16 衣装、莲母 hair_s19、独立通用女性基脸。此包不是已还原专属捏脸的完整运行时角色，也没有 AV03 或新武器。

身份：FashionBody3402139（排除新喜3402140）→ActorTemp1210776→BodySize90005→原 l22_female_skeletonAvatar；ActorVisualPart11602 为 s16。莲母为 HairItem100408 / HairVisual100471。女性也沿用引擎 gMan 骨名，实际矩阵来自女性源，未套男性比例/脸或NPC。

衣装保留16个原可视LOD0实例和各自原FBX骨序（按确切Mesh引用与骨路径CRC验证），空材质模拟代理不显示。莲母原SMR骨列表为空，28槽由原female63骨与原18aux矩阵唯一匹配，7条外部父链接为明确的离线重建，非原序列化骨列表。所有原local SRT、UV、四权重与IBMs保留，当前源姿态可能不同于bind pose。

textures/中的27张原材质PNG、cosmetics/中的3张确切原妆容PNG均保留原尺寸/通道。normal/packed/em/ID/SH/flow/LUT仅保留其真实来源/槽名，未猜作ORM。眼影是通道mask；原iris引用中的male命名保留，不改成李火旺虹膜。专属FacePreset1416403形变和妆容shader组合未应用。莲母染色/SH未恢复：GLB只有明确的中性浅灰alpha预览（derived-preview/白RGB仅留原alpha），不是把ID彩色数据当albedo，也未沿用李火旺黑发/棕眼。原参考图标单独留作源配色对照，不声称取得shader颜色参数。

portable_manifest.json所有实际交付文件均用本目录相对路径与SHA覆盖，验证不需要游戏或原WSL目录。source-evidence/里的原JSON含历史绝对源定位；它们只是来源证据，验证器不读取这些路径。小型原配置record.bin已随包，CAB/GPU压缩数据留在项目外研究目录。

在仓库根目录用有numpy/Pillow的Python执行：

    python scripts/import-bailingmiao.py --verify assets/prepared/naraka/bailingmiao

从原研究成果重新准备：

    python scripts/import-bailingmiao.py --source /path/to/naraka

检验包括实际GLB accessor/四权重/骨索引/IBMs、源层级计算、实际加权骨扰动、每个交付文件SHA和PNG解码。仅离线检查，没有运行游戏、浏览器或截图测试。
