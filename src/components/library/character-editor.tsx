import { saveCharacterAction } from "@/app/library/actions";
import type { CharacterDefinition } from "@/core/character-definition";
import type { ModelBindingSnapshot } from "@/core/player";
import { DirtyFormGuard } from "./dirty-form-guard";

export function CharacterEditor({
  character,
}: {
  readonly character: CharacterDefinition;
}) {
  return (
    <form action={saveCharacterAction} className="space-y-5">
      <DirtyFormGuard />
      <input type="hidden" name="createdAt" value={character.createdAt} />
      <input
        type="hidden"
        name="defaultModelBinding"
        value={modelBindingValue(character.defaultModelBinding)}
      />

      <div className="flex items-start justify-between gap-3 border-b border-zinc-800 pb-3">
        <div className="min-w-0">
          <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
            Character editor
          </p>
          <h2 className="mt-1 truncate text-xl font-semibold text-zinc-50">
            {character.name}
          </h2>
          <p className="mt-1 truncate text-xs text-zinc-500">{character.id}</p>
        </div>
        <span
          className={[
            "rounded border px-2 py-1 text-xs font-medium",
            character.enabled
              ? "border-emerald-500/40 text-emerald-300"
              : "border-zinc-700 text-zinc-500",
          ].join(" ")}
        >
          {character.enabled ? "enabled" : "disabled"}
        </span>
      </div>

      <div className="grid gap-3 md:grid-cols-2">
        <TextField label="Id" name="id" defaultValue={character.id} />
        <TextField label="Name" name="name" defaultValue={character.name} />
      </div>

      <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <TextField
          label="Avatar"
          name="avatar"
          defaultValue={character.avatar ?? ""}
        />
        <TextField
          label="Tags"
          name="tags"
          defaultValue={character.tags.join(",")}
        />
      </div>

      <label className="flex items-center gap-2 text-sm text-zinc-300">
        <input
          type="checkbox"
          name="enabled"
          defaultChecked={character.enabled}
          className="h-4 w-4 accent-cyan-300"
        />
        Enabled
      </label>

      <TextareaField
        label="Persona"
        name="persona"
        defaultValue={character.persona}
        rows={3}
      />
      <TextareaField
        label="Speaking style"
        name="speakingStyle"
        defaultValue={character.speakingStyle}
        rows={3}
      />
      <TextareaField
        label="Reasoning style"
        name="reasoningStyle"
        defaultValue={character.reasoningStyle}
        rows={3}
      />
      <TextareaField
        label="System prompt"
        name="systemPrompt"
        defaultValue={character.systemPrompt}
        rows={7}
      />

      <div className="flex justify-end border-t border-zinc-800 pt-4">
        <button
          type="submit"
          className="rounded bg-zinc-100 px-4 py-2 text-sm font-semibold text-zinc-950 transition hover:bg-white"
        >
          Save character
        </button>
      </div>
    </form>
  );
}

function TextField({
  label,
  name,
  defaultValue,
}: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      <span className="text-xs font-medium uppercase tracking-[0.12em] text-zinc-500">
        {label}
      </span>
      <input
        name={name}
        defaultValue={defaultValue}
        className="mt-1 w-full rounded border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition focus:border-cyan-300"
      />
    </label>
  );
}

function TextareaField({
  label,
  name,
  defaultValue,
  rows,
}: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
  readonly rows: number;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      <span className="text-xs font-medium uppercase tracking-[0.12em] text-zinc-500">
        {label}
      </span>
      <textarea
        name={name}
        defaultValue={defaultValue}
        rows={rows}
        className="mt-1 w-full resize-y rounded border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm leading-6 text-zinc-100 outline-none transition focus:border-cyan-300"
      />
    </label>
  );
}

function modelBindingValue(value: ModelBindingSnapshot | null): string {
  return value === null ? "" : JSON.stringify(value);
}
