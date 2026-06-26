import type { PlaybackItem } from "@/core/playback";

type PlaybackStageProps = {
  readonly items: readonly PlaybackItem[];
};

export function PlaybackStage({ items }: PlaybackStageProps) {
  const current = items.at(-1);

  return (
    <main className="grid min-h-screen place-items-center bg-black p-4 text-white">
      <section className="aspect-video w-full max-w-6xl overflow-hidden bg-zinc-950 shadow-2xl shadow-black">
        <div className="flex h-full flex-col justify-center px-[7%] py-[6%]">
          {current ? (
            <article className="max-w-4xl">
              <div className="mb-5 flex flex-wrap items-center gap-3 text-sm uppercase tracking-[0.2em] text-zinc-500">
                <span>#{current.index}</span>
                <span>{current.phase}</span>
              </div>
              <h1 className="break-words text-4xl font-semibold leading-tight text-zinc-50 sm:text-5xl lg:text-6xl">
                {current.title}
              </h1>
              {current.text ? (
                <p className="mt-6 whitespace-pre-wrap break-words text-xl leading-8 text-zinc-300 sm:text-2xl sm:leading-10">
                  {current.text}
                </p>
              ) : null}
            </article>
          ) : (
            <div className="text-center">
              <div className="text-sm uppercase tracking-[0.2em] text-zinc-600">
                Playback
              </div>
              <p className="mt-4 text-3xl font-semibold text-zinc-300">
                Waiting for public event
              </p>
            </div>
          )}
        </div>
      </section>
    </main>
  );
}
