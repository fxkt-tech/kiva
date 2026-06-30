import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { ModelBindingSnapshot } from "@/core/player";
import type { RoleDefinition } from "@/core/role-definition";
import { seedRoles } from "@/seeds/roles";
import { RoleEditor } from "./role-editor";

const werewolfRole = seedRoles.find((role) => role.id === "werewolf")!;

const modelBinding = {
  provider: "mock",
  model: "role-model",
  temperature: 0.4,
  maxTokens: 900,
  responseFormat: "json",
} satisfies ModelBindingSnapshot;

describe("RoleEditor", () => {
  it("renders role mechanics and prompts", () => {
    const html = renderToStaticMarkup(
      React.createElement(RoleEditor, { role: werewolfRole }),
    );

    expect(html).toContain('name="id"');
    expect(html).toContain('name="name"');
    expect(html).toContain('name="mechanicKey"');
    expect(html).toContain('name="systemPrompt"');
    expect(html).toContain('name="actionPrompt"');
    expect(html).toContain("Save role");
  });

  it("locks built-in role contracts while preserving hidden values", () => {
    const html = renderToStaticMarkup(
      React.createElement(RoleEditor, { role: werewolfRole }),
    );

    expect(html).toContain('name="faction"');
    expect(html).toContain('name="team"');
    expect(html).toContain('name="mechanicKey"');
    expect(html).toContain('disabled=""');
    expect(html).toMatch(
      /<input type="hidden" name="faction" value="wolves"\/?>/,
    );
    expect(html).toMatch(/<input type="hidden" name="team" value="wolf"\/?>/);
    expect(html).toMatch(
      /<input type="hidden" name="mechanicKey" value="wolf_kill"\/?>/,
    );
  });

  it("preserves default model binding in hidden JSON", () => {
    const role: RoleDefinition = {
      ...werewolfRole,
      id: "custom_wolf",
      defaultModelBinding: modelBinding,
    };

    const html = renderToStaticMarkup(
      React.createElement(RoleEditor, { role }),
    );

    expect(html).toContain('name="defaultModelBinding"');
    expect(decodeHtml(html)).toContain(JSON.stringify(modelBinding));
  });
});

function decodeHtml(value: string): string {
  return value.replaceAll("&quot;", "\"");
}
