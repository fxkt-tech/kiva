"use client";

import { useMemo, useState } from "react";
import {
  createGameFromPresetHomeAction,
  createGameFromSeatAssignmentsAction,
} from "@/app/actions";
import { Button } from "@/components/ui/button";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset, GamePresetSeatAssignment } from "@/core/game-preset";
import type { GameScriptDefinition } from "@/core/game-script";
import type { RoleDefinition } from "@/core/role-definition";
import {
  createRandomSeatSetup,
  validateSeatSetup,
} from "./new-game-setup";

type NewGameDialogProps = {
  readonly presets: readonly GamePreset[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly scripts: readonly GameScriptDefinition[];
};

type Mode = "preset" | "random";

export function NewGameDialog({
  presets,
  roles,
  characters,
  scripts,
}: NewGameDialogProps) {
  const [open, setOpen] = useState(false);
  const [mode, setMode] = useState<Mode>("preset");
  const [selectedPresetId, setSelectedPresetId] = useState(
    presets[0]?.id ?? "",
  );
  const [selectedScriptId, setSelectedScriptId] = useState(
    scripts[0]?.id ?? "",
  );
  const [randomSeats, setRandomSeats] = useState<readonly GamePresetSeatAssignment[]>(
    () => createRandomSeatSetup({ roles, characters, random: () => 0 }),
  );
  const enabledRoles = useMemo(
    () => roles.filter((role) => role.enabled),
    [roles],
  );
  const enabledCharacters = useMemo(
    () => characters.filter((character) => character.enabled),
    [characters],
  );
  const selectedPreset =
    presets.find((preset) => preset.id === selectedPresetId) ?? presets[0] ?? null;
  const validationMessages = validateSeatSetup(
    randomSeats,
    enabledRoles,
    enabledCharacters,
  );

  function openDialog() {
    setRandomSeats(createRandomSeatSetup({ roles: enabledRoles, characters: enabledCharacters }));
    setOpen(true);
  }

  function updateRandomSeat(
    seatNo: number,
    field: "roleId" | "characterId",
    value: string,
  ) {
    setRandomSeats((currentSeats) =>
      currentSeats.map((seat) =>
        seat.seatNo === seatNo ? { ...seat, [field]: value } : seat,
      ),
    );
  }

  return (
    <>
      <Button
        onClick={openDialog}
        className="w-full sm:w-auto"
      >
        New game
      </Button>

      {open ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="new-game-title"
            className="flex max-h-[90vh] w-full max-w-4xl flex-col overflow-hidden rounded-lg border border-border bg-background shadow-2xl"
          >
            <header className="flex items-center justify-between gap-4 border-b border-border px-5 py-4">
              <div>
                <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
                  Setup
                </p>
                <h2
                  id="new-game-title"
                  className="mt-1 text-lg font-semibold text-foreground"
                >
                  New game
                </h2>
              </div>
              <Button
                onClick={() => setOpen(false)}
                className="px-3 py-1.5 text-muted"
              >
                Close
              </Button>
            </header>

            <div className="flex min-h-0 flex-1 flex-col overflow-y-auto p-5">
              <ScriptPicker
                scripts={scripts}
                selectedScriptId={selectedScriptId}
                onSelectScript={setSelectedScriptId}
              />

              <div className="mb-4 inline-flex w-fit rounded border border-border bg-surface-muted p-1">
                <ModeButton active={mode === "preset"} onClick={() => setMode("preset")}>
                  Preset
                </ModeButton>
                <ModeButton active={mode === "random"} onClick={() => setMode("random")}>
                  Random
                </ModeButton>
              </div>

              {mode === "preset" ? (
                <PresetMode
                  presets={presets}
                  roles={roles}
                  characters={characters}
                  selectedPreset={selectedPreset}
                  selectedPresetId={selectedPresetId}
                  onSelectPreset={setSelectedPresetId}
                  selectedScriptId={selectedScriptId}
                />
              ) : (
                <RandomMode
                  seats={randomSeats}
                  roles={enabledRoles}
                  characters={enabledCharacters}
                  validationMessages={validationMessages}
                  onReroll={() =>
                    setRandomSeats(
                      createRandomSeatSetup({
                        roles: enabledRoles,
                        characters: enabledCharacters,
                      }),
                    )
                  }
                  onUpdateSeat={updateRandomSeat}
                  selectedScriptId={selectedScriptId}
                />
              )}
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}

export function ScriptPicker({
  scripts,
  selectedScriptId,
  onSelectScript,
}: {
  readonly scripts: readonly GameScriptDefinition[];
  readonly selectedScriptId: string;
  readonly onSelectScript: (scriptId: string) => void;
}) {
  return (
    <section className="mb-5" aria-label="选择剧本">
      <div className="mb-2 flex items-end justify-between gap-4">
        <div>
          <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-cyan-300">Script</p>
          <h3 className="mt-1 text-base font-semibold text-foreground">选择本局剧本</h3>
        </div>
        <p className="text-xs text-subtle">角色固定，剧本决定共同背景与视觉包装</p>
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        {scripts.map((script) => {
          const active = script.id === selectedScriptId;
          return (
            <Button
              key={script.id}
              onClick={() => onSelectScript(script.id)}
              unstyled
              className={[
                "group overflow-hidden rounded border text-left transition",
                active ? "border-cyan-300 bg-[#0b191a] shadow-lg shadow-cyan-950/30" : "border-border bg-surface/60 hover:border-interactive-border-hover",
              ].join(" ")}
            >
              <div className="grid min-h-32 grid-cols-[132px_1fr]">
                <img className="h-full min-h-32 w-full object-cover" src={script.presentation.coverImage} alt="" />
                <div className="p-3.5">
                  <div className="flex items-start justify-between gap-3">
                    <span className="text-lg font-semibold text-foreground">{script.name}</span>
                    <span className={active ? "text-cyan-300" : "text-subtle"}>{active ? "已选择" : "选择"}</span>
                  </div>
                  <p className="mt-1 text-xs font-medium text-cyan-200/80">{script.theme}</p>
                  <p className="mt-2 line-clamp-2 text-xs leading-5 text-muted">{script.background}</p>
                  <div className="mt-3 flex flex-wrap gap-1.5">
                    {script.atmosphere.slice(0, 4).map((item) => (
                      <span key={item} className="rounded-sm border border-cyan-800/60 px-1.5 py-0.5 text-[10px] text-cyan-100/70">{item}</span>
                    ))}
                  </div>
                </div>
              </div>
            </Button>
          );
        })}
      </div>
    </section>
  );
}

function ModeButton({
  active,
  onClick,
  children,
}: {
  readonly active: boolean;
  readonly onClick: () => void;
  readonly children: React.ReactNode;
}) {
  return (
    <Button
      onClick={onClick}
      className={[
        "cursor-pointer rounded px-3 py-1.5 text-sm font-medium transition",
        active
          ? "border border-interactive-border bg-surface/90 text-foreground shadow-sm shadow-black/[0.03]"
          : "border border-transparent text-muted hover:bg-surface/70 hover:text-foreground",
      ].join(" ")}
      unstyled
    >
      {children}
    </Button>
  );
}

function PresetMode({
  presets,
  roles,
  characters,
  selectedPreset,
  selectedPresetId,
  onSelectPreset,
  selectedScriptId,
}: {
  readonly presets: readonly GamePreset[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly selectedPreset: GamePreset | null;
  readonly selectedPresetId: string;
  readonly onSelectPreset: (presetId: string) => void;
  readonly selectedScriptId: string;
}) {
  return (
    <div className="grid min-h-0 gap-4 lg:grid-cols-[260px_minmax(0,1fr)]">
      <div className="space-y-2">
        {presets.length === 0 ? (
          <p className="rounded border border-border bg-surface-muted/60 p-3 text-sm text-subtle">
            No presets available.
          </p>
        ) : (
          presets.map((preset) => (
            <Button
              key={preset.id}
              onClick={() => onSelectPreset(preset.id)}
              className={[
                "block w-full cursor-pointer rounded border px-3 py-3 text-left transition",
                preset.id === selectedPresetId
                  ? "border-accent bg-surface-strong text-foreground"
                  : "border-border bg-surface/55 text-muted hover:border-interactive-border-hover",
              ].join(" ")}
              unstyled
            >
              <span className="block truncate text-sm font-medium">
                {preset.name}
              </span>
              <span className="mt-1 block font-mono text-xs text-subtle">
                {preset.id}
              </span>
            </Button>
          ))
        )}
      </div>

      <div className="space-y-4">
        {selectedPreset ? (
          <>
            <ReadOnlySeatTable
              seats={seatsForPreset(selectedPreset)}
              roles={roles}
              characters={characters}
            />
            <form action={createGameFromPresetHomeAction.bind(null, selectedPreset.id)}>
              <input type="hidden" name="scriptId" value={selectedScriptId} />
              <div className="flex justify-end border-t border-border pt-4">
                <Button
                  type="submit"
                  className="font-semibold"
                >
                  Create game
                </Button>
              </div>
            </form>
          </>
        ) : null}
      </div>
    </div>
  );
}

function RandomMode({
  seats,
  roles,
  characters,
  validationMessages,
  onReroll,
  onUpdateSeat,
  selectedScriptId,
}: {
  readonly seats: readonly GamePresetSeatAssignment[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly validationMessages: readonly string[];
  readonly onReroll: () => void;
  readonly onUpdateSeat: (
    seatNo: number,
    field: "roleId" | "characterId",
    value: string,
  ) => void;
  readonly selectedScriptId: string;
}) {
  return (
    <form action={createGameFromSeatAssignmentsAction} className="space-y-4">
      <input type="hidden" name="scriptId" value={selectedScriptId} />
      <div className="flex justify-end">
        <Button
          onClick={onReroll}
          className="px-3"
        >
          Reroll
        </Button>
      </div>

      <EditableSeatTable
        seats={seats}
        roles={roles}
        characters={characters}
        onUpdateSeat={onUpdateSeat}
      />

      {validationMessages.length > 0 ? (
        <ul className="space-y-2 text-sm text-warning-badge-foreground">
          {validationMessages.map((message) => (
            <li key={message} className="rounded bg-warning-badge px-3 py-2">
              {message}
            </li>
          ))}
        </ul>
      ) : null}

      <div className="flex justify-end border-t border-border pt-4">
        <Button
          type="submit"
          disabled={validationMessages.length > 0}
          className="font-semibold"
        >
          Create game
        </Button>
      </div>
    </form>
  );
}

function ReadOnlySeatTable({
  seats,
  roles,
  characters,
}: {
  readonly seats: readonly GamePresetSeatAssignment[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
}) {
  return (
    <SeatTableFrame>
      {seats.map((seat) => (
        <SeatRow
          key={seat.seatNo}
          seatNo={seat.seatNo}
          role={roleLabel(roles, seat.roleId)}
          character={characterLabel(characters, seat.characterId)}
        />
      ))}
    </SeatTableFrame>
  );
}

function EditableSeatTable({
  seats,
  roles,
  characters,
  onUpdateSeat,
}: {
  readonly seats: readonly GamePresetSeatAssignment[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly onUpdateSeat: (
    seatNo: number,
    field: "roleId" | "characterId",
    value: string,
  ) => void;
}) {
  return (
    <SeatTableFrame>
      {seats.map((seat) => (
        <div
          key={seat.seatNo}
          className="grid grid-cols-[64px_minmax(0,1fr)_minmax(0,1fr)] gap-3 border-b border-border px-3 py-2 last:border-b-0"
        >
          <span className="self-center font-mono text-xs text-subtle">
            Seat {seat.seatNo}
          </span>
          <select
            name={`seat.${seat.seatNo}.roleId`}
            value={seat.roleId}
            onChange={(event) =>
              onUpdateSeat(seat.seatNo, "roleId", event.target.value)
            }
            className="min-w-0 rounded border border-interactive-border bg-background px-2 py-1.5 text-sm text-foreground outline-none"
          >
            {roles.map((role) => (
              <option key={role.id} value={role.id}>
                {role.name} · {role.id}
              </option>
            ))}
          </select>
          <select
            name={`seat.${seat.seatNo}.characterId`}
            value={seat.characterId}
            onChange={(event) =>
              onUpdateSeat(seat.seatNo, "characterId", event.target.value)
            }
            className="min-w-0 rounded border border-interactive-border bg-background px-2 py-1.5 text-sm text-foreground outline-none"
          >
            {characters.map((character) => (
              <option key={character.id} value={character.id}>
                {character.name} · {character.id}
              </option>
            ))}
          </select>
        </div>
      ))}
    </SeatTableFrame>
  );
}

function SeatTableFrame({ children }: { readonly children: React.ReactNode }) {
  return (
    <div className="overflow-hidden rounded border border-border">
      <div className="grid grid-cols-[64px_minmax(0,1fr)_minmax(0,1fr)] gap-3 border-b border-border bg-surface-muted/60 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.12em] text-subtle">
        <span>Seat</span>
        <span>Role</span>
        <span>Character</span>
      </div>
      {children}
    </div>
  );
}

function SeatRow({
  seatNo,
  role,
  character,
}: {
  readonly seatNo: number;
  readonly role: string;
  readonly character: string;
}) {
  return (
    <div className="grid grid-cols-[64px_minmax(0,1fr)_minmax(0,1fr)] gap-3 border-b border-border px-3 py-2 text-sm last:border-b-0">
      <span className="font-mono text-xs text-subtle">Seat {seatNo}</span>
      <span className="min-w-0 truncate text-foreground">{role}</span>
      <span className="min-w-0 truncate text-foreground">{character}</span>
    </div>
  );
}

function seatsForPreset(preset: GamePreset): readonly GamePresetSeatAssignment[] {
  if (preset.seatAssignments !== null) {
    return preset.seatAssignments;
  }

  return Array.from({ length: preset.playerCount }, (_, index) => ({
    seatNo: index + 1,
    roleId: preset.roleIds[index] ?? "",
    characterId: preset.characterIds[index] ?? "",
    modelBindingOverride: null,
  }));
}

function roleLabel(roles: readonly RoleDefinition[], roleId: string): string {
  const role = roles.find((item) => item.id === roleId);
  return role ? `${role.name} · ${role.id}` : roleId;
}

function characterLabel(
  characters: readonly CharacterDefinition[],
  characterId: string,
): string {
  const character = characters.find((item) => item.id === characterId);
  return character ? `${character.name} · ${character.id}` : characterId;
}
