import type { LibraryActionsRecord } from "@/server/library-actions";
import type { LibraryTab } from "./library-workspace";

type ValidationPanelProps = {
  readonly library: LibraryActionsRecord;
  readonly activeTab: LibraryTab;
  readonly selectedId: string | null;
};

export function ValidationPanel({
  library,
  activeTab,
  selectedId,
}: ValidationPanelProps) {
  const diagnostic =
    selectedId === null
      ? undefined
      : library.diagnostics[activeTab][selectedId];

  return (
    <aside
      aria-label="Validation region"
      className="min-h-0 overflow-y-auto border-l border-zinc-800 bg-zinc-950/70 p-4"
    >
      <div className="border-b border-zinc-800 pb-3">
        <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
          Validation
        </p>
        <h2 className="mt-1 text-sm font-semibold text-zinc-100">
          {selectedId ?? "No selection"}
        </h2>
      </div>
      {diagnostic === undefined ? (
        <p className="mt-4 text-sm text-zinc-500">No diagnostics available.</p>
      ) : (
        <div className="mt-4 space-y-4">
          <div className="grid grid-cols-2 gap-2 text-sm">
            <Status
              label="Validation"
              value={diagnostic.valid ? "valid" : "invalid"}
              tone={diagnostic.valid ? "good" : "bad"}
            />
            <Status
              label="Refs"
              value={String(diagnostic.references.length)}
            />
            {typeof diagnostic.canCreateGame === "boolean" ? (
              <Status
                label="Can create game"
                value={diagnostic.canCreateGame ? "yes" : "no"}
                tone={diagnostic.canCreateGame ? "good" : "bad"}
              />
            ) : null}
          </div>
          <section>
            <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
              References
            </h3>
            {diagnostic.references.length === 0 ? (
              <p className="mt-2 text-sm text-zinc-500">No references.</p>
            ) : (
              <ul className="mt-2 space-y-2 text-sm text-zinc-300">
                {diagnostic.references.map((reference) => (
                  <li
                    key={reference}
                    className="rounded border border-zinc-800 bg-zinc-900/55 px-2 py-1"
                  >
                    {reference}
                  </li>
                ))}
              </ul>
            )}
          </section>
          <section>
            <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
              Messages
            </h3>
            {diagnostic.messages.length === 0 ? (
              <p className="mt-2 text-sm text-zinc-500">No messages.</p>
            ) : (
              <ul className="mt-2 space-y-2 text-sm text-zinc-300">
                {diagnostic.messages.map((message) => (
                  <li key={message} className="rounded bg-zinc-900 p-2">
                    {message}
                  </li>
                ))}
              </ul>
            )}
          </section>
          <section>
            <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
              Prompt preview
            </h3>
            <pre className="mt-2 max-h-80 overflow-auto whitespace-pre-wrap rounded border border-zinc-800 bg-zinc-900/55 p-3 text-xs leading-5 text-zinc-300">
              {diagnostic.promptPreview || "No preview."}
            </pre>
          </section>
        </div>
      )}
    </aside>
  );
}

function Status({
  label,
  value,
  tone = "neutral",
}: {
  readonly label: string;
  readonly value: string;
  readonly tone?: "neutral" | "good" | "bad";
}) {
  return (
    <div className="rounded border border-zinc-800 bg-zinc-900/55 px-3 py-2">
      <div className="text-xs text-zinc-500">{label}</div>
      <div
        className={[
          "mt-1 font-mono text-sm",
          tone === "good"
            ? "text-emerald-300"
            : tone === "bad"
              ? "text-red-300"
              : "text-zinc-100",
        ].join(" ")}
      >
        {value}
      </div>
    </div>
  );
}
