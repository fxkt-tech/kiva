import {
  duplicateScriptAction,
  saveScriptAction,
} from "@/app/library/actions";
import { Button } from "@/components/ui/button";
import type { GameScriptDefinition } from "@/core/game-script";
import { DirtyFormGuard } from "./dirty-form-guard";

export function ScriptEditor({
  script,
}: {
  readonly script: GameScriptDefinition;
}) {
  return (
    <form action={saveScriptAction} className="space-y-4">
      <DirtyFormGuard />
      <input type="hidden" name="createdAt" value={script.createdAt} />
      <input type="hidden" name="id" value={script.id} />
      <header className="flex items-start justify-between gap-3 border-b border-border pb-3">
        <div>
          <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
            Theme Script
          </p>
          <h2 className="mt-1 text-xl font-semibold">{script.name}</h2>
          <p className="mt-1 font-mono text-xs text-subtle">{script.id}</p>
        </div>
        <Button
          type="submit"
          formAction={duplicateScriptAction.bind(null, script.id)}
        >
          复制
        </Button>
      </header>
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="Stable ID" name="id" value={script.id} disabled />
        <Field label="名称" name="name" value={script.name} />
      </div>
      <Field label="主题" name="theme" value={script.theme} />
      <Area label="共同背景" name="background" value={script.background} />
      <Field
        label="氛围（逗号分隔）"
        name="atmosphere"
        value={script.atmosphere.join("，")}
      />
      <section className="space-y-3 rounded border border-border bg-surface/30 p-3">
        <h3 className="text-sm font-semibold">视觉包装</h3>
        <Field label="封面" name="presentation.coverImage" value={script.presentation.coverImage} />
        <Field label="白天背景" name="presentation.dayBackground" value={script.presentation.dayBackground} />
        <Field label="夜晚背景" name="presentation.nightBackground" value={script.presentation.nightBackground} />
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
          {(Object.keys(script.presentation.colors) as Array<keyof typeof script.presentation.colors>).map((key) => (
            <label key={key} className="text-xs text-subtle">
              {key}
              <input
                type="color"
                name={`presentation.colors.${key}`}
                defaultValue={script.presentation.colors[key]}
                required
                className="mt-1 h-9 w-full rounded border border-border bg-transparent"
              />
            </label>
          ))}
        </div>
      </section>
      <label className="flex items-center gap-2 text-sm text-muted">
        <input type="checkbox" name="enabled" defaultChecked={script.enabled} />
        可用于新对局
      </label>
      <div className="flex justify-end border-t border-border pt-4">
        <Button type="submit">保存主题脚本</Button>
      </div>
    </form>
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
    <label className="block text-xs text-subtle">
      {label}
      <input name={name} defaultValue={value} disabled={disabled} required className="mt-1 w-full rounded border border-interactive-border bg-background px-3 py-2 text-sm disabled:opacity-60" />
    </label>
  );
}

function Area({ label, name, value }: { readonly label: string; readonly name: string; readonly value: string }) {
  return (
    <label className="block text-xs text-subtle">
      {label}
      <textarea name={name} defaultValue={value} rows={5} required className="mt-1 w-full resize-y rounded border border-interactive-border bg-background px-3 py-2 text-sm leading-6" />
    </label>
  );
}
