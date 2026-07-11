# Domain Context

## Presenter Definition

A reusable host identity available to games. A presenter does not occupy a
seat, receive a role, belong to a faction, take night actions, vote, or affect
the win condition. Each Presenter Definition owns its complete independent
script and voice mapping; presenters do not inherit a shared script.
Presenter Definitions are configured as first-class Library objects; edits are
validated and persisted to `kivdb/presenters.json`.

## Game Presenter

The Presenter Definition selected for one Game. A Game has exactly one Game
Presenter, and changing the library definition later must not rewrite the
identity or behavior already selected for an existing Game.

## Player Character

A seat-bound identity used by a Player. A Player Character is distinct from a
Presenter Definition even when both have names, portraits, and speaking styles.
