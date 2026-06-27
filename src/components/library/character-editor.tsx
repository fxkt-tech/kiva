import type { CharacterDefinition } from "@/core/character-definition";

export function CharacterEditor({
  character,
}: {
  readonly character: CharacterDefinition;
}) {
  return (
    <div className="space-y-4">
      <div className="border-b border-zinc-800 pb-3">
        <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
          Character editor
        </p>
        <h2 className="mt-1 text-xl font-semibold text-zinc-50">
          {character.name}
        </h2>
        <p className="mt-1 text-xs text-zinc-500">{character.id}</p>
      </div>
      <dl className="grid grid-cols-2 gap-x-4 gap-y-3 text-sm">
        <Field label="Persona" value={character.persona} />
        <Field label="Speaking" value={character.speakingStyle} />
        <Field label="Reasoning" value={character.reasoningStyle} />
        <Field
          label="Tags"
          value={character.tags.length > 0 ? character.tags.join(", ") : "none"}
        />
      </dl>
      <section>
        <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
          System prompt
        </h3>
        <p className="mt-2 whitespace-pre-wrap rounded border border-zinc-800 bg-zinc-900/55 p-3 text-sm leading-6 text-zinc-300">
          {character.systemPrompt}
        </p>
      </section>
    </div>
  );
}

function Field({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}) {
  return (
    <div className="min-w-0">
      <dt className="text-xs text-zinc-500">{label}</dt>
      <dd className="mt-1 truncate font-medium text-zinc-200">{value}</dd>
    </div>
  );
}
