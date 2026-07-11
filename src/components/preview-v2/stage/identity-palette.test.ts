import { describe, expect, it } from "vitest";
import {
  presenterIdentityTone,
  roleIdentityTone,
} from "./identity-palette";

describe("preview v2 identity palette", () => {
  it.each([
    ["狼人", "#FF767B"],
    ["平民", "#D0D5D1"],
    ["预言家", "#76E2B6"],
    ["女巫", "#CBA4F4"],
    ["守卫", "#80C8FB"],
    ["猎人", "#E5BD6E"],
  ])("maps %s to its cinematic identity color", (roleName, foreground) => {
    expect(roleIdentityTone(roleName).foreground).toBe(foreground);
  });

  it("supports descriptive role names and a neutral unknown-role fallback", () => {
    expect(roleIdentityTone("白狼王", "night").foreground).toBe("#FF767B");
    expect(roleIdentityTone("村民", "night").foreground).toBe("#D0D5D1");
    expect(roleIdentityTone("未知身份", "night").foreground).toBe("#D0D5D1");
  });

  it("adapts identity and presenter colors for daylight", () => {
    expect(roleIdentityTone("狼人", "day").foreground).toBe("#FF696E");
    expect(roleIdentityTone("守卫", "day").foreground).toBe("#66C1FA");
    expect(presenterIdentityTone("night").foreground).toBe("#F0D084");
    expect(presenterIdentityTone("day").foreground).toBe("#F4C45C");
  });
});
