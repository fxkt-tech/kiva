import { savePresenterAction } from "@/app/library/actions";
import { Button } from "@/components/ui/button";
import {
  PRESENTER_LINE_VARIABLES,
  type PresenterCopyKey,
  type PresenterDefinition,
  type PresenterSeatLine,
  type PresenterStandardLine,
  type PresenterVoice,
} from "@/core/presenter-definition";
import { DirtyFormGuard } from "./dirty-form-guard";

type PresenterEditorProps = {
  readonly presenter: PresenterDefinition;
};

export function PresenterEditor({ presenter }: PresenterEditorProps) {
  const groups = groupedCopyKeys();

  return (
    <form action={savePresenterAction} className="space-y-5">
      <DirtyFormGuard />
      <input type="hidden" name="createdAt" value={presenter.createdAt} />

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
        <TextField label="Id" name="id" defaultValue={presenter.id} />
        <TextField label="Name" name="name" defaultValue={presenter.name} />
      </div>
      <TextField
        label="Avatar"
        name="avatar"
        defaultValue={presenter.avatar ?? ""}
        placeholder="留空时显示名称首字"
      />
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
            <h3 className="text-sm font-semibold text-foreground">主持文案与录音</h3>
            <p className="mt-1 text-xs text-subtle">
              模板变量由事件契约固定；录音留空即静音，pending 不会进入播放。
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
}: {
  readonly copyKey: PresenterCopyKey;
  readonly line: PresenterDefinition["lines"][PresenterCopyKey];
}) {
  return (
    <article className="rounded border border-border bg-background/65 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <code className="text-xs font-semibold text-cyan-200">{copyKey}</code>
        <span className="text-xs text-subtle">
          variables: {line.variables.length ? line.variables.join(", ") : "none"}
        </span>
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
      {"voiceBySeat" in line ? (
        <SeatVoiceEditor copyKey={copyKey} line={line as PresenterSeatLine} />
      ) : (
        <VoiceEditor
          prefix={`line.${copyKey}.voice`}
          voice={(line as PresenterStandardLine).voice}
        />
      )}
    </article>
  );
}

function SeatVoiceEditor({
  copyKey,
  line,
}: {
  readonly copyKey: PresenterCopyKey;
  readonly line: PresenterSeatLine;
}) {
  return (
    <div className="mt-3 overflow-hidden rounded border border-border">
      <div className="grid grid-cols-[48px_minmax(0,1fr)_110px] gap-2 border-b border-border bg-surface-muted/60 px-2 py-1.5 text-[11px] font-semibold text-subtle">
        <span>座位</span>
        <span>录音文件</span>
        <span>状态</span>
      </div>
      {Array.from({ length: 12 }, (_, index) => {
        const seatNo = index + 1;
        const voice = line.voiceBySeat[String(seatNo)] ?? null;
        return (
          <div
            key={seatNo}
            className="grid grid-cols-[48px_minmax(0,1fr)_110px] gap-2 border-b border-border px-2 py-1.5 last:border-b-0"
          >
            <span className="self-center font-mono text-xs text-subtle">{seatNo}</span>
            <input
              name={`line.${copyKey}.voice.${seatNo}.file`}
              defaultValue={voice?.file ?? ""}
              placeholder="留空"
              className="min-w-0 rounded border border-interactive-border bg-background px-2 py-1 text-xs text-foreground outline-none"
            />
            <VoiceStatusSelect
              name={`line.${copyKey}.voice.${seatNo}.status`}
              status={voice?.status}
            />
          </div>
        );
      })}
    </div>
  );
}

function VoiceEditor({
  prefix,
  voice,
}: {
  readonly prefix: string;
  readonly voice: PresenterVoice | null;
}) {
  return (
    <div className="mt-3 grid gap-2 md:grid-cols-[minmax(0,1fr)_140px]">
      <TextField
        label="录音文件"
        name={`${prefix}.file`}
        defaultValue={voice?.file ?? ""}
        placeholder="留空即静音，例如 phase_night.mp3"
      />
      <label className="block text-sm text-muted">
        <span className="text-xs font-medium text-subtle">录音状态</span>
        <VoiceStatusSelect name={`${prefix}.status`} status={voice?.status} />
      </label>
    </div>
  );
}

function VoiceStatusSelect({
  name,
  status,
}: {
  readonly name: string;
  readonly status: PresenterVoice["status"] | undefined;
}) {
  return (
    <select
      name={name}
      defaultValue={status ?? "pending"}
      className="mt-1 w-full rounded border border-interactive-border bg-background px-2 py-1 text-xs text-foreground outline-none"
    >
      <option value="pending">pending</option>
      <option value="ready">ready</option>
    </select>
  );
}

function TextField({
  label,
  name,
  defaultValue,
  placeholder,
}: {
  readonly label: string;
  readonly name: string;
  readonly defaultValue: string;
  readonly placeholder?: string;
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
