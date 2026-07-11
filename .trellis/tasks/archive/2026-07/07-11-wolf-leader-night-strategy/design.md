# 狼人领头人夜间战术讨论：技术设计

## 目标流程

第一夜：

`守卫行动 → Editor 指定领头人 → 领头人发布初始战术 → 其他狼人依次发表意见 → 全体狼人依次密票 → 结票/必要时人工裁定 → 预言家与女巫流程 → 夜间结算`

第二夜及以后：

`守卫行动 → 全体存活狼人依次发表意见 → 全体狼人依次密票 → 结票/必要时人工裁定 → 预言家与女巫流程 → 夜间结算`

第一夜领头人的特殊性仅是多一个“发布初始战术”步骤。投票时与其他狼人完全等权；后续夜晚不再承担特殊职责，死亡后不补选。

## 事件与草稿契约

直接替换现有单一 `wolf_kill_selected` 流程，不为旧记录保留兼容分支。

### `wolf_leader_selected`

- `actorPlayerId`：被主理人选中的狼人。
- `payload.leaderPlayerId`：同一玩家 ID，便于表单编辑与领域校验。
- `visibility`：`host_only`。
- 仅允许第一夜、狼队讨论开始前创建和确认。
- 这是 Editor 控制事件；播放编译器必须显式排除，导演 Preview 也不生成场景。

### `wolf_strategy_given`

- `actorPlayerId`：第一夜选中的领头人。
- `payload`：`{ playerId, text, dayNumber: 1 }`。
- `visibility`：狼阵营私密。
- 仅第一夜一次，文本可编辑、可重新生成。
- Preview 以玩家发言场景展示，且这是第一夜首个可见的狼队协作场景。

### `wolf_opinion_given`

- `actorPlayerId`：发表意见的存活狼人。
- `payload`：`{ playerId, text, dayNumber }`。
- `visibility`：狼阵营私密。
- 第一夜排除领头人；后续夜晚包含所有存活狼人。
- 按座位顺序逐条生成和确认，后发言者能在上下文中看到此前意见。

### `wolf_vote_cast`

- `actorPlayerId`：投票狼人。
- `payload`：`{ voterPlayerId, targetPlayerId, dayNumber }`。
- `visibility`：`host_only`，从数据可见性上保证其他狼人无法提前看到票。
- 每名当夜存活狼人一票，合法目标沿用“存活且非狼人”。
- 由 LLM 根据初始战术、当夜全部意见及此前公开历史生成目标；提示上下文不得包含同夜其他票。
- 草稿可由主理人编辑或重新生成，确认后在全部票完成前不进入 Preview。

### `wolf_vote_resolved`

- `payload` 包含：
  - `votes: { voterPlayerId, targetPlayerId }[]`
  - `tallies: { targetPlayerId, count }[]`
  - `tiedTargetPlayerIds: PlayerId[]`
  - `targetPlayerId: PlayerId | null`
  - `resolution: "majority" | "host_tiebreak"`
- `visibility`：狼阵营私密。
- 唯一最高票时规划器自动填入 `targetPlayerId` 和 `majority`，仍经主理人确认草稿。
- 平票时创建 `targetPlayerId: null` 的 Editor 草稿，只提供并列最高票候选；禁用 AI 生成，必须人工选择后才能确认，确认后标记 `host_tiebreak`。
- Preview 一次性展示每只狼的具体票、各目标票数、最终刀口及必要的人工裁定标记。
- 夜间后续逻辑以该事件的 `targetPlayerId` 作为唯一狼刀输入。

## 规划器状态机

在 `planNightDraft` 中把狼人阶段抽成独立规划函数，避免继续扩大现有长函数。规划器从当夜已确认事件推导下一步，不引入额外可变会话状态：

1. 第一夜缺少 `wolf_leader_selected`：返回领头人选择草稿。
2. 第一夜缺少 `wolf_strategy_given`：返回领头人战术草稿。
3. 按当夜意见参与者顺序，返回下一名尚未发言狼人的意见草稿。
4. 按存活狼人座位顺序，返回下一名尚未投票狼人的密票草稿。
5. 全部投完后统计票数并返回结票草稿。
6. 结票确认且有最终目标后，继续预言家、女巫和夜间结算。

领域辅助函数集中负责：当夜存活狼人、第一夜领头人、应发言者、已投票者、票数统计、并列候选和最终刀口校验。Editor 表单和服务端确认必须调用同一套校验，不能只靠下拉框限制。

## LLM 生成边界

- `wolf_strategy_given` 和 `wolf_opinion_given` 走文本生成，复用现有玩家人格、角色提示词、规则和可见事件格式化基础设施，并增加各自明确的任务指令。
- `wolf_vote_cast` 走结构化行动生成，只输出合法 `targetPlayerId` 和内部 generation reasoning；reasoning 不作为第二条狼队发言进入 Preview。
- `wolf_leader_selected`、`wolf_vote_resolved` 均不调用 LLM。
- 密票事件使用 `host_only`，因此现有玩家可见性投影天然排除其他狼人的票；另加测试防止后续上下文拼装绕过投影。

## Editor 交互

- 领头人草稿：只显示存活狼人候选，不显示重新生成按钮。
- 战术/意见草稿：显示多行文本编辑和重新生成。
- 投票草稿：显示合法非狼人目标，并允许重新生成。
- 唯一最高票结票草稿：只读展示票型和最终刀口，允许确认/删除重算，不允许 AI 生成。
- 平票结票草稿：展示票型，下拉框仅包含并列最高票目标；未选择时确认在服务端失败。

## Preview 与主持文案

- 播放编译器显式过滤 `wolf_leader_selected` 与单张 `wolf_vote_cast`；它们只用于 Editor 和状态推导。
- `wolf_strategy_given`、`wolf_opinion_given`、`wolf_vote_resolved` 在导演 Preview 中形成场景。
- 为三种内容增加 Presenter copy key 和事件格式化：领头人战术、狼人意见、狼队结票。
- 结票场景使用结构化票型生成展示数据，避免从文案反向解析投票。
- 公共 audience 继续只接收公开事件，狼队内容不进入公共播放。

## 夜间结算

女巫获知的被袭击玩家、解药合法目标及 `resolveNightDeathDetails` 的 `wolfKillTargetId` 全部改为读取当夜 `wolf_vote_resolved.payload.targetPlayerId`。守卫、解药、毒药对刀口的现有语义不变。

## 取舍

- 使用独立“意见”和“投票”事件，生成次数更多，但能真正实现先讨论后密票，并支持 Preview 展示讨论而不泄漏投票。
- 领头人选择保留为内部事件，而不是 Game 配置字段，使第一夜状态机、审计和删除草稿后的重算保持事件驱动；通过播放过滤满足“不展示选择过程”。
- 不做旧事件兼容与迁移，减少双轨逻辑，但已有游戏数据在新版本下不保证可继续运行或播放。

## 回滚边界

事件联合类型、夜间规划器、生成器和 Preview 是同一个契约变更，必须作为一个完整切片交付。若验证失败，应整体回滚新狼人阶段，不保留同时支持新旧狼刀流程的临时分支。
