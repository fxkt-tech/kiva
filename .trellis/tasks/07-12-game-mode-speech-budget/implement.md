# 游戏模式发言预算实施计划

## 1. 基线

- [x] 增加只读样本时长诊断，固化 public/director 构成。
- [x] 记录现有 57 段文本、真实 TTS 时长和 76.04 分钟基线。

## 2. 共享预算模块

- [x] 新增预算类型、任务矩阵、统一字符计数和时长估算。
- [x] 测试 Unicode、空白、标点、换行和边界值。

## 3. Prompt 与生成

- [x] Task Spec 关联预算 key 和最低有效表达。
- [x] Prompt 注入目标/硬上限并禁止复述、票表和舞台动作。
- [x] Speech parser 校验硬上限，复用唯一 repair。
- [x] 人工编辑/确认复用评估函数。

## 4. 上下文压缩

- [x] 集中实现当前日、上一日和更早历史选择。
- [x] 增加结构化票型与最新立场投影。
- [x] 扩展 visibility 负向测试。

## 5. 时长观测

- [x] 计算已承诺时长、预算档位和预计导演版区间。
- [x] 在 Editor/Preview 展示，不改变规则或视频内容。

## 6. 回放与门禁

- [ ] 回放样本 57 个发言局面并统计 hard-limit/repair。
- [x] 重新编排预计导演版，目标 28–32 分钟。
- [ ] 人工复核关键推理和自然度。
- [ ] 运行全量 typecheck/test/build/diff-check。

## Validation

- 相关 core 单测
- pnpm typecheck
- pnpm test
- pnpm build
- git diff --check

## Rollback

- 预算、上下文、Pacing 保持独立提交点。
- 若动态档位不稳定，保留固定任务预算和硬上限，先回滚 pacing。
- 不修改样本文件，回放失败只清理临时输出。
