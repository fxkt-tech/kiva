import Link from "next/link";
import { textButtonClassName } from "@/components/ui/button-styles";
import {
  compileActorAuthorCard,
  compileActorRuntimeCard,
  type ActorDefinition,
} from "@/core/actor-definition";
import {
  compileActorComparisonMatrix,
  diagnoseActorPool,
  diagnoseActorSelection,
} from "@/core/actor-diagnostics";
import {
  createGameScriptSnapshot,
  type GameScriptDefinition,
} from "@/core/game-script";
import type { Lineup } from "@/core/lineup";
import {
  createGamePresenterSnapshot,
  type PresenterDefinition,
} from "@/core/presenter-definition";
import type { RuleRole } from "@/core/rule-role";
import type { ContentCatalogSnapshot } from "@/server/content-catalog";
import type { PresenterVoiceManifest } from "@/core/presenter-voice";
import { ActorEditor } from "./actor-editor";
import { LibraryList, type LibraryListItem } from "./library-list";
import { LineupEditor } from "./lineup-editor";
import { PresenterEditor } from "./presenter-editor";
import { RuleRoleViewer } from "./rule-role-viewer";
import { ScriptEditor } from "./script-editor";

export type LibraryTab =
  | "actors"
  | "lineups"
  | "scripts"
  | "presenters"
  | "rules";

type SelectedItem =
  | { readonly kind: "actor"; readonly id: string; readonly item: ActorDefinition }
  | { readonly kind: "new_actor"; readonly id: string; readonly item: ActorDefinition }
  | { readonly kind: "lineup"; readonly id: string; readonly item: Lineup }
  | { readonly kind: "script"; readonly id: string; readonly item: GameScriptDefinition }
  | { readonly kind: "presenter"; readonly id: string; readonly item: PresenterDefinition }
  | { readonly kind: "rule"; readonly id: string; readonly item: RuleRole };

const tabs: readonly LibraryTab[] = [
  "actors",
  "lineups",
  "scripts",
  "presenters",
  "rules",
];

export function LibraryWorkspace({
  activeTab,
  selectedId,
  catalog,
  presenterVoiceManifests = {},
}: {
  readonly activeTab: LibraryTab;
  readonly selectedId: string | null;
  readonly catalog: ContentCatalogSnapshot;
  readonly presenterVoiceManifests?: Readonly<Record<string, PresenterVoiceManifest>>;
}) {
  const selected = selectItem(catalog, activeTab, selectedId);
  return (
    <main className="h-screen overflow-hidden bg-background text-foreground">
      <div className="flex h-full min-h-0 flex-col">
        <header className="flex h-14 shrink-0 items-center justify-between border-b border-border px-4">
          <div className="flex items-center gap-3">
            <Link href="/" className={textButtonClassName("px-2.5 py-1.5 text-xs text-muted")}>Back</Link>
            <div>
              <h1 className="text-base font-semibold">Kiva Library Studio</h1>
              <p className="text-xs text-subtle">Actor、阵容与节目内容的唯一来源</p>
            </div>
          </div>
          <Link href="/library?tab=lineups" className={textButtonClassName("px-3 py-1.5 text-xs font-semibold")}>
            创建对局
          </Link>
        </header>

        <div className="grid min-h-0 flex-1 grid-cols-[126px_260px_minmax(0,1fr)_340px] overflow-hidden">
          <nav className="min-h-0 border-r border-border px-2 py-3">
            <div className="mb-2 px-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-subtle">Catalog</div>
            <div className="space-y-1">
              {tabs.map((tab) => (
                <Link
                  key={tab}
                  href={`/library?tab=${tab}`}
                  aria-current={tab === activeTab ? "page" : undefined}
                  className={[
                    "block rounded px-2.5 py-2 text-sm transition",
                    tab === activeTab ? "bg-surface-strong text-foreground" : "text-muted hover:bg-surface-muted",
                  ].join(" ")}
                >
                  <span className="block font-medium">{tabLabel(tab)}</span>
                  <span className="mt-0.5 block text-[11px] text-subtle">{String(itemsForTab(catalog, tab).length).padStart(2, "0")}</span>
                </Link>
              ))}
            </div>
          </nav>

          <LibraryList
            tab={activeTab}
            selectedId={selected?.id ?? null}
            items={listItems(catalog, activeTab)}
          />

          <section aria-label="Detail region" className="min-h-0 overflow-y-auto p-4">
            {!selected ? (
              <div className="grid h-full place-items-center text-sm text-subtle">No catalog object selected.</div>
            ) : selected.kind === "actor" || selected.kind === "new_actor" ? (
              <ActorEditor
                key={`actor:${selected.id}`}
                actor={selected.item}
                mode={selected.kind === "new_actor" ? "create" : "edit"}
              />
            ) : selected.kind === "lineup" ? (
              <LineupEditor
                key={`lineup:${selected.id}`}
                lineup={selected.item}
                actors={catalog.actors}
                ruleRoles={catalog.ruleRoles}
                presenters={catalog.presenters}
                scripts={catalog.scripts}
              />
            ) : selected.kind === "script" ? (
              <ScriptEditor key={`script:${selected.id}`} script={selected.item} />
            ) : selected.kind === "presenter" ? (
              <PresenterEditor
                key={`presenter:${selected.id}`}
                presenter={selected.item}
                voiceManifest={presenterVoiceManifests[selected.id]}
              />
            ) : (
              <RuleRoleViewer role={selected.item} />
            )}
          </section>

          <CompilerPreview selected={selected} catalog={catalog} />
        </div>
      </div>
    </main>
  );
}

function CompilerPreview({
  selected,
  catalog,
}: {
  readonly selected: SelectedItem | null;
  readonly catalog: ContentCatalogSnapshot;
}) {
  let title = "Compiler preview";
  let body: unknown = null;
  if (selected?.kind === "actor") {
    title = "Actor cards";
    const runtime = compileActorRuntimeCard(selected.item);
    const author = compileActorAuthorCard(selected.item);
    body = {
      runtime,
      author,
      production: selected.item.production,
      compiledSize: {
        runtimeCharacters: JSON.stringify(runtime).length,
        authorCharacters: JSON.stringify(author).length,
      },
      poolDiagnostics: diagnoseActorPool(catalog.actors),
      poolMatrix: compileActorComparisonMatrix(catalog.actors),
    };
  } else if (selected?.kind === "new_actor") {
    title = "New Actor contract";
    body = {
      requiredSections: [
        "identity",
        "core",
        "cognition",
        "interaction",
        "expression",
        "production",
      ],
      note: "保存后才会进入 Runtime/Author 编译与全池诊断。",
    };
  } else if (selected?.kind === "lineup") {
    title = "Resolved cast matrix";
    body = {
      selectionDiagnostics: diagnoseActorSelection({
        actors: catalog.actors,
        actorIds: selected.item.seats.map((seat) => seat.actorId),
      }),
      cast: selected.item.seats.map((seat) => ({
        seatNo: seat.seatNo,
        ruleRole: catalog.ruleRoles.find((role) => role.id === seat.ruleRoleId)?.name,
        actor: catalog.actors.find((actor) => actor.id === seat.actorId)?.identity.name,
        actorId: seat.actorId,
      })),
    };
  } else if (selected?.kind === "rule") {
    title = "Engine-owned contract";
    body = selected.item;
  } else if (selected?.kind === "script") {
    title = "Theme snapshot";
    body = createGameScriptSnapshot(selected.item);
  } else if (selected?.kind === "presenter") {
    title = "Show voice";
    body = {
      snapshot: createGamePresenterSnapshot(selected.item),
      lineCount: Object.keys(selected.item.lines).length,
    };
  }
  return (
    <aside className="min-h-0 overflow-y-auto border-l border-border bg-background/70 p-4">
      <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">{title}</p>
      <p className="mt-2 text-xs leading-5 text-subtle">
        这里展示真正送往运行时或 Script Author 的结构，不展示手写自由 Prompt。
      </p>
      <pre className="mt-4 whitespace-pre-wrap break-words rounded border border-border bg-surface/45 p-3 text-xs leading-5 text-muted">
        {body === null ? "No preview." : JSON.stringify(body, null, 2)}
      </pre>
    </aside>
  );
}

function listItems(
  catalog: ContentCatalogSnapshot,
  tab: LibraryTab,
): readonly LibraryListItem[] {
  if (tab === "actors") {
    return catalog.actors.map((actor) => ({
      id: actor.id,
      name: actor.identity.name,
      enabled: actor.enabled,
      meta: actor.identity.tags.join(" · "),
    }));
  }
  if (tab === "lineups") {
    return catalog.lineups.map((lineup) => ({
      id: lineup.id,
      name: lineup.name,
      enabled: lineup.enabled,
      meta: `${lineup.seats.length} seats · r${lineup.revision}`,
    }));
  }
  if (tab === "scripts") {
    return catalog.scripts.map((script) => ({
      id: script.id,
      name: script.name,
      enabled: script.enabled,
      meta: script.theme,
    }));
  }
  if (tab === "presenters") {
    return catalog.presenters.map((presenter) => ({
      id: presenter.id,
      name: presenter.name,
      enabled: presenter.enabled,
      meta: `${Object.keys(presenter.lines).length} lines`,
    }));
  }
  return catalog.ruleRoles.map((role) => ({
    id: role.id,
    name: role.name,
    enabled: true,
    meta: `${role.team} · ${role.mechanicKey}`,
  }));
}

function itemsForTab(catalog: ContentCatalogSnapshot, tab: LibraryTab) {
  if (tab === "actors") return catalog.actors;
  if (tab === "lineups") return catalog.lineups;
  if (tab === "scripts") return catalog.scripts;
  if (tab === "presenters") return catalog.presenters;
  return catalog.ruleRoles;
}

function selectItem(
  catalog: ContentCatalogSnapshot,
  tab: LibraryTab,
  selectedId: string | null,
): SelectedItem | null {
  if (tab === "actors" && selectedId === "__new_actor__") {
    return {
      kind: "new_actor",
      id: "__new_actor__",
      item: newActorTemplate(catalog.actors[0]),
    };
  }
  const item = itemsForTab(catalog, tab).find((value) => value.id === selectedId) ?? itemsForTab(catalog, tab)[0];
  if (!item) return null;
  if (tab === "actors") return { kind: "actor", id: item.id, item: item as ActorDefinition };
  if (tab === "lineups") return { kind: "lineup", id: item.id, item: item as Lineup };
  if (tab === "scripts") return { kind: "script", id: item.id, item: item as GameScriptDefinition };
  if (tab === "presenters") return { kind: "presenter", id: item.id, item: item as PresenterDefinition };
  return { kind: "rule", id: item.id, item: item as RuleRole };
}

function newActorTemplate(
  productionSource: ActorDefinition | undefined,
): ActorDefinition {
  if (!productionSource) {
    throw new Error("Creating an Actor requires one production profile template");
  }
  return {
    id: "",
    identity: { name: "", portrait: "", tags: [], visualAnchor: "" },
    core: { stableCore: "", drive: "", blindSpot: "", changeBoundary: "" },
    cognition: {
      attention: "",
      evidencePolicy: "",
      decisionPolicy: "",
      correctionTrigger: "",
    },
    interaction: {
      tableFunction: "",
      socialStrategy: "",
      pressureResponse: "",
      conflictAxes: [],
    },
    expression: { cadence: "", diction: "", rhetoricalMoves: [], avoid: [] },
    production: structuredClone(productionSource.production),
    enabled: true,
    revision: 0,
  };
}

function tabLabel(tab: LibraryTab): string {
  if (tab === "actors") return "Actors";
  if (tab === "lineups") return "Lineups";
  if (tab === "scripts") return "Scripts";
  if (tab === "presenters") return "Show";
  return "Rules";
}
