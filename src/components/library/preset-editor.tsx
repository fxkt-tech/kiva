import {
  createGameFromPresetAction,
  savePresetAction,
} from "@/app/library/actions";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset, GamePresetSeatAssignment } from "@/core/game-preset";
import type { ModelBindingSnapshot } from "@/core/player";
import type { RoleDefinition } from "@/core/role-definition";
import { DirtyFormGuard } from "./dirty-form-guard";

type PresetEditorProps = {
  readonly preset: GamePreset;
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
};

export function PresetEditor({ preset, roles, characters }: PresetEditorProps) {
  const seats = seatsForPreset(preset);

  return (
    <div className="space-y-5">
      <form action={savePresetAction} className="space-y-5">
        <DirtyFormGuard />
        <input type="hidden" name="createdAt" value={preset.createdAt} />

        <div className="flex items-start justify-between gap-3 border-b border-zinc-800 pb-3">
          <div className="min-w-0">
            <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
              Preset editor
            </p>
            <h2 className="mt-1 truncate text-xl font-semibold text-zinc-50">
              {preset.name}
            </h2>
            <p className="mt-1 truncate text-xs text-zinc-500">
              {preset.id} · {preset.rulesetId}
            </p>
          </div>
          <span
            className={[
              "rounded border px-2 py-1 text-xs font-medium",
              preset.enabled
                ? "border-emerald-500/40 text-emerald-300"
                : "border-zinc-700 text-zinc-500",
            ].join(" ")}
          >
            {preset.enabled ? "enabled" : "disabled"}
          </span>
        </div>

        <div className="grid gap-3 md:grid-cols-2">
          <TextField label="Id" name="id" defaultValue={preset.id} />
          <TextField label="Name" name="name" defaultValue={preset.name} />
        </div>

        <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_160px]">
          <TextField
            label="Ruleset"
            name="rulesetId"
            defaultValue={preset.rulesetId}
          />
          <TextField
            label="Player count"
            name="playerCount"
            defaultValue={preset.playerCount.toString()}
          />
        </div>

        <label className="flex items-center gap-2 text-sm text-zinc-300">
          <input
            type="checkbox"
            name="enabled"
            defaultChecked={preset.enabled}
            className="h-4 w-4 accent-cyan-300"
          />
          Enabled
        </label>

        <section>
          <div className="flex items-center justify-between gap-3">
            <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
              Seat assignments
            </h3>
            <span className="font-mono text-xs text-zinc-500">
              {seats.length} seats
            </span>
          </div>
          <div className="mt-2 overflow-hidden rounded border border-zinc-800">
            <div className="grid grid-cols-[64px_minmax(0,1fr)_minmax(0,1fr)] gap-3 border-b border-zinc-800 bg-zinc-900/60 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.12em] text-zinc-500">
              <span>Seat</span>
              <span>Role</span>
              <span>Character</span>
            </div>
            {seats.map((seat) => (
              <div
                key={seat.seatNo}
                className="grid grid-cols-[64px_minmax(0,1fr)_minmax(0,1fr)] gap-3 border-b border-zinc-800 px-3 py-2 last:border-b-0"
              >
                <span className="self-center font-mono text-xs text-zinc-500">
                  Seat {seat.seatNo}
                </span>
                <select
                  name={`seat.${seat.seatNo}.roleId`}
                  defaultValue={seat.roleId}
                  className="min-w-0 rounded border border-zinc-700 bg-zinc-950 px-2 py-1.5 text-sm text-zinc-100 outline-none focus:border-cyan-300"
                >
                  {roles.map((role) => (
                    <option key={role.id} value={role.id}>
                      {role.name} · {role.id}
                    </option>
                  ))}
                </select>
                <select
                  name={`seat.${seat.seatNo}.characterId`}
                  defaultValue={seat.characterId}
                  className="min-w-0 rounded border border-zinc-700 bg-zinc-950 px-2 py-1.5 text-sm text-zinc-100 outline-none focus:border-cyan-300"
                >
                  {characters.map((character) => (
                    <option key={character.id} value={character.id}>
                      {character.name} · {character.id}
                    </option>
                  ))}
                </select>
                <input
                  type="hidden"
                  name={`seat.${seat.seatNo}.modelBindingOverride`}
                  value={modelBindingValue(seat.modelBindingOverride)}
                />
              </div>
            ))}
          </div>
        </section>

        <div className="flex justify-end border-t border-zinc-800 pt-4">
          <button
            type="submit"
            className="rounded bg-zinc-100 px-4 py-2 text-sm font-semibold text-zinc-950 transition hover:bg-white"
          >
            Save preset
          </button>
        </div>
      </form>

      <form
        action={createGameFromPresetAction.bind(null, preset.id)}
        className="flex justify-end border-t border-zinc-800 pt-4"
      >
        <button
          type="submit"
          className="rounded border border-emerald-500/60 px-4 py-2 text-sm font-semibold text-emerald-200 transition hover:border-emerald-300 hover:text-emerald-100"
        >
          Create game
        </button>
      </form>
    </div>
  );
}

function TextField({
  label,
  name,
  defaultValue,
}: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
}) {
  return (
    <label className="block text-sm text-zinc-300">
      <span className="text-xs font-medium uppercase tracking-[0.12em] text-zinc-500">
        {label}
      </span>
      <input
        name={name}
        defaultValue={defaultValue}
        className="mt-1 w-full rounded border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition focus:border-cyan-300"
      />
    </label>
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

function modelBindingValue(value: ModelBindingSnapshot | null): string {
  return value === null ? "" : JSON.stringify(value);
}
