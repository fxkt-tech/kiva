# 永劫无间动作离线提取实施

## 前置

依赖父任务的 animation-assets 与 humanoid-conversion 研究；不依赖战斗代码。表现子任务必须等本任务通过真实转换门槛，不能用占位片段先接入。

## 顺序

1. 复核原状态→clip精简清单，导出原Avatar、首个横击及原默认左右手层；保留全部authored mask，核实layer顺序/权重。
2. 实现/验证准确曲线采样与肌肉索引，再验证原轴转换、身体/手指/根处理；记录原生求值尚未对照的部分。
   H0关口：先取得可信原生/独立转换逐帧对照，至少attack95/idle75/jump-loop55与完整左右手层，通过父design规定的方向/quat/LBS验收。H0未过不能进入第3步或把数学候选设为默认。
3. 转换核心三段/振刀/成功片段，通过后再补通用与可信远程动作。
4. 添加含baked locals的准备包、运行时binary、来源清单；普通verify重打包与完整rebake区分，记录后者实际环境，加入真实错误输入测试。

## 校验

原片段0/中/末与全帧有限性；原曲线与烘焙采样误差；原骨/指长度和手掌轴；source跨基础/极值姿态无翻转；导入器--verify。

适用Rust/Web变更执行make check和WASM Clippy；资产变更执行明确的离线verify，不能用语法检查代替解码/重建。所有切片保留可回滚边界；不覆盖同伴文件、不改用户客户端、不启动常驻服务。

## 实施结果

已完成44段原片段/Avatar冻结、真正原生Graph求值、partial body/finger证据、KV2独立rig格式、有界自适应补采、独立137Hz偏移逐时刻FK/实际手部LBS验收及可移植完整verify/repackage。最终指标、来源链、风险和命令见 [delivery.md](delivery.md) 与准备包README。原生环境的f1/f2补丁差异和默认层Kiva适配均明确保留，未改运行时或提交/推送。
