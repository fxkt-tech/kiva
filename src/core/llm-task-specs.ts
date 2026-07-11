import type { DraftEvent } from "./drafts";

export const LLM_SPEECH_DRAFT_TYPES = [
  "wolf_strategy_given",
  "wolf_opinion_given",
  "day_speech_given",
  "last_words_given",
  "pk_speech_given",
] as const;

export const LLM_ACTION_DRAFT_TYPES = [
  "guard_protect_selected",
  "wolf_vote_cast",
  "seer_check_selected",
  "witch_antidote_decided",
  "witch_poison_decided",
  "hunter_shot_decided",
  "vote_cast",
] as const;

export type LlmSpeechDraft = Extract<
  DraftEvent,
  { type: (typeof LLM_SPEECH_DRAFT_TYPES)[number] }
>;

export type LlmActionDraft = Extract<
  DraftEvent,
  { type: (typeof LLM_ACTION_DRAFT_TYPES)[number] }
>;

export type LlmDraft = LlmSpeechDraft | LlmActionDraft;

export type PromptRuleKey =
  | "role_roster"
  | "win_condition"
  | "wolf_kill"
  | "guard"
  | "seer"
  | "witch_antidote"
  | "witch_poison"
  | "hunter"
  | "vote"
  | "pk_vote";

export type PromptTaskSpec = {
  readonly scene: string;
  readonly channel:
    | "公开发言"
    | "狼人私聊"
    | "玩家私有行动"
    | "密封选择"
    | "投票选择"
    | "公开触发行动";
  readonly audience: string;
  readonly progress: string;
  readonly disclosure: string;
  readonly objective: string;
  readonly mustCover: readonly string[];
  readonly mustNot: readonly string[];
  readonly ruleKeys: readonly PromptRuleKey[];
  readonly includePublicClaims: boolean;
  readonly includeFactionDiscussion: boolean;
  readonly includeRoleActionPrompt: boolean;
  readonly outputKind: "speech" | "target" | "optional_action";
};

export type PromptTaskState = {
  readonly hasPriorDaySpeech?: boolean;
};

export function taskSpecForDraft(
  draft: LlmDraft,
  state: PromptTaskState = {},
): PromptTaskSpec {
  switch (draft.type) {
    case "wolf_strategy_given":
      return {
        scene: "首夜狼队制定整体战术",
        channel: "狼人私聊",
        audience: "所有存活狼人队友；可以直接谈论狼人身份和队内策略",
        progress: "狼队尚未密票，夜晚尚未结算，任何玩家都还没有死亡",
        disclosure: "本轮内容只对狼队可见，可以直接点出队友；不要假装在白天公开发言",
        objective: "提出狼队首夜可执行的整体方案，为后续队友意见和密票建立共同方向",
        mustCover: [
          "给出一个主刀候选和一个备选候选",
          "说明没有行为信息时主备顺序只是中立取舍，不代表任何候选更像神职或平民",
          "提出白天伪装、站位或队友协作方向",
        ],
        mustNot: [
          "不得声称某位尚未发言的玩家表现可疑、沉默或像神职",
          "不得把任一非狼人候选称为已知神职、平民，或声称其身份概率高于其他对称候选",
          "不得编造数值概率、已完成的投票、死亡或夜间结果",
        ],
        ruleKeys: ["role_roster", "win_condition", "wolf_kill", "guard"],
        includePublicClaims: false,
        includeFactionDiscussion: true,
        includeRoleActionPrompt: false,
        outputKind: "speech",
      };

    case "wolf_opinion_given":
      return {
        scene: `第 ${draft.payload.dayNumber} 夜狼队内部讨论`,
        channel: "狼人私聊",
        audience: "所有存活狼人队友；可以直接谈论狼人身份和队内策略",
        progress: "部分队友已经提出方案，但狼队尚未密票，本夜尚未结算",
        disclosure: "本轮内容只对狼队可见，不需要隐藏自己或队友的狼人身份",
        objective: "直接回应已有战术和意见，补充一个能帮助狼队完成密票的独立判断",
        mustCover: [
          "明确赞成、反对或修正已有主方案",
          "给出自己的主刀候选以及备选或一个具体风险点；无新增证据时明确沿用主目标只是维持团队共识，主备都属于中立取舍",
          "避免机械复述队友已经说过的整套方案",
        ],
        mustNot: [
          "不得把本轮说成白天公开发言",
          "不得把提案、投票或袭击当成已经发生的结果",
          "不得根据尚未发生的白天发言编造玩家特征",
          "队友提到或没有提到某候选，不是该候选的行为证据，不得据此提高其神职或平民概率",
          "不得把身份未知、没有反证或没有额外信息包装成选择某位候选的差异化证据",
        ],
        ruleKeys: ["role_roster", "win_condition", "wolf_kill", "guard"],
        includePublicClaims: true,
        includeFactionDiscussion: true,
        includeRoleActionPrompt: false,
        outputKind: "speech",
      };

    case "day_speech_given":
      return {
        scene: `第 ${draft.payload.dayNumber} 天依次公开发言`,
        channel: "公开发言",
        audience: "所有存活玩家",
        progress: state.hasPriorDaySpeech
          ? "本日已有玩家发言，现在轮到你；本日放逐投票尚未开始"
          : "你是本日第一位发言者；还没有其他玩家在本日发言，放逐投票尚未开始",
        disclosure:
          "公开文本会被所有玩家听到。狼队秘密永远不得直接披露；自己的神职身份和私有结果只有在你明确选择公开时才能说出",
        objective: state.hasPriorDaySpeech
          ? "回应本日已有发言，形成当前立场并留下后续可以验证的问题或方向"
          : "在没有他人发言可分析时，诚实说明信息有限，并提出有用的观察框架、关注点或暂定策略",
        mustCover: state.hasPriorDaySpeech
          ? ["具体回应至少一条已有公开主张", "给出当前立场、怀疑点或待验证问题"]
          : ["提供至少一个后续可观察或可验证的关注点", "不得只复述昨夜结果后直接过麦"],
        mustNot: [
          "不得把其他玩家的说法自动当成确认事实",
          "不得泄露狼队私聊、狼队友、密票或未主动公开的神职私有信息",
          "不得为了符合人物口头禅而牺牲事实一致性或任务贡献",
        ],
        ruleKeys: ["role_roster", "win_condition"],
        includePublicClaims: true,
        includeFactionDiscussion: true,
        includeRoleActionPrompt: false,
        outputKind: "speech",
      };

    case "last_words_given":
      return {
        scene: `第 ${draft.payload.dayNumber} 天公开遗言`,
        channel: "公开发言",
        audience: "所有仍在场的玩家",
        progress: `你已因 ${lastWordsReasonLabel(draft.payload.reason)} 出局，这是你的最后一次公开发言`,
        disclosure:
          "你可以策略性公开自己的身份或私有结果，但狼队秘密不得作为确认事实直接泄露",
        objective: "交付最后有价值的信息、最重要判断和留给场上玩家的后续建议",
        mustCover: ["指出最重要的确认信息或判断", "给出一个后续验证、站边或放逐建议"],
        mustNot: ["不得假装自己仍会继续行动或投票", "不得引用出局后尚未发生的信息"],
        ruleKeys: ["role_roster", "win_condition", "hunter"],
        includePublicClaims: true,
        includeFactionDiscussion: true,
        includeRoleActionPrompt: false,
        outputKind: "speech",
      };

    case "pk_speech_given":
      return {
        scene: `第 ${draft.payload.dayNumber} 天第 ${draft.payload.round} 轮 PK 公开发言`,
        channel: "公开发言",
        audience: "全场玩家以及本轮有资格投票的玩家",
        progress: "上一轮放逐票出现平票，你是平票候选之一；PK 投票尚未开始",
        disclosure:
          "公开文本会被所有玩家听到；遵守与白天公开发言相同的秘密披露边界",
        objective: "回应自己进入 PK 的原因，完成有针对性的生存辩护并争取合法投票者",
        mustCover: ["回应指向自己的关键理由", "说明自己与另一位平票候选的差异或更合理的放逐方向"],
        mustNot: ["不得写成与 PK 无关的普通漫谈", "不得假装自己已经完成投票"],
        ruleKeys: ["role_roster", "win_condition", "pk_vote"],
        includePublicClaims: true,
        includeFactionDiscussion: true,
        includeRoleActionPrompt: false,
        outputKind: "speech",
      };

    case "guard_protect_selected":
      return privateActionSpec({
        scene: "守卫选择本夜守护目标",
        objective: "从合法候选中选择本夜最值得保护的玩家",
        mustCover: ["结合上夜守护限制和公开局势说明选择"],
        mustNot: ["不得声称知道狼人本夜的真实刀口"],
        ruleKeys: ["guard"],
        includeRoleActionPrompt: true,
        outputKind: "target",
      });

    case "wolf_vote_cast":
      return {
        ...privateActionSpec({
          scene: `第 ${draft.payload.dayNumber} 夜狼人密封刀票`,
          objective: "结合狼队内部讨论，独立投出本夜袭击目标",
          mustCover: ["优先响应团队已经形成的方向，或明确说明偏离原因；沿用目标只需说明团队共识"],
          mustNot: [
            "不得猜测或引用其他狼人已经提交的密封票",
            "不得返回姓名代替合法 playerId",
            "狼队讨论中的身份、沉默、发言和概率判断仍是未证实猜测，不得在 decisionSummary 中当作目标证据复述",
          ],
          ruleKeys: ["wolf_kill", "guard"],
          includeRoleActionPrompt: true,
          outputKind: "target",
        }),
        channel: "密封选择",
        audience: "仅系统记录；其他狼人看不到你的单独票",
        disclosure: "可以使用狼队身份和内部讨论；较早密封票不会提供给你",
        includeFactionDiscussion: true,
      };

    case "seer_check_selected":
      return privateActionSpec({
        scene: "预言家选择本夜查验目标",
        objective: "从尚未查验的合法候选中选择信息价值最高的目标",
        mustCover: ["结合历史查验和公开信息说明信息价值；若候选完全对称则明确这是中立选择"],
        mustNot: ["不得编造尚未发生的发言或可疑行为", "默认不得重复查验已有结果的玩家"],
        ruleKeys: ["seer"],
        includeRoleActionPrompt: true,
        outputKind: "target",
      });

    case "witch_antidote_decided":
      return privateActionSpec({
        scene: "女巫决定是否使用解药",
        objective: "根据本夜被袭击者、药品库存和局势决定是否消耗解药",
        mustCover: ["权衡保存资源与避免当前损失"],
        mustNot: [
          "不得由刀口继续推断其具体身份是平民或神职",
          "不使用时 targetPlayerId 必须为 null",
        ],
        ruleKeys: ["witch_antidote", "guard"],
        includeRoleActionPrompt: true,
        outputKind: "optional_action",
      });

    case "witch_poison_decided":
      return privateActionSpec({
        scene: "女巫决定是否使用毒药",
        objective: "根据毒药库存、本夜解药决定和公开嫌疑决定是否用毒及目标",
        mustCover: ["权衡毒药命中风险与保留资源的价值"],
        mustNot: ["双药规则禁止时不得在已使用解药后再用毒", "不使用时 targetPlayerId 必须为 null"],
        ruleKeys: ["witch_poison", "hunter"],
        includeRoleActionPrompt: true,
        outputKind: "optional_action",
      });

    case "hunter_shot_decided":
      return {
        ...privateActionSpec({
          scene: "猎人出局后选择开枪目标",
          objective: "根据公开证据从存活合法候选中选择要带走的玩家",
          mustCover: ["结合发言、票型或死亡触发说明最终目标"],
          mustNot: ["不得选择自己、死亡玩家或其他非法目标"],
          ruleKeys: ["hunter"],
          includeRoleActionPrompt: true,
          outputKind: "target",
        }),
        channel: "公开触发行动",
        audience: "全场玩家；最终开枪目标会公开",
        disclosure: "可以直接说明自己的猎人身份和开枪判断",
      };

    case "vote_cast": {
      switch (draft.payload.voteType) {
        case "exile":
          return {
            scene: `第 ${draft.payload.dayNumber} 天放逐投票`,
            channel: "投票选择",
            audience: "系统记录；提交后的公开时机由本局投票规则决定",
            progress: "本日公开发言已经结束，现在需要提交放逐票",
            disclosure: "根据公开发言和自己的身份目标投票；不得引用尚未公开的他人单票",
            objective: "从合法候选中选择本日最应该被放逐的玩家，或在规则允许时弃票",
            mustCover: ["让投票与自己的公开立场或阵营策略保持一致"],
            mustNot: ["不得使用角色的夜间技能指令", "不得看到或猜作事实的未公开单票"],
            ruleKeys: ["vote"],
            includePublicClaims: true,
            includeFactionDiscussion: true,
            includeRoleActionPrompt: false,
            outputKind: "target",
          };

        case "pk":
          return {
            scene: `第 ${draft.payload.dayNumber} 天第 ${draft.payload.round} 轮 PK 投票`,
            channel: "投票选择",
            audience: "系统记录；提交后的公开时机由本局投票规则决定",
            progress: "平票候选已经完成 PK 发言，现在只在平票候选中投票",
            disclosure: "根据公开发言和票型投票；不得引用尚未公开的他人单票",
            objective: "只从本轮平票候选中选择要放逐的玩家，或在规则允许时弃票",
            mustCover: ["比较 PK 候选的发言和上一轮争议"],
            mustNot: ["不得投给非 PK 候选", "不得使用角色的夜间技能指令"],
            ruleKeys: ["pk_vote"],
            includePublicClaims: true,
            includeFactionDiscussion: true,
            includeRoleActionPrompt: false,
            outputKind: "target",
          };

        case "sheriff":
          throw new Error("LLM sheriff voting is not supported");
      }
    }
  }
}

export function isLlmSpeechDraft(draft: DraftEvent): draft is LlmSpeechDraft {
  return (LLM_SPEECH_DRAFT_TYPES as readonly DraftEvent["type"][]).includes(
    draft.type,
  );
}

export function isLlmActionDraft(draft: DraftEvent): draft is LlmActionDraft {
  return (LLM_ACTION_DRAFT_TYPES as readonly DraftEvent["type"][]).includes(
    draft.type,
  );
}

function privateActionSpec(
  input: Pick<
    PromptTaskSpec,
    | "scene"
    | "objective"
    | "mustCover"
    | "mustNot"
    | "ruleKeys"
    | "includeRoleActionPrompt"
    | "outputKind"
  >,
): PromptTaskSpec {
  return {
    ...input,
    channel: "玩家私有行动",
    audience: "仅你本人和系统记录",
    progress: "当前行动尚未提交，也尚未产生行动结果",
    disclosure: "可以使用自己的私有身份和记录；其他玩家不会看到这份决策摘要",
    includePublicClaims: true,
    includeFactionDiscussion: false,
  };
}

function lastWordsReasonLabel(
  reason: Extract<DraftEvent, { type: "last_words_given" }>["payload"]["reason"],
): string {
  switch (reason) {
    case "night_death":
      return "昨夜死亡";
    case "exile":
      return "放逐";
    case "hunter_shot":
      return "猎人开枪";
  }
}
