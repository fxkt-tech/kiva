import type { Options } from "html-to-image/lib/types";

type CopyCurrentFrameDependencies = {
  readonly createClipboardItem: (
    items: Record<string, Blob>,
  ) => ClipboardItem;
  readonly toBlob: (
    element: HTMLElement,
    options: Options,
  ) => Promise<Blob | null>;
  readonly write: (items: readonly ClipboardItem[]) => Promise<void>;
};

export async function copyCurrentFrame(
  element: HTMLElement,
  dependencies: CopyCurrentFrameDependencies,
): Promise<void> {
  const blob = await dependencies.toBlob(element, {
    backgroundColor: "#000000",
    cacheBust: true,
    canvasHeight: 1080,
    canvasWidth: 1920,
    pixelRatio: 1,
  });
  if (!blob) {
    throw new Error("无法生成当前帧图片");
  }

  await dependencies.write([
    dependencies.createClipboardItem({ "image/png": blob }),
  ]);
}
