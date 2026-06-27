# Role Library UI Design

## Purpose

The role library UI is the configuration surface for reusable game-generation assets. It is not a generic JSON editor.

Its job is to keep three things correct before a game is created:

- The selected roles match the current ruleset and action mechanics.
- AI characters remain separate from gameplay identities.
- Presets can create a complete, valid game without hidden broken references.

The UI must help the host understand whether a library item is safe to use, where it is referenced, and what prompt material it will contribute to a game.

## Navigation

Add a dedicated Library workspace at `/library`.

The home page should expose two top-level actions:

- `New game`: opens a preset selection flow instead of immediately creating the default game.
- `Library`: opens `/library` for maintaining roles, characters, and presets.

The editor remains focused on one game. It should not become the place where global library assets are edited.

## Information Architecture

Use a Library workspace with tabs:

- `Roles`
- `Characters`
- `Presets`

The accepted layout is:

- Header: `Back`, `Kiva Library`, optional import/export actions.
- Left navigation: the three library sections and a create-game entry point.
- Object list: items in the active section.
- Detail editor: the selected item form.
- Validation panel: references, rule compatibility, prompt preview, and save errors.

This is intentionally not a one-screen dashboard. Editing one object type at a time keeps the host from mixing role mechanics, character personality, and preset seat assignment.

## Roles Tab

Roles represent gameplay identities such as werewolf, seer, witch, and villager.

Editable fields:

- `id`
- `name`
- `enabled`
- `faction`
- `team`
- `mechanicKey`
- `visibilityRules`
- `nightOrder`
- `systemPrompt`
- `actionPrompt`
- `defaultModelBinding`

For current built-in roles, the rule contract fields are locked:

- `werewolf`: `faction=wolves`, `team=wolf`, `mechanicKey=wolf_kill`
- `seer`: `faction=good`, `team=god`, `mechanicKey=seer_check`
- `witch`: `faction=good`, `team=god`, `mechanicKey=witch_medicine`
- `villager`: `faction=good`, `team=villager`, `mechanicKey=none`

For those roles, the UI can still edit display name, prompts, model binding, enabled status, and timing fields where supported. It must not let the host save a built-in role with a mismatched faction, team, or mechanic.

New unsupported role ids can be added to the library, but first-version presets cannot use them to create a game unless the current ruleset supports them. The UI should surface this as a validation warning, not silently fail during game creation.

## Characters Tab

Characters represent AI performers. They do not define gameplay identity.

Editable fields:

- `id`
- `name`
- `avatar`
- `tags`
- `enabled`
- `persona`
- `speakingStyle`
- `reasoningStyle`
- `systemPrompt`
- `defaultModelBinding`

The UI must not include role-only fields in the character editor:

- role
- faction
- team
- mechanic
- visibility rules
- night order

The character detail page should include a prompt preview that shows character prompt material by itself. Role prompt composition is shown from presets or generated player snapshots, because a character has no role until assigned to a seat.

## Presets Tab

Presets are game creation recipes.

Editable fields:

- `id`
- `name`
- `rulesetId`
- `playerCount`
- `enabled`
- seat assignments

The main preset editor should be a seat table, not a raw field list.

Each row represents one seat:

- seat number
- selected role
- selected character
- optional model binding override

The validation panel must show:

- role counts versus the selected ruleset
- disabled role references
- disabled character references
- duplicate seat numbers
- missing seat assignments
- unsupported role ids
- whether the preset can create a game

The preset detail page should include a `Create game` action. When successful, it redirects to the editor for the new game.

## Create Game Flow

The home page `New game` action should no longer create the default preset immediately.

First-version flow:

1. Host opens `New game`.
2. The app shows enabled presets with validation status.
3. Host selects one preset.
4. The app shows the preset seat table and validation panel.
5. Host clicks `Create game`.
6. The server creates the game from the selected preset.
7. The app redirects to `/games/:gameId/editor`.

This makes preset choice explicit and avoids hiding invalid library configuration behind a one-click action.

## Save Behavior

Use explicit save, not autosave.

Reason: role library changes affect future games and LLM prompts. Autosave can persist half-complete prompt edits, invalid references, or temporarily broken presets.

Save flow:

1. Host edits an item locally in the form.
2. Host clicks `Save`.
3. Server validates the full collection boundary affected by the item.
4. If valid, write the JSON repository and refresh the page state.
5. If invalid, keep the form dirty and show field-specific or object-specific errors.

The UI should show unsaved state. Navigating away with unsaved edits should warn the host.

## Disable, Duplicate, And Delete

First version supports:

- `New`
- `Save`
- `Duplicate`
- `Disable` / `Enable`

First version does not support hard delete.

Reason: roles, characters, and presets can be referenced by historical games or other presets. Existing game snapshots remain immutable, but hard deletion still creates avoidable confusion in the library UI. Disable is enough to prevent future use while preserving explainability.

## Validation And Diagnostics

The right-side validation panel is required.

For every selected object, it should show:

- Whether the object is valid.
- Whether it is enabled.
- Where it is referenced.
- Whether it can be used by enabled presets.
- The relevant prompt preview.
- The last save error, if any.

Errors should expose the same root cause as server validation, for example:

- `Role werewolf does not match the current ruleset contract`
- `Game preset six_player_standard roleIds[0] references disabled role: werewolf`
- `Game preset six_player_standard seatAssignments[2] references unknown character: zhou_zhi`

The UI should not collapse these into a generic `failed` message.

## Prompt Preview

Prompt preview is diagnostic, not editable generated text.

Roles should preview:

- role system prompt
- role action prompt

Characters should preview:

- character system prompt
- persona
- speaking style
- reasoning style

Presets should preview one selected seat's eventual prompt composition:

- character system prompt
- role system prompt
- role action prompt
- resolved model binding

This mirrors how games snapshot data at creation time and helps the host see why a player will speak or act a certain way.

## Server/API Boundary

The UI should use server actions or route handlers over `LibraryRepository`.

Required operations:

- list library record
- save role
- save character
- save preset
- duplicate role
- duplicate character
- duplicate preset
- enable or disable item
- create game from preset

Saves should validate the full relevant collection before writing:

- saving a role validates all roles and any presets that reference roles
- saving a character validates all characters and any presets that reference characters
- saving a preset validates roles, characters, and presets together

This keeps repository JSON files from entering a state where each file looks valid alone but the combined library cannot create games.

## First Version Non-Goals

The first UI version will not include:

- hard delete
- bulk edit
- role mechanic authoring for new runtime rules
- visual avatar upload
- complex import UI
- random seat assignment controls
- autosave
- generated prompt rewriting tools

Seed import can remain script-based. A visible `Import seeds` button is optional and should only be added if it is backed by a clear overwrite/merge behavior.

## Testing

Core tests should cover:

- library save validation
- unsupported role warnings for presets
- disabled item references
- duplicate behavior
- create-game-from-preset selection

UI or integration tests should cover:

- opening `/library`
- switching tabs
- editing and saving a role prompt
- editing and saving a character prompt
- editing a preset seat assignment
- failed save showing the server validation error
- creating a game from a selected preset and redirecting to editor

## Open Decisions

No open decisions remain for the first implementation plan.
