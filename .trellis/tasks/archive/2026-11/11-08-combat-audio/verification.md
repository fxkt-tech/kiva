# Audio implementation evidence

## Implemented boundaries

- `src/client/audio.rs` / `audio/tests.rs`: separate fixed-step reader after
  SimulationSet::Step and before GameplaySet::Outcome; committed facts, ability
  identity, source life/position, visibility, own life notifications, hearing
  distance, cosmetic variation, bounded queue and independent display transport.
- `web/audio.js`: one Web Audio context, trusted gesture activation, preload,
  bounded priority mix, ended cleanup, stale/session/life protection, pause/focus
  cleanup, volume/mute and read-only observable diagnostics. No simulation rule
  or World mutation is implemented in JavaScript.
- `assets/audio`: 25 local SFX exports, 433,273 bytes total, from the pinned current banks plus the separately documented shared respawn source. Ashe
  PROJECT includes empowered attacks and its actual respawn event; Jarvan uses
  base SFX, with the explicitly documented shared respawn cue. No old SFX or VO
  fallback is shipped. Source/chunk hashes, play events and WEM IDs are recorded.
- `scripts/fetch-league-audio.py` is the main agent's range reconstruction tool;
  `scripts/import-audio.py` resolves event/container graphs and decodes/encodes
  the reviewed layers. All optional tools are import-time only.

## Checks completed by audio worker

- `node --test scripts/audio-unit.test.cjs`: **8 passed**, covering actual owner
  logic for one trusted device, no decode/autoplay backlog, pause/focus/reset
  cleanup, old-frame/old-life protection, 16-voice priority bound, settings, and
  observable genuine load failure. Device behavior is mocked here; this is not
  claimed as browser decoding evidence.
- `node --check web/audio.js` and `node --check scripts/browser-audio.cjs`: pass.
- `rustfmt --edition 2024 src/client/audio.rs src/client/audio/tests.rs`: pass.
- `python3 scripts/import-audio.py --verify`: all 25 SHA-256 values match.
- Every OGG fully decoded through ffmpeg. A second decode/encode from an empty
  cache into a different output directory produced identical hashes for all 25
  files; the total remains 433,273 bytes.

## Integration checks originally delegated to main agent

The audio worker deliberately did not run Cargo/Trunk or compete for GPU/browser
resources while other work was underway. These checks are prepared, not claimed
as passed here:

- The 8 native adapter tests use real simulation commands and committed results:
  rejection/once-only, mixed loadout, actual four-AA→Q→empowered AA, death and
  respawn between presentation frames, hidden/visible enemy casts, pause/reset,
  queue/stale priority and attenuation/variation.
- `node scripts/browser-audio.cjs` uses the built application, trusted selection
  and launch clicks, 25 real decode completions, real Jarvan/Ashe casts, actual
  analyser amplitude, pause cleanup, mute with no replay, volume keyboard input,
  and a new selected hero/session. It uses existing browser fixture helpers and
  a temporary static server, leaving the user's development service alone.
- Native/WASM compiler, complete test/lint checks and combined lifecycle/model
  integration remain the main agent's serial quality gate.

## Main agent final integration result

On 2026-11-08, the complete `make check` passed 343 Rust tests, including all
eight real-command audio adapter tests (two existing long soak tests remain
ignored). Native and WASM Clippy passed, as did all 40 Node tests and the final
temporary Trunk build. On that final build, `browser-audio.cjs` passed in headed
Chromium/WebGPU: all 25 real files decoded, trusted activation unlocked the one
device, Jarvan W/Q and PROJECT Ashe W produced nonzero analyser output, and
pause, mute without replay, volume, and session replacement behaved correctly.
The script closed its own browser and fixture without touching the user's
development service. See the parent verification file for the source/build
fingerprint and `/tmp/kiva-presentation-final-browser-audio.log` for the result.

## Scope limits

Selected source layers are actual Riot bank media. Full Wwise RTPC, pitch and
material-switch graph emulation is intentionally outside the lightweight local
cue exporter; runtime gain/pan comes from client presentation. The Jarvan source
bank has no skin-specific respawn cue, so that one cue uses the separately
verified shared League sound. Audio failure remains cosmetic and never blocks
entering a match.
