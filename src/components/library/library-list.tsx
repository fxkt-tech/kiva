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
    <aside className="min-h-0 overflow-y-auto border-r border-zinc-800 bg-zinc-950/45">
      <div className="sticky top-0 z-10 border-b border-zinc-800 bg-zinc-950/95 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
        Objects
      </div>
      {items.length === 0 ? (
        <div className="px-3 py-6 text-sm text-zinc-500">No objects.</div>
      ) : (
        <div className="divide-y divide-zinc-800/80">
          {items.map((item) => {
            const selected = item.id === selectedId;

            return (
              <Link
                key={item.id}
                href={`/library?tab=${tab}&id=${item.id}`}
                aria-current={selected ? "page" : undefined}
                className={[
                  "block border-l-2 px-3 py-3 transition",
                  selected
                    ? "border-cyan-300 bg-zinc-800/80"
                    : "border-transparent hover:bg-zinc-900",
                ].join(" ")}
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="min-w-0 truncate text-sm font-medium text-zinc-100">
                    {item.name}
                  </div>
                  <span
                    className={[
                      "h-2 w-2 shrink-0 rounded-full",
                      item.valid === false
                        ? "bg-amber-400"
                        : item.enabled
                          ? "bg-emerald-400"
                          : "bg-zinc-600",
                    ].join(" ")}
                    aria-label={statusLabel(item)}
                  />
                </div>
                <div className="mt-1 truncate text-xs text-zinc-500">
                  {item.id}
                </div>
                <div className="mt-1 truncate text-xs text-zinc-400">
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
