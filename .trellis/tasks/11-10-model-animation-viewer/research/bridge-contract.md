# Implementation contract: viewer Web / Rust

Agreed by web and runtime implementers, 2026-11-10. This is an implementation coordination artifact, not a new product requirement.

## Composition and authority

`client::run` installs DefaultPlugins and loading::StartupPlugin, then dispatches the immutable mode read by `web::viewer_mode()`. Viewer calls `viewer::configure(&mut App)`; normal game composition remains separate. Viewer configures libraries and startup/update systems. JavaScript never advances a pose/playback clock.

Web mode script installs frozen `window.kivaPage.isViewer()` before any game script. Only exact `view=model` chooses the viewer. Game-only scripts return before listeners in that mode. Navigation uses ordinary relative query URLs, preserving deployment directory.

## Command channel

Rust `web::viewer_take_commands()->String` consumes `window.kivaViewer.takeCommands()` once. Each command is one newline-delimited line with tab-delimited primitive fields. IDs cannot contain tabs/newlines/control characters and are bounded to 160 characters. Queue at most 128 commands and 16 KiB, pointers aggregate/coalesce when possible; Rust independently validates arity, IDs, finite numeric values and bounds.

```
preset <catalog ID>
appearance <catalog ID>
weapon <catalog ID>
mode game|source|reference
action <catalog ID>
play 0|1
seek <seconds>
step -1|1
speed .25|.5|1|1.5|2
loop 0|1
debug grid|axes|bones|grips|bounds 0|1
part <part ID> 0|1
orbit <CSS dx> <CSS dy>
pan <CSS dx> <CSS dy>
zoom <normalized wheel pixels>
view front|back|left|right|top|three-quarter
fit
reset
```

Angle and time formulas stay Rust-side. Pointer drag begins only on canvas, uses pointer capture (never pointer lock), clears on release/cancel/blur. Native bridge returns no browser commands.

## Snapshot channel

`web::viewer_render(&str)` calls `window.kivaViewer.render(JSONstr)` after Rust state/evaluation. JSON strings must escape arbitrary part/source labels. JS validates snapshot shape and uses textContent/createElement only. It preserves prior catalog if omitted from a subsequent snapshot.

```json
{"version":1,"revision":1,"ready":false,"failed":false,"appearance":"catalog-id","weapon":"catalog-id","preset":"catalog-id-or-custom","mode":"game","action":"stable-action-id","time":0,"duration":2,"frame":0,"playing":false,"speed":1,"looping":true,"trace":"actual sampling info","warning":"","status":"正在准备模型","camera":{"yaw":0,"pitch":0,"distance":3,"target":[0,1,0]},"debug":{"grid":true,"axes":true,"bones":false,"grips":false,"bounds":false},"catalog":{"appearances":[{"id":"id","label":"角色"}],"weapons":[{"id":"id","label":"武器"}],"presets":[{"id":"id","label":"实际配置"}],"actions":[{"id":"stable-id","label":"待机","mode":"game","group":"游戏单体","duration":2}]},"parts":[{"id":"id","label":"部件","visible":true}],"inspection":"copyable inspection text"}
```

Catalog IDs and action durations come from Rust. Source and game catalogs are displayed separately with explicit labels. `inspection` is optional; UI can copy a safe bounded serialization of current confirmed snapshot. No local playback simulation or optimistic acknowledgement.

## Render / selection readiness

Viewer camera inserts `rendering::MainView`, shared with game via rendering setup. `rendering::setup_viewer(&mut World)` configures shared PBR/HDR/FXAA without game wetland SSR.

Runtime calls `loading::require_model(world, revision, Vec<Entity>)` with selected body Mesh3d entities, after a new selection; never floor or weapon alone. `loading::model_ready(world, revision)` reads the render-world confirmation. Render observation requires a current selected body mesh visible and specialized, every visible mesh specialized, all needed pipelines ready, and an actual submitted view target. Revisions prevent stale confirmation. Global startup ready remains once-only; later selections have independent readiness. `loading::model_failed(world, revision)` supplies terminal selection error state; snapshot `failed` presents reload recovery. The main-world render-error callback calls `web::viewer_fail(reason)` / `kivaViewer.fail(reason)` immediately before delegating to Bevy's existing policy, so a stopped render loop still has a visible reload control.
