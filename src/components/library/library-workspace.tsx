import Link from "next/link";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";
import type { LibraryActionsRecord } from "@/server/library-actions";
import { CharacterEditor } from "./character-editor";
import { LibraryList, type LibraryListItem } from "./library-list";
import { PresetEditor } from "./preset-editor";
import { RoleEditor } from "./role-editor";
import { ValidationPanel } from "./validation-panel";

export type LibraryTab = "roles" | "characters" | "presets";

type LibraryWorkspaceProps = {
  readonly activeTab: LibraryTab;
  readonly selectedId: string | null;
  readonly library: LibraryActionsRecord;
};

type SelectedItem =
  | { readonly kind: "role"; readonly id: string; readonly item: RoleDefinition }
  | {
      readonly kind: "character";
      readonly id: string;
      readonly item: CharacterDefinition;
    }
  | {
      readonly kind: "preset";
      readonly id: string;
      readonly item: GamePreset;
    };

const tabs: readonly LibraryTab[] = ["roles", "characters", "presets"];

export function LibraryWorkspace({
  activeTab,
  selectedId,
  library,
}: LibraryWorkspaceProps) {
  const selected = selectItem(library, activeTab, selectedId);
  const effectiveSelectedId = selected?.id ?? null;

  return (
    <main className="h-screen overflow-hidden bg-zinc-950 text-zinc-100">
      <div className="flex h-full min-h-0 flex-col">
        <header className="flex h-14 shrink-0 items-center justify-between border-b border-zinc-800 bg-zinc-950 px-4">
          <div className="flex min-w-0 items-center gap-3">
            <Link
              href="/"
              className="rounded border border-zinc-700 px-2.5 py-1.5 text-xs font-medium text-zinc-300 transition hover:border-zinc-500 hover:text-white"
            >
              Back
            </Link>
            <div className="min-w-0">
              <h1 className="truncate text-base font-semibold text-zinc-50">
                Kiva Library
              </h1>
              <p className="text-xs text-zinc-500">Director configuration desk</p>
            </div>
          </div>
          <Link
            href="/library?tab=presets"
            className="rounded bg-cyan-200 px-3 py-1.5 text-xs font-semibold text-zinc-950 transition hover:bg-cyan-100"
          >
            {activeTab === "presets" ? "Presets entry" : "Create game"}
          </Link>
        </header>

        <div className="grid min-h-0 flex-1 grid-cols-[128px_280px_minmax(0,1fr)_320px] overflow-hidden">
          <nav className="min-h-0 border-r border-zinc-800 bg-zinc-950 px-2 py-3">
            <div className="mb-2 px-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-zinc-600">
              Library
            </div>
            <div className="space-y-1">
              {tabs.map((tab) => (
                <Link
                  key={tab}
                  href={`/library?tab=${tab}`}
                  aria-current={tab === activeTab ? "page" : undefined}
                  className={[
                    "block rounded px-2.5 py-2 text-sm transition",
                    tab === activeTab
                      ? "bg-zinc-800 text-zinc-50"
                      : "text-zinc-400 hover:bg-zinc-900 hover:text-zinc-100",
                  ].join(" ")}
                >
                  <span className="block font-medium">{labelForTab(tab)}</span>
                  <span className="mt-0.5 block text-[11px] text-zinc-500">
                    {countForTab(library, tab)}
                  </span>
                </Link>
              ))}
            </div>
          </nav>

          <LibraryList
            tab={activeTab}
            selectedId={effectiveSelectedId}
            items={listItemsForTab(library, activeTab)}
          />

          <section
            aria-label="Detail region"
            className="min-h-0 overflow-y-auto bg-zinc-950 p-4"
          >
            {selected === null ? (
              <EmptyDetail />
            ) : selected.kind === "role" ? (
              <RoleEditor role={selected.item} />
            ) : selected.kind === "character" ? (
              <CharacterEditor character={selected.item} />
            ) : (
              <PresetEditor
                preset={selected.item}
                roles={library.roles}
                characters={library.characters}
              />
            )}
          </section>

          <ValidationPanel
            library={library}
            activeTab={activeTab}
            selectedId={effectiveSelectedId}
          />
        </div>
      </div>
    </main>
  );
}

function EmptyDetail() {
  return (
    <div className="flex h-full items-center justify-center text-sm text-zinc-500">
      No library object selected.
    </div>
  );
}

function labelForTab(tab: LibraryTab): string {
  if (tab === "characters") {
    return "Characters";
  }

  if (tab === "presets") {
    return "Presets";
  }

  return "Roles";
}

function countForTab(library: LibraryActionsRecord, tab: LibraryTab): string {
  return String(itemsForTab(library, tab).length).padStart(2, "0");
}

function listItemsForTab(
  library: LibraryActionsRecord,
  tab: LibraryTab,
): readonly LibraryListItem[] {
  if (tab === "characters") {
    return library.characters.map((character) => ({
      id: character.id,
      name: character.name,
      enabled: character.enabled,
      valid: library.diagnostics.characters[character.id]?.valid,
      meta: character.tags.length > 0 ? character.tags.join(", ") : "no tags",
    }));
  }

  if (tab === "presets") {
    return library.presets.map((preset) => ({
      id: preset.id,
      name: preset.name,
      enabled: preset.enabled,
      valid: library.diagnostics.presets[preset.id]?.valid,
      meta: `${preset.playerCount} seats · ${preset.rulesetId}`,
    }));
  }

  return library.roles.map((role) => ({
    id: role.id,
    name: role.name,
    enabled: role.enabled,
    valid: library.diagnostics.roles[role.id]?.valid,
    meta: `${role.team} · ${role.mechanicKey}`,
  }));
}

function itemsForTab(library: LibraryActionsRecord, tab: LibraryTab) {
  if (tab === "characters") {
    return library.characters;
  }

  if (tab === "presets") {
    return library.presets;
  }

  return library.roles;
}

function selectItem(
  library: LibraryActionsRecord,
  tab: LibraryTab,
  selectedId: string | null,
): SelectedItem | null {
  const items = itemsForTab(library, tab);
  const item = items.find((candidate) => candidate.id === selectedId) ?? items[0];

  if (item === undefined) {
    return null;
  }

  if (tab === "characters") {
    return {
      kind: "character",
      id: item.id,
      item: item as CharacterDefinition,
    };
  }

  if (tab === "presets") {
    return {
      kind: "preset",
      id: item.id,
      item: item as GamePreset,
    };
  }

  return {
    kind: "role",
    id: item.id,
    item: item as RoleDefinition,
  };
}
