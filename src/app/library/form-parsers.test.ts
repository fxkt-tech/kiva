import { describe, expect, it } from "vitest";
import { seedActors } from "@/seeds/actors";
import { seedLineups } from "@/seeds/lineups";
import { actorFromFormData, lineupFromFormData } from "./form-parsers";

describe("Library Studio form parsers", () => {
  it("round-trips the structured Actor form and increments revision", () => {
    const actor = seedActors[1]!;
    const form = new FormData();
    appendActor(form, actor);
    expect(actorFromFormData(form)).toMatchObject({
      id: actor.id,
      identity: actor.identity,
      core: actor.core,
      cognition: actor.cognition,
      revision: actor.revision + 1,
    });
  });

  it("keeps Qin Chuan's stable id/name pair", () => {
    const form = new FormData();
    appendActor(form, seedActors[0]!);
    form.set("identity.name", "别名");
    expect(() => actorFromFormData(form)).toThrow("稳定 Actor");
  });

  it("parses one Actor and Rule Role per fixed seat", () => {
    const lineup = seedLineups[0]!;
    const form = new FormData();
    form.set("id", lineup.id);
    form.set("name", lineup.name);
    form.set("revision", String(lineup.revision));
    form.set("enabled", "on");
    for (const seat of lineup.seats) {
      form.set(`seat.${seat.seatNo}.ruleRoleId`, seat.ruleRoleId);
      form.set(`seat.${seat.seatNo}.actorId`, seat.actorId);
    }
    expect(lineupFromFormData(form)).toEqual({
      ...lineup,
      revision: lineup.revision + 1,
    });
  });
});

function appendActor(form: FormData, actor: (typeof seedActors)[number]) {
  form.set("id", actor.id);
  form.set("revision", String(actor.revision));
  form.set("enabled", "on");
  for (const [key, value] of Object.entries({
    "identity.name": actor.identity.name,
    "identity.portrait": actor.identity.portrait,
    "identity.tags": actor.identity.tags.join(","),
    "identity.visualAnchor": actor.identity.visualAnchor,
    "core.stableCore": actor.core.stableCore,
    "core.drive": actor.core.drive,
    "core.blindSpot": actor.core.blindSpot,
    "core.changeBoundary": actor.core.changeBoundary,
    "cognition.attention": actor.cognition.attention,
    "cognition.evidencePolicy": actor.cognition.evidencePolicy,
    "cognition.decisionPolicy": actor.cognition.decisionPolicy,
    "cognition.correctionTrigger": actor.cognition.correctionTrigger,
    "interaction.tableFunction": actor.interaction.tableFunction,
    "interaction.socialStrategy": actor.interaction.socialStrategy,
    "interaction.pressureResponse": actor.interaction.pressureResponse,
    "interaction.conflictAxes": actor.interaction.conflictAxes.join(","),
    "expression.cadence": actor.expression.cadence,
    "expression.diction": actor.expression.diction,
    "expression.rhetoricalMoves": actor.expression.rhetoricalMoves.join(","),
    "expression.avoid": actor.expression.avoid.join(","),
    "production.modelBinding.provider": actor.production.modelBinding.provider,
    "production.modelBinding.model": actor.production.modelBinding.model,
    "production.modelBinding.fallbackModel": actor.production.modelBinding.fallbackModel ?? "",
    "production.voice.voice": actor.production.voice.voice,
    "production.voice.rate": actor.production.voice.rate,
    "production.voice.pitch": actor.production.voice.pitch,
    "production.voice.volume": actor.production.voice.volume,
  })) form.set(key, value);
}
