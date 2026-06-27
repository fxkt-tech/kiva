import type { RoleDefinition } from "@/core/role-definition";

export function RoleEditor({ role }: { readonly role: RoleDefinition }) {
  return (
    <div className="space-y-4">
      <EditorHeader eyebrow="Role editor" title={role.name} id={role.id} />
      <dl className="grid grid-cols-2 gap-x-4 gap-y-3 text-sm">
        <Field label="Faction" value={role.faction} />
        <Field label="Team" value={role.team} />
        <Field label="Mechanic" value={role.mechanicKey} />
        <Field label="Night order" value={role.nightOrder ?? "none"} />
      </dl>
      <PromptBlock title="System prompt" value={role.systemPrompt} />
      <PromptBlock title="Action prompt" value={role.actionPrompt ?? "none"} />
    </div>
  );
}

function EditorHeader({
  eyebrow,
  title,
  id,
}: {
  readonly eyebrow: string;
  readonly title: string;
  readonly id: string;
}) {
  return (
    <div className="border-b border-zinc-800 pb-3">
      <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
        {eyebrow}
      </p>
      <h2 className="mt-1 text-xl font-semibold text-zinc-50">{title}</h2>
      <p className="mt-1 text-xs text-zinc-500">{id}</p>
    </div>
  );
}

function Field({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string | number;
}) {
  return (
    <div>
      <dt className="text-xs text-zinc-500">{label}</dt>
      <dd className="mt-1 truncate font-medium text-zinc-200">{value}</dd>
    </div>
  );
}

function PromptBlock({
  title,
  value,
}: {
  readonly title: string;
  readonly value: string;
}) {
  return (
    <section>
      <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
        {title}
      </h3>
      <p className="mt-2 whitespace-pre-wrap rounded border border-zinc-800 bg-zinc-900/55 p-3 text-sm leading-6 text-zinc-300">
        {value}
      </p>
    </section>
  );
}
