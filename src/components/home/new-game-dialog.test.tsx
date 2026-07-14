import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { seedScripts } from "@/seeds/scripts";
import {
  EditableSeatTable,
  NewGameDialog,
  ReadOnlySeatTable,
  RunModePicker,
  ScriptPicker,
} from "./new-game-dialog";
import { roleIdentityColor } from "./role-identity-color";

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

  it("colors role text, selected values, and dropdown options by identity", () => {
    const seats = seedPresets[0]!.seatAssignments!;
    const readOnlyHtml = renderToStaticMarkup(
      React.createElement(ReadOnlySeatTable, {
        seats,
        roles: seedRoles,
        characters: seedCharacters,
      }),
    );
    const editableHtml = renderToStaticMarkup(
      React.createElement(EditableSeatTable, {
        seats,
        roles: seedRoles,
        characters: seedCharacters,
        onUpdateSeat: () => {},
      }),
    );

    for (const role of seedRoles) {
      const color = roleIdentityColor(role.id);
      expect(readOnlyHtml).toContain(`data-role-id="${role.id}"`);
      expect(readOnlyHtml).toContain(`color:${color}`);
      expect(editableHtml).toContain(`data-role-id="${role.id}"`);
      expect(editableHtml).toContain(`color:${color}`);
    }
    expect(editableHtml).toContain('data-selected-role-id="villager"');
    expect(readOnlyHtml).not.toContain("background-color:");
    expect(readOnlyHtml).not.toContain("border-color:");
    expect(roleIdentityColor("werewolf")).toBe(
      "var(--role-werewolf, #b4233e)",
    );
  });

  it("shows all twelve seats as two six-seat overview columns on desktop", () => {
    const seats = seedPresets[0]!.seatAssignments!;
    const readOnlyHtml = renderToStaticMarkup(
      React.createElement(ReadOnlySeatTable, {
        seats,
        roles: seedRoles,
        characters: seedCharacters,
      }),
    );
    const editableHtml = renderToStaticMarkup(
      React.createElement(EditableSeatTable, {
        seats,
        roles: seedRoles,
        characters: seedCharacters,
        onUpdateSeat: () => {},
      }),
    );

    for (const html of [readOnlyHtml, editableHtml]) {
      expect(html).toContain("lg:grid-cols-2");
      expect(html.match(/data-seat-column=/g)).toHaveLength(2);
      expect(html.match(/data-seat-no=/g)).toHaveLength(12);
      expect(html.match(/data-seat-column-size="6"/g)).toHaveLength(2);
    }
  });
});
