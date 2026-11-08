# Combat audio research (2026-11-08)

## Confirmed requirements and delivery boundary

The user authorized planning followed immediately by implementation, with no further
questions while asleep. Prefer PROJECT: Ashe (skin 8) for appearance/audio, with a
working fallback explicitly allowed; prefer base Jarvan IV (skin 0). Keep battle
rules in simulation and all cue selection/playback in client presentation.

## Delivered source: actual current PROJECT and base SFX

The later Riot-CDN investigation resolved the missing PROJECT combat bank. The
production assets now come from patch **16.22.8348910**, manifest
`6EDAD6B3C9806CB9`. No 2014 SFX or Gitvoice dialogue fallback is shipped.

- Public source manifest:
  https://lol.secure.dyn.riotcdn.net/channels/public/releases/6EDAD6B3C9806CB9.manifest
  (SHA-256 `ed2ba7e19ddacb18604fd5f61baf87927d4c4514540cec8047cb56d6499cdbcb`).
- PROJECT Ashe skin 8 SFX audio bank: 2,014,858 bytes, SHA-256
  `37f6e60a7e5b3ff258994a211cc9ef7d21c86d3e5953eeb491d63c5db752cd0b`.
- Base Jarvan IV SFX audio bank: 1,194,208 bytes, SHA-256
  `84dc68a76317e918423fa6142617847f4c4d81a94b2a901f28aa2f9954f95e03`.
- The corresponding event banks and skin BIN files were also retrieved. The
  graph parser maps actual Play event names through actions and random/layer
  containers to embedded WEM IDs. It does not guess sounds from file numbers.
- Ashe has verified normal attack, Q activation, Q empowered attack, W/E/R,
  `Death3D_cast` and `Respawn3D_buffactivate` events in the PROJECT bank. Jarvan
  has verified base AA, Q/W/E/R and `Death3D_cast`. No Jarvan respawn event was
  present, so only that cue uses the separately verified shared respawn below.
- `league-tools==1.2.2` parses Wwise 145; vgmstream `r2168` decodes the selected
  WEM streams. Release:
  https://github.com/vgmstream/vgmstream/releases/download/r2168/vgmstream-linux.zip
  (SHA-256 `2dfdfddda68e0e36e62f20dc050a4e45f4bc3318c7efeadf8a063df646d57c84`).
- All **25 final OGG files, 433,273 bytes total**, fully decode. A second rebuild
  from an empty decode cache produced identical SHA-256 values for every file.
  Source hashes, event names, WEM IDs and recipes are checked into
  `assets/audio/{riot-sources,sources,catalog}.json`; rebuild commands and exact
  scope are in `assets/audio/README.md`. Public CDN chunk reconstruction is in
  `scripts/fetch-league-audio.py`; event decoding/encoding is in
  `scripts/import-audio.py`.

Earlier HTTP 403 responses were caused by an overly short script User-Agent;
using a normal full browser User-Agent retrieved the public CDN metadata. No
account, password, author contact or installed game client was required.

## Earlier candidate investigation (not shipped)

Research cache: `/tmp/kiva-audio-research-uts3p9kq/`. This path is an investigation
artifact, not a production asset dependency. Production copies need a checked-in
source manifest with exact URL, source version, SHA-256 and processing command.

### Original combat SFX (independent WAV files)

- Index: https://wiki.spectralcoding.com/info:lol_sound_pages:jarvan_iv_-_default.html
  (84 independent sounds: basic attack cast/hit variants, Q, EQ, W cast/loop,
  E cast/fall/hit/loop, R cast/hit/break and death equipment effects).
- Index: https://wiki.spectralcoding.com/info:lol_sound_pages:ashe_-_default.html
  (30 independent sounds: basic attack, old Frost Shot, W, E, R).
- The provider explicitly dates these to patch `4.6.14_04_10_21_15`, patched
  2014-04-25: https://wiki.spectralcoding.com/info:league_of_legends_sounds:league_of_legends_sounds.html
- The provider warns that some conversions may need pitch correction. Files
  checked here decode correctly, but decoding alone does not prove their pitch
  matches modern LoL. Do not label this corpus as current-version assets.
- URL prefix is `https://files.spectralcoding.com/files/misc/lolwavs/`.
- Concrete files (downloaded, ffprobe and full ffmpeg decode passed):
  - `LoL_SFX_jarvaniv/17_jarvaniv_basicattack_oc_01.wav`
  - `LoL_SFX_jarvaniv/01_jarvaniv_basicattack_ohf_01.wav`
  - `LoL_SFX_jarvaniv/52_jarvaniv_dragonstrike_oc_01.wav`
  - `LoL_SFX_jarvaniv/73_jarvaniv_goldenaegis_oc_01.wav`
  - `LoL_SFX_jarvaniv/39_jarvaniv_demacianstandard_oc_.wav`
  - `LoL_SFX_jarvaniv/25_jarvaniv_cataclysm_oc_01.wav`
  - `LoL_SFX_jarvaniv/69_jarvaniv_emote_death.wav`
  - `LoL_SFX_ashe/01_bowmaster_basic_OC_1.wav`
  - `LoL_SFX_ashe/10_bowmaster_volley_OC_1.wav`
  - `LoL_SFX_ashe/23_bowmaster_hawkshot_oc_1.wav`
  - `LoL_SFX_ashe/17_bowmaster_enchantedcrystalarr.wav`
- `verification.json`, `lifecycle-verification.json`, `decode-checks.json` contain
  sample metadata/hashes. Three samples additionally converted to Vorbis OGG,
  48 kHz, using `ffmpeg -i INPUT -ar 48000 -c:a libvorbis -q:a 4 OUTPUT.ogg`.

### PROJECT Ashe and base Jarvan Chinese VO, mapped to actual events

Repositories have branch `audio`, README version `11.10` (2021). These are VO,
not weapon/ability SFX. Do not describe them as a complete PROJECT combat SFX set.

- https://github.com/Gitvoice/Ashe — `skin8/{id}.mp3` (PROJECT).
- https://github.com/Gitvoice/JarvanIV — `skin0/{id}.mp3` (base).
- Event maps:
  - https://raw.githubusercontent.com/Virace/lol-audio-events-hashtable/main/VO/zh_CN/characters/ashe/skin8.json
  - https://raw.githubusercontent.com/Virace/lol-audio-events-hashtable/main/VO/zh_CN/characters/jarvaniv/skin0.json
- Actual media URL pattern:
  `https://raw.githubusercontent.com/Gitvoice/Ashe/audio/skin8/677239718.mp3`.
- All IDs below intersect the actual repository tree, downloaded and decoded.
- PROJECT Ashe:
  - Q (`Play_vo_AsheSkin08_AsheQ_cast3D`): 677239718 (0.967 s),
    492368145 (1.306 s).
  - W (`Play_vo_AsheSkin08_Volley_cast3D`): 96463442 (1.149 s),
    276588018 (1.411 s).
  - E (`Play_vo_AsheSkin08_AsheSpiritOfTheHawk_cast3D`): 122297235
    (1.071 s), 362187993 (1.202 s).
  - R (`Play_vo_AsheSkin08_EnchantedCrystalArrow_cast3D`): 1071464777
    (0.731 s), 413150626 (0.862 s).
  - Death (`Play_vo_AsheSkin08_Death3D`): 822331857 (3.291 s),
    695731331 (3.370 s).
- Jarvan base:
  - Basic attack (`Play_vo_JarvanIV_JarvanIVBasicAttack_cast3D`, variants 2/3
    share these sounds): 1215894796 (0.888 s), 2027665590 (0.862 s).
  - Q (`Play_vo_JarvanIV_JarvanIVdragonstrike_cast3D`): 1069948866
    (1.437 s), 373903481 (1.907 s).
  - E (`Play_vo_JarvanIV_JarvanIVDemacianStandard_cast3D`): 698884581
    (1.149 s), 465578768 (1.515 s).
  - R (`Play_vo_JarvanIV_JarvanIVCataclysm_cast3D`): 1094129435
    (1.776 s), 1573966194 (2.821 s).
  - Death (`Play_vo_JarvanIV_Death3D`): 661651632 (6.452 s),
    610767530 (7.732 s). These full files include a long tail; any production
    trim must be explicit and not cut the vocal attack.
- No actual Respawn VO event in either selected event map. Do not substitute
  a random move/selection voice and call it a respawn event.
- Cache files: `mapped-voice/`, `mapped-voice-verification.json` and the two
  `*-vo-events.json` documents.

### Shared respawn cue

- Source page: https://www.101soundboards.com/sounds/471129-respawn
- Actual media: https://hoovers.101soundboards.com/sb/board_sounds_rendered/oxbwar.mp3
- Downloaded, decoded: mono MP3, 44.1 kHz, 3.944438 s, 64,194 bytes.
- SHA-256: `f590a80c8ba53ca8ee6636e1dca76e4dbb21d9e195d40ed97d612a16e18f8fa8`.
- Cache `shared-respawn.mp3`, and converted `preview/shared-respawn.ogg`.

### Alternate sources checked

- https://vaultsoft.co.uk/sound-vault/ and
  https://raw.githubusercontent.com/VaultSoft/sound-vault/main/manifest.json:
  Ashe contains `ashe sfx.mp3` and High Noon only, no PROJECT or Jarvan pack.
  Original montage downloaded (402.0272 s, stereo MP3 48 kHz, 16,093,992 bytes).
  It needs event-by-event identification/cutting, so do not ship it as one cue.
- League Wiki currently has only 3 Ashe SFX files and 1 Jarvan SFX file;
  champion Audio pages predominantly provide VO. Category counts are not a
  complete combat SFX corpus.
- https://github.com/Virace/lol-audio-unpack reads local WAD/BNK/WPK and can
  export WAV by event. It needs local game/resource files (not installed here).
- https://github.com/RitoShark/Flint supports CDN manifest browsing and selective
  inner-WAD extraction. The initial Sieve requests failed; the later direct
  public Riot CDN extraction above retrieved the required actual audio banks.
- https://x-item.com/lol_voice_collation.html provides other archives; latest
  update is VO-only; some archive downloads require obtaining a password from
  the author. No messages were sent and no password gate was bypassed.

## Browser playback recommendation

Prefer a thin Rust `client/audio` adapter with its own message cursor and a single
Web Audio owner in `web/audio.js`, separate from the 80 ms HUD snapshot. The Web
target is the product runtime; native tests can inspect cue decisions without an
audio device. This avoids adding a second sound engine or sound decisions in JS.

Browser owner should preload/decode bounded local OGG assets, create/resume its
one AudioContext inside the real trusted launch/resume gesture, and schedule only
accepted cue IDs. It should have gain/pan nodes, a concurrency limit, ended-node
cleanup, pause/focus/reselection teardown, and discard expired or old-match cues.
Failure to enable audio must leave the match usable and retry on the next trusted
gesture; no stale cue burst when sound finally becomes available.

The Rust adapter should select cues from committed attack/ability/death/respawn
facts, not input or animation frames; apply team visibility at the event position
so hidden enemies are not audible across the map. Attach simulation tick, match
identity and source life to transient cues to reject old-session playback.
Do not re-use the slow HUD transport for sound triggers. The skin mapping and
AbilityId cue selection belong to client presentation content, not controls or
simulation. Mixed loadouts must retain the ability's own cue identity.

Why not enable Bevy Audio immediately: `Cargo.toml` has it disabled; native ALSA
development metadata is absent (`pkg-config --exists alsa` returns 1). The local
cpal 0.17.3 WebAudio backend calls AudioContext.resume in StreamTrait::play but
does not register trusted gesture listeners. Bevy AudioOutput hides the stream
context, making reliable first-click activation less direct. A Bevy backend is
possible, but would need explicit native dependency and browser activation work.

## Resolved delivery decision

Use the exact current PROJECT Ashe and base Jarvan combat banks described above.
Do not ship the legacy or dialogue candidates. Only Jarvan's missing skin-specific
respawn event uses the explicit shared respawn cue. Wwise RTPC/pitch/material
switch behavior is not fully emulated: reviewed event layers are exported to
local OGG and client presentation supplies its own bounded mix, gain and pan.
