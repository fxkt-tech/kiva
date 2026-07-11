# Design: Selectable KivDB Presenter Definitions

## 1. Domain Model

A **Presenter Definition** is a reusable KivDB library entity. It owns one host
identity and one complete, independent script/audio catalog. It is not a Player
Character: it has no seat, role, faction, private knowledge, action, vote, or
win-condition participation.

A **Game Presenter** is the Presenter Definition selected when a Game is
created. The Game stores a complete snapshot. Later library edits, disabling,
or deletion never rewrite an existing Game Presenter.

Every Game has exactly one Game Presenter. Presets do not choose one; the user
makes a per-game selection in the new-game flow.

## 2. KivDB Collection

Create `kivdb/presenters.json` as an array, consistent with `roles.json`,
`characters.json`, and `presets.json`:

```json
[
  {
    "id": "night_watch",
    "name": "守夜人",
    "avatar": null,
    "lines": {
      "phase.night": {
        "template": "天黑请闭眼。所有玩家保持安静，夜间行动开始。",
        "variables": [],
        "voice": {
          "file": "phase_night_start.mp3",
          "status": "pending"
        }
      },
      "prompt.speech": {
        "template": "请{seatNo}号玩家开始发言。",
        "variables": ["seatNo"],
        "voiceBySeat": {
          "1": {
            "file": "player_1_speech_prompt.mp3",
            "status": "ready"
          },
          "7": null
        }
      }
    },
    "enabled": true,
    "createdAt": "2026-07-11T00:00:00.000Z",
    "updatedAt": "2026-07-11T00:00:00.000Z"
  }
]
```

The initial collection contains:

- `night_watch` / `守夜人`: the reviewed atmospheric copy;
- `judge` / `法官`: a complete concise, directive script using traditional
  in-person Werewolf moderator phrasing.

Both avatars start as null. The stage uses the first glyph of the selected name
as fallback (`守` / `法`). No image generation is part of this delivery.

## 3. Definition and Snapshot Contracts

Add `src/core/presenter-definition.ts`:

```ts
type PresenterVoice = {
  readonly file: string;
  readonly status: "ready" | "pending";
};

type PresenterLine = {
  readonly template: string;
  readonly variables: readonly string[];
  readonly voice?: PresenterVoice;
  readonly voiceBySeat?: Readonly<Record<string, PresenterVoice | null>>;
};

type PresenterDefinition = {
  readonly id: string;
  readonly name: string;
  readonly avatar: string | null;
  readonly lines: PresenterLineCatalog;
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

type GamePresenterSnapshot = {
  readonly presenterSourceId: string;
  readonly name: string;
  readonly avatar: string | null;
  readonly lines: PresenterLineCatalog;
};

function validatePresenterDefinitions(value: unknown): readonly PresenterDefinition[];
function createGamePresenterSnapshot(
  definition: PresenterDefinition,
): GamePresenterSnapshot;
```

The snapshot omits library lifecycle fields but copies identity, avatar, every
template, voice filename, and ready/pending state.

## 4. Semantic Line Contract

Each definition must contain the same exhaustive semantic keys:

- all eight phases;
- role assignment and every night action/result variant;
- night/dawn death and safe-night variants;
- vote cast/abstain, exile, tie, and no-exile variants;
- all four game-end reasons;
- speech, last-word, and PK prompts;
- announcement, empty-speech, vote, and resolution fallbacks.

The code owns the required key/variable contract. KivDB owns the actual words
and voice mappings. There is no global script and no inheritance between
presenters.

Validation requires:

- an array with unique, trimmed stable IDs;
- non-blank names and valid null/non-blank avatar paths;
- enabled boolean and ISO timestamps;
- every required semantic key, with unknown typo keys rejected;
- exactly the permitted placeholder list for each key;
- every `{placeholder}` declared and used, with no unsupported braces;
- `ready`/`pending` voice status;
- safe `.mp3` basenames matching `^[a-zA-Z0-9_-]+\.mp3$`;
- prompt seat maps containing 1–12, each mapped to a valid voice or null.

Templates perform literal named-placeholder replacement only. Values come from
typed event/player data. KivDB cannot execute expressions, property paths,
HTML, Markdown, or JavaScript.

All rewritten `守夜人` narration begins `pending`. Preserved 1–6 player prompt
recordings may remain `ready`; missing 7–12 recordings are null. `法官` starts
without ready audio unless a recording is known to match its exact text.

## 5. Library and Game Creation

Extend `LibraryRecord` and `LibraryRepository` with presenter definitions and
`getPresenters()` / `savePresenters()`. `saveAll()` atomically includes
`presenters.json`. Library diagnostics validate presenters.

The Library exposes a `presenters` tab beside roles, characters, and presets.
Its editor submits identity, avatar, enabled state, all semantic templates,
standard voice mappings, and the three 1–12 seat voice maps. Semantic keys,
placeholder declarations, and seat-map shape are generated from
`PRESENTER_LINE_VARIABLES`; they are displayed but cannot be redefined by the
form. The server parser rebuilds the complete catalog and `saveAll()` applies
the authoritative collection validator before writing. Blank voice filenames
become null mappings; non-blank mappings require `ready` or `pending`.

The home page loads enabled presenters and passes them to `NewGameDialog`. A
single presenter selector sits above the preset/random mode content, so both
creation paths submit `presenterId`.

Server actions reject blank, unknown, or disabled IDs. `createGameFromPreset()`
receives the selected definition and stores `createGamePresenterSnapshot()` on
the new `Game.presenter` field.

`GamePreset` remains unchanged: selecting a presenter is a property of this
Game creation, not of the reusable seating/rules preset.

The current `kivdb/games/*.json` records are edited directly to include the
`night_watch` snapshot. There is deliberately no default presenter, missing-
field normalizer, lazy migration, or legacy fallback. Non-migrated game data is
invalid.

## 6. Deep Resolution Module

Add `src/core/presenter.ts` with the playback-compilation seam:

```ts
type PresenterCue = {
  readonly copyKey: PresenterCopyKey;
  readonly text: string;
  readonly voiceFile: string | null;
};

type PresenterResolution = {
  readonly presenterName: string;
  readonly presenterAvatar: string | null;
  readonly transcriptSpeaker: "presenter" | "player";
  readonly transcriptText: string;
  readonly cue: PresenterCue;
};

function resolvePresenter(
  presenter: GamePresenterSnapshot,
  event: GameEvent | DraftEvent,
  players: readonly PlayerSnapshot[],
): PresenterResolution;

function declaredPresenterVoiceFiles(
  definitions: readonly PresenterDefinition[],
): ReadonlySet<string>;
```

Event-to-key selection, variant choice, safe player labels, template rendering,
prompt selection, and ready/pending behavior remain behind this interface. This
is an in-process deep module; there is no hypothetical filesystem port.

## 7. Playback Snapshot

`compilePublicPlayback()` receives `presenter: GamePresenterSnapshot` in its
options and extends every `PlaybackItem` with:

```ts
type PlaybackItem = {
  // existing fields remain
  readonly presenterName: string;
  readonly presenterAvatar: string | null;
  readonly transcriptSpeaker: "presenter" | "player";
  readonly presenterCue: {
    readonly copyKey: string;
    readonly text: string;
    readonly voiceFile: string | null;
  };
};
```

For non-speech events, `text` is the selected presenter's transcript and the
speaker is `presenter`. For non-blank player speech/last words, `text` stays
player-authored, the speaker is `player`, and `presenterCue` contains the
selected presenter's player-addressed prompt. Empty speech uses that
presenter's semantic fallback and presenter attribution.

`event-presenter.ts` continues to own neutral editor/event-log titles and
details. Playback no longer treats its descriptive text as presenter dialogue.

## 8. Preview, Audio, and Export Flow

```text
kivdb/presenters.json --new game selection--> Game.presenter snapshot
                                                   |
                                                   v
                                      compilePublicPlayback(events)
                                         /                    \
                                        v                      v
                          transcript text/speaker      presenterCue.voiceFile
                                        |                      |
                                        v                      v
                              shared HTML stage       preview/export audio cue
```

`stage-copy.ts` removes the global `PRESENTER_NAME`, phase prose, and fallback
prose. It uses the resolved snapshot name/avatar/speaker/text. The composition
asset collector includes the selected presenter avatar when non-null so export
copies it with player avatars.

`systemVoiceSourceForScene()` becomes a path adapter over
`presenterCue.voiceFile`. It performs no title or Chinese-text matching.
`pending` and null mappings yield no cue.

The system-voice route loads `presenters.json` and permits only safe basenames
declared by at least one presenter definition, removing its source-code
allowlist. Both pending and ready declared files may be fetched directly; only
ready mappings are selected by playback.

Export creation uses the Game Presenter snapshot already stored in the game.
It copies resolved ready audio and avatar assets into the immutable job
snapshot. Later library edits cannot change queued or completed exports.

The shared HTML stage owns the approved C “smoke glass” dual palette in
`stage-palette.ts`. Night uses translucent cool-black surfaces with warm moon
gold; day uses translucent warm charcoal with brighter amber. Identity tones
also have explicit day/night values in `identity-palette.ts`. Presenter and
player speaker names use the phase's shared warm white. Low-opacity same-hue
glow supports phone-scale legibility. Dead players recede across the full card
hierarchy while preserving enough role hue to keep identity recognizable.

Seat tracks remain full height and use space-between to distribute six fixed
compact rows from the top edge to the bottom edge. Cards retain the outer number / inner avatar mirror, while the middle name and
role block is centered and enlarged. Dead cards use a darker surface, weak
border/accent, strongly muted avatar and number, and reduced role opacity. The
active speaker uses a brighter border, wider inner-edge bar, and a directional
gold field that mirrors between left and right without covering readable text.

The transcript band uses a compact hierarchy: identity-colored avatar frame,
one speaker row containing the name plus the identity value directly, and
narration below. It does not render an `身份` prefix. Player-owned speech projects
the active player's role; presenter-owned narration projects only gold
`主理人`. The narration divider is anchored directly under the speaker row rather
than vertically centering with the text, so short presenter copy and longer
player speech keep identical spacing. For presenter narration only, a safe text projection matches player
full seat/name labels, names, or seat labels and colors each mention by that
player's role. Player-authored speech is free text and remains entirely neutral;
it is never scanned for mentions or role words.

The center visual-stage row is intentionally an unstyled empty grid slot. It
must not render a card surface, border, shadow, texture, or entrance translation.
The parent grid's single row gap is the only spacing source above and below it.

Configured day/night backgrounds render as a single full-frame `Img` with only
`object-cover`. The source image is not dimmed, desaturated, vignetted, or
covered by scanlines. A cinematic fallback gradient exists only for a missing
background asset.

The Preview v2 shell may rasterize the Player container with `html-to-image`
for an explicit copy-frame action. The PNG output is fixed to the composition's
1920×1080 native size with a black fallback surface, then written as
`image/png` through `ClipboardItem`. Unsupported clipboard APIs and rasterizer
failures are non-destructive UI errors; capture never seeks the Player.

## 9. Persisted Data Policy

This delivery intentionally performs a direct schema transition:

- all checked-in game JSON is edited to include `Game.presenter`;
- all test fixtures are updated;
- video composition schema is bumped and only the new shape is accepted;
- no legacy game or composition normalization is added.

This is safe for the current workspace because KivDB has one game record and no
historical export-job snapshot directory. External stale data must be migrated
before use.

## 10. Failure Matrix

| Condition | Behavior |
|---|---|
| Malformed/incomplete presenter definition | KivDB library load fails with definition/key path |
| Blank/unknown/disabled presenter submitted | Game creation fails; no game is written |
| Game record lacks presenter snapshot | Game load fails; no implicit selection |
| Voice mapping is `pending` or null | Text renders; no cue is emitted |
| Ready mapping points to missing MP3 | Preview is silent; export warns and continues |
| Presenter definition edited after game creation | Existing game remains unchanged |
| Incomplete/invalid Library presenter form | Save fails validation; existing KivDB collection remains unchanged |

## 11. Verification Strategy

Test through public domain interfaces:

- presenter collection validation, exhaustive keys, placeholders, audio maps;
- definition-to-game snapshot isolation;
- `守夜人` and `法官` resolve the same event to distinct text;
- new-game selection in preset and random flows, including invalid IDs;
- Library presenter form parsing, complete editor field coverage, diagnostics,
  and storage round-trip;
- direct checked-in game migration and strict missing-presenter rejection;
- player transcript versus presenter prompt ownership;
- title-independent audio selection and pending silence;
- route allow/deny behavior from all definitions;
- preview/export equality and snapshot immutability;
- full tests, typecheck, Next build, and preview-v2 render smoke.
