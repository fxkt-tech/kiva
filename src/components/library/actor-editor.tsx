import {
  createActorAction,
  duplicateActorAction,
  saveActorAction,
  setActorEnabledAction,
} from "@/app/library/actions";
import { Button } from "@/components/ui/button";
import type { ActorDefinition } from "@/core/actor-definition";
import { DirtyFormGuard } from "./dirty-form-guard";

export function ActorEditor({
  actor,
  mode = "edit",
}: {
  readonly actor: ActorDefinition;
  readonly mode?: "create" | "edit";
}) {
  const isNew = mode === "create";
  const isQinChuan = actor.id === "qin_chuan";
  return (
    <div className="space-y-4">
      <header className="flex items-start justify-between gap-3 border-b border-border pb-3">
        <div className="flex min-w-0 items-center gap-3">
          {actor.identity.portrait ? (
            <img
              src={actor.identity.portrait}
              alt=""
              className="h-16 w-16 rounded-full border border-border object-cover"
            />
          ) : (
            <div className="grid h-16 w-16 place-items-center rounded-full border border-dashed border-border text-xs text-subtle">
              New
            </div>
          )}
          <div className="min-w-0">
            <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
              Actor v2 · {isNew ? "new" : `revision ${actor.revision}`}
            </p>
            <h2 className="mt-1 truncate text-xl font-semibold">
              {isNew ? "创建 Actor" : actor.identity.name}
            </h2>
            {!isNew ? (
              <p className="mt-1 font-mono text-xs text-subtle">{actor.id}</p>
            ) : null}
          </div>
        </div>
        {!isNew ? (
          <div className="flex gap-2">
            <form action={duplicateActorAction.bind(null, actor.id)}>
              <Button type="submit">复制</Button>
            </form>
            <form
              action={setActorEnabledAction.bind(null, actor.id, !actor.enabled)}
            >
              <Button type="submit">
                {actor.enabled ? "停用" : "启用"}
              </Button>
            </form>
          </div>
        ) : null}
      </header>

      <form action={isNew ? createActorAction : saveActorAction} className="space-y-5">
        <DirtyFormGuard />
        <input type="hidden" name="revision" value={actor.revision} />
        {!isNew ? <input type="hidden" name="id" value={actor.id} /> : null}
        {isQinChuan ? (
          <input
            type="hidden"
            name="identity.name"
            value={actor.identity.name}
          />
        ) : null}

        <Section title="身份锚点" note="只定义这个人是谁，不绑定任何规则身份。">
          <div className="grid gap-3 md:grid-cols-2">
            <Field
              label="Stable ID"
              name="id"
              value={actor.id}
              disabled={!isNew}
            />
            <Field
              label="姓名"
              name="identity.name"
              value={actor.identity.name}
              disabled={isQinChuan}
            />
          </div>
          <Field
            label="头像"
            name="identity.portrait"
            value={actor.identity.portrait}
          />
          <Field
            label="标签（逗号分隔）"
            name="identity.tags"
            value={actor.identity.tags.join("，")}
          />
          <Area
            label="视觉锚点"
            name="identity.visualAnchor"
            value={actor.identity.visualAnchor}
          />
        </Section>

        <Section title="稳定人格" note="跨对局保持，身份变化不能覆盖。">
          <Area label="稳定内核" name="core.stableCore" value={actor.core.stableCore} />
          <Area label="驱动力" name="core.drive" value={actor.core.drive} />
          <Area label="盲点" name="core.blindSpot" value={actor.core.blindSpot} />
          <Area
            label="变化边界"
            name="core.changeBoundary"
            value={actor.core.changeBoundary}
          />
        </Section>

        <Section title="认知策略" note="决定先看什么、何时改口，而不是替他写结论。">
          <Area label="注意焦点" name="cognition.attention" value={actor.cognition.attention} />
          <Area label="证据纪律" name="cognition.evidencePolicy" value={actor.cognition.evidencePolicy} />
          <Area label="决策方式" name="cognition.decisionPolicy" value={actor.cognition.decisionPolicy} />
          <Area label="纠错触发" name="cognition.correctionTrigger" value={actor.cognition.correctionTrigger} />
        </Section>

        <Section title="桌上互动" note="描述与任意同桌人的互动方式，不写固定他人姓名。">
          <Area label="桌上功能" name="interaction.tableFunction" value={actor.interaction.tableFunction} />
          <Area label="社交策略" name="interaction.socialStrategy" value={actor.interaction.socialStrategy} />
          <Area label="受压反应" name="interaction.pressureResponse" value={actor.interaction.pressureResponse} />
          <Field
            label="冲突轴（逗号分隔）"
            name="interaction.conflictAxes"
            value={actor.interaction.conflictAxes.join("，")}
          />
        </Section>

        <Section title="表达控制" note="表演阶段使用；不改变已决定的事实和行动。">
          <Area label="节奏" name="expression.cadence" value={actor.expression.cadence} />
          <Area label="用词" name="expression.diction" value={actor.expression.diction} />
          <Field
            label="修辞动作（逗号分隔）"
            name="expression.rhetoricalMoves"
            value={actor.expression.rhetoricalMoves.join("，")}
          />
          <Field
            label="避免项（逗号分隔）"
            name="expression.avoid"
            value={actor.expression.avoid.join("，")}
          />
        </Section>

        <Section title="生产配置" note="模型和声音随 Actor 快照进入对局。">
          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Provider" name="production.modelBinding.provider" value={actor.production.modelBinding.provider} />
            <Field label="Model" name="production.modelBinding.model" value={actor.production.modelBinding.model} />
            <Field label="Fallback model" name="production.modelBinding.fallbackModel" value={actor.production.modelBinding.fallbackModel ?? ""} required={false} />
            <Field label="Voice" name="production.voice.voice" value={actor.production.voice.voice} />
            <Field label="Rate" name="production.voice.rate" value={actor.production.voice.rate} />
            <Field label="Pitch" name="production.voice.pitch" value={actor.production.voice.pitch} />
            <Field label="Volume" name="production.voice.volume" value={actor.production.voice.volume} />
          </div>
        </Section>

        <label className="flex items-center gap-2 text-sm text-muted">
          <input type="checkbox" name="enabled" defaultChecked={actor.enabled} />
          可用于新对局
        </label>
        <div className="sticky bottom-0 flex justify-end border-t border-border bg-background/95 py-4 backdrop-blur">
          <Button type="submit">{isNew ? "创建 Actor" : "保存 Actor"}</Button>
        </div>
      </form>
    </div>
  );
}

function Section({
  title,
  note,
  children,
}: {
  readonly title: string;
  readonly note: string;
  readonly children: React.ReactNode;
}) {
  return (
    <section className="space-y-3 rounded border border-border bg-surface/30 p-3">
      <div>
        <h3 className="text-sm font-semibold">{title}</h3>
        <p className="mt-1 text-xs text-subtle">{note}</p>
      </div>
      {children}
    </section>
  );
}

function Field({
  label,
  name,
  value,
  disabled = false,
  required = true,
}: {
  readonly label: string;
  readonly name: string;
  readonly value: string;
  readonly disabled?: boolean;
  readonly required?: boolean;
}) {
  return (
    <label className="block text-xs text-subtle">
      {label}
      <input
        name={name}
        defaultValue={value}
        disabled={disabled}
        required={required}
        className="mt-1 w-full rounded border border-interactive-border bg-background px-3 py-2 text-sm text-foreground outline-none disabled:opacity-60"
      />
    </label>
  );
}

function Area({
  label,
  name,
  value,
}: {
  readonly label: string;
  readonly name: string;
  readonly value: string;
}) {
  return (
    <label className="block text-xs text-subtle">
      {label}
      <textarea
        name={name}
        defaultValue={value}
        rows={2}
        required
        className="mt-1 w-full resize-y rounded border border-interactive-border bg-background px-3 py-2 text-sm leading-6 text-foreground outline-none"
      />
    </label>
  );
}
