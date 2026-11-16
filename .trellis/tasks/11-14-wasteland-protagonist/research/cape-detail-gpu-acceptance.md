# 披风精修实际WebGPU验收

复用用户3221开发服务，独立Windows Edge157窗口/CDP；没有启动/停止用户服务。QA页隔离Trunk更新消息，避免写截图造成页面重置。

最终运行SHA2564ccf4c8392b99c99ce075ace84eabb95cdfef6f478ad4e31e8052f1cfd053c23，运行源964a0949…c004，高精源b741e39c…2d4e。浏览器no-store读取assets/characters/zhuque.bin并用crypto.subtle计算同一SHA；用户dist同hash，页面实际部件含新shoulder/cowl/cape，不是旧缓存截图。

## 实际画面
art/wasteland/cape-refined/validation/gpu-qa.json保存21张截图对应配置/时间/源动作trace。默认李火旺，然后切朱雀等待真实GPU就绪，保存正/背/斜视。两组武器分别三段普攻、蹲伏、跑、跳、右闪、防御、受击0.400s各一帧，共18动作图；0页面错误。播放后暂停等待600ms，时间保持。

Root已看背面/实际首段攻击/链斧蹲伏：新披风右侧长尾/宽下摆、连续肩颈和曲面褶皱实际进入游戏；无之前的上背可见黑洞，武器跟手。主披布仍随骨骼运动，非物理布料；有些动作会整体抬起，不能将静帧验收称为完整实时布料碰撞通过。这里只报告这些实际查看器姿态，不冒称连续游戏战斗全覆盖。

## 源与数值
高精整人430128tri，运行整人139152tri（显式预算150000），保留身体33328tri；5材质/20张2K实际图。164保留component与rig/sourceRest独立指纹一致。实际顶点证明、LOD差异、FILE/PNG色空间、GLB厚度/AO由导出与独立checker分别记录。

## 用户Blender
已打开最终高精源，Material Preview、后3/4视角。原窗口dirty，先保存副本C:/Users/ASUS/AppData/Local/Temp/kiva-before-cape-detail-20261115.blend再切换；新源未因UI视角变化重新保存，冻结hash保持。

这是本轮披风精修交付，仍需用户审阅美术观感。其他人物网格沿用前版，未新增实时布料或面部系统。
