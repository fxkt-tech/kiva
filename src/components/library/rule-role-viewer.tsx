import type { RuleRole } from "@/core/rule-role";

export function RuleRoleViewer({ role }: { readonly role: RuleRole }) {
  return (
    <article className="space-y-5">
      <header className="border-b border-border pb-3">
        <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
          Read-only engine contract
        </p>
        <h2 className="mt-1 text-xl font-semibold">{role.name}</h2>
        <p className="mt-1 font-mono text-xs text-subtle">{role.id}</p>
      </header>
      <div className="grid gap-3 sm:grid-cols-2">
        <Value label="阵营" value={role.faction} />
        <Value label="屠边分组" value={role.team} />
        <Value label="机制键" value={role.mechanicKey} />
        <Value label="夜间顺序" value={role.nightOrder?.toString() ?? "none"} />
      </div>
      <section className="rounded border border-border bg-surface/35 p-4">
        <h3 className="text-sm font-semibold">初始私有知识</h3>
        <ul className="mt-3 space-y-2 text-sm text-muted">
          {role.initialPrivateKnowledge.map((item) => (
            <li key={item} className="rounded bg-background/65 px-3 py-2 font-mono text-xs">
              {item}
            </li>
          ))}
        </ul>
      </section>
      <p className="rounded border border-cyan-900/70 bg-cyan-950/20 p-3 text-sm leading-6 text-muted">
        Rule Role 由规则引擎拥有：它只定义胜负、可见性、合法行动和夜间顺序，不承载人设、自由 Prompt、模型或声音配置。
      </p>
    </article>
  );
}

function Value({ label, value }: { readonly label: string; readonly value: string }) {
  return (
    <div className="rounded border border-border bg-surface/35 p-3">
      <div className="text-xs text-subtle">{label}</div>
      <div className="mt-1 font-mono text-sm text-foreground">{value}</div>
    </div>
  );
}
