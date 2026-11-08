# 首轮GPU截图的源素材核对

本轮先只读对照真实截图、原SCB与已解析原BIN，随后按root明确授权修正军旗矛的导入策略。
未运行Cargo/Trunk/GPU、未操作服务或git；W运行时由独立checker负责。

## W 地面白楔

截图：`/tmp/kiva-browser-tests/artifacts/skill-visuals/jarvan-w-shield-1280x800-dpr1.png`。

原 `JarvanIV_Base_W_nova_01/shockwave2` 明确包含：

- `uvRotation.constantValue=90.0`。
- `birthUvScrollRate.constantValue=(0.01999999955,0)`。
- `Color` RGB约`(.80392,.68627,1)`，alpha峰值仅`.15686275`，生命周期两端alpha为0。

对原 `JarvanIV_Base_W_RingMesh03.scb` 实测：U沿圆周0..1；V沿半径，中心约`.0180846`、外缘`.9918887`。
该贴图是沿U从亮到暗的灰阶坡度。若漏掉原UV旋转，则会沿圆周变亮，直接形成截图中的亮扇区；
绕UV中心旋转+90°后`U'=1−V`，才能得到径向亮边。当前默认白色burst强度也明显超过原alpha峰值。
这两项原参数已补入`assets/vfx/parameters.json`，仍对应原BIN SHA；不改图片或这个网格几何。

## E 标准矛全黑

截图：`/tmp/kiva-browser-tests/artifacts/skill-visuals/jarvan-e-standard-1280x800-dpr1.png`。

原 `JarvanIV_weapon.scb` 带旧`HAS_VCP` flag=1：

- 641个vertex RGBA全部`(1,1,1,1)`。
- 1230三角的3690个face-corner RGB全部`(0,0,0)`。
- 原 `JarvanIV_Base_E_tar_green/Weapon` 使用有色hero atlas与blend3，不能将该未使用的旧face-lighting通道作为PBR基色乘数。

原导入将face RGB乘入COLOR_0，因而输出整矛黑色；UV或源atlas本身并未变黑。
root授权后，`JarvanStandard`配方显式选择`ignore_zero_legacy_vcp`。导入器先确认旧VCP确实全0才忽略它，
遇到新增非0face色会拒绝，避免将新作者颜色默默删除。其余8个mesh都为flag2、没有此零VCP问题，保持原策略。

离线证据：新旧Standard POSITION/NORMAL/TEXCOORD_0逐字节相同；alpha逐项相同；
只有COLOR_0 RGB从0恢复为源vertex白色，原SCB来源SHA保留。
新GLB SHA为`600194707bee528f570c7b7ffdcf40d99077e35f02b9a6e29cc9df9e409477d2`。
原生GLB测试增加Standard实际RGB非黑及完整UV范围断言；另外直接变更源VCP为非0的负例已验证会被拒绝。

## 持续护盾未见

原图与元数据正确。生产`jarvan-shield.png`完整解码结果：256×256，alpha范围0..235，
46,473像素非透明、1,842像素alpha≥128，最大RGB×alpha/255为221.8，具备明确可见环。
原 `JarvanIV_Base_W_buf_01/outerring` 使用这张图、blend4和原偏移`(0,80,0)`。

因此不能以空图/全透明解释持续盾缺失。运行时还应核对实际live shield是否存在、billboard世界中心/平面朝向和遮挡；
当前layer的局部Y偏移也被group的camera旋转带走，与原沿世界Y的偏移语义不同。此部分只读建议已交root/checker，未修改renderer。

## 最终交付状态

35产物哈希验证及当前manifest完整内存重建通过；只变更Standard mesh产物和补充的参数来源记录。
42文件快照`/tmp/kiva-vfx-final-files.json` canonical SHA：
`126abbaaf50375f92eec0989cd32b06afb9fea7f3f60364788b6b8922a1bb20b`。
资产与运行代码再次冻结，最终GPU重拍由root负责。
