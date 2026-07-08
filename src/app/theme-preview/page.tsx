import Link from "next/link";
import type { CSSProperties } from "react";

type ThemeOption = {
  readonly name: string;
  readonly direction: string;
  readonly fit: string;
  readonly variables: CSSProperties;
};

const themeOptions: readonly ThemeOption[] = [
  {
    name: "Alpine Dossier",
    direction: "cool, quiet, operational",
    fit: "Best default if the app should feel like a durable internal tool.",
    variables: {
      "--preview-background": "#f5f8f7",
      "--preview-foreground": "#14201d",
      "--preview-surface": "#ffffff",
      "--preview-surface-muted": "#eaf1ee",
      "--preview-surface-strong": "#d7e2de",
      "--preview-field": "#fbfdfc",
      "--preview-border": "#d7e2de",
      "--preview-border-strong": "#b8c9c3",
      "--preview-muted": "#52645f",
      "--preview-subtle": "#71827d",
      "--preview-control": "#14201d",
      "--preview-control-foreground": "#f8fbfa",
      "--preview-accent": "#16756b",
      "--preview-accent-hover": "#0f5f57",
      "--preview-accent-foreground": "#f8fffd",
    } as CSSProperties,
  },
  {
    name: "Steel Ledger",
    direction: "crisp, analytic, low-noise",
    fit: "Best if lists, forms, and repeated review work are the priority.",
    variables: {
      "--preview-background": "#f4f7fb",
      "--preview-foreground": "#111827",
      "--preview-surface": "#ffffff",
      "--preview-surface-muted": "#e9eff7",
      "--preview-surface-strong": "#d6e1ee",
      "--preview-field": "#fbfdff",
      "--preview-border": "#d6e1ee",
      "--preview-border-strong": "#b8c8db",
      "--preview-muted": "#506174",
      "--preview-subtle": "#748398",
      "--preview-control": "#111827",
      "--preview-control-foreground": "#f8fbff",
      "--preview-accent": "#0f6c81",
      "--preview-accent-hover": "#09586a",
      "--preview-accent-foreground": "#f7fcff",
    } as CSSProperties,
  },
  {
    name: "Porcelain Ink",
    direction: "sharp, editorial, high contrast",
    fit: "Best if light mode should feel cleaner and less earthy.",
    variables: {
      "--preview-background": "#f7f7fb",
      "--preview-foreground": "#171621",
      "--preview-surface": "#ffffff",
      "--preview-surface-muted": "#ececf4",
      "--preview-surface-strong": "#dadae8",
      "--preview-field": "#fdfdff",
      "--preview-border": "#dadae8",
      "--preview-border-strong": "#bebed0",
      "--preview-muted": "#56546a",
      "--preview-subtle": "#7c7a90",
      "--preview-control": "#171621",
      "--preview-control-foreground": "#f9f9ff",
      "--preview-accent": "#315c9a",
      "--preview-accent-hover": "#244a82",
      "--preview-accent-foreground": "#f8fbff",
    } as CSSProperties,
  },
  {
    name: "Evidence Rose",
    direction: "warm accent, neutral base",
    fit: "Best if the game needs more character without making the UI decorative.",
    variables: {
      "--preview-background": "#f7f6f4",
      "--preview-foreground": "#211b1d",
      "--preview-surface": "#fffefd",
      "--preview-surface-muted": "#efebe8",
      "--preview-surface-strong": "#dfd7d2",
      "--preview-field": "#fffdfb",
      "--preview-border": "#dfd7d2",
      "--preview-border-strong": "#c7b9b1",
      "--preview-muted": "#665b5d",
      "--preview-subtle": "#887c7b",
      "--preview-control": "#211b1d",
      "--preview-control-foreground": "#fffaf8",
      "--preview-accent": "#9f1239",
      "--preview-accent-hover": "#86112f",
      "--preview-accent-foreground": "#fff7f8",
    } as CSSProperties,
  },
];

export default function ThemePreviewPage() {
  return (
    <main className="min-h-screen bg-background px-4 py-6 text-foreground sm:px-6 lg:px-8">
      <div className="mx-auto flex max-w-7xl flex-col gap-6">
        <header className="flex flex-col gap-3 border-b border-border pb-5 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-xs font-medium uppercase tracking-[0.18em] text-subtle">
              Kiva theme lab
            </p>
            <h1 className="mt-2 text-2xl font-semibold text-foreground">
              Light theme candidates
            </h1>
          </div>
          <Link
            className="w-fit rounded-md border border-interactive-border bg-surface/70 px-4 py-2 text-sm font-medium text-foreground transition hover:border-interactive-border-hover"
            href="/"
          >
            Back
          </Link>
        </header>

        <div className="grid gap-5 xl:grid-cols-2">
          {themeOptions.map((theme) => (
            <ThemeCard key={theme.name} theme={theme} />
          ))}
        </div>
      </div>
    </main>
  );
}

function ThemeCard({ theme }: { readonly theme: ThemeOption }) {
  return (
    <section
      className="overflow-hidden rounded-lg border border-[var(--preview-border)] bg-[var(--preview-background)] text-[var(--preview-foreground)] shadow-sm"
      style={theme.variables}
    >
      <div className="border-b border-[var(--preview-border)] px-4 py-3">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold">{theme.name}</h2>
            <p className="mt-1 text-xs font-medium uppercase tracking-[0.14em] text-[var(--preview-subtle)]">
              {theme.direction}
            </p>
          </div>
          <Swatches theme={theme} />
        </div>
        <p className="mt-3 text-sm text-[var(--preview-muted)]">{theme.fit}</p>
      </div>

      <div className="grid gap-4 p-4 lg:grid-cols-[1fr_0.95fr]">
        <div className="space-y-4">
          <HomeSample />
          <EditorSample />
        </div>
        <div className="space-y-4">
          <LibrarySample />
          <PreviewSample />
        </div>
      </div>
    </section>
  );
}

function Swatches({ theme }: { readonly theme: ThemeOption }) {
  const variables = theme.variables as Record<string, string>;
  const swatches = [
    "--preview-background",
    "--preview-surface",
    "--preview-surface-strong",
    "--preview-accent",
    "--preview-foreground",
  ];

  return (
    <div className="flex gap-1.5" aria-label={`${theme.name} swatches`}>
      {swatches.map((swatch) => (
        <span
          className="h-5 w-5 rounded-sm border border-black/10"
          key={swatch}
          style={{ background: variables[swatch] }}
          title={swatch}
        />
      ))}
    </div>
  );
}

function HomeSample() {
  return (
    <div className="overflow-hidden rounded-md border border-[var(--preview-border)] bg-[var(--preview-surface)]">
      <div className="flex items-center justify-between border-b border-[var(--preview-border)] px-3 py-2">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--preview-subtle)]">
            Home
          </div>
          <div className="mt-0.5 text-sm font-semibold">Local games</div>
        </div>
        <button
          className="rounded-md border border-[var(--preview-border-strong)] bg-[var(--preview-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--preview-foreground)] shadow-sm transition hover:bg-[var(--preview-surface-muted)]"
          type="button"
        >
          New game
        </button>
      </div>
      <div className="divide-y divide-[var(--preview-border)] text-sm">
        {["四方诛杀", "月下档案", "夜谈局"].map((title, index) => (
          <div className="grid grid-cols-[1fr_auto_auto] gap-3 px-3 py-2" key={title}>
            <span className="font-medium">{title}</span>
            <span className="text-[var(--preview-muted)]">{index + 8} events</span>
            <span className="text-[var(--preview-subtle)]">12:{index}4</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function EditorSample() {
  return (
    <div className="grid grid-cols-[0.85fr_1fr] gap-3">
      <section className="rounded-md border border-[var(--preview-border)] bg-[var(--preview-surface)]">
        <div className="border-b border-[var(--preview-border)] px-3 py-2 text-sm font-semibold">
          Timeline
        </div>
        <div className="space-y-2 p-3">
          <div className="rounded bg-[var(--preview-background)] px-3 py-2">
            <div className="text-xs text-[var(--preview-subtle)]">01 / night</div>
            <div className="mt-1 text-sm font-medium">第 1 夜开始</div>
          </div>
          <div className="rounded bg-[var(--preview-surface-muted)] px-3 py-2 opacity-75">
            <div className="text-xs text-[var(--preview-subtle)]">02 / speech</div>
            <div className="mt-1 text-sm">玩家发言</div>
          </div>
        </div>
      </section>
      <section className="rounded-md border border-[var(--preview-border)] bg-[var(--preview-surface)]">
        <div className="border-b border-[var(--preview-border)] px-3 py-2 text-sm font-semibold">
          Draft
        </div>
        <div className="p-3">
          <div className="text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--preview-subtle)]">
            Action
          </div>
          <div className="mt-1 text-sm font-medium">确认夜晚开始</div>
          <p className="mt-2 text-sm leading-5 text-[var(--preview-muted)]">
            生成下一段公开叙事，并保留导演可编辑的草稿。
          </p>
        </div>
      </section>
    </div>
  );
}

function LibrarySample() {
  return (
    <div className="grid grid-cols-[110px_1fr] overflow-hidden rounded-md border border-[var(--preview-border)] bg-[var(--preview-surface)]">
      <nav className="border-r border-[var(--preview-border)] bg-[var(--preview-background)] p-2">
        {["Roles", "Characters", "Presets"].map((item, index) => (
          <div
            className={[
              "rounded px-2 py-1.5 text-xs",
              index === 1
                ? "bg-[var(--preview-surface-strong)] font-semibold"
                : "text-[var(--preview-muted)]",
            ].join(" ")}
            key={item}
          >
            {item}
          </div>
        ))}
      </nav>
      <div className="p-3">
        <div className="flex items-center justify-between border-b border-[var(--preview-border)] pb-2">
          <div>
            <div className="text-sm font-semibold">Kiva Library</div>
            <div className="text-xs text-[var(--preview-subtle)]">Director configuration desk</div>
          </div>
          <span className="rounded bg-[var(--preview-accent)] px-2.5 py-1 text-xs font-semibold text-[var(--preview-accent-foreground)]">
            Create
          </span>
        </div>
        <div className="mt-3 grid gap-2">
          <input
            className="rounded border border-[var(--preview-border-strong)] bg-[var(--preview-field)] px-2 py-1.5 text-sm"
            defaultValue="预言家"
            readOnly
          />
          <div className="rounded border border-[var(--preview-border)] bg-[var(--preview-surface-muted)] px-2 py-1.5 text-xs text-[var(--preview-muted)]">
            references: preset.default
          </div>
        </div>
      </div>
    </div>
  );
}

function PreviewSample() {
  return (
    <div className="rounded-md border border-[var(--preview-border)] bg-[var(--preview-background)] p-3">
      <div className="aspect-video rounded bg-black">
        <div className="flex h-full items-center justify-center text-xs font-semibold uppercase tracking-[0.18em] text-white/55">
          Canvas remains black
        </div>
      </div>
      <div className="mt-3 border border-[var(--preview-border)] bg-[var(--preview-surface)] px-3 py-2">
        <div className="flex justify-between text-xs text-[var(--preview-subtle)]">
          <span>1 / 35</span>
          <span>0:00 / 2:30</span>
        </div>
        <div className="mt-2 h-1.5 rounded-full bg-[var(--preview-surface-strong)]">
          <div className="h-1.5 w-1/5 rounded-full bg-[var(--preview-accent)]" />
        </div>
      </div>
    </div>
  );
}
