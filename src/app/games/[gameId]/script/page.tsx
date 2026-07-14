import { ArrowLeft, Check, Film, LockKeyhole, RefreshCw } from "lucide-react";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";
import {
  approveEpisodeScriptAction,
  generateEpisodeScriptAction,
} from "@/app/actions";
import { FormSubmitButton } from "@/components/editor/form-submit-button";
import { LlmGenerationDetails } from "@/components/editor/llm-generation-details";
import { EpisodeEnsembleReview } from "@/components/script/episode-ensemble-review";
import { EpisodeGeneratingRefresh } from "@/components/script/episode-generating-refresh";
import { iconButtonClassName } from "@/components/ui/button-styles";
import type { EpisodeAuthorRequestRecord } from "@/core/episode-script";
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

  return (
    <main className="min-h-screen bg-background px-4 py-6 text-foreground sm:px-6 lg:px-8">
      <div className="mx-auto flex max-w-5xl flex-col gap-5">
        <header className="flex items-center gap-3 border-b border-border pb-4">
          <Link
            href="/"
            aria-label="Back"
            title="Back"
            className={iconButtonClassName()}
          >
            <ArrowLeft aria-hidden="true" className="h-4 w-4" />
          </Link>
          <div>
            <p className="text-xs font-medium uppercase tracking-[0.16em] text-cyan-300">
              Scripted mode
            </p>
            <h1 className="mt-1 text-xl font-semibold">{record.game.title}</h1>
          </div>
        </header>

        <section className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
          <article className="rounded-lg border border-border bg-surface/45 p-5">
            <EpisodeWorkspace record={record} />
          </article>

          <aside className="rounded-lg border border-warning-badge/60 bg-warning-badge/25 p-5">
            <div className="flex items-center gap-2 text-sm font-semibold text-warning-badge-foreground">
              <LockKeyhole aria-hidden="true" className="h-4 w-4" />
              开局已锁定
            </div>
            <p className="mt-3 text-xs leading-5 text-muted">
              {record.episodeScript?.status === "approved"
                ? "剧本已经批准，结构已锁定。"
                : "剧本批准前不会推进游戏、生成玩家配音或允许导出。"}
            </p>
            <dl className="mt-4 space-y-3 text-xs">
              <div>
                <dt className="text-subtle">主题模板</dt>
                <dd className="mt-1 text-foreground">{record.game.script.name}</dd>
              </div>
              <div>
                <dt className="text-subtle">玩家数量</dt>
                <dd className="mt-1 text-foreground">{record.game.players.length}</dd>
              </div>
              <div>
                <dt className="text-subtle">目标时长</dt>
                <dd className="mt-1 text-foreground">28–32 分钟</dd>
              </div>
            </dl>
          </aside>
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
  const state = record.episodeScript;
  if (!state) {
    throw new Error("Scripted game is missing its episode script state");
  }
  if (state.status === "review") {
    const script = state.candidate;
    const speechSteps = script.steps.filter((step) => step.speechBeat);
    return (
      <div>
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="text-xs font-medium uppercase tracking-[0.16em] text-cyan-300">Director review</p>
            <h2 className="mt-1 text-xl font-semibold">{script.title}</h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-muted">{script.logline}</p>
          </div>
          <span className={state.report.valid ? "rounded-full bg-good-badge px-3 py-1 text-xs text-good-badge-foreground" : "rounded-full bg-danger-badge px-3 py-1 text-xs text-danger-badge-foreground"}>
            {state.report.valid ? "校验通过" : "校验失败"}
          </span>
        </div>

        <div className="mt-5 grid gap-3 sm:grid-cols-4">
          <Metric label="计划胜方" value={script.plannedWinner === "good" ? "好人" : "狼人"} />
          <Metric label="计划天数" value={`${script.plannedDayCount} 天`} />
          <Metric label="预计时长" value={formatDuration(script.targetDurationMs)} />
          <Metric label="结构规模" value={`${script.steps.length} 步 / ${speechSteps.length} 段发言`} />
        </div>

        <EpisodeAuthorRequests requests={state.requests ?? []} />

        <section className="mt-5">
          <h3 className="text-sm font-semibold">剧情幕</h3>
          <div className="mt-2 grid gap-2 sm:grid-cols-3">
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
          <form action={generateEpisodeScriptAction.bind(null, record.game.id)}>
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
      <div>
        <h2 className="text-base font-semibold text-danger-badge-foreground">剧本生成失败</h2>
        <p className="mt-2 rounded border border-danger-badge/60 bg-danger-badge/25 px-3 py-2 text-sm text-danger-badge-foreground">{state.error}</p>
        <EpisodeAuthorRequests requests={state.requests ?? []} />
        <form className="mt-4" action={generateEpisodeScriptAction.bind(null, record.game.id)}>
          <FormSubmitButton label="重试生成" pendingLabel="正在重试…" className={primaryActionClass} />
        </form>
      </div>
    );
  }

  if (state.status === "generating") {
    return (
      <div className="py-10 text-center">
        <EpisodeGeneratingRefresh />
        <div className="mx-auto h-8 w-8 animate-spin rounded-full border-2 border-border border-t-cyan-300" />
        <h2 className="mt-4 text-base font-semibold">正在生成并模拟整局剧本</h2>
        <p className="mt-2 text-sm text-muted">规则 dry-run 与主题编写完成后会进入导演审核。</p>
        <form className="mt-5" action={generateEpisodeScriptAction.bind(null, record.game.id)}>
          <FormSubmitButton
            label="任务长时间无响应？重新开始"
            pendingLabel="正在重新开始…"
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
      <div className="flex items-start gap-3">
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
      <form className="mt-5 flex justify-end" action={generateEpisodeScriptAction.bind(null, record.game.id)}>
        <FormSubmitButton label="开始生成剧本" pendingLabel="正在生成并模拟…" className={primaryActionClass} />
      </form>
    </div>
  );
}

export function EpisodeAuthorRequests({
  requests,
}: {
  readonly requests: readonly EpisodeAuthorRequestRecord[];
}) {
  if (requests.length === 0) return null;
  const labels = episodeAuthorRequestLabels(requests);
  return (
    <section className="mt-5">
      <div className="flex items-baseline justify-between gap-3">
        <h3 className="text-sm font-semibold">LLM requests</h3>
        <span className="text-xs text-subtle">
          {requests.length} requests · {formatRequestTokens(requests)} tokens
        </span>
      </div>
      <div className="mt-2 divide-y divide-border overflow-hidden rounded border border-border bg-background/40">
        {requests.map((request, index) => (
          <div
            key={request.id}
            className="flex items-center justify-between gap-3 px-3 py-2 text-xs"
          >
            <div className="min-w-0">
              <div className="font-medium text-foreground">
                {labels[index]}
              </div>
              <div className="mt-0.5 truncate text-subtle">
                {request.provider}/{request.model} · {request.status} ·{" "}
                {request.stepIndexes.length > 0
                  ? `steps ${request.stepIndexes.join(", ")}`
                  : "full cast"}
              </div>
            </div>
            <div className="flex shrink-0 items-center gap-2">
              <span className="text-muted">{formatRequestTokens([request])}</span>
              <LlmGenerationDetails
                generation={request}
                label={`${labels[index]} LLM details`}
                showDecisionSummary={false}
              />
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function episodeAuthorRequestLabels(
  requests: readonly EpisodeAuthorRequestRecord[],
): readonly string[] {
  let run = 0;
  let beatBatch = 0;
  return requests.map((request) => {
    if (request.kind === "outline") {
      run += 1;
      beatBatch = 0;
      return run === 1 ? "Outline" : `Outline · run ${run}`;
    }
    beatBatch += 1;
    return run <= 1
      ? `Beat batch ${beatBatch}`
      : `Beat batch ${beatBatch} · run ${run}`;
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
const secondaryActionClass = "inline-flex items-center gap-2 rounded-md border border-border px-4 py-2 text-sm font-medium text-muted hover:bg-surface";
