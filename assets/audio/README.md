# Combat audio

The checked-in OGG files are the runtime assets. Builds and gameplay do not fetch
the game client, run the importer, require an installed League client, or load a
second audio engine.

## Content and provenance

- Ashe: **PROJECT, skin 8**, Riot patch `16.22.8348910`; basic attack, Q activation,
  Q empowered attacks, W, E, R, death and respawn SFX.
- Jarvan IV: **base skin 0**, the same patch; basic attack, Q, W, E, R and death
  SFX. His selected bank has no respawn event, so his respawn uses the separately
  documented shared League respawn cue.
- These are actual event-mapped combat SFX, rather than dialogue clips or the old
  2014 sound corpus investigated before the current banks were retrieved.
- Twenty-five local, mono, 48 kHz Vorbis files provide separate random variants
  for the common attacks and abilities. `catalog.json` maps presentation-only
  CharacterId and AbilityId values to cues. Mixed loadouts use the ability's cue.

`riot-sources.json` pins the public Riot CDN version, original WAD paths, exact
compressed chunk byte ranges and decoded SHA-256 values. `sources.json` pins the
selected play events, deterministic random-container variant, resulting WEM IDs,
decoder version, processing recipe and each final OGG SHA-256. The sole shared
MP3 source is also URL- and SHA-pinned there. League assets belong to Riot Games;
these manifests record provenance, not a separate grant of rights.

## Rebuild offline assets

Optional import tools are **Python 3, pyzstd, league-tools 1.2.2, ffmpeg with
libvorbis**, and **vgmstream r2168**. Install Python dependencies in an isolated
environment. vgmstream's exact release URL and archive SHA-256 are recorded in
`sources.json`; verify that archive before extracting its CLI.

```sh
python3 -m venv /tmp/kiva-audio-tools
/tmp/kiva-audio-tools/bin/pip install pyzstd league-tools==1.2.2
/tmp/kiva-audio-tools/bin/python scripts/fetch-league-audio.py --output /tmp/kiva-banks
/tmp/kiva-audio-tools/bin/python scripts/import-audio.py \
  --banks /tmp/kiva-banks --vgmstream /path/to/vgmstream-cli \
  --output /tmp/kiva-rebuilt-audio
python3 scripts/import-audio.py --verify --output /tmp/kiva-rebuilt-audio
```

The downloader fetches the reviewed source chunks, verifies their decoded hash,
and reconstructs only the pinned banks/BIN files. The importer verifies those
files again, resolves each actual Play event through the event/action/container
graph, chooses the specified random variant, preserves parallel layers, decodes
WEM with `vgmstream-cli -i`, trims silence, mixes layers, bounds duration, applies
a short final fade and loudness normalization, and encodes Vorbis. It fully
decodes every output to catch corrupt media. It fails on missing media, unmapped
events, unsupported switch containers or hash mismatches.

This deliberately does not emulate Wwise's entire RTPC, material-switch, pitch
or spatialization graph. The selected cast/launch/death/respawn recipes do not
require a guessed switch. Rust supplies audibility, attenuation and pan, and the
browser mixes those cues. The source WEM layers remain identifiable in the
manifest rather than being described as bit-identical recordings of live LoL.

Encoding is reproducible with the recorded ffmpeg build; different encoder
versions may change OGG bytes. `--record` updates reviewed hashes and WEM IDs when
deliberately replacing assets. Normal rebuilds compare against existing hashes.
For a quick check without any optional tool or network access:

```sh
python3 scripts/import-audio.py --verify
node --test scripts/audio-unit.test.cjs
```

## Runtime ownership

`src/client/audio.rs` consumes committed simulation facts once per fixed step,
before outcome/cleanup. It snapshots the source life and position, filters team
visibility, applies hearing distance, and publishes a bounded transient packet
independent of HUD throttling. No simulation rule depends on sound availability.

`web/audio.js` owns one Web Audio context. Trusted gestures unlock it; local
files preload/decode independently. Undecoded, muted, blocked, expired and old
session cues are discarded without a delayed playback queue. Pause, pointer-lock
loss, blur and session replacement stop and disconnect all active nodes. The mix
has a sixteen-voice bound, priorities, volume/mute controls and ended cleanup.
`window.__kivaAudio` is read-only playback diagnostics, including actual analyser
peak and load/decode errors. It provides no simulation mutation API.

`node scripts/browser-audio.cjs` exercises the built application with trusted
input and real decoded audio. It uses a temporary static fixture server, closes
it on exit, and leaves the user's development service alone.
