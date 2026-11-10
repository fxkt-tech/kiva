# 模型接入技术设计

## 边界

现有李火旺已在 AvatarAssets 缓存中，改变玩家 AppearanceSelection 默认值而不改变全局敌人默认外观。
白灵淼从准备包三个 GLB 离线转换，优先复用 KIVAAV03 与现有 RigPose→女性结构参考驱动／palette流程；CompanionView 保持独立，不引入 Body／Health 或新施法时钟。
远程武器离线解析真实源层级／skin／逆绑定并生成随包运行资源；武器的局部射击源与现有 source_position 对齐。沿用具名妹妹动作和固定步时间线。
源包所有文件保持冻结；运行导出写新 assets/bailingmiao、assets/yinbingluo、assets/yizhiying 目录，生成器及证明单独存放。

## 数据与接口

研究产物写 research/prepared-model-evidence.md；转换者先发布 research/model-runtime-contract.md，列明二进制格式、源单位／轴、女性参考驱动、文件名、挂点／射击源约定，再通知运行时实现者。
无新增运行时第三方 GLTF／serde 依赖；现有小端有界读取、PNG载入和缓存方式优先复用。
保留原材料中已明确的基础色／alpha／单双面；未知 packed／SH 等不猜为 ORM。原 normal 的约定未经证明则不随意引入。

## 验证与迁移

先以真实 GLB／原骨架数值验证转换，再以 World 测试验证几何、蒙皮、源枪口、姿态、缓存和状态隔离。
保留第一版战斗／输入回归。默认李火旺、白灵淼出场和两套远程皮肤的 UI 文字跟随当前资产身份。
所有改动保持 client→gameplay→simulation，运行时只修改 client；不触碰模拟规则。文档更新在接入事实落地后进行。
