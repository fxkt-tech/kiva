# 单局剧本核心实施计划

1. 定义 Episode contract、slot、step、beat、snapshot、report 与 validator。
2. 实现稳定 input hash 和确定性 dry-run compiler。
3. 实现 Script Author prompt/解析，将 narrative 严格映射到 speech step。
4. 实现运行时 planner binding 与 Actor Projection。
5. 覆盖稳定 trace、终局、非法 snapshot/hash/slot 和 projection 测试。
6. 运行 core/server targeted tests、typecheck、build、diff-check。
