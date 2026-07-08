"use client";

import { useMemo, useState } from "react";
import {
  createGameFromPresetHomeAction,
  createGameFromSeatAssignmentsAction,
} from "@/app/actions";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset, GamePresetSeatAssignment } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";
import { textButtonClassName } from "@/components/ui/button-styles";
import {
  createRandomSeatSetup,
  validateSeatSetup,
} from "./new-game-setup";

type NewGameDialogProps = {
  readonly presets: readonly GamePreset[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
};

type Mode = "preset" | "random";

export function NewGameDialog({
  presets,
  roles,
  characters,
}: NewGameDialogProps) {
  const [open, setOpen] = useState(false);
  const [mode, setMode] = useState<Mode>("preset");
  const [selectedPresetId, setSelectedPresetId] = useState(
    presets[0]?.id ?? "",
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
      <button
        type="button"
        onClick={openDialog}
        className={textButtonClassName("w-full sm:w-auto")}
      >
        New game
      </button>

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
              <button
                type="button"
                onClick={() => setOpen(false)}
                className={textButtonClassName("px-3 py-1.5 text-muted")}
              >
                Close
              </button>
            </header>

            <div className="flex min-h-0 flex-1 flex-col overflow-y-auto p-5">
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
                />
              )}
            </div>
          </div>
        </div>
      ) : null}
    </>
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
    <button
      type="button"
      onClick={onClick}
      className={[
        "rounded px-3 py-1.5 text-sm font-medium transition",
        active
          ? "border border-interactive-border bg-surface/90 text-foreground shadow-sm shadow-black/[0.03]"
          : "border border-transparent text-muted hover:bg-surface/70 hover:text-foreground",
      ].join(" ")}
    >
      {children}
    </button>
  );
}

function PresetMode({
  presets,
  roles,
  characters,
  selectedPreset,
  selectedPresetId,
  onSelectPreset,
}: {
  readonly presets: readonly GamePreset[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly selectedPreset: GamePreset | null;
  readonly selectedPresetId: string;
  readonly onSelectPreset: (presetId: string) => void;
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
            <button
              key={preset.id}
              type="button"
              onClick={() => onSelectPreset(preset.id)}
              className={[
                "block w-full rounded border px-3 py-3 text-left transition",
                preset.id === selectedPresetId
                  ? "border-accent bg-surface-strong text-foreground"
                  : "border-border bg-surface/55 text-muted hover:border-interactive-border-hover",
              ].join(" ")}
            >
              <span className="block truncate text-sm font-medium">
                {preset.name}
              </span>
              <span className="mt-1 block font-mono text-xs text-subtle">
                {preset.id}
              </span>
            </button>
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
              <div className="flex justify-end border-t border-border pt-4">
                <button
                  type="submit"
                  className={textButtonClassName("font-semibold")}
                >
                  Create game
                </button>
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
}) {
  return (
    <form action={createGameFromSeatAssignmentsAction} className="space-y-4">
      <div className="flex justify-end">
        <button
          type="button"
          onClick={onReroll}
          className={textButtonClassName("px-3")}
        >
          Reroll
        </button>
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
        <button
          type="submit"
          disabled={validationMessages.length > 0}
          className={textButtonClassName("font-semibold")}
        >
          Create game
        </button>
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
            className="min-w-0 rounded border border-interactive-border bg-background px-2 py-1.5 text-sm text-foreground outline-none focus:border-accent"
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
            className="min-w-0 rounded border border-interactive-border bg-background px-2 py-1.5 text-sm text-foreground outline-none focus:border-accent"
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
