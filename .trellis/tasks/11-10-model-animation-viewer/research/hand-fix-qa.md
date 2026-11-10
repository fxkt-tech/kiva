# 手部修复与验收

日期：2026-11-11。用户确认截图为默认暂停状态，并授权查清后修复。修复位于游戏／验收页共用层，代码和验证已完成，用户随后授权本轮提交并推送main。

## 根因及回归

1. 默认为CC0 Sword_Idle＋持握，燕云源权重为0。目标参考臂向与donor相差48.794°／56.854°，旧全局增量保留了方向偏差；骨长正确与游戏／查看一致不能证明姿态正确。
2. 真实腕→中指MCP与原语义挂点+Y点积0.4513，约63°；柄轴与掌横向轴线约30°偏差。皮肤与挂点没有共用真实掌框。
3. 原燕云skill_ylk_1有有效指轨，小指相对腕91.63°曾在目标指骨变成0°；30个目标指节被归到手腕，运动在跨骨架传递中丢失。

先红后绿测试和数据见[诊断](hand-diagnostic.md)，独立公式审查见[掌轴审查](hand-basis-review.md)。

## 实现

- avatar.rs校准采样臂段方向，FK保留自身长度，显式妹妹握点IK在后。
- avatar/hand.rs把声明CC0手框／当前源掌框校准到目标自身掌参考，挂接从当前真实腕/MCP求值。语义scale=ONE，保留+0.055m但方向进入真实掌内；柄头朝食指／拇指。
- 三节源指链按语义和自身原生轴映射，局部旋转混合后FK。不完整palette祖先用真实资产并集补足，不造Nub或长度。
- 掌轴修正前后默认武器目标位置差<1e-6m，实际手部几何对齐握持，没有只挪武器。
- 原始资产、sourceRestModel、IBM、完整原生愁铃客源palette及模拟规则保留；未改解包器或加查看页专用补偿。

## 最终检查

- [独立审查](hand-independent-review.md)通过；reviewer只补reference实际掌轴测试，没有修改运行时代码。
- make check全绿：251单元＋9集成＝260 Rust，94 Node，0失败/0ignored；原生/WASM Clippy、格式、WASM check通过。
- 回归覆盖双Naraka／双手／五指方向、正向柄头、权重0/.25/.5/.75/1的指节长度、真实LBS响应、source→reference、原生源矩阵、妹妹持铳／持弓及资源复用。
- 固定离线Trunk：WASM 217301021字节；SHA256 2c34a4f9588a05b1160974c72861b4096134a094d21c1ab12ab186102129a918。JS/WASM两份SRI核对通过。
- 最终10条同帧诊断及13条完整页面记录通过，成功路径0 JS异常/console error。默认近景、两近战source权重1攻击、原片段/参考、三角色五配置、暂停换装/相机/部件/复制、根及子路径、游戏首次进入/Esc均复核。
- 环境为独立有界面Xvfb＋SwiftShader软件GPU，不宣称硬件GPU验收；未接管用户服务。

## 留存证据

修复前近景：/tmp/kiva-hand-diagnosis-browser/before-close/01-li-default-close.png
修复后近景：/tmp/kiva-hand-diagnosis-browser/final/01-li-default-close.png
同帧动作与来源：各目录report.json；完整回归：viewer-regression/report.json；构建摘要：build-final.json。

使用新固定dist重跑：

    KIVA_XVFB_SIZE=1800x1200x24 python3 /tmp/kiva-browser-tests/with-isolated-xvfb.py -- node /tmp/kiva-hand-diagnosis-browser/repro.cjs <dist> <output> --close --attacks

夹具回收自身浏览器、HTTP与Xvfb。忽略式诊断探针只保存在研究补丁/文本，生产测试没有ignored；验收后清理自建dist，保留图片和报告。

## 范围边界

35片段没有独立twist轨，不生造原作twist控制器。旧愁铃客全局身体混合的中间掌形可能不刚性，本次让挂点跟随实际MCP，未重写全部身体混合。长刀片段的锤/链斧适配、衣装穿插与布料仍属已有美术适配范围，不宣称恢复未知武器专属动作/原作控制器。硬件GPU继续跟踪，任务不提前归档。
