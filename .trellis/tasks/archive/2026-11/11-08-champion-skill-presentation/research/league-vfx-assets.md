# 英雄技能原素材调研（2026-11-08）

状态：只读研究；未改产品代码、规范或运行资源，未启动服务/构建/GPU。以下是实际下载与解析结果，不是仅由文件名推断可用性。

## 结论与推荐边界

原皮皇子与源计划寒冰的 QWER 原素材可得。建议离线选择对应的原 PNG、SCB 静态网格及少量已核对的 emitter 参数，转换为普通 GLB/贴图与小型客户端配方；不实现整个 League 粒子运行时。当前模拟定义、真实载体和场上对象仍决定位置、范围、命中、持续时间和销毁；源素材只决定外观。

实际样本缓存：`/tmp/kiva-vfx-research-xz0f3gqf/`。33 张 PNG 已完整验证各 chunk CRC、zlib 解压、逐行反滤波与 RGBA；9 个 SCB 已完整解析至 EOF，验证索引范围、顶点/UV 有限值并导出带 UV/颜色的标准 glTF 2.0 `.proof.glb`。没有运行 Bevy/GPU，原网格面向/实际 blend 和最终大小仍须实现阶段浏览器确认。

## 固定源与获取方法

1. PNG 和已转为 JSON 的 BIN 来自固定 CommunityDragon `16.22/game/`，不能用移动 `latest`。完整 User-Agent/Referer 与上轮一致。
2. SCB 与链接 BIN 来自 Riot 官方 patch `16.22.8348910`，manifest `6EDAD6B3C9806CB9`：<https://lol.secure.dyn.riotcdn.net/channels/public/releases/6EDAD6B3C9806CB9.manifest>，manifest SHA-256 `ed2ba7e19ddacb18604fd5f61baf87927d4c4514540cec8047cb56d6499cdbcb`。
3. 重用上轮缓存 `bank-index.json` 中完整 WAD entry/chunk 表，以 lowercase 资源路径的 XXH64 定位 entry，仅下载覆盖它的 bundle Range；校验压缩 entry XXH3、解压大小和产物 SHA-256。未下载整个客户端。
4. 本轮 Range/offset/encoding/source SHA 清单：缓存 `riot-vfx-sources.json`。字段与已提交 `scripts/fetch-league-audio.py` 的 `--manifest` 输入兼容，可原样通过该脚本重建 SCB/BIN（不依赖临时缓存）。产品实施时应抽取通用获取器或单独命名 VFX 获取脚本，避免资源层依赖音频模块。
5. PNG URL/原始哈希在缓存 `sample-acquisition.json`、`indicator-acquisition.json`；完整像素验证在 `png-verification.json`。下载程序为 `fetch-samples.py`，解析/标准 GLB 证明为 `validate-samples.py`。这些仅为研究工具，尚未加入仓库构建流程。

### 实际 BIN 解析

本地原二进制由 [CDTB binfile.py](https://raw.githubusercontent.com/CommunityDragon/CDTB/master/cdtb/binfile.py) 解析，辅助名称表来自 CommunityDragon/Data `hashes/lol/hashes.{binfields,bintypes,binentries}.txt`。精确工具与表哈希在缓存 `bin-parser-acquisition.json`。JSON 镜像的真实路径是 `game/data/characters/...`，省略 `data/` 会 404。

- `Ashe-skin8.bin`：324,373 bytes，SHA `c63872b2de8e43a1a510258ccc2a109edd26e06e8e4d81902735a9c55321f891`；58 entry，其中 56 个 VFX，6 个链接 BIN。合并非动画链接后共59个 VFX。
- `JarvanIV-skin0.bin`：9,413 bytes，SHA `79270d5f483ae35ee1f3668370ce89c201e4bf730c61631a77b62ee6431a4f7c`；仅5 entry，其中3个 E 覆盖，10个链接。**仅看 skin0 会误判 Q/W/R 不存在**。
- 皇子完整 Q/W/R 等在 `JarvanIV_Multi_Skins_...` 链接文件中；已全部下载并解析，合并后35个 VFX。长名字链接的 CDragon JSON URL 返回404，但官方 WAD 中均存在且按哈希可准确取出。
- 机器可读合并结果 `Ashe-merged.json`、`JarvanIV-merged.json`；各链接原 BIN/解析 JSON 在同一缓存；精选发射器原值在 `ashe-vfx-extracted-parameters.json`、`jarvan-vfx-extracted-parameters.json`。
- [寒冰8 JSON](https://raw.communitydragon.org/16.22/game/data/characters/ashe/skins/skin8.bin.json)、[皇子0 JSON](https://raw.communitydragon.org/16.22/game/data/characters/jarvaniv/skins/skin0.bin.json)、[皇子技能定义 JSON](https://raw.communitydragon.org/16.22/game/data/characters/jarvaniv/jarvaniv.bin.json) 均实际获取成功。

## 每技能最小选材

| 技能 | 原 VFX 事实与首版选材 | 跟随当前项目的事实 |
| --- | --- | --- |
| 寒冰 Q | `Ashe_Skin08_Q_ready/Q_buf/Q_Buf_attack/Q_mis`；原 Q 标志 mesh+`q_ready_symbol`、原 `q_mis` 箭/数字闪片 | 实际充能/empower；只对真正强化的攻击显示增强箭，不能仅按当前 Q 按键 |
| 寒冰 W | `Ashe_Skin08_W_Cast/W_mis/W_tar`；`w_mis`+`w_glow_mis` 双层箭与稀疏残影 | 实际每支 fan missile 的方向/当前位置/消失；不另造九个视觉碰撞载体 |
| 寒冰 E | `Ashe_Skin08_E_mis/E_tar_explode/E_tar_linger`；原 Drone mesh+`e_asset_md` 图集、`e_scanring` | 当前 Scout/Reveal 真对象，飞行和到达扫描持续均读模拟 |
| 寒冰 R | `Ashe_Skin08_R_mis/R_tar`；原 Arrow_Head mesh+`r_arrowhead`、透明尾迹/地面闪光/命中环 | 真实导弹当前位置/方向/命中或到期；source 20秒仅是粒子上限，不延长导弹寿命 |
| 皇子 Q | `JarvanIV_Base_Q_buff_01/Q_hit_01/Q_crown_01`；金色 `q_streak/q_speedlines`、真实枪模型动作，命中短闪 | 真实提交方向和投递宽/长；EQ 只跟随已有实际位移/旗帜规则 |
| 皇子 W | `JarvanIV_Base_W_nova_01/W_buf_01`；原 RingMesh03 扩散环、`template_shield` 护盾纹理 | 提交时外扩，持续外壳读 shield/状态；破盾不留5.1秒假盾 |
| 皇子 E | `JarvanIV_Base_E_mis_01/E_tar_green/E_tar_red/E_flag_hld_01/E_hit_01`；原 `JarvanIV_weapon.scb`+英雄 atlas，蓝/黄条旗带 | 实际 Standard 的下落/active/remaining；旗带色贴图只是64×64两色，不是完整旗帜模型 |
| 皇子 R | `JarvanIV_Base_R_cas_01/R_tar_01/R_Tar02/R_Wall_rock_01/R_Wall_death_01`；原 Rocks01/02 meshes+匹配岩石贴图，裂地与粉尘 | 岩石摆放和半径跟随真实 Terrain 几何；拆墙与到期读对象消失，不能再造导航障碍 |

源 R 岩石02含整个较大的破地/周边块（源 XZ 约±1000）；不能直接在游戏每个 wall segment 放完整02。建议岩墙周围实例化小块 Rocks01，02只作为按实际半径归一化的一次落地破地层。两者必须在实际截图中检查不盖住交互。

## 指示器原资源与语义

英雄 BIN 中 `mSpell.mClientData.mTargeterDefinitions` 已解析：

- 皇子 Q 为 Range+Line（源 lineWidth70）；E 为 Range+受范围限制 AOE，明确 `assets/spells/textures/aoe.tex`；R 为 Range+目标中心 AOE（源340），明确 `aoe_brand.tex`。
- 寒冰 W 是多条不同 angleOffset 的 Line+Range，R 是 GlobalLine+Range/Minimap；E 是 Range/Minimap；自身技能为 Range。
- 这说明指示器主要是公共 line/aoe 素材配合技能几何，并非每款皮肤都另有一张整技能 PNG。原范围/等级配置只用于识别样式，**当前实际 AbilityDefinition、投递宽度/扇形/半径优先**，不能把LoL源70/340直接替换项目数值。
- 已实际下载/查看：`aoe.png`（E）、`aoe_brand.png`（R）、`circularrangeindicator_empty.png`、`skillshot_rectangle_{base,tip}_r01_v01.png`。固定目录 <https://raw.communitydragon.org/16.22/game/assets/spells/textures/>。
- 推荐：方向技能投影长条/箭头；W展示真实扇区边缘与每支箭的预览方向；点技能落点圆+射程；Enemy显示当前可观察敌人中心圆及接近提示；SelfCast显示自身作用范围。非法/等待接近边界继续服从现有 preview_cast，不由贴图决定合法性。

## 已验证的格式/混合陷阱

- `aoe.png`、`aoe_brand.png` 及寒冰 `r_arrowhead.png` 的像素 alpha 全255。前两张原本就是黑底发光线，不能用普通 Blend 地面方片，否则是黑方块；使用 `AlphaMode::Add` 或专门的mask材质。本地 Bevy0.20.1 `bevy_material/src/alpha.rs` 明确 Add 中黑色不产生效果。不能把原青色RGB直接乘红色作为非法态（会接近黑）；可以保留纹理配独立红色边线，或专用材质按原图亮度着色。
- W箭PNG透明alpha范围0..246；皇子盾0..235；它们可做普通透明或受控Add层。E drone atlas与岩石为不透明表面，应作为mesh贴图。
- 原 source `blendMode` 数字并非 Bevy枚举值；1/3/4/5需显式配方映射。已确认W主要箭发射器为4，R箭头同时有1与4双层，Q箭为5。没有验证完整League blend表，不能声称逐项原生等价；首版选透明主体+Add辉光并截图核对即可。
- SCB源样本为3.2，真实face包含3×u32 indices、64字节材质名、3×u与3×v，共100字节；额外face RGB为9字节/face。LTK Wiki页对应字段长度与实际文件不一致，已对照 [StaticMesh.cs](https://raw.githubusercontent.com/LeagueToolkit/LeagueToolkit/main/src/LeagueToolkit/Core/Mesh/StaticMesh.cs) 与 [StaticMeshFace.cs](https://raw.githubusercontent.com/LeagueToolkit/LeagueToolkit/main/src/LeagueToolkit/Core/Mesh/StaticMeshFace.cs) 验证后完整解析。
- SCB per-vertex BGRA 与face RGB都保留为COLOR_0；R箭头/尾迹存在alpha0顶点，丢掉颜色会把原本淡出的边缘变实心。
- mesh材质名（如lambert8）不等于贴图名，必须以该 emitter 的texture引用绑定。导入器不能按mesh文件名猜PNG。
- 源header bbox部分为零或min+size表达，实际边界应由顶点计算。R箭长轴是源Y，不可直接照搬英雄GLB的+Z约定；配方须明确粒子局部轴→载体前向，并用实拍验证。
- 示例GLB仅把原坐标乘0.01，保留原轴、UV、RGBA及原贴图；尚未加入最终glow材质/源比例动画。源birthScale不是当前Ability范围，不能照抄就认为尺寸正确。

## 最小离线导入和客户端方案建议

1. 建独立 `assets/vfx/{sources,catalog}.json` 和离线脚本。下载校验源PNG/SCB/BIN，选择固定发射器、保留原值来源与SHA；处理结果也固定SHA。日常构建/游戏不下载网络资源。
2. PNG原样发布；SCB展开每face的POSITION/TEXCOORD_0/COLOR_0（保留接缝）导出小GLB。原始SCB parser只存在离线脚本，Bevy运行时不解析League格式；9个proof GLB证明所需格式均可导出。
3. `client::skill_effects` 边界拥有材质/mesh句柄、纯外观目录和固定步采样，不在scene.rs继续加英雄分支。配方按实际 AbilityId 11–14/21–24选择，身份只决定皮肤覆盖，混合装配不按槽位套错效果。
4. 最小显示primitive只需原贴图billboard/ground quad、沿方向trail strip、原SCB静态mesh和少量确定性的短命闪片；支持颜色/尺度曲线及UV帧格可覆盖主要识别形态。不支持的概率噪声、flowmap扭曲、层级子发射器明确简化，不伪称完整原生效果。
5. 瞬发/命中效果来自实际提交/投递事实；持续效果从真实Projectile/Standard/Terrain/Scout/Reveal/Shield绑定。客户端拥有显示寿命上限，不新建伤害/碰撞/计时规则。按来源生命代次、会话、可观察位置裁剪，暂停读PresentationFrame冻结，重选回收子树。
6. 原mesh VFX不带PickGeometry，不参与单位选中；地面透明层只作表现，HUD仍隔离输入。原模型武器/施法动作与音效保持原时钟。
7. 实现验收须真实WebGPU：箭方向、黑底/透明、岩墙大小与实际通路、W扇形几何、暂停、命中/消失、死亡/新生命、隐藏敌方与重选残留；离线结构验证不代替截图。

## 样本清单

下表PNG URL均为 `https://raw.communitydragon.org/16.22/game/` 加对应源路径；原目录分为寒冰skin08、皇子base粒子及公共spells/textures。完整URL可在缓存两个 acquisition JSON 中直接复制。

| PNG | 宽×高 | 原容器SHA-256 |
| --- | --- | --- |
| `aoe.png` | 512×512 | `4135288ceaf767b7a61bc31f7fb30bd079f35a8be03ad732cf621bca13c61efa` |
| `aoe_brand.png` | 512×512 | `1c8f43b05b171cb0261bc8cf44e95d63e6064cc4c4854b8bd991cedc2f3d129d` |
| `ashe_skin08_e_asset_md.png` | 256×256 | `696c859654f5ce8d0d8dbe0fee09c3659eb00b4ad075b55e1bc89a388652d878` |
| `ashe_skin08_e_scanring.png` | 512×512 | `2ae1152cc32cfe01bb718dc38ef1b9773036d43efa2ae567d5fc4c39e8ce4388` |
| `ashe_skin08_q_mis.png` | 512×128 | `f7b7522393a9c9fd796c34fce57b851527fd6d839b5deed15343252010dee926` |
| `ashe_skin08_q_ready_symbol.png` | 512×256 | `fb664d13731f10198e90234fd9c0cc4b9fcdd168b463d8432d921a86aa20404e` |
| `ashe_skin08_r_aoering.png` | 256×512 | `e4b009da983dfa8a22397679c46db12a8560f49957b1702c2ff9fea7d8e5a553` |
| `ashe_skin08_r_arrowhead.png` | 512×256 | `c4fb77a4a8108865a81a84c2a3bd2093174125b044d72fd2c6bb7c0a41b7c491` |
| `ashe_skin08_r_groundstar_flash.png` | 128×128 | `6d6211f3e1a15593c9f9a732374158dcd4a1f97502c21c267b43366fd4389f99` |
| `ashe_skin08_w_glow_mis.png` | 256×128 | `6dd5a6911660cf00aad9e6d334e87dca42af3d578ab8625efcfad5454bbc1b4a` |
| `ashe_skin08_w_mis.png` | 256×128 | `f5ec5e8da4d67ba7c816c74db218481d165adff9597f4812f3953733141d142c` |
| `ashe_skin08_z_cape_bits.png` | 512×256 | `026ff8fee456ceb4b26c407892668c8f2db00b3c9f4105fcf5b0ed40d0d4153f` |
| `circularrangeindicator_empty.png` | 1024×1024 | `2a63de2630794a3fd996aa2650e894098b82043d3b5c54ad32d191053d5b1429` |
| `conicrangeindicator.png` | 512×512 | `5c18c67be3960e405cee5ef55b82213b26d6fb490b90fe59ce3dfc0d50b45cc3` |
| `crosshair.png` | 64×64 | `d22aeebf5b67fdc1cdd7071a9b929e434f55643c4b38241f09823337fd460cd2` |
| `globallinemissilebase.png` | 512×512 | `be72f13262fa5e467045ae5eebfaad29555e3c297cbd21329b0a710960e0bad5` |
| `globallinemissiletarget.png` | 512×512 | `7e7f1e91ebf3864eca7e602fc016a4e90a647652990cdf92504208007132127f` |
| `jarvan_base_r_wall_tupdate.png` | 256×256 | `57f2f26de6dda1b7d95a4a50758222173cd05bef864249f792b6017872ba5132` |
| `jarvan_base_r_wall_tupdate02.png` | 512×512 | `15d23fa51a21e2c3bc4922090d417c44cea466eece353108c2966676f1689e7d` |
| `jarvaniv_base_e_flag.png` | 64×64 | `99edb6902d2d064379b47d7cb3b39d1aefd80795f08812783ec80d2077ee6c8a` |
| `jarvaniv_base_q_globalhit_blue_tar_outerring_p-rgba.png` | 128×128 | `2ac864392c108f5cf9ceb312ab5adc0f8afd6a1a52c5d67bd67f0a83818267f6` |
| `jarvaniv_base_q_speedlines.png` | 256×512 | `5e70f4a760f193c9bb338d702c0f9110c6e5b815c5e369a03c6b781b77d36a7d` |
| `jarvaniv_base_q_streak.png` | 64×128 | `10ca68003238c948936f52ebde528f0bc615668f65ccf769ebfecc77c5cd2563` |
| `jarvaniv_base_r_slicegroundcracks.png` | 512×512 | `aef5b995dfdac81a53288e7f1b85e17f6878329b65ed391a8df07dc5c94b598e` |
| `jarvaniv_base_tx_cm.png` | 512×512 | `334af2a1b0affc6a3b1c3f28fa09464e4e9fb36426518efee3aa8c5e25f5e29a` |
| `jarvaniv_base_w_template_shield.png` | 256×256 | `1fe4424ea9a4cb298be439cfeac1f1cbd42aa6eb5598f9783a17965f1688b406` |
| `jarvaniv_base_w_template_shockwave_blue.png` | 256×256 | `842c962ee7d9d30d0cd76bbbfa1cedcb8e668f33853b28a509ed416c6b0ada13` |
| `locallinemissilebase.png` | 512×512 | `0e998e2530f16b870d19a639f420c8f6199f5ba7b015b7e53f99c6602dbf4747` |
| `locallinemissilebase_gold.png` | 512×512 | `e1a3747ffc897d6bdacecad8886d78016bd3618b8c145caffa8c7268bf4d045d` |
| `locallinemissiletarget.png` | 512×512 | `412d8d66bcbffebeada1d026fea7c68454be5d88df6f58d3dc9a826948e92371` |
| `locallinemissiletarget_gold.png` | 512×512 | `323f505814404407d8d86a648bb948334019e6a398e08d3a933cca91aa3c2efd` |
| `skillshot_rectangle_base_r01_v01.png` | 128×512 | `d3ddcc0efd55ee585825ce6028d61544f1129c71a872c50e573a17e7b4d06f9b` |
| `skillshot_rectangle_tip_r01_v01.png` | 512×512 | `0c62c69bc8cc71e6f1e1e9bbb46ad804873921046463d1ff203a6bc4e8839d3b` |

### 官方WAD静态网格

源路径在 `ASSETS/Characters/{Ashe/Skins/skin08,JarvanIV/Skins/Base}/Particles/` 下；每项精确bundle区间/decoded SHA见缓存 `riot-vfx-sources.json`。已有标准GLB样本都在同一缓存，后缀 `.proof.glb`。

| SCB | 顶点/三角 | 原SCB SHA-256 |
| --- | --- | --- |
| `Ashe_Skin08_E_Drone.scb` | 372/680 | `f8a342948bf6ec246af110c87cf729e39bc39c84376514a8c282294903cb9ef3` |
| `Ashe_Skin08_Q_ready_Symbol.scb` | 40/80 | `cb8bc7de31118e53d895251d6df9fc159923219a9cc8792031c70e62f8a0d210` |
| `Ashe_Skin08_R_AOEring_Tar.scb` | 90/140 | `0bf1e1d226e17e3e4278d499741aeace20ce2e50465a2ef8c1ad1680f9c54cff` |
| `Ashe_Skin08_R_Arrow_Head.scb` | 160/238 | `8707dda699e48872fefea0ffd3bf3fb9909a1e70ea2372f9a73bee453b31d39b` |
| `Ashe_Skin08_R_Arrow_Trail.scb` | 8/6 | `c107c33cd58cd1b0224fab2caf19adbe14fab7874b48babe099538f2b3c79dc7` |
| `JarvanIV_Base_W_RingMesh03.scb` | 181/330 | `267c5cf915d0fe9704e06a23c5fa93a1cbc69693e47cbec1bfe6a8b4c92daa8e` |
| `JarvanIV_weapon.scb` | 641/1230 | `63bdf2a8d82b4d72f4d36023f79ddd532d61729def01f5c4e62e691becf35ad8` |
| `Jarvin_Base_R_CatacylsmRocks01.scb` | 78/135 | `e1b36c73684a57d1c16e326b0dde69e6f62763280a71322b8c03817afc8d74be` |
| `Jarvin_Base_R_CatacylsmRocks02.scb` | 1246/1380 | `66ae5fdf62afac5d4522f27b1c27897adb04d45b31a24356ec13d5fc7ec173a9` |

复现研究验证（只读研究工具，并非已实现的生产导入入口）：

```sh
python3 -B /tmp/kiva-vfx-research-xz0f3gqf/validate-samples.py
# 完整解码33张PNG；9个SCB解析至EOF并导出标准GLB。
```

尚待实现阶段验证：GLB通过真实Bevy装配、源mesh前向与贴图V方向的实拍、最终Add/Blend层和场景亮度、实际生命周期/可见性、资源发布清单以及浏览器验收。

## 冻结给两个 Renderer 的共享资源接口建议

以下是经研究收敛的**待实现接口与固定目录映射**，不属于当前已实现规范。产品写入必须等子任务进入实施。

```rust
// client::skill_assets：只拥有资产与外观元数据，不读写战斗规则。
pub(super) struct SkillAssetsPlugin;
pub(super) struct SkillAssets;
impl SkillAssets {
    pub fn image(&self, key: TextureKey) -> Option<&Handle<Image>>;
    pub fn mesh(&self, key: MeshKey) -> Option<&Handle<Mesh>>;
    pub fn material(&self, key: TextureKey) -> Option<&Handle<StandardMaterial>>;
    pub fn status(&self) -> &AssetStatus;
}
pub(super) fn texture_spec(key: TextureKey) -> &'static TextureSpec;
pub(super) fn mesh_spec(key: MeshKey) -> &'static MeshSpec;
// AssetStatus { ready: bool, errors: Vec<String> }。
// TextureSpec { path, blend, atlas, sprite_forward, source_system, source_emitter }。
// AtlasSpec { columns: u8, rows: u8, frames: u8, default_frame: u8 }。
// MeshSpec { path, texture: TextureKey, source_bounds, bounds, source_to_world,
//            source_to_game, forward: Option<Vec3>, up: Vec3 }。
// bounds 指实际已导入网格坐标；source_bounds 为SCB原始坐标。
```

`TextureKey`/`MeshKey` 为 Copy+Eq+Hash 的封闭枚举，下表就是完整首版键集。路径均为 AssetServer 相对 `assets/` 的路径。
`material(key)` 是按此表混合、图集初始帧构造的共享只读模板，Renderer 不得直接改变它影响所有实例；需要各自透明度或颜色时复制模板到受生命周期管理的实例材质。
Default frame 仅选择清晰静态帧，不伪造源文件没有给出的 frameRate。需要变帧时按 metadata 选择原帧，并由模拟表现时钟驱动。
非均匀 mesh UV atlas（无人机/英雄身体/箭头）不切成均匀格；atlas 设1×1，保留原mesh UV。

### 固定 TextureKey → 原图 → 产物与材质

| TextureKey | 原PNG basename | 产物路径（`vfx/textures/`下） | 默认Blend | uniform grid /帧 | Sprite正向 |
| --- | --- | --- | --- | --- | --- |
| AsheQArrow | ashe_skin08_q_mis.png | ashe-q-arrow.png | Add | 1×1 /0 | 图像右 |
| AsheQSymbol | ashe_skin08_q_ready_symbol.png | ashe-q-symbol.png | Blend | 2×1 /0..1，默认0 | 无；mesh UV |
| AsheVolleyArrow | ashe_skin08_w_mis.png | ashe-volley-arrow.png | Add | 1×1 /0 | 图像右 |
| AsheVolleyGlow | ashe_skin08_w_glow_mis.png | ashe-volley-glow.png | Add | 1×1 /0 | 图像右 |
| AsheScoutAtlas | ashe_skin08_e_asset_md.png | ashe-scout-atlas.png | Opaque | 非均匀mesh atlas，1×1 /0 | 无；mesh UV |
| AsheScanRing | ashe_skin08_e_scanring.png | ashe-scan-ring.png | Add | 1×1 /0 | 无；地面 |
| AsheCrystalArrow | ashe_skin08_r_arrowhead.png | ashe-crystal-arrow.png | Blend | 非均匀mesh atlas，1×1 /0 | 无；mesh UV |
| AsheCrystalTrail | ashe_skin08_z_cape_bits.png | ashe-crystal-trail.png | Add | 所选ArrowTrail1是1×1 /0 | 无；mesh UV |
| AsheImpactRing | ashe_skin08_r_aoering.png | ashe-impact-ring.png | Add | 1×1 /0 | 无；必须原环mesh，不能方片 |
| AsheImpactFlash | ashe_skin08_r_groundstar_flash.png | ashe-impact-flash.png | Add | 1×1 /0 | 无；地面 |
| JarvanThrustStreak | jarvaniv_base_q_streak.png | jarvan-thrust-streak.png | Add | 1×1 /0 | 图像上 |
| JarvanShield | jarvaniv_base_w_template_shield.png | jarvan-shield.png | Add | 1×1 /0 | 无；billboard |
| JarvanNovaRing | jarvaniv_base_q_globalhit_blue_tar_outerring_p-rgba.png | jarvan-nova-ring.png | Add | 1×1 /0 | 无；原RingMesh UV |
| JarvanStandardAtlas | jarvaniv_base_tx_cm.png | jarvan-standard-atlas.png | Opaque | 非均匀mesh atlas，1×1 /0 | 无；mesh UV |
| JarvanFlag | jarvaniv_base_e_flag.png | jarvan-flag.png | Blend | 1×1 /0，两个颜色不是两帧 | 无；trail/cloth |
| JarvanRockAtlas | jarvan_base_r_wall_tupdate02.png | jarvan-rock-atlas.png | Opaque | 2×2 /0..3，默认0 | 无；mesh UV+格变换 |
| JarvanGroundAtlas | jarvan_base_r_wall_tupdate.png | jarvan-ground-atlas.png | Opaque | 非均匀mesh atlas，1×1 /0 | 无；mesh UV |
| JarvanCracks | jarvaniv_base_r_slicegroundcracks.png | jarvan-cracks.png | Add | 1×1 /0 | 无；地面 |
| IndicatorAoe | aoe.png | indicator-aoe.png | Add | 1×1 /0 | 无；圆 |
| IndicatorTarget | aoe_brand.png | indicator-target.png | Add | 1×1 /0 | 无；圆/目标 |
| IndicatorRange | circularrangeindicator_empty.png | indicator-range.png | Add | 1×1 /0 | 无；圆 |
| IndicatorLineBody | skillshot_rectangle_base_r01_v01.png | indicator-line-body.png | Add | 1×1 /0 | 图像上 |
| IndicatorLineHead | skillshot_rectangle_tip_r01_v01.png | indicator-line-head.png | Add | 1×1 /0 | 图像上 |
| IndicatorFan | conicrangeindicator.png | indicator-fan.png | Add | 1×1 /0 | 尖端在下，瞄准方向向图像上 |

这里 Blend 是选择的Bevy表现配方，不宣称与每个League数字blend完全一一对应；原数字/发射器在源元数据保留。AsheCrystalArrow 需要增强辉光时可用同mesh/texture的Add实例层，但不能丢掉原vertex alpha。

图集上下文已逐一核对：Q符号取 `Ashe_Skin08_Q_ready/MeshSymbol` 的2×1；岩石取 `JarvanIV_Base_R_Wall_rock_01/rock1` 的2×2。不能把同一文件在其它发射器中的texDiv挪过来（例如cape bits另有12×1.5的UV重复用途，并非该ArrowTrail1的帧格）。

公共6键已全部实际取图/目检。`quartercircle.png` 在16.22已404，正确可用扇形资源是 `conicrangeindicator.png`（原图可见约60°扇形、起点在下）。实现可把它映射到由实际spread_angle构成的扇区网格，或以圆环贴图配边线明确裁切；不能为了原图角度改玩法。
另已取得原版 `crosshair`、`locallinemissile{base,target}`、对应gold版、`globallinemissile{base,target}`候选，位于缓存，不列入首版键集。需要单独金色超距样式时有原图可用，不必直接把青色乘黄色/红色。

### 固定 MeshKey → SCB → 产物与轴

导入每个SCB先保留原UV、面/顶点颜色，再乘 `source_to_world=0.01`（本项目100 League距离单位=1世界单位）；尺寸合法性不依赖这个比例，Renderer仍以实际投递宽度、对象半径/高度做fit。
R箭/尾迹源尖端为+Y，离线转换 `Rx(-π/2)`，即 `(x,y,z)→(x,z,-y)`，使实际前向变为游戏-Z、厚度变成Y。其余首版网格保留源Y-up，中心/高度由bounds显式适配；E源粒子还有92°预转与粒子速度框架组合，本方案保留几何平面Y-up并在浏览器确认无人机姿态，不宣称完整还原其复杂局部框架。

| MeshKey | 原SCB | 产物路径（`vfx/meshes/`下） | 默认TextureKey | 转换后轴 |
| --- | --- | --- | --- | --- |
| AsheQSymbol | Ashe_Skin08_Q_ready_Symbol | ashe-q-symbol.glb | AsheQSymbol | Y-up；无纵向前向 |
| AsheCrystalArrow | Ashe_Skin08_R_Arrow_Head | ashe-crystal-arrow.glb | AsheCrystalArrow | -Z-forward，Y-up |
| AsheCrystalTrail | Ashe_Skin08_R_Arrow_Trail | ashe-crystal-trail.glb | AsheCrystalTrail | -Z-forward，Y-up |
| AsheScout | Ashe_Skin08_E_Drone | ashe-scout.glb | AsheScoutAtlas | Y-up；按载体方向-Yaw旋转 |
| AsheImpactRing | Ashe_Skin08_R_AOEring_Tar | ashe-impact-ring.glb | AsheImpactRing | XZ地面，Y-up；源本身是偏心环弧 |
| JarvanStandard | JarvanIV_weapon | jarvan-standard.glb | JarvanStandardAtlas | +Y竖向，原E发射器用Rxπ翻转 |
| JarvanRock | Jarvin_Base_R_CatacylsmRocks01 | jarvan-rock.glb | JarvanRockAtlas | Y-up；源中心在中段 |
| JarvanImpactGround | Jarvin_Base_R_CatacylsmRocks02 | jarvan-impact-ground.glb | JarvanGroundAtlas | XZ地面，Y-up |
| JarvanNovaRing | JarvanIV_Base_W_RingMesh03 | jarvan-nova-ring.glb | JarvanNovaRing | XZ地面，Y-up |

下列为实际原SCB的min/max，绝不是header推测；最终`mesh_spec(key).bounds`必须提供转换后的精确值，防止renderer按名字猜大小。`source_bounds`可原样保存此表。

| MeshKey | source min | source max |
| --- | --- | --- |
| AsheQSymbol | `(-16.565769,-1.020000,-14.225450)` | `(16.371861,3.011034,10.007669)` |
| JarvanStandard | `(-20.895201,-151.201996,-6.708000)` | `(20.895201,151.201996,6.708000)` |
| AsheImpactRing | `(-171.808151,0.000000,-113.279869)` | `(-40.365711,0.000000,33.517910)` |
| AsheScout | `(-74.169395,-26.283852,-53.850632)` | `(74.169411,26.483850,53.850632)` |
| AsheCrystalTrail | `(-19.593199,-113.945686,1.575771)` | `(19.593199,-44.245415,1.575771)` |
| JarvanRock | `(-144.862091,-217.546814,-164.919037)` | `(144.862091,217.546814,164.918991)` |
| AsheCrystalArrow | `(-10.979300,-41.631317,-12.723736)` | `(10.687950,73.050781,10.524324)` |
| JarvanImpactGround | `(-1026.603027,-144.233032,-985.299377)` | `(1012.344971,63.277184,1033.735962)` |
| JarvanNovaRing | `(-153.061798,-0.000601,-152.223526)` | `(153.061905,0.000745,152.223526)` |

资源status只表示本地加载/校验状态，不决定技能合法性。网络缺图/缺mesh必须给明确一次诊断，显示renderer保留可理解回退；不能把未ready当空指令或让VFX失败阻断玩法。

## 实施中补充：PNG运输层不能按容器字节认图

空缓存复现实际遇到Cloudflare Polish：Q PNG的响应头明确 `cf-polished: ok, orig_size=51822`，容器变45043字节而完整RGBA相同；Q标志另有387个alpha=0像素的RGB从(0,1,2)清为(0,0,0)，非透明RGB和alpha均不变。它还会将全不透明RGBA无损转为RGB PNG。

已据此在离线导入器统一PNG容器编码，保留原容器SHA、原RGBA SHA以及“只将alpha=0下RGB归零”的严格像素指纹。所有alpha及非透明RGB发生变化均拒绝，PNG CRC/长度/解压/过滤均验证；SCB/BIN继续严格字节SHA。产品图没有缩放或换色，运输重压缩不再造成虚假的源版本变化。

最终本地包选24张PNG与9个GLB，catalog/parameters另计；所有最终输出SHA、元数据、源文件及五个选定BIN来源见 `assets/vfx/sources.json`。`scripts/import-league-vfx.py` 无参数离线验证，`--source-dir`精确重建，`--download`从固定源取素材。资源模块已按上述固定API落地，最终编译/真实浏览器仍由父任务完成。
