import Link from "next/link";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import type { PresenterDefinition } from "@/core/presenter-definition";
import type { PresenterVoiceManifest } from "@/core/presenter-voice";
import type { RoleDefinition } from "@/core/role-definition";
import type { LibraryActionsRecord } from "@/server/library-actions";
import { textButtonClassName } from "@/components/ui/button-styles";
import { CharacterEditor } from "./character-editor";
import { LibraryList, type LibraryListItem } from "./library-list";
import { PresetEditor } from "./preset-editor";
import { PresenterEditor } from "./presenter-editor";
import { RoleEditor } from "./role-editor";
import { ValidationPanel } from "./validation-panel";

export type LibraryTab = "roles" | "characters" | "presenters" | "presets";

type LibraryWorkspaceProps = {
  readonly activeTab: LibraryTab;
  readonly selectedId: string | null;
  readonly library: LibraryActionsRecord;
  readonly presenterVoiceManifests?: Readonly<Record<string, PresenterVoiceManifest>>;
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
    }
  | {
      readonly kind: "presenter";
      readonly id: string;
      readonly item: PresenterDefinition;
    };

const tabs: readonly LibraryTab[] = [
  "roles",
  "characters",
  "presenters",
  "presets",
];

export function LibraryWorkspace({
  activeTab,
  selectedId,
  library,
  presenterVoiceManifests = {},
}: LibraryWorkspaceProps) {
  const selected = selectItem(library, activeTab, selectedId);
  const effectiveSelectedId = selected?.id ?? null;

  return (
    <main className="h-screen overflow-hidden bg-background text-foreground">
      <div className="flex h-full min-h-0 flex-col">
        <header className="flex h-14 shrink-0 items-center justify-between border-b border-border bg-background px-4">
          <div className="flex min-w-0 items-center gap-3">
            <Link
              href="/"
              className={textButtonClassName("px-2.5 py-1.5 text-xs text-muted")}
            >
              Back
            </Link>
            <div className="min-w-0">
              <h1 className="truncate text-base font-semibold text-foreground">
                Kiva Library
              </h1>
              <p className="text-xs text-subtle">Director configuration desk</p>
            </div>
          </div>
          <Link
            href="/library?tab=presets"
            className={textButtonClassName("px-3 py-1.5 text-xs font-semibold")}
          >
            {activeTab === "presets" ? "Presets entry" : "Create game"}
          </Link>
        </header>

        <div className="grid min-h-0 flex-1 grid-cols-[128px_280px_minmax(0,1fr)_320px] overflow-hidden">
          <nav className="min-h-0 border-r border-border bg-background px-2 py-3">
            <div className="mb-2 px-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-subtle">
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
                      ? "bg-surface-strong text-foreground"
                      : "text-muted hover:bg-surface-muted hover:text-foreground",
                  ].join(" ")}
                >
                  <span className="block font-medium">{labelForTab(tab)}</span>
                  <span className="mt-0.5 block text-[11px] text-subtle">
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
            className="min-h-0 overflow-y-auto bg-background p-4"
          >
            {selected === null ? (
              <EmptyDetail />
            ) : selected.kind === "role" ? (
              <RoleEditor
                key={selectedEditorKey(selected)}
                role={selected.item}
              />
            ) : selected.kind === "character" ? (
              <CharacterEditor
                key={selectedEditorKey(selected)}
                character={selected.item}
              />
            ) : selected.kind === "presenter" ? (
              <PresenterEditor
                key={selectedEditorKey(selected)}
                presenter={selected.item}
                voiceManifest={presenterVoiceManifests[selected.item.id]}
              />
            ) : (
              <PresetEditor
                key={selectedEditorKey(selected)}
                preset={selected.item}
                roles={library.roles}
                characters={library.characters}
                presenters={library.presenters.filter(
                  (presenter) => presenter.enabled,
                )}
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

export function selectedEditorKey(
  selected: Pick<SelectedItem, "kind" | "id"> | null,
): string | null {
  return selected === null ? null : `${selected.kind}:${selected.id}`;
}

function EmptyDetail() {
  return (
    <div className="flex h-full items-center justify-center text-sm text-subtle">
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

  if (tab === "presenters") {
    return "Presenters";
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

  if (tab === "presenters") {
    return library.presenters.map((presenter) => ({
      id: presenter.id,
      name: presenter.name,
      enabled: presenter.enabled,
      valid: library.diagnostics.presenters[presenter.id]?.valid,
      meta: `${Object.keys(presenter.lines).length} lines`,
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

  if (tab === "presenters") {
    return library.presenters;
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

  if (tab === "presenters") {
    return {
      kind: "presenter",
      id: item.id,
      item: item as PresenterDefinition,
    };
  }

  return {
    kind: "role",
    id: item.id,
    item: item as RoleDefinition,
  };
}
