import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";

type PresetEditorProps = {
  readonly preset: GamePreset;
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
};

export function PresetEditor({ preset, roles, characters }: PresetEditorProps) {
  const roleNames = new Map(roles.map((role) => [role.id, role.name]));
  const characterNames = new Map(
    characters.map((character) => [character.id, character.name]),
  );

  return (
    <div className="space-y-4">
      <div className="border-b border-zinc-800 pb-3">
        <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
          Preset editor
        </p>
        <h2 className="mt-1 text-xl font-semibold text-zinc-50">
          {preset.name}
        </h2>
        <p className="mt-1 text-xs text-zinc-500">
          {preset.id} · {preset.rulesetId}
        </p>
      </div>
      <div className="grid grid-cols-3 gap-3 text-sm">
        <Metric label="Seats" value={preset.playerCount} />
        <Metric label="Roles" value={preset.roleIds.length} />
        <Metric label="Characters" value={preset.characterIds.length} />
      </div>
      <section>
        <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
          Seat assignments
        </h3>
        <div className="mt-2 divide-y divide-zinc-800 rounded border border-zinc-800">
          {(preset.seatAssignments ?? []).map((seat) => (
            <div
              key={seat.seatNo}
              className="grid grid-cols-[52px_minmax(0,1fr)_minmax(0,1fr)] gap-3 px-3 py-2 text-sm"
            >
              <span className="font-mono text-xs text-zinc-500">
                #{seat.seatNo}
              </span>
              <span className="truncate text-zinc-200">
                {roleNames.get(seat.roleId) ?? seat.roleId}
              </span>
              <span className="truncate text-zinc-400">
                {characterNames.get(seat.characterId) ?? seat.characterId}
              </span>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

function Metric({
  label,
  value,
}: {
  readonly label: string;
  readonly value: number;
}) {
  return (
    <div className="rounded border border-zinc-800 bg-zinc-900/55 px-3 py-2">
      <div className="text-xs text-zinc-500">{label}</div>
      <div className="mt-1 font-mono text-lg text-zinc-100">{value}</div>
    </div>
  );
}
