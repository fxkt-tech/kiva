import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, describe, expect, it } from "vitest";
import { ThemeProvider } from "./theme-provider";

describe("ThemeProvider", () => {
  afterEach(() => {
    Reflect.deleteProperty(globalThis, "window");
  });

  it("does not render next-themes bootstrap scripts during client renders", () => {
    Object.defineProperty(globalThis, "window", {
      configurable: true,
      value: {},
    });

    const html = renderToStaticMarkup(
      <ThemeProvider>
        <span>content</span>
      </ThemeProvider>,
    );

    expect(html).toContain("content");
    expect(html).not.toContain("<script");
  });
});
