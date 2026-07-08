import type { PlaybackSceneKind } from "@/core/playback";
import type { RenderablePlayer, ShotFrame } from "../shot-engine/types";

export type CaseBoardTone = "day" | "night" | "danger" | "good" | "neutral";

export type CaseBoardRowTone =
  | "detail"
  | "alive-player"
  | "dead-player"
  | "neutral"
  | "result";

export type CaseBoardRow = {
  readonly tone: CaseBoardRowTone;
  readonly label: string;
  readonly text: string;
  readonly meta?: string;
  readonly count?: number;
};

export type CaseBoardMetric = {
  readonly tone: CaseBoardRowTone;
  readonly label: string;
  readonly value: string;
  readonly caption?: string;
};

export type CaseBoardParticipant = {
  readonly tone: CaseBoardRowTone;
  readonly label: string;
  readonly name: string;
  readonly roleName?: string;
};

export type CaseBoardRelation = {
  readonly actor: CaseBoardParticipant | null;
  readonly action: string;
  readonly target: CaseBoardParticipant | null;
  readonly result?: string;
};

export type CaseBoardContentVariant =
  | "phase"
  | "night-action"
  | "death"
  | "standard"
  | "vote"
  | "ending";

export type CaseBoardContent = {
  readonly variant: CaseBoardContentVariant;
  readonly tone: CaseBoardTone;
  readonly kindLabel: string;
  readonly title: string;
  readonly hero: string;
  readonly body: string;
  readonly metrics: readonly CaseBoardMetric[];
  readonly relation: CaseBoardRelation | null;
  readonly rows: readonly CaseBoardRow[];
};

export function caseBoardContentForFrame(frame: ShotFrame): CaseBoardContent {
  if (isEndingScene(frame)) {
    const winner = winnerTitle(frame.scene.title);
    return {
      variant: "ending",
      tone: winner.includes("狼人") ? "danger" : "good",
      kindLabel: "终局",
      title: "游戏结束",
      hero: winner,
      body: endReason(frame.scene.text),
      metrics: [],
      relation: null,
      rows: [],
    };
  }

  if (isVoteResolutionScene(frame)) {
    return {
      variant: "vote",
      tone: "danger",
      kindLabel: "投票结算",
      title: frame.scene.title,
      hero: exileHero(frame.scene.text),
      body: "",
      metrics: [],
      relation: null,
      rows: voteRows(frame.scene.details),
    };
  }

  if (frame.scene.kind === "phase") {
    return {
      variant: "phase",
      tone: toneForPhase(frame.scene.phase),
      kindLabel: phaseKindLabel(frame.scene.phase),
      title: trimStarted(frame.scene.title),
      hero: phaseHero(frame.scene.phase),
      body: "",
      metrics: [],
      relation: null,
      rows: [],
    };
  }

  if (isDeathScene(frame)) {
    const death = deathContent(frame);
    return {
      variant: "death",
      tone: death.tone,
      kindLabel: "死讯",
      title: frame.scene.title,
      hero: death.hero,
      body: death.body,
      metrics: [],
      relation: null,
      rows: highlightedRows(frame),
    };
  }

  if (isNightActionScene(frame)) {
    const action = actionContent(frame);
    return {
      variant: "night-action",
      tone: toneForNightAction(frame.scene.title),
      kindLabel: "夜间行动",
      title: frame.scene.title,
      hero: action.hero,
      body: action.body,
      metrics: action.metrics,
      relation: action.relation,
      rows: [],
    };
  }

  const detailRows = frame.scene.details.map<CaseBoardRow>((detail, index) => ({
    tone: "detail",
    label: formatNumberLabel(index + 1),
    text: detail,
  }));
  const playerRows = frame.highlightedPlayers.map(playerRow);

  return {
    variant: "standard",
    tone: toneForPhase(frame.scene.phase),
    kindLabel: kindLabelForScene(frame.scene.kind),
    title: frame.scene.title,
    hero: frame.scene.text || frame.scene.title,
    body: frame.scene.text ? "" : "等待下一步对局信息",
    metrics: [],
    relation: null,
    rows: detailRows.length > 0
      ? detailRows
      : playerRows.length > 0
        ? playerRows
        : [{ tone: "neutral", label: "--", text: "暂无补充信息" }],
  };
}

function isEndingScene(frame: ShotFrame): boolean {
  return frame.scene.title.startsWith("游戏结束：");
}

function isVoteResolutionScene(frame: ShotFrame): boolean {
  return (
    frame.scene.kind === "resolution" &&
    (frame.scene.title === "投票结算" || frame.scene.title === "PK 结算") &&
    frame.scene.details.some((detail) => detail.includes("->"))
  );
}

function isDeathScene(frame: ShotFrame): boolean {
  return frame.scene.title === "昨夜死讯" || frame.scene.title === "夜间结算";
}

function isNightActionScene(frame: ShotFrame): boolean {
  return [
    "狼人刀人",
    "预言家查验",
    "查验结果",
    "女巫死亡信息",
    "女巫解药",
    "女巫毒药",
    "守卫守护",
    "猎人开枪",
  ].includes(frame.scene.title);
}

function actionContent(frame: ShotFrame): {
  readonly hero: string;
  readonly body: string;
  readonly metrics: readonly CaseBoardMetric[];
  readonly relation: CaseBoardRelation | null;
} {
  const text = frame.scene.text;
  const actor = actorFromText(frame, text);
  const target = targetFromText(text);
  switch (frame.scene.title) {
    case "预言家查验":
      return {
        hero: target ?? "选择查验目标",
        body: "查验目标",
        metrics: [],
        relation: relationFor(frame, actor, "查验", target),
      };
    case "查验结果":
      return {
        hero: resultFromText(text) ?? "查验完成",
        body: target ?? "",
        metrics: [],
        relation: relationFor(frame, actor, "查验", target, resultFromText(text)?.replace(/^阵营：/, "")),
      };
    case "女巫死亡信息":
      return {
        hero: text.includes("无人") ? "无人被袭击" : (targetAfter(text, "被袭击：") ?? "获得夜间死讯"),
        body: "夜间被袭击目标",
        metrics: [],
        relation: relationFor(
          frame,
          actor,
          "获知",
          text.includes("无人") ? null : targetAfter(text, "被袭击："),
        ),
      };
    case "女巫解药":
      return medicineContent(frame, text, "解药", actor);
    case "女巫毒药":
      return medicineContent(frame, text, "毒药", actor);
    case "狼人刀人":
      return {
        hero: target ?? "选择击杀目标",
        body: "狼人击杀目标",
        metrics: [],
        relation: relationFor(frame, actor, "刀人", target),
      };
    case "守卫守护":
      return {
        hero: target ?? "选择守护目标",
        body: "守护目标",
        metrics: [],
        relation: relationFor(frame, actor, "守护", target),
      };
    case "猎人开枪":
      return {
        hero: target ?? "开枪目标",
        body: "猎人带走目标",
        metrics: [],
        relation: relationFor(frame, actor, "开枪", target),
      };
    default:
      return { hero: text, body: "", metrics: [], relation: null };
  }
}

function medicineContent(
  frame: ShotFrame,
  text: string,
  label: "解药" | "毒药",
  actor: string | null,
): {
  readonly hero: string;
  readonly body: string;
  readonly metrics: readonly CaseBoardMetric[];
  readonly relation: CaseBoardRelation | null;
} {
  const used = !text.includes("不使用");
  const target = used ? targetFromText(text) : null;
  return {
    hero: used ? `使用${label}` : `未使用${label}`,
    body: target ?? "",
    metrics: [],
    relation: relationFor(frame, actor, label, target, used ? "已使用" : "未使用"),
  };
}

function deathContent(frame: ShotFrame): {
  readonly tone: CaseBoardTone;
  readonly hero: string;
  readonly body: string;
} {
  const text = frame.scene.text;
  if (text.includes("平安夜") || text.includes("无人死亡")) {
    return { tone: "good", hero: "平安夜", body: "无人死亡" };
  }

  const deadPlayers = frame.highlightedPlayers
    .filter((player) => player.status === "dead")
    .map((player) => `${player.seatNo} 号 ${player.name}`);
  return {
    tone: "danger",
    hero: deadPlayers.length > 0 ? deadPlayers.join("、") : "出现死亡",
    body: frame.scene.title === "昨夜死讯" ? "昨夜死亡" : "夜间死亡",
  };
}

function voteRows(details: readonly string[]): readonly CaseBoardRow[] {
  const buckets = new Map<string, string[]>();

  details.forEach((detail) => {
    const [voter, target] = detail.split("->").map((part) => part.trim());
    if (!voter || !target) {
      return;
    }

    const voters = buckets.get(target) ?? [];
    voters.push(shortSeatLabel(voter));
    buckets.set(target, voters);
  });

  return [...buckets.entries()]
    .sort(([, left], [, right]) => right.length - left.length)
    .map(([target, voters], index) => ({
      tone: index === 0 && target !== "弃票" ? "result" : "detail",
      label: target === "弃票" ? "--" : formatNumberLabel(seatNoFromLabel(target)),
      text: target,
      meta: voters.join("  "),
      count: voters.length,
    }));
}

function highlightedRows(frame: ShotFrame): readonly CaseBoardRow[] {
  return frame.highlightedPlayers.length > 0
    ? frame.highlightedPlayers.map(playerRow)
    : [{ tone: "neutral", label: "--", text: "无目标" }];
}

function winnerTitle(title: string): string {
  return title.replace(/^游戏结束：/, "");
}

function endReason(text: string): string {
  const match = /原因：(.+?)。?$/.exec(text);
  return match ? `原因：${match[1]}` : text;
}

function exileHero(text: string): string {
  const match = /(?:放逐出局|PK 出局)：(.+?)。?$/.exec(text);
  return match ? `${match[1]} 出局` : text;
}

function targetFromText(text: string): string | null {
  const patterns = [
    /(?:选择查验|查验|选择击杀|使用解药救|使用毒药毒|选择守护|开枪带走)\s*(.+?)。?$/,
    /目标：\s*(.+?)。?$/,
  ];
  for (const pattern of patterns) {
    const match = pattern.exec(text);
    if (match?.[1]) {
      return match[1].replace(/，结果：.+$/, "");
    }
  }
  return null;
}

function actorFromText(frame: ShotFrame, text: string): string | null {
  const match = /^(\d+\s*号\s*[^，。 ]+)/.exec(text);
  if (match?.[1]) {
    return match[1];
  }
  const actor = frame.highlightedPlayers.find((player) => (
    text.startsWith(`${player.seatNo} 号 ${player.name}`) ||
    text.startsWith(`${player.seatNo}号${player.name}`)
  ));
  return actor ? `${actor.seatNo} 号 ${actor.name}` : null;
}

function relationFor(
  frame: ShotFrame,
  actorLabel: string | null,
  action: string,
  targetLabel: string | null,
  result?: string,
): CaseBoardRelation {
  return {
    actor: participantFromLabel(frame, actorLabel),
    action,
    target: participantFromLabel(frame, targetLabel),
    result,
  };
}

function participantFromLabel(
  frame: ShotFrame,
  label: string | null,
): CaseBoardParticipant | null {
  if (!label) {
    return null;
  }
  const seatNo = seatNoFromLabel(label);
  const player = frame.players.find((candidate) => candidate.seatNo === seatNo);
  return {
    tone: player?.status === "dead" ? "dead-player" : "alive-player",
    label: seatNo > 0 ? formatNumberLabel(seatNo) : "--",
    name: player?.name ?? label.replace(/^\d+\s*号\s*/, ""),
    roleName: player?.roleName,
  };
}

function targetAfter(text: string, marker: string): string | null {
  const index = text.indexOf(marker);
  if (index < 0) {
    return null;
  }
  return text.slice(index + marker.length).replace(/。$/, "");
}

function resultFromText(text: string): string | null {
  const match = /结果：(.+?)。?$/.exec(text);
  return match ? `阵营：${match[1]}` : null;
}

function toneForNightAction(title: string): CaseBoardTone {
  if (title === "狼人刀人" || title === "女巫毒药" || title === "猎人开枪") {
    return "danger";
  }
  if (title === "预言家查验" || title === "查验结果" || title === "守卫守护") {
    return "good";
  }
  return "night";
}

function toneForPhase(phase: ShotFrame["scene"]["phase"]): CaseBoardTone {
  if (phase === "night") return "night";
  if (phase === "day" || phase === "speech") return "day";
  if (phase === "vote" || phase === "pk" || phase === "last_words") return "danger";
  return "neutral";
}

function phaseKindLabel(phase: ShotFrame["scene"]["phase"]): string {
  switch (phase) {
    case "night":
      return "夜晚";
    case "day":
      return "天亮";
    case "speech":
      return "发言";
    case "vote":
      return "投票";
    case "pk":
      return "PK";
    case "last_words":
      return "遗言";
    case "ended":
      return "终局";
    case "setup":
      return "准备";
  }
}

function phaseHero(phase: ShotFrame["scene"]["phase"]): string {
  switch (phase) {
    case "night":
      return "夜间行动";
    case "day":
      return "天亮公布";
    case "speech":
      return "依次发言";
    case "vote":
      return "开始投票";
    case "pk":
      return "进入 PK";
    case "last_words":
      return "遗言阶段";
    case "ended":
      return "对局结束";
    case "setup":
      return "准备开始";
  }
}

function trimStarted(title: string): string {
  return title.replace(/开始$/, "");
}

function shortSeatLabel(label: string): string {
  return formatNumberLabel(seatNoFromLabel(label));
}

function seatNoFromLabel(label: string): number {
  const match = /^(\d+)\s*号/.exec(label);
  return match ? Number(match[1]) : 0;
}

function playerRow(player: RenderablePlayer): CaseBoardRow {
  return {
    tone: player.status === "dead" ? "dead-player" : "alive-player",
    label: formatNumberLabel(player.seatNo),
    text: player.name,
    meta: player.roleName,
  };
}

function kindLabelForScene(kind: PlaybackSceneKind): string {
  switch (kind) {
    case "phase":
      return "阶段";
    case "announcement":
      return "播报";
    case "vote":
      return "投票";
    case "resolution":
      return "结算";
    case "speech":
      return "发言";
  }
}

function formatNumberLabel(value: number): string {
  return String(value).padStart(2, "0");
}
