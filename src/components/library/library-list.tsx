import Link from "next/link";

export type LibraryListItem = {
  readonly id: string;
  readonly name: string;
  readonly enabled: boolean;
  readonly meta: string;
  readonly valid?: boolean;
};

type LibraryListProps = {
  readonly tab: "roles" | "characters" | "presets";
  readonly selectedId: string | null;
  readonly items: readonly LibraryListItem[];
};

export function LibraryList({ tab, selectedId, items }: LibraryListProps) {
  return (
    <aside className="min-h-0 overflow-y-auto border-r border-border bg-background/45">
      <div className="sticky top-0 z-10 border-b border-border bg-background/95 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-subtle">
        Objects
      </div>
      {items.length === 0 ? (
        <div className="px-3 py-6 text-sm text-subtle">No objects.</div>
      ) : (
        <div className="divide-y divide-border/80">
          {items.map((item) => {
            const selected = item.id === selectedId;

            return (
              <Link
                key={item.id}
                href={libraryItemHref(tab, item.id)}
                aria-current={selected ? "page" : undefined}
                className={[
                  "block border-l-2 px-3 py-3 transition",
                  selected
                    ? "border-accent bg-surface-strong/80"
                    : "border-transparent hover:bg-surface-muted",
                ].join(" ")}
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="min-w-0 truncate text-sm font-medium text-foreground">
                    {item.name}
                  </div>
                  <span
                    className={[
                      "h-2 w-2 shrink-0 rounded-full",
                      item.valid === false
                        ? "bg-amber-400"
                        : item.enabled
                          ? "bg-emerald-400"
                          : "bg-subtle",
                    ].join(" ")}
                    aria-label={statusLabel(item)}
                  />
                </div>
                <div className="mt-1 truncate text-xs text-subtle">
                  {item.id}
                </div>
                <div className="mt-1 truncate text-xs text-muted">
                  {item.meta}
                </div>
              </Link>
            );
          })}
        </div>
      )}
    </aside>
  );
}

function statusLabel(item: LibraryListItem): string {
  if (item.valid === false) {
    return "invalid";
  }

  return item.enabled ? "enabled" : "disabled";
}

function libraryItemHref(tab: LibraryListProps["tab"], id: string): string {
  const params = new URLSearchParams({ tab, id });
  return `/library?${params.toString()}`;
}
