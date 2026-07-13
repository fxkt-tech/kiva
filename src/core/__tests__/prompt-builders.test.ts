import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { legalActionOptions } from "../llm-action-options";
import { buildPlayerLlmContext } from "../player-context";
import { speechBudgetForKey } from "../speech-budget";
import {
  ACTION_PROMPT_VERSION,
  buildActionPrompt,
  buildSpeechPrompt,
  SPEECH_PROMPT_VERSION,
} from "../prompt-builders";
import type {
  DraftId,
  EventId,
  Faction,
  GameId,
  GameRole,
  PlayerId,
} from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const wolf = playerByRole("werewolf");
const secondWolf = game.players.find(
  (player) => player.gameRole === "werewolf" && player.playerId !== wolf.playerId,
)!;
const seer = playerByRole("seer");
const witch = playerByRole("witch");
const villager = playerByRole("villager");

describe("prompt builders v2", () => {
  it("puts scene and task before semantically separated player knowledge", () => {
    const checkedWolfId = wolf.playerId;
    const events = [
      roleAssigned(1, seer.playerId, "seer", "good"),
      phaseStarted(2, "night", 1),
      {
        ...baseEvent(3),
        type: "seer_check_result",
        phase: "night",
        actorPlayerId: seer.playerId,
        targetPlayerIds: [checkedWolfId],
        visibility: { kind: "player_private", playerIds: [seer.playerId] },
        payload: { targetPlayerId: checkedWolfId, result: "wolves" },
      },
      {
        ...baseEvent(4),
        type: "death_announced",
        phase: "day",
        targetPlayerIds: [],
        visibility: { kind: "public" },
        payload: { deadPlayerIds: [] },
      },
      phaseStarted(5, "speech", 1),
      daySpeech(6, villager.playerId, "我觉得昨夜平安需要继续观察。"),
    ] satisfies readonly GameEvent[];
    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: seer.playerId,
    });
    const prompt = buildSpeechPrompt({
      context,
      draft: daySpeechDraft(seer.playerId),
    });
    const content = prompt.messages[0]!.content;

    expect(prompt.promptVersion).toBe(SPEECH_PROMPT_VERSION);
    expect(prompt.promptVersion).toBe("speech:v2");
    expect(prompt.schemaName).toBe("werewolf_speech_v2");
    expect(prompt.systemPrompt).toContain("【执行优先级】");
    expect(prompt.systemPrompt).toContain(seer.roleSystemPromptSnapshot);
    expect(prompt.systemPrompt).toContain(seer.persona);
    expect(prompt.systemPrompt).toContain("人物信息只是倾向，不是固定台词模板");
    expect(content.startsWith("【当前场景——本轮最高优先级】")).toBe(true);
    expect(content.indexOf("【本轮唯一任务】")).toBeLessThan(
      content.indexOf("【已确认的公开事实】"),
    );
    expect(content).toContain("【本局剧本背景——只用于自然表达，不是身份事实】");
    expect(content).toContain("剧本：未明档案");
    expect(content).toContain("剧本不提供任何玩家身份、行为、关系或可信度证据");
    expect(content).toContain("【其他玩家的公开主张——可能真实、误判或撒谎】");
    expect(content).toContain("【未知信息——没有提供就不得推断为事实】");
    expect(content).toContain("我觉得昨夜平安需要继续观察。");
    expect(content).toContain("【你的私有事实——默认不可公开");
    expect(content).toContain("查验结果");
    expect(content).toContain("disclosure");
    expect(content).toContain("decisionSummary");
    expect(content).not.toContain('"reasoning"');
    expect(content).not.toContain(
      `${wolf.playerId} | ${wolf.seatNo} 号 | ${wolf.name} | 狼人`,
    );
  });

  it("makes wolf opinions an explicit faction-private response task", () => {
    const events = [
      roleAssigned(1, wolf.playerId, "werewolf", "wolves"),
      roleAssigned(2, secondWolf.playerId, "werewolf", "wolves"),
      phaseStarted(3, "night", 1),
      {
        ...baseEvent(4),
        type: "wolf_strategy_given",
        phase: "night",
        actorPlayerId: wolf.playerId,
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: {
          playerId: wolf.playerId,
          text: "主刀 8 号，备选 9 号，但目前没有行为信息。",
          dayNumber: 1,
        },
      },
    ] satisfies readonly GameEvent[];
    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: secondWolf.playerId,
    });
    const content = buildSpeechPrompt({
      context,
      draft: wolfOpinionDraft(secondWolf.playerId),
    }).messages[0]!.content;

    expect(content).toContain("频道：狼人私聊");
    expect(content).toContain("不需要隐藏自己或队友的狼人身份");
    expect(content).toContain("狼队尚未密票，本夜尚未结算");
    expect(content).toContain("明确赞成、反对或修正已有主方案");
    expect(content).toContain("主刀 8 号，备选 9 号");
    expect(content).toContain("【合法候选】");
    expect(content).not.toContain("现在是第一天白天");
  });

  it("gives the first day speaker a useful no-information objective", () => {
    const events = [
      roleAssigned(1, villager.playerId, "villager", "good"),
      phaseStarted(2, "speech", 1),
    ];
    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: villager.playerId,
    });
    const prompt = buildSpeechPrompt({
      context,
      draft: daySpeechDraft(villager.playerId),
    });
    const content = prompt.messages[0]!.content;

    expect(content).toContain("你是本日第一位发言者");
    expect(content).toContain("观察框架、关注点或暂定策略");
    expect(content).toContain("不得只复述昨夜结果后直接过麦");
    expect(prompt.speechBudget).toMatchObject({
      key: "day_first",
      targetMinCharacters: 100,
      targetMaxCharacters: 140,
      hardMaxCharacters: 170,
    });
    expect(content).toContain("【表达长度——正文必须遵守】");
    expect(content).toContain("text 目标长度：100–140 个非空白字符");
    expect(content).toContain("text 硬上限：170 个非空白字符");
    expect(content).toContain("不得复述完整时间线、完整票型");
    expect(content).toContain("不要写括号舞台动作");
  });

  it("uses the responding day-speech budget after a prior speaker", () => {
    const source = game.players.find(
      (player) => player.playerId !== villager.playerId,
    )!;
    const events = [
      roleAssigned(1, villager.playerId, "villager", "good"),
      phaseStarted(2, "speech", 1),
      {
        ...baseEvent(3),
        type: "day_speech_given",
        phase: "speech",
        actorPlayerId: source.playerId,
        visibility: { kind: "public" },
        payload: {
          playerId: source.playerId,
          text: "先听后面的发言。",
          dayNumber: 1,
          round: 1,
        },
      },
    ] satisfies readonly GameEvent[];
    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: villager.playerId,
    });

    expect(
      buildSpeechPrompt({
        context,
        draft: daySpeechDraft(villager.playerId),
      }).speechBudget,
    ).toMatchObject({
      key: "day_response",
      targetMinCharacters: 140,
      targetMaxCharacters: 180,
      hardMaxCharacters: 220,
    });
  });

  it("renders only the current scripted actor brief", () => {
    const context = buildPlayerLlmContext({
      game,
      events: [roleAssigned(1, villager.playerId, "villager", "good")],
      viewerPlayerId: villager.playerId,
    });
    const prompt = buildSpeechPrompt({
      context,
      draft: daySpeechDraft(villager.playerId),
      actorBrief: {
        stepIndex: 18,
        scene: "红印存根第一次出现矛盾",
        objective: "迫使上一位发言者解释证词差异",
        stance: "暂时质疑三号，但保留复核空间",
        disclosure: "conceal",
        themeHook: "让本轮质疑成为下一次投票验证的档案版本冲突",
        characterHook: "用谨慎拆词的习惯追问，不突然变成咄咄逼人",
        arcMove: "第一次让谨慎从旁观转为承担一次明确判断",
        relationshipMove: "把与三号的礼貌分歧推进为可公开复核的竞争",
        budget: speechBudgetForKey("day_first", "critical"),
      },
    });
    const content = prompt.messages[0]!.content;
    const combined = `${prompt.systemPrompt}\n${content}`;
    const otherPlayer = game.players.find(
      (player) => player.playerId !== villager.playerId,
    )!;

    expect(content).toContain("【本场剧本指引——只执行当前场，不得推断未来】");
    expect(content).toContain("红印存根第一次出现矛盾");
    expect(content).toContain("用谨慎拆词的习惯追问");
    expect(content).toContain("谨慎从旁观转为承担一次明确判断");
    expect(content).toContain("礼貌分歧推进为可公开复核的竞争");
    expect(content).toContain("不是游戏事实、身份线索或可信度证据");
    expect(content).toContain("你看不到完整剧本");
    expect(content).not.toContain("计划胜方");
    expect(content).toContain("text 硬上限：136 个非空白字符");
    expect(combined).toContain(villager.persona);
    expect(combined).not.toContain(otherPlayer.persona);
    expect(combined).not.toContain("另一位演员的未来弧线");
  });

  it("renders action candidates with stable id, seat, and name", () => {
    const events = [
      roleAssigned(1, seer.playerId, "seer", "good"),
      phaseStarted(2, "night", 1),
    ];
    const draft = seerDraft(villager.playerId);
    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: seer.playerId,
    });
    const options = legalActionOptions({ game, events, draft });
    const prompt = buildActionPrompt({ context, draft, options });
    const content = prompt.messages[0]!.content;

    expect(prompt.promptVersion).toBe(ACTION_PROMPT_VERSION);
    expect(prompt.promptVersion).toBe("action:v2");
    expect(prompt.schemaName).toBe("werewolf_target_action_v2");
    expect(content).toContain("频道：玩家私有行动");
    expect(content).toContain("候选完全对称则明确这是中立选择");
    expect(content).toContain(
      `${villager.playerId} | ${villager.seatNo} 号 | ${villager.name}`,
    );
    expect(content).toContain("decisionSummary");
    expect(content).not.toContain("PK 投票者");
  });

  it("does not inject a night action prompt into exile voting", () => {
    const strictGame = {
      ...game,
      ruleset: { ...game.ruleset, allowAbstainVote: false },
    };
    const events = [
      roleAssigned(1, wolf.playerId, "werewolf", "wolves"),
      phaseStarted(2, "vote", 1),
    ];
    const draft = voteDraft(wolf.playerId, "exile");
    const context = buildPlayerLlmContext({
      game: strictGame,
      events,
      viewerPlayerId: wolf.playerId,
    });
    const options = legalActionOptions({ game: strictGame, events, draft });
    const prompt = buildActionPrompt({ context, draft, options });
    const combined = `${prompt.systemPrompt}\n${prompt.messages[0]!.content}`;

    expect(combined).toContain("第 1 天放逐投票");
    expect(combined).toContain("本局不允许弃票");
    expect(combined).not.toContain(wolf.roleActionPromptSnapshot ?? "missing");
    expect(combined).not.toContain('"targetPlayerId":null');
  });

  it("shows witch resources and forced no-use when dual use is forbidden", () => {
    const rescuedPlayerId = game.players[0]!.playerId;
    const events = [
      roleAssigned(1, witch.playerId, "witch", "good"),
      phaseStarted(2, "night", 1),
      {
        ...baseEvent(3),
        type: "witch_antidote_decided",
        phase: "night",
        actorPlayerId: witch.playerId,
        targetPlayerIds: [rescuedPlayerId],
        visibility: { kind: "player_private", playerIds: [witch.playerId] },
        payload: { used: true, targetPlayerId: rescuedPlayerId },
      },
    ] satisfies readonly GameEvent[];
    const draft = witchPoisonDraft();
    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: witch.playerId,
    });
    const options = legalActionOptions({ game, events, draft });
    const prompt = buildActionPrompt({ context, draft, options });
    const content = prompt.messages[0]!.content;

    expect(prompt.schemaName).toBe("werewolf_optional_action_v2");
    expect(content).toContain("解药：已用完");
    expect(content).toContain("本夜解药决定：已使用");
    expect(content).toContain("本夜毒药决定：当前正在决定");
    expect(content).toContain("本次行动允许使用：否");
    expect(content).toContain(
      '{"used":false,"targetPlayerId":null,"decisionSummary"',
    );
  });

  it("does not inject another role's action advice into an invalid draft", () => {
    const events = [
      roleAssigned(1, seer.playerId, "seer", "good"),
      phaseStarted(2, "night", 1),
    ];
    const draft = {
      ...wolfVoteDraft(villager.playerId),
      actorPlayerId: seer.playerId,
    };
    const context = buildPlayerLlmContext({
      game,
      events,
      viewerPlayerId: seer.playerId,
    });
    const prompt = buildActionPrompt({
      context,
      draft,
      options: { targetPlayerIds: [], allowNoTarget: false },
    });

    expect(prompt.systemPrompt).not.toContain(
      seer.roleActionPromptSnapshot ?? "missing role action prompt",
    );
  });

});

function daySpeechDraft(
  playerId: PlayerId,
): Extract<DraftEvent, { type: "day_speech_given" }> {
  return {
    ...draftBase("day_speech_given"),
    phase: "speech",
    actorPlayerId: playerId,
    visibility: { kind: "public" },
    payload: { playerId, text: "", dayNumber: 1, round: 1 },
  };
}

function wolfOpinionDraft(
  playerId: PlayerId,
): Extract<DraftEvent, { type: "wolf_opinion_given" }> {
  return {
    ...draftBase("wolf_opinion_given"),
    phase: "night",
    actorPlayerId: playerId,
    visibility: { kind: "faction_private", faction: "wolves" },
    payload: { playerId, text: "", dayNumber: 1 },
  };
}

function seerDraft(
  targetPlayerId: PlayerId,
): Extract<DraftEvent, { type: "seer_check_selected" }> {
  return {
    ...draftBase("seer_check_selected"),
    phase: "night",
    actorPlayerId: seer.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "player_private", playerIds: [seer.playerId] },
    payload: { targetPlayerId },
  };
}

function wolfVoteDraft(
  targetPlayerId: PlayerId,
): Extract<DraftEvent, { type: "wolf_vote_cast" }> {
  return {
    ...draftBase("wolf_vote_cast"),
    phase: "night",
    actorPlayerId: wolf.playerId,
    visibility: { kind: "host_only" },
    payload: {
      voterPlayerId: wolf.playerId,
      targetPlayerId,
      dayNumber: 1,
    },
  };
}

function voteDraft(
  voterPlayerId: PlayerId,
  voteType: "exile" | "pk",
): Extract<DraftEvent, { type: "vote_cast" }> {
  return {
    ...draftBase("vote_cast"),
    phase: "vote",
    actorPlayerId: voterPlayerId,
    visibility: { kind: "host_only" },
    payload: {
      voterPlayerId,
      targetPlayerId: null,
      dayNumber: 1,
      round: voteType === "pk" ? 2 : 1,
      voteType,
    },
  };
}

function witchPoisonDraft(): Extract<
  DraftEvent,
  { type: "witch_poison_decided" }
> {
  return {
    ...draftBase("witch_poison_decided"),
    phase: "night",
    actorPlayerId: witch.playerId,
    visibility: { kind: "player_private", playerIds: [witch.playerId] },
    payload: { used: false, targetPlayerId: null },
  };
}

function daySpeech(
  index: number,
  playerId: PlayerId,
  text: string,
): Extract<GameEvent, { type: "day_speech_given" }> {
  return {
    ...baseEvent(index),
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: playerId,
    visibility: { kind: "public" },
    payload: { playerId, text, dayNumber: 1, round: 1 },
  };
}

function roleAssigned(
  index: number,
  playerId: PlayerId,
  role: GameRole,
  faction: Faction,
): Extract<GameEvent, { type: "role_assigned" }> {
  return {
    ...baseEvent(index),
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [playerId],
    visibility: { kind: "player_private", playerIds: [playerId] },
    payload: { playerId, role, faction },
  };
}

function phaseStarted(
  index: number,
  phase: Extract<GameEvent, { type: "phase_started" }>["payload"]["phase"],
  dayNumber: number,
): Extract<GameEvent, { type: "phase_started" }> {
  return {
    ...baseEvent(index),
    type: "phase_started",
    phase,
    visibility: { kind: "public" },
    payload: { phase, dayNumber },
  };
}

function draftBase<Type extends DraftEvent["type"]>(type: Type) {
  return {
    id: `draft_${type}` as DraftId,
    gameId,
    status: "draft" as const,
    type,
    createdAt,
  };
}

function baseEvent(index: number) {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active" as const,
    createdAt,
  };
}

function playerByRole(role: GameRole) {
  const player = game.players.find((candidate) => candidate.gameRole === role);
  if (!player) throw new Error(`Missing player for role ${role}`);
  return player;
}
