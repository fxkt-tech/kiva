import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RULE_ROLES } from "@/core/rule-role";
import { seedActors } from "@/seeds/actors";
import { seedLineups } from "@/seeds/lineups";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";
import { LibraryWorkspace } from "./library-workspace";

const catalog = {
  ruleRoles: RULE_ROLES,
  actors: seedActors,
  lineups: seedLineups,
  presenters: seedPresenters,
  scripts: seedScripts,
};

describe("LibraryWorkspace v2", () => {
  it("shows structured Actor cards instead of free-form role prompts", () => {
    const html = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "actors",
        selectedId: "qin_chuan",
        catalog,
      }),
    );
    expect(html).toContain("Actor v2");
    expect(html).toContain("Actor cards");
    expect(html).toContain("runtime");
    expect(html).toContain("author");
    expect(html).toContain("poolDiagnostics");
    expect(html).toContain("poolMatrix");
    expect(html).toContain("distinctVoiceCount");
    expect(html).not.toContain("systemPrompt");
    expect(html).toContain("id=__new_actor__");
  });

  it("renders an unsaved exact-v2 form for a new Actor", () => {
    const html = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "actors",
        selectedId: "__new_actor__",
        catalog,
      }),
    );
    expect(html).toContain("Actor v2 · new");
    expect(html).toContain("创建 Actor");
    expect(html).toContain("New Actor contract");
    expect(html).toContain('name="id"');
  });

  it("renders Rule Roles as read-only engine contracts", () => {
    const html = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "rules",
        selectedId: "werewolf",
        catalog,
      }),
    );
    expect(html).toContain("Read-only engine contract");
    expect(html).toContain("wolf_kill");
    expect(html).not.toContain("Save Rule");
  });
});
