import { describe, expect, it } from "vitest";
import {
  clamp01,
  easeInOutCubic,
  easeOutCubic,
  fadeIn,
  shake,
  slideIn,
  stagedReveal,
  zoom,
} from "./animation";

describe("preview animation helpers", () => {
  it("clamps progress values", () => {
    expect(clamp01(-1)).toBe(0);
    expect(clamp01(0.5)).toBe(0.5);
    expect(clamp01(2)).toBe(1);
    expect(clamp01(Number.NaN)).toBe(0);
  });

  it("eases deterministically", () => {
    expect(easeOutCubic(0)).toBe(0);
    expect(easeOutCubic(1)).toBe(1);
    expect(easeInOutCubic(0.5)).toBe(0.5);
    expect(fadeIn(1)).toBe(1);
  });

  it("derives simple motion values from progress", () => {
    expect(slideIn(0, 40)).toBe(40);
    expect(slideIn(1, 40)).toBe(0);
    expect(zoom(1, 0.9, 1)).toBe(1);
    expect(Math.abs(shake(1, 12))).toBeLessThan(0.0001);
  });

  it("reveals staged rows by index", () => {
    expect(stagedReveal(0.1, 0, 3)).toBeGreaterThan(0);
    expect(stagedReveal(0.1, 1, 3)).toBe(0);
    expect(stagedReveal(1, 2, 3)).toBe(1);
  });
});
