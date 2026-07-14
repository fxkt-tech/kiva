"use client";

import { useMemo, useState } from "react";
import {
  createGameFromPresetHomeAction,
  createGameFromSeatAssignmentsAction,
} from "@/app/actions";
import { Button } from "@/components/ui/button";
import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePreset, GamePresetSeatAssignment } from "@/core/game-preset";
import type { GameRunMode } from "@/core/game-run-mode";
import type { GameScriptDefinition } from "@/core/game-script";
import type { RoleDefinition } from "@/core/role-definition";
import {
  createRandomSeatSetup,
  validateSeatSetup,
} from "./new-game-setup";
import { roleIdentityColor } from "./role-identity-color";

type NewGameDialogProps = {
  readonly presets: readonly GamePreset[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly scripts: readonly GameScriptDefinition[];
};

type SeatMode = "preset" | "random";
type SetupStep = "experience" | "lineup";

const SEAT_GRID_CLASS =
  "grid grid-cols-[52px_minmax(0,0.9fr)_minmax(0,1.1fr)] items-center gap-2";

export function NewGameDialog({
  presets,
  roles,
  characters,
  scripts,
}: NewGameDialogProps) {
  const [open, setOpen] = useState(false);
  const [setupStep, setSetupStep] = useState<SetupStep>("experience");
  const [runMode, setRunMode] = useState<GameRunMode>("game");
  const [seatMode, setSeatMode] = useState<SeatMode>("preset");
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
    setSetupStep("experience");
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

  function rerollRandomSeats() {
    setRandomSeats(
      createRandomSeatSetup({
        roles: enabledRoles,
        characters: enabledCharacters,
      }),
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
            className="flex max-h-[92vh] w-full max-w-6xl flex-col overflow-hidden rounded-xl border border-border bg-background shadow-2xl"
          >
            <header className="grid gap-4 border-b border-border px-5 py-4 sm:grid-cols-[1fr_auto_auto] sm:items-center">
              <div>
                <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
                  Create a game
                </p>
                <h2
                  id="new-game-title"
                  className="mt-1 text-lg font-semibold text-foreground"
                >
                  New game
                </h2>
              </div>
              <StepIndicator current={setupStep} />
              <Button
                onClick={() => setOpen(false)}
                unstyled
                className="rounded-md border border-border px-3 py-2 text-xs text-muted transition hover:bg-surface hover:text-foreground"
              >
                Close
              </Button>
            </header>

            <div
              key={setupStep}
              className="min-h-0 flex-1 overflow-y-auto"
            >
              {setupStep === "experience" ? (
                <div className="mx-auto flex max-w-4xl flex-col gap-7 px-5 py-6">
                  <RunModePicker runMode={runMode} onSelectRunMode={setRunMode} />
                  <ScriptPicker
                    scripts={scripts}
                    selectedScriptId={selectedScriptId}
                    onSelectScript={setSelectedScriptId}
                  />
                  <div className="flex justify-end border-t border-border pt-4">
                    <Button
                      type="button"
                      disabled={!selectedScriptId}
                      onClick={() => setSetupStep("lineup")}
                      className="min-w-28 font-semibold"
                    >
                      下一步
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="p-4 sm:p-5">
                  <div className="mb-3 grid gap-3 lg:grid-cols-[auto_minmax(0,1fr)_auto] lg:items-center">
                    <div>
                      <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-cyan-300">Lineup</p>
                      <h3 className="mt-1 text-base font-semibold text-foreground">配置本局阵容</h3>
                    </div>
                    <SelectionSummary
                      runMode={runMode}
                      script={scripts.find((script) => script.id === selectedScriptId) ?? null}
                    />
                    <div className="flex flex-wrap items-center justify-between gap-2 lg:justify-end">
                      <div className="inline-flex rounded-lg border border-border bg-surface-muted p-1">
                        <ModeButton active={seatMode === "preset"} onClick={() => setSeatMode("preset")}>
                          使用预设
                        </ModeButton>
                        <ModeButton active={seatMode === "random"} onClick={() => setSeatMode("random")}>
                          随机阵容
                        </ModeButton>
                      </div>
                      {seatMode === "random" ? (
                        <Button
                          type="button"
                          onClick={rerollRandomSeats}
                          className="px-3"
                        >
                          Reroll
                        </Button>
                      ) : null}
                    </div>
                  </div>

                  {seatMode === "preset" ? (
                    <PresetMode
                      presets={presets}
                      roles={roles}
                      characters={characters}
                      selectedPreset={selectedPreset}
                      selectedPresetId={selectedPresetId}
                      onSelectPreset={setSelectedPresetId}
                      selectedScriptId={selectedScriptId}
                      runMode={runMode}
                      onBack={() => setSetupStep("experience")}
                    />
                  ) : (
                    <RandomMode
                      seats={randomSeats}
                      roles={enabledRoles}
                      characters={enabledCharacters}
                      validationMessages={validationMessages}
                      onUpdateSeat={updateRandomSeat}
                      selectedScriptId={selectedScriptId}
                      runMode={runMode}
                      onBack={() => setSetupStep("experience")}
                    />
                  )}
                </div>
              )}
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}

function StepIndicator({ current }: { readonly current: SetupStep }) {
  return (
    <div className="flex items-center gap-2 text-xs" aria-label="创建进度">
      <span className={current === "experience" ? "text-cyan-300" : "text-muted"}>
        <span className="mr-1.5 inline-grid h-6 w-6 place-items-center rounded-full border border-current">1</span>
        玩法与主题
      </span>
      <span className="h-px w-8 bg-border" aria-hidden="true" />
      <span className={current === "lineup" ? "text-cyan-300" : "text-subtle"}>
        <span className="mr-1.5 inline-grid h-6 w-6 place-items-center rounded-full border border-current">2</span>
        阵容与创建
      </span>
    </div>
  );
}

function SelectionSummary({
  runMode,
  script,
}: {
  readonly runMode: GameRunMode;
  readonly script: GameScriptDefinition | null;
}) {
  return (
    <section className="flex min-w-0 flex-wrap items-center gap-2 rounded-lg border border-border bg-surface/55 px-3 py-2">
      <span className="text-xs font-medium text-subtle">本局方案</span>
      <span className="rounded-full border border-cyan-800/70 bg-cyan-950/30 px-2.5 py-1 text-xs text-cyan-100">
        {runMode === "scripted" ? "剧本模式" : "游戏模式"}
      </span>
      <span className="rounded-full border border-border bg-background/60 px-2.5 py-1 text-xs text-muted">
        {script?.name ?? "未选择主题"}
      </span>
      <span className="ml-auto text-xs text-subtle">
        {runMode === "scripted" ? "创建后进入剧本准备" : "创建后直接进入 Editor"}
      </span>
    </section>
  );
}

export function RunModePicker({
  runMode,
  onSelectRunMode,
}: {
  readonly runMode: GameRunMode;
  readonly onSelectRunMode: (runMode: GameRunMode) => void;
}) {
  return (
    <section aria-label="选择运行模式">
      <div className="mb-2">
        <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-cyan-300">
          Run mode
        </p>
        <h3 className="mt-1 text-base font-semibold text-foreground">
          这局游戏如何进行？
        </h3>
      </div>
      <div className="grid gap-3 md:grid-cols-2">
        <RunModeCard
          active={runMode === "game"}
          title="游戏模式"
          description="玩家根据可见信息自主发言、投票和行动；发言受时长预算约束。"
          flow="直接开局 · 目标约 30 分钟"
          onClick={() => onSelectRunMode("game")}
        />
        <RunModeCard
          active={runMode === "scripted"}
          title="剧本模式"
          description="先生成整局结构剧本，经导演审核批准后，再让玩家按剧情节拍执行。"
          flow="生成剧本 → 审核 → 批准 → 开局"
          onClick={() => onSelectRunMode("scripted")}
        />
      </div>
      {runMode === "scripted" ? (
        <p className="mt-2 rounded border border-warning-badge/60 bg-warning-badge/35 px-3 py-2 text-xs leading-5 text-warning-badge-foreground">
          创建后将进入剧本准备页；剧本批准前不会推进游戏、生成配音或导出视频。
        </p>
      ) : null}
    </section>
  );
}

function RunModeCard({
  active,
  title,
  description,
  flow,
  onClick,
}: {
  readonly active: boolean;
  readonly title: string;
  readonly description: string;
  readonly flow: string;
  readonly onClick: () => void;
}) {
  return (
    <Button
      type="button"
      onClick={onClick}
      unstyled
      className={[
        "rounded border p-4 text-left transition",
        active
          ? "border-accent bg-surface-strong shadow-sm"
          : "border-border bg-surface/60 hover:border-interactive-border-hover",
      ].join(" ")}
    >
      <span className="flex items-center justify-between gap-3">
        <span className="text-base font-semibold text-foreground">{title}</span>
        <span className={active ? "text-xs text-cyan-300" : "text-xs text-subtle"}>
          {active ? "已选择" : "选择"}
        </span>
      </span>
      <span className="mt-2 block text-xs leading-5 text-muted">{description}</span>
      <span className="mt-3 block text-xs font-medium text-cyan-200/80">{flow}</span>
    </Button>
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
    <section aria-label="选择主题模板">
      <div className="mb-2 flex items-end justify-between gap-4">
        <div>
          <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-cyan-300">Theme</p>
          <h3 className="mt-1 text-base font-semibold text-foreground">选择主题模板</h3>
        </div>
        <p className="text-xs text-subtle">主题决定共同背景与视觉包装，不等于单局剧情</p>
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
                active ? "border-accent bg-surface-strong shadow-sm" : "border-border bg-surface/60 hover:border-interactive-border-hover",
              ].join(" ")}
            >
              <div className="grid min-h-24 grid-cols-[96px_1fr]">
                <img className="h-full min-h-24 w-full object-cover" src={script.presentation.coverImage} alt="" />
                <div className="p-3">
                  <div className="flex items-start justify-between gap-3">
                    <span className="text-sm font-semibold text-foreground">{script.name}</span>
                    <span className={active ? "text-cyan-300" : "text-subtle"}>{active ? "已选择" : "选择"}</span>
                  </div>
                  <p className="mt-1 text-xs font-medium text-cyan-200/80">{script.theme}</p>
                  <p className="mt-1.5 line-clamp-2 text-xs leading-5 text-muted">{script.background}</p>
                  <div className="mt-2 flex flex-wrap gap-1.5">
                    {script.atmosphere.slice(0, 3).map((item) => (
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
  runMode,
  onBack,
}: {
  readonly presets: readonly GamePreset[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly selectedPreset: GamePreset | null;
  readonly selectedPresetId: string;
  readonly onSelectPreset: (presetId: string) => void;
  readonly selectedScriptId: string;
  readonly runMode: GameRunMode;
  readonly onBack: () => void;
}) {
  return (
    <div className="grid min-h-0 gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
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

      <div className="space-y-3">
        {selectedPreset ? (
          <>
            <ReadOnlySeatTable
              seats={seatsForPreset(selectedPreset)}
              roles={roles}
              characters={characters}
            />
            <form action={createGameFromPresetHomeAction.bind(null, selectedPreset.id)}>
              <input type="hidden" name="scriptId" value={selectedScriptId} />
              <input type="hidden" name="runMode" value={runMode} />
              <div className="flex justify-between border-t border-border pt-3">
                <Button type="button" onClick={onBack} unstyled className="rounded-md border border-border px-4 py-2 text-sm text-muted hover:bg-surface">
                  上一步
                </Button>
                <Button
                  type="submit"
                  className="font-semibold"
                >
                  {createButtonLabel(runMode)}
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
  onUpdateSeat,
  selectedScriptId,
  runMode,
  onBack,
}: {
  readonly seats: readonly GamePresetSeatAssignment[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly validationMessages: readonly string[];
  readonly onUpdateSeat: (
    seatNo: number,
    field: "roleId" | "characterId",
    value: string,
  ) => void;
  readonly selectedScriptId: string;
  readonly runMode: GameRunMode;
  readonly onBack: () => void;
}) {
  return (
    <form action={createGameFromSeatAssignmentsAction} className="space-y-3">
      <input type="hidden" name="scriptId" value={selectedScriptId} />
      <input type="hidden" name="runMode" value={runMode} />

      <EditableSeatTable
        seats={seats}
        roles={roles}
        characters={characters}
        onUpdateSeat={onUpdateSeat}
      />

      {validationMessages.length > 0 ? (
        <ul className="grid gap-2 text-sm text-warning-badge-foreground lg:grid-cols-2">
          {validationMessages.map((message) => (
            <li key={message} className="rounded bg-warning-badge px-3 py-2">
              {message}
            </li>
          ))}
        </ul>
      ) : null}

      <div className="flex justify-between border-t border-border pt-3">
        <Button type="button" onClick={onBack} unstyled className="rounded-md border border-border px-4 py-2 text-sm text-muted hover:bg-surface">
          上一步
        </Button>
        <Button
          type="submit"
          disabled={validationMessages.length > 0}
          className="font-semibold"
        >
          {createButtonLabel(runMode)}
        </Button>
      </div>
    </form>
  );
}

export function ReadOnlySeatTable({
  seats,
  roles,
  characters,
}: {
  readonly seats: readonly GamePresetSeatAssignment[];
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
}) {
  return (
    <SeatTableFrame
      seats={seats}
      renderSeat={(seat) => (
        <SeatRow
          key={seat.seatNo}
          seatNo={seat.seatNo}
          roleId={seat.roleId}
          role={roleLabel(roles, seat.roleId)}
          character={characterLabel(characters, seat.characterId)}
        />
      )}
    />
  );
}

export function EditableSeatTable({
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
    <SeatTableFrame
      seats={seats}
      renderSeat={(seat) => (
        <div
          key={seat.seatNo}
          data-seat-no={seat.seatNo}
          className={`${SEAT_GRID_CLASS} border-b border-border px-2.5 py-1.5 last:border-b-0`}
        >
          <span className="self-center font-mono text-xs text-subtle">
            Seat {seat.seatNo}
          </span>
          <select
            name={`seat.${seat.seatNo}.roleId`}
            value={seat.roleId}
            data-selected-role-id={seat.roleId}
            onChange={(event) =>
              onUpdateSeat(seat.seatNo, "roleId", event.target.value)
            }
            className="min-w-0 rounded border border-interactive-border bg-background px-2 py-1.5 text-xs font-semibold outline-none"
            style={{ color: roleIdentityColor(seat.roleId) }}
          >
            {roles.map((role) => (
              <option
                key={role.id}
                value={role.id}
                data-role-id={role.id}
                style={{ color: roleIdentityColor(role.id) }}
              >
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
            className="min-w-0 rounded border border-interactive-border bg-background px-2 py-1.5 text-xs text-foreground outline-none"
          >
            {characters.map((character) => (
              <option key={character.id} value={character.id}>
                {character.name} · {character.id}
              </option>
            ))}
          </select>
        </div>
      )}
    />
  );
}

function SeatTableFrame({
  seats,
  renderSeat,
}: {
  readonly seats: readonly GamePresetSeatAssignment[];
  readonly renderSeat: (seat: GamePresetSeatAssignment) => React.ReactNode;
}) {
  const columnSize = Math.ceil(seats.length / 2);
  const columns = [seats.slice(0, columnSize), seats.slice(columnSize)];

  return (
    <div
      data-seat-overview="true"
      className="grid overflow-hidden rounded border border-border lg:grid-cols-2"
    >
      {columns.map((column, index) => (
        <div
          key={index}
          data-seat-column={index + 1}
          data-seat-column-size={column.length}
          className={
            index === 0
              ? "min-w-0"
              : "min-w-0 border-t border-border lg:border-l lg:border-t-0"
          }
        >
          <div className={`${SEAT_GRID_CLASS} border-b border-border bg-surface-muted/60 px-2.5 py-1.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-subtle`}>
            <span>Seat</span>
            <span>Role</span>
            <span>Character</span>
          </div>
          {column.map(renderSeat)}
        </div>
      ))}
    </div>
  );
}

function SeatRow({
  seatNo,
  roleId,
  role,
  character,
}: {
  readonly seatNo: number;
  readonly roleId: string;
  readonly role: string;
  readonly character: string;
}) {
  return (
    <div
      data-seat-no={seatNo}
      className={`${SEAT_GRID_CLASS} border-b border-border px-2.5 py-2 text-xs last:border-b-0`}
    >
      <span className="font-mono text-xs text-subtle">Seat {seatNo}</span>
      <span
        className="min-w-0 truncate font-semibold"
        data-role-id={roleId}
        style={{ color: roleIdentityColor(roleId) }}
      >
        {role}
      </span>
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

function createButtonLabel(runMode: GameRunMode): string {
  return runMode === "scripted" ? "创建并进入剧本页" : "创建并进入游戏";
}

function characterLabel(
  characters: readonly CharacterDefinition[],
  characterId: string,
): string {
  const character = characters.find((item) => item.id === characterId);
  return character ? `${character.name} · ${character.id}` : characterId;
}
