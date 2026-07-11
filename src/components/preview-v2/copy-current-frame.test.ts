import { describe, expect, it, vi } from "vitest";
import { copyCurrentFrame } from "./copy-current-frame";

describe("copyCurrentFrame", () => {
  it("rasterizes the player at the native video size and writes a PNG", async () => {
    const element = {} as HTMLElement;
    const blob = new Blob(["png"], { type: "image/png" });
    const toBlob = vi.fn(async () => blob);
    const clipboardItem = { type: "clipboard-item" } as unknown as ClipboardItem;
    const createClipboardItem = vi.fn(() => clipboardItem);
    const write = vi.fn(async () => undefined);

    await copyCurrentFrame(element, {
      createClipboardItem,
      toBlob,
      write,
    });

    expect(toBlob).toHaveBeenCalledWith(element, {
      backgroundColor: "#000000",
      cacheBust: true,
      canvasHeight: 1080,
      canvasWidth: 1920,
      pixelRatio: 1,
    });
    expect(createClipboardItem).toHaveBeenCalledWith({ "image/png": blob });
    expect(write).toHaveBeenCalledWith([clipboardItem]);
  });

  it("fails when the current frame cannot be rasterized", async () => {
    await expect(
      copyCurrentFrame({} as HTMLElement, {
        createClipboardItem: vi.fn(),
        toBlob: vi.fn(async () => null),
        write: vi.fn(),
      }),
    ).rejects.toThrow("无法生成当前帧图片");
  });
});
