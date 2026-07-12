# 单局剧本核心技术设计

`compileEpisodePlan(game)` 以空事件开始循环调用现有 planner，使用 Draft 默认合法 payload 确认事件，直到 `game_ended`。它输出无随机 ID 的 `EpisodePlanStep[]`。`authorEpisodeNarrative()` 把压缩 trace 发给 LLM，只接受 title/logline/acts/beats；beats 用 step index 关联 speech step。

`EpisodeScriptSnapshot` 保存 compiler/schema/inputHash、计划胜方/天数、预计时长、steps、acts 和 author metadata。`planNextEpisodeDraft()` 先调用现有 planner，再匹配当前 step slot，并应用批准的结构 payload。Speech Draft 保留当前可生成 text，同时返回当前 ActorBrief。

持久化状态由产品层拥有；核心只提供纯编译、验证、hash、绑定与 Actor Projection。
