import { ArrowLeft, Check, Clapperboard, Film, RefreshCw } from "lucide-react";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";
import {
  approveEpisodeScriptAction,
  generateEpisodeScriptAction,
} from "@/app/actions";
import { FormSubmitButton } from "@/components/editor/form-submit-button";
import { LlmGenerationDetails } from "@/components/editor/llm-generation-details";
import { EpisodeAutoContinueButton } from "@/components/script/episode-auto-continue";
import { EpisodeEnsembleReview } from "@/components/script/episode-ensemble-review";
import { EpisodeGeneratingRefresh } from "@/components/script/episode-generating-refresh";
import { iconButtonClassName } from "@/components/ui/button-styles";
import { episodeAuthorProgress } from "@/core/episode-author";
import type {
  EpisodeAuthorRequestRecord,
  EpisodeAuthorWorkspace,
  EpisodeScriptState,
} from "@/core/episode-script";
import { tokenCount } from "@/core/token-usage";
import type { GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import type { GameRecord } from "@/server/game-repository";

export const dynamic = "force-dynamic";

export default async function ScriptPreparationPage({
  params,
}: {
  readonly params: Promise<{ readonly gameId: string }>;
}) {
  const { gameId } = await params;
  const record = await createGameActions(
    createGameRepository(process.env.KIVA_DATA_DIR),
  ).getGame(gameId as GameId);
  if (!record) notFound();
  if (record.game.runMode !== "scripted") {
    redirect(`/games/${record.game.id}/editor`);
  }
  const requests = episodeAuthorRequestsForRecord(record);

  return (
    <main className="h-screen overflow-hidden bg-background p-3 text-foreground sm:p-4">
      <div className="grid h-full min-h-0 grid-rows-[auto_minmax(0,1fr)] gap-3">
        <header className="flex h-10 min-w-0 shrink-0 items-center gap-3 border-b border-border pb-2">
          <Link
            href="/"
            aria-label="Back"
            title="Back"
            className={iconButtonClassName()}
          >
            <ArrowLeft aria-hidden="true" className="h-4 w-4" />
          </Link>
          <div className="min-w-0">
            <p className="text-[10px] font-medium uppercase leading-none tracking-[0.16em] text-cyan-300">
              Scripted mode
            </p>
            <h1 className="mt-1 truncate text-base font-semibold">
              {record.game.title}
            </h1>
          </div>
        </header>

        <section className="grid min-h-0 gap-3 overflow-hidden grid-rows-[minmax(280px,0.95fr)_minmax(320px,1.05fr)] lg:grid-cols-[minmax(360px,0.76fr)_minmax(520px,1.24fr)] lg:grid-rows-1">
          <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-border bg-surface/45">
            <header className="flex h-12 shrink-0 items-center justify-between gap-3 border-b border-border px-4">
              <div className="flex items-center gap-2 text-sm font-semibold">
                <Clapperboard
                  aria-hidden="true"
                  className="h-4 w-4 text-cyan-300"
                />
                <h2>剧本控制台</h2>
              </div>
              <span className="font-mono text-[10px] uppercase tracking-[0.12em] text-subtle">
                Script Author
              </span>
            </header>
            <div className="min-h-0 flex-1 overflow-y-auto p-3 sm:p-4">
              <div className="rounded-lg border border-warning-badge/60 bg-warning-badge/25 px-3 py-2.5 text-xs leading-5 text-warning-badge-foreground">
                {record.episodeScript?.status === "approved"
                  ? "剧本已经批准，结构已锁定。"
                  : "剧本批准前不会推进游戏、生成玩家配音或允许导出。当前阵容与合法轨迹保持锁定。"}
              </div>
              <div className="mt-3">
                <EpisodeWorkspace record={record} />
              </div>
              <LockedGameFacts record={record} />
            </div>
          </section>

          <EpisodeAuthorRequests requests={requests} />
        </section>
      </div>
    </main>
  );
}

export function EpisodeWorkspace({
  record,
}: {
  readonly record: GameRecord;
}) {
  const persistedState = record.episodeScript;
  if (!persistedState) {
    throw new Error("Scripted game is missing its episode script state");
  }
  const state = episodeScriptDisplayState(persistedState);
  const automationControl = (
    <div className="flex items-center justify-between gap-3 rounded-lg border border-border bg-background/35 p-3">
      <div>
        <div className="text-xs font-medium text-foreground">自动生成下一步</div>
        <p className="mt-1 text-[11px] leading-4 text-subtle">
          每个检查点完成 1 秒后继续
        </p>
      </div>
      <EpisodeAutoContinueButton
        gameId={record.game.id}
        expectedJobId={"jobId" in state ? state.jobId : null}
        ready={state.status === "ready"}
      />
    </div>
  );
  if (state.status === "review") {
    const script = state.candidate;
    const speechSteps = script.steps.filter((step) => step.speechBeat);
    return (
      <div>
        <EpisodeWorkspaceStatusCard
          state={state}
          playerCount={record.game.players.length}
        />
        <div className="mt-4 flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="text-xs font-medium uppercase tracking-[0.16em] text-cyan-300">Director review</p>
            <h2 className="mt-1 text-xl font-semibold">{script.title}</h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-muted">{script.logline}</p>
          </div>
          <span className={state.report.valid ? "rounded-full bg-good-badge px-3 py-1 text-xs text-good-badge-foreground" : "rounded-full bg-danger-badge px-3 py-1 text-xs text-danger-badge-foreground"}>
            {state.report.valid ? "校验通过" : "校验失败"}
          </span>
        </div>

        <div className="mt-4 grid grid-cols-2 gap-2">
          <Metric label="计划胜方" value={script.plannedWinner === "good" ? "好人" : "狼人"} />
          <Metric label="计划天数" value={`${script.plannedDayCount} 天`} />
          <Metric label="预计时长" value={formatDuration(script.targetDurationMs)} />
          <Metric label="结构规模" value={`${script.steps.length} 步 / ${speechSteps.length} 段发言`} />
        </div>

        <div className="mt-3">{automationControl}</div>

        <section className="mt-4">
          <h3 className="text-sm font-semibold">剧情幕</h3>
          <div className="mt-2 grid gap-2 sm:grid-cols-2">
            {script.acts.map((act) => (
              <article key={act.title} className="rounded border border-border bg-background/50 p-3">
                <h4 className="text-sm font-medium">{act.title}</h4>
                <p className="mt-1 text-xs leading-5 text-muted">{act.summary}</p>
              </article>
            ))}
          </div>
        </section>

        <EpisodeEnsembleReview
          script={script}
          players={record.game.players}
        />

        <details className="mt-5 rounded border border-border bg-background/40">
          <summary className="cursor-pointer px-4 py-3 text-sm font-medium">查看完整结构与发言节拍</summary>
          <div className="max-h-96 space-y-2 overflow-y-auto border-t border-border p-3">
            {script.steps.map((step) => (
              <div key={step.index} className="rounded border border-border/70 px-3 py-2 text-xs">
                <div className="font-mono text-subtle">#{step.index} · {step.slot.phase} · {step.slot.type}</div>
                <div className="mt-1 text-muted">{step.summary}</div>
                {step.speechBeat ? (
                  <div className="mt-2 border-l-2 border-cyan-700 pl-2 text-muted">
                    {step.speechBeat.objective} · {step.speechBeat.themeHook}
                  </div>
                ) : null}
              </div>
            ))}
          </div>
        </details>

        {state.report.warnings.length > 0 ? (
          <ul className="mt-4 space-y-1 text-xs text-warning-badge-foreground">
            {state.report.warnings.map((warning) => <li key={warning}>{warning}</li>)}
          </ul>
        ) : null}

        <div className="mt-5 flex flex-wrap justify-end gap-2 border-t border-border pt-4">
          <form
            action={generateEpisodeScriptAction.bind(
              null,
              record.game.id,
              state.jobId,
            )}
          >
            <FormSubmitButton label={<><RefreshCw className="h-4 w-4" /> 重新生成</>} pendingLabel="重新生成中…" className={secondaryActionClass} />
          </form>
          <form action={approveEpisodeScriptAction.bind(null, record.game.id, state.jobId, script.id)}>
            <FormSubmitButton label={<><Check className="h-4 w-4" /> 批准并开局</>} pendingLabel="批准中…" disabled={!state.report.valid} className={primaryActionClass} />
          </form>
        </div>
      </div>
    );
  }

  if (state.status === "failed") {
    return (
      <div className="text-left">
        <EpisodeWorkspaceStatusCard
          state={state}
          playerCount={record.game.players.length}
        />
        <p className="mt-3 rounded border border-danger-badge/60 bg-danger-badge/25 px-3 py-2 text-sm text-danger-badge-foreground">
          {state.error}
        </p>
        <div className="mt-4">{automationControl}</div>
        <form
          className="mt-4"
          action={generateEpisodeScriptAction.bind(
            null,
            record.game.id,
            state.jobId,
          )}
        >
          <FormSubmitButton
            label="重试当前阶段"
            pendingLabel="正在重试当前阶段…"
            className={primaryActionClass}
          />
        </form>
      </div>
    );
  }

  if (state.status === "ready") {
    return (
      <div className="text-left">
        <EpisodeWorkspaceStatusCard
          state={state}
          playerCount={record.game.players.length}
        />
        <div className="mt-4">{automationControl}</div>
        <form
          className="mt-3"
          action={generateEpisodeScriptAction.bind(
            null,
            record.game.id,
            state.jobId,
          )}
        >
          <FormSubmitButton
            label="生成下一步"
            pendingLabel="正在开始下一步…"
            className={widePrimaryActionClass}
          />
        </form>
      </div>
    );
  }

  if (state.status === "generating") {
    return (
      <div className="text-left">
        <EpisodeGeneratingRefresh />
        <EpisodeWorkspaceStatusCard
          state={state}
          playerCount={record.game.players.length}
        />
        <div className="mt-3">{automationControl}</div>
        <form
          className="mt-3"
          action={generateEpisodeScriptAction.bind(
            null,
            record.game.id,
            state.jobId,
          )}
        >
          <FormSubmitButton
            label="重试当前阶段"
            pendingLabel="正在重试当前阶段…"
            className="rounded-md border border-border px-3 py-2 text-xs text-muted"
          />
        </form>
      </div>
    );
  }

  if (state.status === "approved") {
    redirect(`/games/${record.game.id}/editor`);
  }

  return (
    <div>
      <EpisodeWorkspaceStatusCard
        state={state}
        playerCount={record.game.players.length}
      />
      <div className="mt-4 flex items-start gap-3">
        <Film aria-hidden="true" className="mt-0.5 h-5 w-5 text-cyan-300" />
        <div>
          <h2 className="text-base font-semibold">生成一份可执行的单局剧本</h2>
          <p className="mt-2 text-sm leading-6 text-muted">
            系统会先用当前角色与规则模拟完整终局，再让 Script Author 为合法轨迹编写主题剧情和逐场发言节拍。
          </p>
        </div>
      </div>
      <ol className="mt-5 grid gap-2 text-sm text-subtle sm:grid-cols-4">
        {["1. 规则规划", "2. 主题编写", "3. 导演审核", "4. 批准开局"].map((step) => (
          <li key={step} className="rounded border border-border bg-background/50 px-3 py-3">{step}</li>
        ))}
      </ol>
      <div className="mt-4">{automationControl}</div>
      <form
        className="mt-3"
        action={generateEpisodeScriptAction.bind(null, record.game.id, null)}
      >
        <FormSubmitButton label="开始生成剧本" pendingLabel="正在生成并模拟…" className={widePrimaryActionClass} />
      </form>
    </div>
  );
}

function EpisodeWorkspaceStatusCard({
  state,
  playerCount,
}: {
  readonly state: EpisodeScriptState;
  readonly playerCount: number;
}) {
  const presentation = episodeWorkspaceStatusPresentation(state, playerCount);
  const progressValue = Math.round(
    (presentation.progress.completed /
      Math.max(presentation.progress.total, 1)) *
      100,
  );

  return (
    <section className="overflow-hidden rounded-lg border border-border bg-background/55">
      <div className="p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0">
            <p className="text-[10px] font-medium uppercase tracking-[0.1em] text-subtle">
              当前状态
            </p>
            <h2 className="mt-1 text-base font-semibold text-foreground">
              {presentation.title}
            </h2>
            <p className="mt-1 text-xs leading-5 text-muted">
              {presentation.copy}
            </p>
          </div>
          <span
            className={`${presentation.badgeClassName} inline-flex shrink-0 items-center gap-1.5 rounded-full border px-2.5 py-1 font-mono text-[10px] font-semibold`}
          >
            <span
              aria-hidden="true"
              className={`h-1.5 w-1.5 rounded-full bg-current ${state.status === "generating" ? "animate-pulse" : ""}`}
            />
            {presentation.badge}
          </span>
        </div>

        <div className="mt-4">
          <div className="mb-2 flex items-center justify-between gap-3 font-mono text-[10px] text-subtle">
            <span className="truncate">{presentation.progress.label}</span>
            <span className="shrink-0">
              {presentation.progress.completed} / {presentation.progress.total}
            </span>
          </div>
          <div
            role="progressbar"
            aria-label={`Script Author ${presentation.progress.label}`}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={progressValue}
            className="h-1 overflow-hidden rounded-full bg-surface-muted"
          >
            <div
              className="h-full rounded-full bg-accent transition-[width] duration-500"
              style={{ width: `${progressValue}%` }}
            />
          </div>
        </div>
      </div>

      <dl className="grid grid-cols-5 border-t border-border">
        <StatusMetric label="故事主轴" value={presentation.story} />
        <StatusMetric label="群像分工" value={presentation.ensemble} />
        <StatusMetric label="角色弧线" value={presentation.actorArcs} />
        <StatusMetric label="关系弧线" value={presentation.relationships} />
        <StatusMetric label="场景节拍" value={presentation.sceneBeats} />
      </dl>
    </section>
  );
}

function StatusMetric({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}) {
  return (
    <div className="min-w-0 border-r border-border px-2 py-2.5 last:border-r-0">
      <dt className="truncate text-[10px] text-subtle">{label}</dt>
      <dd className="mt-1 font-mono text-[11px] font-semibold text-foreground">
        {value}
      </dd>
    </div>
  );
}

function episodeWorkspaceStatusPresentation(
  state: EpisodeScriptState,
  playerCount: number,
) {
  if (state.status === "idle") {
    return {
      badge: "IDLE",
      badgeClassName: "border-border bg-surface-muted text-muted",
      title: "等待生成剧本",
      copy: "尚未创建 Script Author 工作区。",
      progress: { label: "等待开始", completed: 0, total: 1 },
      story: "0 / 1",
      ensemble: "0 / 1",
      actorArcs: `0 / ${playerCount}`,
      relationships: "0 / —",
      sceneBeats: "0 / —",
    };
  }

  if (state.status === "review" || state.status === "approved") {
    const script = state.status === "review" ? state.candidate : state.script;
    return {
      badge: state.status === "review" ? "REVIEW" : "APPROVED",
      badgeClassName:
        "border-good-badge/70 bg-good-badge text-good-badge-foreground",
      title:
        state.status === "review" ? "剧本候选已生成" : "剧本已经批准",
      copy:
        state.status === "review"
          ? "完整结构已经组装并校验，等待导演审核。"
          : "剧本结构已经锁定，可以进入游戏编辑器。",
      progress: { label: "导演审核", completed: 1, total: 1 },
      story: "1 / 1",
      ensemble: "1 / 1",
      actorArcs: `${script.castDirections.length} / ${playerCount}`,
      relationships: `${script.relationships.length} / ${script.relationships.length}`,
      sceneBeats: `${state.report.speechCount} / ${state.report.speechCount}`,
    };
  }

  const progress = episodeAuthorProgress(state.workspace);
  const relationshipTotal = state.workspace.ensemble?.relationshipSeeds.length;
  const common = {
    progress,
    story: `${state.workspace.story ? 1 : 0} / 1`,
    ensemble: `${state.workspace.ensemble ? 1 : 0} / 1`,
    actorArcs: `${state.workspace.castDirections.length} / ${playerCount}`,
    relationships: `${state.workspace.relationships.length} / ${relationshipTotal ?? "—"}`,
    sceneBeats: `${state.workspace.beats.length} / ${state.workspace.speechStepCount}`,
  };

  if (state.status === "generating") {
    return {
      ...common,
      badge: "GENERATING",
      badgeClassName:
        "border-info-badge/70 bg-info-badge text-info-badge-foreground",
      title: "正在由 Script Author Agent 细化剧本",
      copy: "正在处理当前任务，已完成结果会持续写入工作区。",
    };
  }

  if (state.status === "failed") {
    return {
      ...common,
      badge: "FAILED",
      badgeClassName:
        "border-danger-badge/70 bg-danger-badge text-danger-badge-foreground",
      title: "剧本生成失败",
      copy: `已保留 ${state.workspace.castDirections.length} 个角色弧线和 ${state.workspace.beats.length} 个场景节拍；重试会从当前任务继续。`,
    };
  }

  return {
    ...common,
    badge: "READY",
    badgeClassName:
      "border-good-badge/70 bg-good-badge text-good-badge-foreground",
    title: "当前步骤已完成",
    copy: `下一步：${progress.label} · ${progress.completed}/${progress.total}`,
  };
}

export function EpisodeAuthorGeneratingStatus({
  workspace,
}: {
  readonly workspace: EpisodeAuthorWorkspace;
}) {
  const progress = episodeAuthorProgress(workspace);
  return (
    <>
      <h2 className="mt-4 text-base font-semibold">
        正在由 Script Author Agent 细化剧本
      </h2>
      <p className="mt-2 text-sm text-muted">
        {progress.label} · {progress.completed}/{progress.total}
      </p>
    </>
  );
}

export function EpisodeAuthorRequests({
  requests,
}: {
  readonly requests: readonly EpisodeAuthorRequestRecord[];
}) {
  const labels = episodeAuthorRequestLabels(requests);
  const entries = requests
    .map((request, index) => ({
      request,
      label: labels[index]!,
      sequence: index + 1,
    }))
    .reverse();
  return (
    <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-border bg-surface/45">
      <header className="flex h-12 shrink-0 items-center justify-between gap-3 border-b border-border px-4">
        <h2 className="text-sm font-semibold">LLM requests</h2>
        <span className="font-mono text-[10px] uppercase tracking-[0.08em] text-subtle">
          {requests.length} requests · {formatRequestTokens(requests)} tokens
        </span>
      </header>
      {requests.length === 0 ? (
        <div className="grid min-h-0 flex-1 place-items-center p-6 text-center">
          <div>
            <Film aria-hidden="true" className="mx-auto h-5 w-5 text-subtle" />
            <p className="mt-2 text-xs text-subtle">
              开始生成后，请求记录会按执行顺序出现在这里。
            </p>
          </div>
        </div>
      ) : (
        <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain">
          <div className="divide-y divide-border">
            {entries.map(({ request, label, sequence }) => (
              <div
                key={request.id}
                className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-3 px-4 py-3 text-xs transition-colors hover:bg-surface-muted/35"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-[10px] text-subtle">
                      {String(sequence).padStart(2, "0")}
                    </span>
                    <div className="truncate font-medium text-foreground">
                      {label}
                    </div>
                  </div>
                  <div className="mt-1 flex min-w-0 items-center gap-1.5 pl-7 font-mono text-[10px] text-subtle">
                    <span
                      aria-hidden="true"
                      className={
                        request.status === "success"
                          ? "h-1.5 w-1.5 shrink-0 rounded-full bg-good-badge-foreground"
                          : "h-1.5 w-1.5 shrink-0 rounded-full bg-danger-badge-foreground"
                      }
                    />
                    <span className="truncate">
                      {request.provider}/{request.model} · {request.status} ·{" "}
                      {request.task.kind === "beats"
                        ? `steps ${request.task.stepIndexes.join(", ")}`
                        : request.task.kind}
                    </span>
                  </div>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <span className="font-mono text-[10px] text-muted">
                    {formatRequestTokens([request])} tokens
                  </span>
                  <LlmGenerationDetails
                    generation={request}
                    label={`${label} LLM details`}
                    showDecisionSummary={false}
                  />
                </div>
              </div>
            ))}
          </div>
          <p className="px-4 py-5 text-center text-[10px] text-subtle">
            后续请求会追加到这里，不改变左侧操作区位置。
          </p>
        </div>
      )}
    </section>
  );
}

function LockedGameFacts({ record }: { readonly record: GameRecord }) {
  return (
    <section className="mt-5">
      <h3 className="text-[10px] font-medium uppercase tracking-[0.12em] text-subtle">
        本局信息
      </h3>
      <dl className="mt-2 overflow-hidden rounded-lg border border-border text-xs">
        <GameFact label="主题模板" value={record.game.script.name} />
        <GameFact
          label="演员阵容"
          value={`${record.game.players.length} 人 · 已快照`}
        />
        <GameFact label="目标时长" value="28–32 分钟" />
        <GameFact label="运行模式" value="Scripted" />
      </dl>
    </section>
  );
}

function GameFact({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}) {
  return (
    <div className="grid grid-cols-[96px_minmax(0,1fr)] gap-3 border-b border-border px-3 py-2.5 last:border-b-0">
      <dt className="text-subtle">{label}</dt>
      <dd className="truncate text-right text-foreground">{value}</dd>
    </div>
  );
}

function episodeAuthorRequestsForRecord(
  record: GameRecord,
): readonly EpisodeAuthorRequestRecord[] {
  const state = record.episodeScript;
  return state && "requests" in state ? state.requests : [];
}

function episodeScriptDisplayState(
  state: EpisodeScriptState,
): EpisodeScriptState {
  if (
    state.status !== "generating" ||
    !generatingTaskResultIsPersisted(state)
  ) {
    return state;
  }
  return {
    status: "ready",
    jobId: state.jobId,
    workspace: state.workspace,
    requests: state.requests,
  };
}

function generatingTaskResultIsPersisted(
  state: Extract<EpisodeScriptState, { readonly status: "generating" }>,
): boolean {
  const currentJobRequests = state.requests.filter(
    (request) => request.createdAt >= state.startedAt,
  );
  const latestRequest = currentJobRequests.at(-1);
  if (!latestRequest || latestRequest.status !== "success") return false;
  const task = latestRequest.task;

  switch (task.kind) {
    case "story":
      return state.workspace.story !== null;
    case "ensemble":
      return state.workspace.ensemble !== null;
    case "actor_arc":
      return state.workspace.castDirections.some(
        (direction) => direction.playerId === task.playerId,
      );
    case "relationship":
      return state.workspace.relationships.some((relationship) =>
        task.playerIds.every((playerId) =>
          relationship.playerIds.includes(playerId),
        ),
      );
    case "beats": {
      const rootSplitRequest = currentJobRequests.find(
        (request) =>
          request.status === "failed" && request.task.kind === "beats",
      );
      const requiredStepIndexes =
        rootSplitRequest?.task.kind === "beats"
          ? rootSplitRequest.task.stepIndexes
          : task.stepIndexes;
      const completedStepIndexes = new Set(
        state.workspace.beats.map((beat) => beat.stepIndex),
      );
      return requiredStepIndexes.every((stepIndex) =>
        completedStepIndexes.has(stepIndex),
      );
    }
  }
}

function episodeAuthorRequestLabels(
  requests: readonly EpisodeAuthorRequestRecord[],
): readonly string[] {
  let run = 0;
  let beatBatch = 0;
  return requests.map((request) => {
    if (request.task.kind === "story") {
      run += 1;
      beatBatch = 0;
      return run === 1 ? "Story spine" : `Story spine · run ${run}`;
    }
    switch (request.task.kind) {
      case "ensemble":
        return "Ensemble map";
      case "actor_arc":
        return `Actor arc · ${request.task.playerId}`;
      case "relationship":
        return `Relationship · ${request.task.playerIds.join("/")}`;
      case "beats":
        beatBatch += 1;
        return run <= 1
          ? `Scene beats ${beatBatch}`
          : `Scene beats ${beatBatch} · run ${run}`;
    }
  });
}

function formatRequestTokens(
  requests: readonly EpisodeAuthorRequestRecord[],
): string {
  const total = requests.reduce((sum, request) => {
    if (!request.tokenUsage) return sum;
    return (
      sum +
      (tokenCount(request.tokenUsage.totalTokens) ||
        tokenCount(request.tokenUsage.promptTokens) +
          tokenCount(request.tokenUsage.completionTokens))
    );
  }, 0);
  return new Intl.NumberFormat("en-US").format(total);
}

function Metric({ label, value }: { readonly label: string; readonly value: string }) {
  return <div className="rounded border border-border bg-background/50 p-3"><div className="text-[11px] text-subtle">{label}</div><div className="mt-1 text-sm font-semibold">{value}</div></div>;
}

function formatDuration(durationMs: number): string {
  const totalSeconds = Math.round(durationMs / 1_000);
  return `${Math.floor(totalSeconds / 60)}:${String(totalSeconds % 60).padStart(2, "0")}`;
}

const primaryActionClass = "inline-flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-semibold text-accent-foreground disabled:opacity-50";
const widePrimaryActionClass = `${primaryActionClass} w-full justify-center`;
const secondaryActionClass = "inline-flex items-center gap-2 rounded-md border border-border px-4 py-2 text-sm font-medium text-muted hover:bg-surface";
