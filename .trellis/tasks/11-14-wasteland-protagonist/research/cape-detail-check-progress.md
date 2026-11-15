# 披风精修独立检查进度

## 当前阶段
最终已冻结并完成独立审计，见cape-detail-check-final.md；以下为准备阶段记录，不作为最终资产身份。

## 静态合同审查
- 删除旧域按component provenance明确列举：旧后披布、旧围巾连续面与两条卷褶、旧肩披、五条旧ember guide、旧fire域。独立checker以原textured源b984…为基准，比较保留域每个面的世界坐标/loop UV/命名骨权重，不依赖作者报告。
- 新面为三套独立quad控制面；细分曲率与Solidify真实厚度在Armature前，保留编辑面与自身绑定；运行LOD必须无有效Subsurf，厚度可保留修改器或真实烘焙闭合网格。
- UV使用三块独立矩形，Solidify内外面/边缘共享布料UV是有记录的设计，不冒称整个厚布唯一UV或独立AO烘焙。
- sourceRest/核心骨架/身体网格保持，肩部只复用已有颈/锁骨/Spine2与三个既有cape辅助骨；不创建来源不明骨链或实时布料系统。
- 导出器triangle-budget默认40k，显式接受1..250k整数；详细披风作者源不受旧40k限制，最终运行LOD目标以主会话记录的150k为准。

## 待实际检查
`cape-detail-check.py`已准备并通过Ruff/语法检查；将在独立后台Blender只读重开，检查源和LOD保留域/rig指纹、控制quad/求值闭合流形/法线/零面积、归一化权重、自身绑定实际run/attack顶点，及真实保留上衣/躯干BVH的上背净空。最终还需主会话看图确认形体、羽缘、覆肩连接与攻击表现。
