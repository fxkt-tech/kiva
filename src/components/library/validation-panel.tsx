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
      className="min-h-0 overflow-y-auto border-l border-border bg-background/70 p-4"
    >
      <div className="border-b border-border pb-3">
        <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
          Validation
        </p>
        <h2 className="mt-1 text-sm font-semibold text-foreground">
          {selectedId ?? "No selection"}
        </h2>
      </div>
      {diagnostic === undefined ? (
        <p className="mt-4 text-sm text-subtle">No diagnostics available.</p>
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
            <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-subtle">
              References
            </h3>
            {diagnostic.references.length === 0 ? (
              <p className="mt-2 text-sm text-subtle">No references.</p>
            ) : (
              <ul className="mt-2 space-y-2 text-sm text-muted">
                {diagnostic.references.map((reference) => (
                  <li
                    key={reference}
                    className="rounded border border-border bg-surface/55 px-2 py-1"
                  >
                    {reference}
                  </li>
                ))}
              </ul>
            )}
          </section>
          <section>
            <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-subtle">
              Messages
            </h3>
            {diagnostic.messages.length === 0 ? (
              <p className="mt-2 text-sm text-subtle">No messages.</p>
            ) : (
              <ul className="mt-2 space-y-2 text-sm text-muted">
                {diagnostic.messages.map((message) => (
                  <li key={message} className="rounded bg-surface-muted p-2">
                    {message}
                  </li>
                ))}
              </ul>
            )}
          </section>
          <section>
            <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-subtle">
              Prompt preview
            </h3>
            <pre className="mt-2 max-h-80 overflow-auto whitespace-pre-wrap rounded border border-border bg-surface/55 p-3 text-xs leading-5 text-muted">
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
    <div className="rounded border border-border bg-surface/55 px-3 py-2">
      <div className="text-xs text-subtle">{label}</div>
      <div
        className={[
          "mt-1 font-mono text-sm",
          tone === "good"
            ? "text-good-badge-foreground"
            : tone === "bad"
              ? "text-danger-badge-foreground"
              : "text-foreground",
        ].join(" ")}
      >
        {value}
      </div>
    </div>
  );
}
