import {
  createGameFromLineupAction,
  duplicateLineupAction,
  saveLineupAction,
  setLineupEnabledAction,
} from "@/app/library/actions";
import { Button } from "@/components/ui/button";
import type { ActorDefinition } from "@/core/actor-definition";
import type { GameScriptDefinition } from "@/core/game-script";
import type { Lineup } from "@/core/lineup";
import type { PresenterDefinition } from "@/core/presenter-definition";
import type { RuleRole } from "@/core/rule-role";
import { DirtyFormGuard } from "./dirty-form-guard";

export function LineupEditor({
  lineup,
  actors,
  ruleRoles,
  presenters,
  scripts,
}: {
  readonly lineup: Lineup;
  readonly actors: readonly ActorDefinition[];
  readonly ruleRoles: readonly RuleRole[];
  readonly presenters: readonly PresenterDefinition[];
  readonly scripts: readonly GameScriptDefinition[];
}) {
  return (
    <div className="space-y-4">
      <header className="flex items-start justify-between gap-3 border-b border-border pb-3">
        <div>
          <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
            12-seat Lineup · revision {lineup.revision}
          </p>
          <h2 className="mt-1 text-xl font-semibold">{lineup.name}</h2>
          <p className="mt-1 font-mono text-xs text-subtle">{lineup.id}</p>
        </div>
        <div className="flex gap-2">
          <form action={duplicateLineupAction.bind(null, lineup.id)}>
            <Button type="submit">复制</Button>
          </form>
          <form
            action={setLineupEnabledAction.bind(
              null,
              lineup.id,
              !lineup.enabled,
            )}
          >
            <Button type="submit">{lineup.enabled ? "停用" : "启用"}</Button>
          </form>
        </div>
      </header>

      <form action={saveLineupAction} className="space-y-4">
        <DirtyFormGuard />
        <input type="hidden" name="revision" value={lineup.revision} />
        <input type="hidden" name="id" value={lineup.id} />
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="Stable ID" name="id" value={lineup.id} disabled />
          <Field label="名称" name="name" value={lineup.name} />
        </div>
        <label className="flex items-center gap-2 text-sm text-muted">
          <input type="checkbox" name="enabled" defaultChecked={lineup.enabled} />
          可用于新对局
        </label>
        <div className="overflow-hidden rounded border border-border">
          <div className="grid grid-cols-[56px_minmax(0,0.9fr)_minmax(0,1.1fr)] bg-surface-muted px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.12em] text-subtle">
            <span>Seat</span><span>Rule Role</span><span>Actor</span>
          </div>
          {lineup.seats.map((seat) => (
            <div
              key={seat.seatNo}
              className="grid grid-cols-[56px_minmax(0,0.9fr)_minmax(0,1.1fr)] gap-2 border-t border-border px-3 py-2"
            >
              <span className="self-center font-mono text-xs text-subtle">
                {seat.seatNo}
              </span>
              <select
                name={`seat.${seat.seatNo}.ruleRoleId`}
                defaultValue={seat.ruleRoleId}
                className="min-w-0 rounded border border-interactive-border bg-background px-2 py-2 text-sm"
              >
                {ruleRoles.map((role) => (
                  <option key={role.id} value={role.id}>
                    {role.name} · {role.id}
                  </option>
                ))}
              </select>
              <select
                name={`seat.${seat.seatNo}.actorId`}
                defaultValue={seat.actorId}
                className="min-w-0 rounded border border-interactive-border bg-background px-2 py-2 text-sm"
              >
                {actors.map((actor) => (
                  <option
                    key={actor.id}
                    value={actor.id}
                    disabled={!actor.enabled && actor.id !== seat.actorId}
                  >
                    {actor.identity.name} · {actor.id}
                  </option>
                ))}
              </select>
            </div>
          ))}
        </div>
        <div className="flex justify-end border-t border-border pt-4">
          <Button type="submit">保存 Lineup</Button>
        </div>
      </form>

      <form
        action={createGameFromLineupAction.bind(null, lineup.id)}
        className="grid gap-3 rounded border border-cyan-800/60 bg-cyan-950/15 p-3 md:grid-cols-[1fr_1fr_auto] md:items-end"
      >
        <Select
          label="主持人"
          name="presenterId"
          options={presenters.filter((item) => item.enabled).map((item) => ({
            id: item.id,
            label: item.name,
          }))}
        />
        <Select
          label="主题脚本"
          name="scriptId"
          options={scripts.filter((item) => item.enabled).map((item) => ({
            id: item.id,
            label: item.name,
          }))}
        />
        <Button type="submit" disabled={!lineup.enabled}>创建对局</Button>
      </form>
    </div>
  );
}

function Field({
  label,
  name,
  value,
  disabled = false,
}: {
  readonly label: string;
  readonly name: string;
  readonly value: string;
  readonly disabled?: boolean;
}) {
  return (
    <label className="text-xs text-subtle">
      {label}
      <input
        name={name}
        defaultValue={value}
        disabled={disabled}
        required
        className="mt-1 w-full rounded border border-interactive-border bg-background px-3 py-2 text-sm"
      />
    </label>
  );
}

function Select({
  label,
  name,
  options,
}: {
  readonly label: string;
  readonly name: string;
  readonly options: readonly { readonly id: string; readonly label: string }[];
}) {
  return (
    <label className="text-xs text-subtle">
      {label}
      <select
        name={name}
        required
        className="mt-1 w-full rounded border border-interactive-border bg-background px-3 py-2 text-sm"
      >
        {options.map((option) => (
          <option key={option.id} value={option.id}>{option.label}</option>
        ))}
      </select>
    </label>
  );
}
