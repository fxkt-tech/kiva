import Link from "next/link";
import { RecordingControls } from "./recording-controls";

type RecordingWorkspaceProps = {
  readonly title: string;
  readonly previewHref: string;
  readonly cleanPreviewHref: string;
};

export function RecordingWorkspace({
  title,
  previewHref,
  cleanPreviewHref,
}: RecordingWorkspaceProps) {
  return (
    <main className="h-screen overflow-hidden bg-black p-4 text-zinc-100">
      <div className="flex h-full min-h-0 flex-col gap-3">
        <header className="flex shrink-0 items-center justify-between gap-3 border-b border-zinc-900 pb-3">
          <div className="min-w-0">
            <div className="text-xs uppercase tracking-[0.18em] text-zinc-600">
              Recording studio
            </div>
            <h1 className="mt-1 truncate text-lg font-semibold text-zinc-100">
              {title}
            </h1>
          </div>
          <Link
            className="shrink-0 border border-zinc-800 px-3 py-2 text-xs text-zinc-300 transition hover:border-zinc-600 hover:text-white"
            href={previewHref}
          >
            Back to preview
          </Link>
        </header>

        <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[minmax(0,1fr)_360px]">
          <section className="flex min-h-0 items-center justify-center">
            <iframe
              className="aspect-video w-full max-w-6xl border border-zinc-900 bg-black"
              src={cleanPreviewHref}
              title="Clean playback preview"
            />
          </section>

          <aside className="flex min-h-0 flex-col gap-4 border border-zinc-900 bg-zinc-950 p-4">
            <div>
              <h2 className="text-sm font-semibold text-zinc-100">Record</h2>
            </div>
            <RecordingControls cleanPreviewHref={cleanPreviewHref} />
          </aside>
        </div>
      </div>
    </main>
  );
}
