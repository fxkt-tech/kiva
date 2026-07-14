import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RULE_ROLES } from "@/core/rule-role";
import { seedActors } from "@/seeds/actors";
import { seedLineups } from "@/seeds/lineups";
import { seedScripts } from "@/seeds/scripts";
import { ReadOnlySeatTable } from "./new-game-dialog";

describe("NewGameDialog lineup view", () => {
  it("renders the actual Actor and Rule Role pairing", () => {
    const html = renderToStaticMarkup(
      React.createElement(ReadOnlySeatTable, {
        seats: seedLineups[0]!.seats,
        ruleRoles: RULE_ROLES,
        actors: seedActors,
      }),
    );
    expect(html).toContain("Seat 1");
    expect(html).toContain(seedActors.find((actor) => actor.id === seedLineups[0]!.seats[0]!.actorId)!.identity.name);
    expect(html).toContain("狼人");
  });
});
