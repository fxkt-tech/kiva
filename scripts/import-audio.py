#!/usr/bin/env python3
"""Build local combat cues from pinned Riot banks and explicit event recipes.

Optional offline tools: league-tools==1.2.2, vgmstream r2168 and ffmpeg.
First fetch banks with scripts/fetch-league-audio.py. No tool runs at game startup.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class SoundBank:
    """Resolve a play event, preserving parallel and random containers."""

    def __init__(self, definition, directory, cache, decoder):
        from league_tools import BIN, BNK, NativeHIRC
        from league_tools.utils.hash import str_fnv_32
        from league_tools.tools import AudioEventMapper

        self.cache = cache
        self.decoder = decoder
        self.hash_name = str_fnv_32
        hirc = NativeHIRC.from_bnk(directory / definition["events"], use_cache=False)
        self.mapping = AudioEventMapper(BIN(directory / definition["skin"]), hirc).build_mapping()
        self.media = {file.id: file for file in BNK(directory / definition["audio"]).extract_files()}
        self.objects = {}
        for bank in hirc.banks.values():
            for field, kind in [("events", "event"), ("event_actions", "action"),
                                ("sounds", "sound"), ("random_containers", "random"),
                                ("switch_containers", "switch"), ("layer_containers", "layer")]:
                for object_id, item in getattr(bank, field).items():
                    self.objects[object_id] = (kind, item)

    def layers(self, name, variation):
        if not name.startswith("Play_") or not self.mapping.find_sounds_by_event_name(name):
            raise ValueError(f"Event is not a mapped play event: {name}")

        def visit(object_id, ancestors):
            if object_id in ancestors:
                raise ValueError(f"Cyclic sound event: {name}")
            kind, item = self.objects[object_id]
            ancestors = ancestors | {object_id}
            if kind == "sound":
                if item.source_id not in self.media:
                    raise ValueError(f"External media {item.source_id} in {name}")
                return [item.source_id]
            if kind == "event":
                return [source for child in item.event_ids for source in visit(child, ancestors)]
            if kind == "action":
                return visit(item.id_ext, ancestors) if item.id_ext else []
            if kind == "random":
                child = item.child_ids[variation % len(item.child_ids)]
                return visit(child, ancestors)
            if kind == "layer":
                return [source for child in item.child_ids for source in visit(child, ancestors)]
            # These cast/launch recipes must not depend on a guessed material
            # switch or Wwise state. Hits need a separate explicit policy.
            raise ValueError(f"Unsupported {kind} container in selected event: {name}")

        return visit(self.hash_name(name), set())

    def decode(self, media_id):
        path = self.cache / f"{media_id}.wav"
        if path.exists():
            return path
        wem = self.cache / f"{media_id}.wem"
        self.media[media_id].save_file(wem)
        subprocess.run([str(self.decoder), "-i", "-o", str(path), str(wem)],
                       check=True, stdout=subprocess.DEVNULL)
        return path


def run():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "assets/audio")
    parser.add_argument("--banks", type=Path, help="Verified output of fetch-league-audio.py")
    parser.add_argument("--vgmstream", type=Path, default=Path("vgmstream-cli"))
    parser.add_argument("--cache", type=Path)
    parser.add_argument("--verify", action="store_true", help="Check local output hashes only")
    parser.add_argument("--record", action="store_true", help="Record reviewed rebuilt output hashes")
    args = parser.parse_args()
    manifest_path = ROOT / "assets/audio/sources.json"
    manifest = json.loads(manifest_path.read_text())
    if args.verify:
        for cue in manifest["outputs"]:
            path = args.output / cue["file"]
            if not path.is_file() or sha256(path) != cue["sha256"]:
                raise SystemExit(f"Invalid audio asset: {path}")
        print(f"Verified {len(manifest['outputs'])} audio assets")
        return
    if args.banks is None:
        parser.error("--banks is required when rebuilding; first use fetch-league-audio.py")
    riot = json.loads((ROOT / "assets/audio/riot-sources.json").read_text())
    for source in riot["files"]:
        path = args.banks / source["file"]
        if not path.exists() or sha256(path) != source["sha256"]:
            raise SystemExit(f"Missing or unverified bank source: {path}")

    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="kiva-audio-import-") as temporary:
        cache = args.cache or Path(temporary)
        cache.mkdir(parents=True, exist_ok=True)
        banks = {}
        for name, definition in manifest["banks"].items():
            directory = cache / name
            directory.mkdir(exist_ok=True)
            banks[name] = SoundBank(definition, args.banks, directory, args.vgmstream)
        shared = {}
        for name, source in manifest["shared_sources"].items():
            path = cache / name
            if not path.exists():
                request = urllib.request.Request(source["url"], headers={
                    "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
                    "(KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                })
                with urllib.request.urlopen(request, timeout=45) as response:
                    path.write_bytes(response.read(1024 * 1024))
            if sha256(path) != source["sha256"]:
                raise SystemExit(f"Source hash mismatch: {name}")
            shared[name] = path

        for cue in manifest["outputs"]:
            paths = []
            media = []
            for event in cue.get("events", []):
                bank = banks[event["bank"]]
                ids = bank.layers(event["name"], cue.get("variation", 0))
                paths.extend(bank.decode(media_id) for media_id in ids)
                media.extend({"bank": event["bank"], "event": event["name"], "wem_id": item} for item in ids)
            paths.extend(shared[name] for name in cue.get("shared", []))
            if not paths:
                raise SystemExit(f"No audio for {cue['file']}")
            command = ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y"]
            for path in paths:
                command.extend(["-i", str(path)])
            normal = ("silenceremove=start_periods=1:start_threshold=-55dB,"
                      "areverse,silenceremove=start_periods=1:start_threshold=-55dB,areverse")
            filters = [f"[{i}:a]{normal}[s{i}]" for i in range(len(paths))]
            if len(paths) > 1:
                inputs = "".join(f"[s{i}]" for i in range(len(paths)))
                filters.append(f"{inputs}amix=inputs={len(paths)}:duration=longest:normalize=0[mixed]")
                mixed = "mixed"
            else:
                mixed = "s0"
            duration = cue["max_seconds"]
            filters.append(
                f"[{mixed}]atrim=duration={duration},afade=t=out:st={duration - 0.08}:d=0.08,"
                "loudnorm=I=-21:TP=-3:LRA=7,aresample=48000[out]"
            )
            path = args.output / cue["file"]
            command.extend([
                "-filter_complex", ";".join(filters), "-map", "[out]", "-ac", "1",
                "-map_metadata", "-1", "-fflags", "+bitexact", "-flags:a", "+bitexact",
                "-c:a", "libvorbis", "-q:a", "4", str(path),
            ])
            subprocess.run(command, check=True)
            subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-i", str(path),
                            "-f", "null", "-"], check=True)
            digest = sha256(path)
            if args.record:
                cue["sha256"] = digest
                cue["media"] = media
            elif digest != cue["sha256"]:
                raise SystemExit(f"Rebuilt asset differs: {path}; check ffmpeg version before --record")
            print(f"{path.name}: {path.stat().st_size} bytes {digest}")

    if args.record:
        manifest["encoder"] = subprocess.check_output(["ffmpeg", "-version"], text=True).splitlines()[0]
        manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    run()
