import { saveRoleAction } from "@/app/library/actions";
import { Button } from "@/components/ui/button";
import type { ModelBindingSnapshot } from "@/core/player";
import type {
  RoleDefinition,
  RoleMechanicKey,
  RoleTeam,
} from "@/core/role-definition";
import type { Faction } from "@/core/types";
import { DirtyFormGuard } from "./dirty-form-guard";

export function RoleEditor({ role }: { readonly role: RoleDefinition }) {
  const lockedContract = isBuiltInRole(role.id);

  return (
    <form action={saveRoleAction} className="space-y-5">
      <DirtyFormGuard />
      <input type="hidden" name="createdAt" value={role.createdAt} />
      <input
        type="hidden"
        name="defaultModelBinding"
        value={modelBindingValue(role.defaultModelBinding)}
      />

      <EditorHeader
        eyebrow="Role editor"
        title={role.name}
        id={role.id}
        enabled={role.enabled}
      />

      <div className="grid gap-3 md:grid-cols-2">
        <TextField label="Id" name="id" defaultValue={role.id} />
        <TextField label="Name" name="name" defaultValue={role.name} />
      </div>

      <label className="flex items-center gap-2 text-sm text-muted">
        <input
          type="checkbox"
          name="enabled"
          defaultChecked={role.enabled}
          className="h-4 w-4 accent-cyan-300"
        />
        Enabled
      </label>

      <div className="grid gap-3 md:grid-cols-3">
        <SelectField
          label="Faction"
          name="faction"
          value={role.faction}
          options={["wolves", "good"] satisfies readonly Faction[]}
          disabled={lockedContract}
        />
        <SelectField
          label="Team"
          name="team"
          value={role.team}
          options={["wolf", "god", "villager"] satisfies readonly RoleTeam[]}
          disabled={lockedContract}
        />
        <SelectField
          label="Mechanic"
          name="mechanicKey"
          value={role.mechanicKey}
          options={
            [
              "wolf_kill",
              "seer_check",
              "witch_medicine",
              "none",
            ] satisfies readonly RoleMechanicKey[]
          }
          disabled={lockedContract}
        />
      </div>

      {lockedContract ? (
        <>
          <input type="hidden" name="faction" value={role.faction} />
          <input type="hidden" name="team" value={role.team} />
          <input type="hidden" name="mechanicKey" value={role.mechanicKey} />
        </>
      ) : null}

      <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_160px]">
        <TextField
          label="Visibility rules"
          name="visibilityRules"
          defaultValue={role.visibilityRules.join(",")}
        />
        <TextField
          label="Night order"
          name="nightOrder"
          defaultValue={role.nightOrder?.toString() ?? ""}
        />
      </div>

      <TextareaField
        label="System prompt"
        name="systemPrompt"
        defaultValue={role.systemPrompt}
        rows={7}
      />
      <TextareaField
        label="Action prompt"
        name="actionPrompt"
        defaultValue={role.actionPrompt ?? ""}
        rows={4}
      />

      <div className="flex justify-end border-t border-border pt-4">
        <Button
          type="submit"
          className="font-semibold"
        >
          Save role
        </Button>
      </div>
    </form>
  );
}

function EditorHeader({
  eyebrow,
  title,
  id,
  enabled,
}: {
  readonly eyebrow: string;
  readonly title: string;
  readonly id: string;
  readonly enabled: boolean;
}) {
  return (
    <div className="flex items-start justify-between gap-3 border-b border-border pb-3">
      <div className="min-w-0">
        <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
          {eyebrow}
        </p>
        <h2 className="mt-1 truncate text-xl font-semibold text-foreground">
          {title}
        </h2>
        <p className="mt-1 truncate text-xs text-subtle">{id}</p>
      </div>
      <span
        className={[
          "rounded-full px-2.5 py-1 text-xs font-medium",
          enabled
            ? "bg-good-badge text-good-badge-foreground"
            : "bg-badge text-badge-foreground",
        ].join(" ")}
      >
        {enabled ? "enabled" : "disabled"}
      </span>
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
    <label className="block text-sm text-muted">
      <span className="text-xs font-medium uppercase tracking-[0.12em] text-subtle">
        {label}
      </span>
      <input
        name={name}
        defaultValue={defaultValue}
        className="mt-1 w-full rounded border border-interactive-border bg-background px-3 py-2 text-sm text-foreground outline-none transition"
      />
    </label>
  );
}

function SelectField<T extends string>({
  label,
  name,
  value,
  options,
  disabled,
}: {
  readonly label: string;
  readonly name: string;
  readonly value: T;
  readonly options: readonly T[];
  readonly disabled?: boolean;
}) {
  return (
    <label className="block text-sm text-muted">
      <span className="text-xs font-medium uppercase tracking-[0.12em] text-subtle">
        {label}
      </span>
      <select
        name={name}
        defaultValue={value}
        disabled={disabled}
        className="mt-1 w-full rounded border border-interactive-border bg-background px-3 py-2 text-sm text-foreground outline-none transition disabled:cursor-not-allowed disabled:text-subtle"
      >
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </label>
  );
}

function TextareaField({
  label,
  name,
  defaultValue,
  rows,
}: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
  readonly rows: number;
}) {
  return (
    <label className="block text-sm text-muted">
      <span className="text-xs font-medium uppercase tracking-[0.12em] text-subtle">
        {label}
      </span>
      <textarea
        name={name}
        defaultValue={defaultValue}
        rows={rows}
        className="mt-1 w-full resize-y rounded border border-interactive-border bg-background px-3 py-2 text-sm leading-6 text-foreground outline-none transition"
      />
    </label>
  );
}

function modelBindingValue(value: ModelBindingSnapshot | null): string {
  return value === null ? "" : JSON.stringify(value);
}

function isBuiltInRole(roleId: string): boolean {
  return ["werewolf", "seer", "witch", "villager"].includes(roleId);
}
