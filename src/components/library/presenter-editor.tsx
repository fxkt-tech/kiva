import { savePresenterAction } from "@/app/library/actions";
import { Button } from "@/components/ui/button";
import {
  PRESENTER_LINE_VARIABLES,
  type PresenterCopyKey,
  type PresenterDefinition,
} from "@/core/presenter-definition";
import {
  resolvePresenterVoiceClips,
  type PresenterVoiceManifest,
} from "@/core/presenter-voice";
import { DirtyFormGuard } from "./dirty-form-guard";
import {
  PresenterAudioPreview,
  type PresenterAudioPreviewOption,
} from "./presenter-audio-preview";

type PresenterEditorProps = {
  readonly presenter: PresenterDefinition;
  readonly voiceManifest?: PresenterVoiceManifest;
};

export function PresenterEditor({ presenter, voiceManifest }: PresenterEditorProps) {
  const groups = groupedCopyKeys();

  return (
    <form action={savePresenterAction} className="space-y-5">
      <DirtyFormGuard />
      <input type="hidden" name="createdAt" value={presenter.createdAt} />
      <input type="hidden" name="id" value={presenter.id} />

      <div className="flex items-start justify-between gap-3 border-b border-border pb-3">
        <div className="flex min-w-0 items-center gap-3">
          <PresenterAvatar presenter={presenter} />
          <div className="min-w-0">
            <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-cyan-300">
              Presenter editor
            </p>
            <h2 className="mt-1 truncate text-xl font-semibold text-foreground">
              {presenter.name}
            </h2>
            <p className="mt-1 truncate text-xs text-subtle">{presenter.id}</p>
          </div>
        </div>
        <span
          className={[
            "rounded-full px-2.5 py-1 text-xs font-medium",
            presenter.enabled
              ? "bg-good-badge text-good-badge-foreground"
              : "bg-badge text-badge-foreground",
          ].join(" ")}
        >
          {presenter.enabled ? "enabled" : "disabled"}
        </span>
      </div>

      <div className="grid gap-3 md:grid-cols-2">
        <TextField label="Stable ID" name="id" defaultValue={presenter.id} disabled />
        <TextField label="Name" name="name" defaultValue={presenter.name} />
      </div>
      <TextField
        label="Avatar"
        name="avatar"
        defaultValue={presenter.avatar ?? ""}
        placeholder="留空时显示名称首字"
      />
      <div className="grid gap-3 md:grid-cols-4">
        <TextField
          label="Edge voice"
          name="voiceProfile.voice"
          defaultValue={presenter.voiceProfile.voice}
        />
        <TextField
          label="Rate"
          name="voiceProfile.rate"
          defaultValue={presenter.voiceProfile.rate}
        />
        <TextField
          label="Pitch"
          name="voiceProfile.pitch"
          defaultValue={presenter.voiceProfile.pitch}
        />
        <TextField
          label="Volume"
          name="voiceProfile.volume"
          defaultValue={presenter.voiceProfile.volume}
        />
      </div>
      <label className="flex items-center gap-2 text-sm text-muted">
        <input
          type="checkbox"
          name="enabled"
          defaultChecked={presenter.enabled}
          className="h-4 w-4 accent-cyan-300"
        />
        Enabled
      </label>

      <section className="space-y-3">
        <div className="flex items-end justify-between gap-3 border-b border-border pb-2">
          <div>
            <h3 className="text-sm font-semibold text-foreground">主持文案</h3>
            <p className="mt-1 text-xs text-subtle">
              模板变量由事件契约固定；配音由预制 manifest 构建命令统一生成。
            </p>
          </div>
          <span className="font-mono text-xs text-subtle">
            {Object.keys(PRESENTER_LINE_VARIABLES).length} lines
          </span>
        </div>

        {groups.map((group, groupIndex) => (
          <details
            key={group.id}
            open={groupIndex === 0}
            className="rounded border border-border bg-surface/35"
          >
            <summary className="cursor-pointer select-none px-3 py-2.5 text-sm font-semibold text-foreground">
              {group.label}
              <span className="ml-2 font-mono text-xs font-normal text-subtle">
                {group.keys.length}
              </span>
            </summary>
            <div className="space-y-3 border-t border-border p-3">
              {group.keys.map((key) => (
                <LineEditor
                  key={key}
                  copyKey={key}
                  line={presenter.lines[key]}
                  previewOptions={previewOptions(presenter.id, voiceManifest, key)}
                />
              ))}
            </div>
          </details>
        ))}
      </section>

      <div className="sticky bottom-0 flex justify-end border-t border-border bg-background/95 py-4 backdrop-blur">
        <Button type="submit" className="font-semibold">
          Save presenter
        </Button>
      </div>
    </form>
  );
}

function PresenterAvatar({ presenter }: PresenterEditorProps) {
  if (presenter.avatar) {
    return (
      <img
        alt=""
        className="h-16 w-16 shrink-0 rounded-full border border-cyan-300/35 object-cover"
        src={presenter.avatar}
      />
    );
  }

  return (
    <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-full border border-cyan-300/35 bg-surface-strong text-2xl font-semibold text-cyan-100">
      {Array.from(presenter.name)[0] ?? "?"}
    </div>
  );
}

function LineEditor({
  copyKey,
  line,
  previewOptions,
}: {
  readonly copyKey: PresenterCopyKey;
  readonly line: PresenterDefinition["lines"][PresenterCopyKey];
  readonly previewOptions: readonly PresenterAudioPreviewOption[];
}) {
  return (
    <article className="rounded border border-border bg-background/65 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <code className="text-xs font-semibold text-cyan-200">{copyKey}</code>
        <div className="flex items-center gap-2">
          <span className="text-xs text-subtle">
            variables: {line.variables.length ? line.variables.join(", ") : "none"}
          </span>
          <PresenterAudioPreview options={previewOptions} />
        </div>
      </div>
      <label className="mt-2 block text-sm text-muted">
        <span className="text-xs font-medium text-subtle">文案模板</span>
        <textarea
          name={`line.${copyKey}.template`}
          defaultValue={line.template}
          rows={2}
          required
          className="mt-1 w-full resize-y rounded border border-interactive-border bg-background px-3 py-2 text-sm leading-6 text-foreground outline-none"
        />
      </label>
    </article>
  );
}

function previewOptions(
  presenterId: string,
  manifest: PresenterVoiceManifest | undefined,
  copyKey: PresenterCopyKey,
): readonly PresenterAudioPreviewOption[] {
  if (!manifest) return [];
  const plan = manifest.plans[copyKey] ?? [];
  const seatVariant = plan.length === 1 && plan[0]?.kind === "seat-variant"
    ? plan[0]
    : null;
  const values = {
    seatNo: "1",
    player: "1号",
    target: "1号",
    voter: "1号",
    players: "1号、2号",
    role: "平民",
  };
  const optionValues = seatVariant
    ? Array.from({ length: 12 }, (_, index) => ({
        label: `${index + 1}号`,
        values: { ...values, [seatVariant.variableName]: seatVariant.variableName === "seatNo" ? String(index + 1) : `${index + 1}号` },
      }))
    : [{ label: "示例", values }];
  return optionValues.map((option) => ({
    label: option.label,
    sources: resolvePresenterVoiceClips(manifest, {
      copyKey,
      text: "",
      values: option.values,
    }).map((clip) =>
      `/api/presenters/${encodeURIComponent(presenterId)}/voice/${encodeURIComponent(clip.file)}`),
  }));
}

function TextField({
  label,
  name,
  defaultValue,
  placeholder,
  disabled = false,
}: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
  readonly placeholder?: string;
  readonly disabled?: boolean;
}) {
  return (
    <label className="block text-sm text-muted">
      <span className="text-xs font-medium uppercase tracking-[0.08em] text-subtle">
        {label}
      </span>
      <input
        name={name}
        defaultValue={defaultValue}
        placeholder={placeholder}
        disabled={disabled}
        className="mt-1 w-full rounded border border-interactive-border bg-background px-3 py-2 text-sm text-foreground outline-none"
      />
    </label>
  );
}

function groupedCopyKeys(): readonly {
  readonly id: string;
  readonly label: string;
  readonly keys: readonly PresenterCopyKey[];
}[] {
  const groups = [
    ["phase", "阶段播报"],
    ["action", "夜间行动"],
    ["announcement", "公布信息"],
    ["prompt", "玩家提示"],
    ["vote", "投票过程"],
    ["resolution", "结算结果"],
    ["game_end", "游戏结束"],
    ["fallback", "兜底文案"],
  ] as const;
  const keys = Object.keys(PRESENTER_LINE_VARIABLES) as PresenterCopyKey[];

  return groups.map(([id, label]) => ({
    id,
    label,
    keys: keys.filter((key) => key.startsWith(`${id}.`)),
  }));
}
