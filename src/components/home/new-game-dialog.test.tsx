import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { seedScripts } from "@/seeds/scripts";
import {
  NewGameDialog,
  RunModePicker,
  ScriptPicker,
} from "./new-game-dialog";

describe("NewGameDialog", () => {
  it("renders a dialog trigger without immediately rendering setup forms", () => {
    const html = renderToStaticMarkup(
      React.createElement(NewGameDialog, {
        presets: seedPresets,
        roles: seedRoles,
        characters: seedCharacters,
        scripts: seedScripts,
      }),
    );

    expect(html).toContain("New game");
    expect(html).not.toContain("Create game");
    expect(html).not.toContain('name="seat.1.roleId"');
  });

  it("renders the selected script with narrative context", () => {
    const html = renderToStaticMarkup(
      React.createElement(ScriptPicker, {
        scripts: seedScripts,
        selectedScriptId: seedScripts[0]!.id,
        onSelectScript: () => {},
      }),
    );

    expect(html).toContain("未明档案");
    expect(html).toContain("东方都市异闻");
    expect(html).toContain("已选择");
  });

  it("explains the two run modes before seat setup", () => {
    const html = renderToStaticMarkup(
      React.createElement(RunModePicker, {
        runMode: "game",
        onSelectRunMode: () => {},
      }),
    );

    expect(html).toContain("游戏模式");
    expect(html).toContain("剧本模式");
    expect(html).toContain("直接开局 · 目标约 30 分钟");
    expect(html).toContain("生成剧本 → 审核 → 批准 → 开局");
  });

  it("warns that scripted games stay locked before approval", () => {
    const html = renderToStaticMarkup(
      React.createElement(RunModePicker, {
        runMode: "scripted",
        onSelectRunMode: () => {},
      }),
    );

    expect(html).toContain("剧本批准前不会推进游戏");
  });
});
