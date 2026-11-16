# 披风精修最终独立检查

## 结论
本轮披风制作源、运行LOD、贴图和严格GLB/AV04检查通过，无新的交付阻断项。真实美术形体/黑灰材质/宽幅焰羽与游戏GPU画面由主会话审阅，记录见cape-detail-gpu-acceptance.md；本报告不以面数代替美术质量，也不声称实时布料或全局零交叉。

## 冻结身份
- 作者源：b741e39c4e48ef26efffacab187c11974fc9f363002c3aebab573eb558e52d4e。
- 运行源：964a094925d12fa9770f62fd5b87701a1c41514d68178c7517f71a0217b0c004。
- AV04：4ccf4c8392b99c99ce075ace84eabb95cdfef6f478ad4e31e8052f1cfd053c23。
- GLB：76c137c504b15d7836b32675fce4ddc8f9089ff38e733c863c3f7431e7b415bf。
- 原textured制作源仍为b9847c7088bcb171e9dbccd8d13bdd42f1b26b7e3aff992a3e67a445ba17f59b，未覆写。

## 保留域、真实厚度与绑定
独立后台Blender分别重开原源、作者源、运行源。按真实component逐面世界坐标/loop UV/命名骨权重重新计算，不采用作者报告代替：164个保留身体/衣裤/头发等域精确相同；11个明确旧披风/围巾/旧火域已移除；rig父链/编辑矩阵/sourceRest/修正元数据精确相同。保留身体几何及其高度未改变。

| 项目 | 作者源 | 运行LOD |
|---|---:|---:|
| 三块新厚布控制笼三角面等价（24,064个quad） | 48,128 | 48,128 |
| 新厚布真实求值三角面 | 389,440 | 98,464 |
| 独立薄火焰面三角面 | 7,360 | 7,360 |
| 保留人物面数 | 33,328 | 33,328 |
| 全身求值三角面 | 430,128 | 139,152 |

作者源保留可编辑quad、一级Subsurf和真实厚度；运行源没有有效Subsurf，仍有真实Solidify闭合厚度。三块厚布求值均无非流形边、零面积面、绕序不一致或非有限坐标/法线。火焰是明确薄面例外，其开放边不冒称流形闭壳。

新披风最多3个归一化权重，最大权重和误差3.08e-8。用真实共享游戏run/attack快照求值实际Subsurf/厚度/Armature网格，与独立自有bind LBS比较：作者源397,912顶点、LOD106,936顶点，最大误差分别3.93e-7m、3.91e-7m。不是只检查抽象矩阵或手工摆姿。

## 形体与局部净空
实看第三版clay领口/背面，主后披布没有早期上背黑洞、独立横板切入，褶皱层级和不对称长尾来自控制几何，非只细分旧管面。
独立以保留躯干/上衣/领口BVH作背向射线：main cape和肩披无负净空采样；cowl下领的109作者点/36LOD点与隐藏Body torso under clothing接触，最深约12.7mm。它们是藏入领口的下层连接，被肩披/主披布遮住，主会话/制作代理复看实际侧后图确认不是可见外衣穿出；报告保留该限制，不把它抹成全局零交叉。

## FILE与实际PNG颜色证据
新增披风4张图在作者/LOD中都是FILE来源、已pack、2048²，packed数据逐字节等于实际PNG；base/emission为sRGB，normal/ORM为Non-Color；Sheen0、normal strength1。
独立Pillow解码PNG并核对两源各4图64×64采样：Blender字节FILE的CPU image.pixels暴露的是编码浮点值，匹配误差最大7.44e-8，不能误称CPU数值就是着色线性辐射。按源Texture节点明确sRGB合同解码，基色线性范围0.005605..0.026241；发光范围0..0.001821。该证据保证文件/pack/颜色空间一致，最终可见黑灰和发光仍以实际GPU审阅为准。
ORM.R全1、B全0，明确未烘几何AO。三块外面板矩形分开；厚布内背面与rim共享布料UV为设计例外，未宣称整件厚壳唯一UV或独立AO。旧16张PNG不改，新建4图独立处理。

## 实际GLB/AV04独立证明
实际AV04精确读至EOF：7部件、5材质、20张2KPNG、92,022序列化顶点、139,152三角面，显式预算150,000，development UV/material绕过均关闭。字段/单位法线/UV/索引/归一字节权重/自己的R和inverse(R)与authored-mesh逐项相符；20张内嵌PNG逐像素匹配原16图和新增4图。

标准GLTF初导出遗漏Solidify厚度的问题已由导出代理修复。独立读最终GLB的JSON/BIN/真实skin/IBM：7个skinned mesh、实际索引三角总数139,152，与LOD/AV04一致；GLB到运行几何和反向各70,901个唯一bind点均覆盖，最大nearest误差5.66e-7m，排除只保留外面、内厚度丢失仍单向通过的漏洞。
5个材质的GLB AO/MR索引、UV一致，嵌入完整ORM payload与实际运行PNG一致。新的披风ORM.R=1是显式无AO值，并非恢复或烘焙未知来源效果。

## 质量与证据文件
- 制作工具Ruff、独立checker Ruff/语法、diff检查通过；未改制作代理文件。
- cape-detail-check-final.json：最终源/LOD保留域、拓扑、绑定、净空与实际姿态顶点。
- cape-detail-check-textures-final.json：FILE/packed/PNG采样与颜色空间合同。
- cape-detail-check-runtime-final.json：实际AV04、PNG、GLB AO和双向全几何覆盖。
- 运行代码未改，本次未重复全套Rust；严格导出/原有运行回归与主会话真实GPU21图、暂停/0pageerror证据分开记录。
