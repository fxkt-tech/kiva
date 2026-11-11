#!/usr/bin/env python3
"""Bounded original ranged track/Bank/Event/media probe; no client edits."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys
import tempfile

ROOT = next(parent for parent in Path(__file__).resolve().parents if (parent / ".trellis").is_dir())
OUT = Path(__file__).resolve().parent
RESEARCH = ROOT / ".trellis/tasks/11-11-naraka-action-set/research"
if not RESEARCH.is_dir():
    candidates = list((ROOT / ".trellis/tasks/archive").glob("*/11-11-naraka-action-set/research"))
    assert len(candidates) == 1, "original bounded research helpers not uniquely located"
    RESEARCH = candidates[0]
CACHE = Path("/home/asuka/kiva-asset-research/naraka/action-set-2026-11-11")
SOURCE = Path("/mnt/c/Program Files (x86)/Steam/steamapps/common/NARAKA BLADEPOINT")
DECODER = Path("/home/asuka/kiva-asset-research/vgmstream-r2168/vgmstream-cli")
sys.path.insert(0, str(ROOT / "scripts"))
import naraka_audio as audio


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    tracks = load("ranged_original_tracks", RESEARCH / "audio_tracks.py")
    tracks.ROOT = OUT
    tracks.KEYS = {
        "female_sniper_ads_attack_01", "female_sniper_attack_01",
        "male_sniper_ads_attack_01", "male_sniper_attack_01",
        "male_bow_attack_01", "male_bow_attack_aim_01", "male_bow_attack_aim_01_pre",
        "male_fk_attack_01", "female_fk_attack_01", "female_fk_ads_attack_01",
    }
    original_argv = sys.argv
    sys.argv = [__file__, "--source-root", str(CACHE)]
    tracks.main()
    sys.argv = original_argv
    (OUT / "audio-tracks-selected.json").replace(OUT / "ranged-track-evidence.json")
    tables = load("ranged_original_tables", RESEARCH / "audio_probe.py")
    dbbl, data = tables.tables()
    events = {r["key"]: r for r in data["WwiseBanks_Event"]}
    banks = {r["key"]: r for r in data["WwiseBanks_Bank"]}
    audio_root = SOURCE / "NarakaBladepoint_Data/StreamingAssets/Audio/GeneratedSoundBanks/Windows"
    result = []
    for name in ["gman_fk_attack_01", "gman_bow_attack_01"]:
        record = events[name]
        reader = tables.Reader(record["raw"])
        assert reader.string() == name
        bank_name = reader.string()
        bank_record = banks[bank_name]
        reader = tables.Reader(bank_record["raw"])
        assert reader.string() == bank_name
        package_name = reader.string()
        path = audio_root / package_name
        package = audio.read_pck(path)
        entry = next(e for e in package["entries"]
                     if e["section"] == "banks" and e["id"] == audio.fnv(bank_name))
        assert entry["bytes"] <= audio.MAX_BANK
        with path.open("rb") as stream:
            stream.seek(entry["offset"])
            raw = stream.read(entry["bytes"])
        bank = audio.read_bank(path, package, audio.fnv(bank_name), audio.sha(raw))
        layers, _ = audio.event_layers(bank["objects"], bank["object_index"], bank["bank_id"], name)
        media, paths = [], []
        for layer in layers:
            for sound in layer["sounds"]:
                info, _ = audio.source_media(bank["objects"][sound["id"]], bank)
                media.append(dict(sound_id=info["sound_id"], plugin_id=info["plugin_id"],
                                  stream_type=info["stream_type"], source_id=info["source_id"],
                                  source_bytes=info["bytes"], source_sha256=info["sha256"],
                                  bank_data_offset=info["bank_data_offset"],
                                  physical_offset=info["physical_offset"],
                                  codec=info["codec"], channels=info["channels"], rate=info["rate"]))
            paths.append(dict(action_id=layer["action_id"], play_action_type=0x403,
                              target_id=layer["target"], target_type=bank["objects"][layer["target"]]["type"],
                              containers=layer["containers"], sounds=[s["id"] for s in layer["sounds"]],
                              action_body_hex=bank["objects"][layer["action_id"]]["body_hex"],
                              original_delay_ms=layer["original_delay_ms"]))
        first = media[0]
        with tempfile.TemporaryDirectory(prefix="kiva-ranged-audio-probe-") as temporary:
            _, wem = audio.source_media(bank["objects"][first["sound_id"]], bank)
            file = Path(temporary) / "source.wem"
            file.write_bytes(wem)
            pcm, measurement = audio.decode_wem(file, DECODER, temporary)
            _, encoded = audio.encode(pcm, measurement, temporary)
        decoded = dict(source_id=first["source_id"], duration=measurement["frames"] / measurement["rate"],
                       samples=measurement["frames"], channels=measurement["channels"], rate=measurement["rate"],
                       peak=measurement["peak"], rms_dbfs=measurement["rms_dbfs"],
                       wav_sha256=measurement["wav_sha256"],
                       ogg_bytes=encoded["bytes"], ogg_sha256=encoded["sha256"])
        result.append(dict(event=name, event_id=audio.fnv(name), bank=bank_name,
                           bank_id=bank["bank_id"], bank_sha256=bank["sha256"], bank_entry=bank["entry"],
                           package={k: v for k, v in package.items() if k != "entries"},
                           event_record_offset=record["offset"], event_record_bytes=record["bytes"],
                           event_record_sha256=audio.sha(record["raw"]),
                           bank_record_offset=bank_record["offset"], bank_record_bytes=bank_record["bytes"],
                           bank_record_sha256=audio.sha(bank_record["raw"]),
                           paths=paths, media=media, decoded=decoded))
        print(name, "layers", len(layers), "variants", len(media),
              "first duration", decoded["duration"])
    (OUT / "ranged-audio-trace.json").write_bytes(audio.json_bytes(
        dict(source_dbbl_sha256=audio.sha(dbbl), decoder_sha256=audio.DECODER_SHA,
             chains=result)))


if __name__ == "__main__":
    main()
