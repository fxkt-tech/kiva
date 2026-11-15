# 完整制作流程：阶段1静态审查

范围：Ember表现模块/测试、AV04材质解码与缓存挂接、外观ID3/模型查看页调用、refine_head/refined_surfaces及builder阶段隔离。纹理工具/最终导出仍在实现，未进行最终Blender/游戏验收。

## 已确认
- SurfaceEffect显式字节0=None/1=Ember，未知值拒绝；Ember只注册独立Blend材质，不按外观或纹理名猜测。
- normal/hit/telegraph各缓存句柄保留独立原始UV/emissive基线；重复注册不捕获已动画值，不逐帧创建材质或网格。Ember改变标准PBR材质参数，保留蒙皮路径。
- 游戏时钟只累加SimulationActive期间dt；查看页用绝对seek时间求值。8秒周期相位及整数纹理平移保持重复边界连续。已有6个针对性测试覆盖原基线、确定性seek、暂停恢复、重复缓存、变体与非法标签。
- AV02/AV03/WP01沿旧材质字段长度读入，AV04增加alpha/opacity/HDR emission/emission texture/u8surfaceEffect；所有f32先检查有限，新模式/范围/纹理索引及Blend效果约束明确验证。
- avatar的三个材质反馈变体均只注册一次；ID3选择、缓存与模型查看页已接入，默认仍李火旺，妹妹不进入玩家外观循环。
- headhelper只做静态顶点变形和基于真实头部表面投射的眉几何，不修改来源文件、核心骨长或构造无证据面部骨。refined构建另存，拒绝直接把refined输出到名为greybox的目录。

## 已提请主会话协调
- 运行导出合同文档需补齐实际已写入/解码的u8surfaceEffect字段。
- export_materials的显式ORM路径缺少其他纹理路径已有的2K/PNG大小检查，严格最终导出应统一验证；当前纹理仍在生成，未据开发版宣称成品通过。
- WIP texture_character.py有一个未使用Vector导入的Ruff告警，已告知实现代理，未并发改其文件。

refine_head/refined_surfaces/builder的Ruff与语法检查通过；全工具py_compile通过。全量Rust检查由运行实现代理拥有，未并发重复运行。未启动、停止或重启用户服务。最终纹理文件、严格导出、Blender文件/贴图/透明发光画面及游戏姿态验收留待后续完整检查。
