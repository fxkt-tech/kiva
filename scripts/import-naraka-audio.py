#!/usr/bin/env python3
"""Prepare/import/verify the version-pinned, original Naraka effect set.

Prepare reads only the selected original PCK banks and requires the pinned
vgmstream decoder. Normal import/--verify use frozen lossless PCM, stdlib Python
and ffmpeg/ffprobe; neither needs the game installation or vgmstream.
"""
from __future__ import annotations

import argparse
import gzip
import json
from pathlib import Path
import struct
import subprocess
import tempfile

from naraka_audio import (
    DECODER_SHA, PEAK_LIMIT, atomic_write, check_file_list, check_wem, decode_wem,
    encode, event_layers, file_sha, fnv, json_bytes, read_bank, read_pck, require,
    safe_file, sha, source_media, unpack_pcm,
)

ROOT = Path(__file__).resolve().parents[1]
PREPARED = ROOT / "assets/prepared/naraka/audio"
OUTPUT = ROOT / "assets/audio"
EFFECT_GAIN = 0.45
AUDIO_RELATIVE = Path("NarakaBladepoint_Data/StreamingAssets/Audio/GeneratedSoundBanks/Windows")
EVIDENCE = [
    "audio_sample_trace.json", "audio-tracks-selected.json",
    "audio-dbbl-verification.json", "audio-config-selected.json",
]
# These are Kiva cue assignments. They do not reconstruct original Wwise RTPC,
# material switching, action cancellation, distance buses or weighted mode.
CUES = [
    ("naraka_attack_1", "gman_hengdao_attack_light_01", 2, "Original horizontal stage 1"),
    ("naraka_attack_2", "gman_hengdao_attack_light_02", 2, "Original horizontal stage 2"),
    ("naraka_attack_3", "gman_hengdao_attack_light_03", 2, "Original horizontal stage 3"),
    ("naraka_riposte", "gman_hengdao_attack_shock_light_01", 2, "Original riposte whoosh; not success collision"),
    ("naraka_hit", "impact_body", 2, "Original default impact switch; armor/strength names unknown"),
    ("naraka_deflect", "gman_all_break_hurt", 3, "Original shared blade clash, adapted to settled deflect"),
    ("naraka_block", "gman_all_break_hurt", 3, "Same shared clash, adapted to settled block"),
    ("naraka_parry", "gman_all_weapon_touch_shock_katana", 3,
     "Shared original metal collision, adapted only to settled parry"),
    ("naraka_jump", "move_jump_takeoff_cloth_fast", 1, "Actual jump track's takeoff cloth event"),
    ("naraka_land", "move_jump_land", 1, "Both original PlayActions; first delayed 100ms"),
    ("naraka_dodge", "gman_dodge_01", 1, "Original four-direction dodge track event"),
    ("naraka_dodge_success", "move_soul_dodge_1p", 3,
     "User-selected soul dodge event adapted to settled successful dodge; not proof of original PVP usage"),
    ("naraka_step", "move_run", 0, "Original default material branch; Kiva actual-distance cadence"),
    ("naraka_death", "gman_all_die_drop", 2, "Original shared body/drop event; Kiva death assignment"),
    ("naraka_death_voice", "gman_dying_voice_male", 2,
     "Optional shared male death voice; not Li Huowang-specific or a female voice"),
    ("naraka_gun", "gman_fk_attack_01", 2,
     "Original FK-family firing event from male_fk_attack_01; Kiva musket adaptation, not skin-exclusive"),
    ("naraka_bow", "gman_bow_attack_01", 2,
     "Original male_bow_attack_01 firing event; shared bow-family adaptation"),
]
REFERENCE = {
    "project": "https://github.com/bnnm/wwiser",
    "commit": "6d4b5bfb45afc52cda9d68fbbe0cbd97de0e26a3",
    "action_property": "v135 AkPropID_128: 0x0F DelayTime, signed milliseconds",
    "references": [
        {"path": "wwiser/parser/wdefs.py",
         "sha256": "8eb5e5450731d5e00f2e1cae9545c93381fc01a78c8aff27a2751841cd2d27d2"},
        {"path": "wwiser/generator/render/bnode_props.py",
         "sha256": "e9bfe9a8f3e2ea223ea8f7dba29c8921fbd7d220e62acaab61ba59bc5d751305"},
    ],
    "note": "Format reference only; this importer does not execute or depend on wwiser.",
}

README = """# 永劫无间音效冻结准备包

原件来自用户本机NARAKA BLADEPOINT，版本由PCK目录/已选bank/WEM与原配置SHA固定。
source-evidence保留原动作到声音轨、中文引用配置、原表重解验证与原HIRC媒体链。
banks只保留实际采用事件经过的原HIRC对象，并保留完整对象ID/type索引验证引用；
sources是未改写的完整Wwise Vorbis WEM；pcm是完整PCM16LE的gzip无损副本。

无归一化、裁剪、拼接、变速或变调；运行时gain只提供固定峰值余量。
每层独立选原变体，保留所有选中分支的随机权重。地表/impact选原default；
未恢复RTPC、原random模式、完整混响总线或proto轨time单位，不冒称完整Wwise复刻。
落地有两个原PlayAction；第一层有真实100ms DelayTime，第二层0ms。
该bank延迟与未知单位的proto轨float是两种不同数据。
远程新增原FK射击与弓射击事件。FK标准分支与Barrett条件分支区分，
不将未知条件数字称皮肤ID，不把无Sound轨的sniper片段说成已恢复声音连接。
枪/弓cue均为武器家族适配，不是引冰落/羿之影或角色专属音效。
成功闪避使用用户选定的move_soul_dodge_1p，保留三个原变体；这是Kiva用途适配，非原作PVP触发语义的确证。

普通import/--verify从本包无损PCM重建Ogg并验证完整媒体链、哈希与格式，
不需要客户端、Unity或vgmstream。需要Python3.11+和ffmpeg/ffprobe。
--redecode另从原WEM调用指定、固定SHA的vgmstream，核对PCM样本；
这不等于普通verify，也不恢复原Wwise运行时。

运行时只复制assets/audio/naraka-*，此准备包不编入游戏/WASM。
原attack/hit/deflect和BGM不属于本导入器，不会被写入。
"""


def prepare(install, evidence, decoder, output):
    evidence = Path(evidence).resolve()
    recipe_bytes = {name: safe_file(evidence, name).read_bytes() for name in EVIDENCE}
    trace = json.loads(recipe_bytes["audio_sample_trace.json"])
    require(trace["decoder_sha256"] == DECODER_SHA, "unrecognized source evidence decoder")
    chains = {row["event"]: row for row in trace["chains"]}
    names = {event for _, event, _, _ in CUES}
    require(names <= chains.keys(), "missing frozen event identity recipe")
    packages, banks, media, events, bank_nodes = {}, {}, {}, {}, {}
    audio_root = Path(install).resolve() / AUDIO_RELATIVE
    with tempfile.TemporaryDirectory(prefix="kiva-naraka-audio-prepare-") as temporary:
        staging = Path(temporary) / "prepared"
        work = Path(temporary) / "work"
        work.mkdir()
        for name, data in recipe_bytes.items():
            atomic_write(staging / "source-evidence" / name, data)
        for name in sorted(names):
            recipe = chains[name]
            package_name = Path(recipe["package"]["file"]).name
            require(package_name in ["SoundActor1.pck", "SoundActor2.pck"], "unexpected source package")
            if package_name not in packages:
                package = read_pck(audio_root / package_name)
                require(all(package[k] == recipe["package"][k]
                            for k in ["file_bytes", "header_bytes", "header_sha256"]),
                        "pinned source PCK directory changed")
                packages[package_name] = package
            bank_name = recipe["bank"]
            if bank_name not in banks:
                require(fnv(bank_name) == recipe["bank_id"], "bank name/hash mismatch")
                bank = read_bank(audio_root / package_name, packages[package_name],
                                 recipe["bank_id"], recipe["bank_sha256"])
                require(bank["entry"] == recipe["bank_entry"], "pinned bank physical entry changed")
                bank["package"] = {k: v for k, v in packages[package_name].items() if k != "entries"}
                banks[bank_name] = bank
                bank_nodes[bank_name] = set()
            bank = banks[bank_name]
            layers, seen = event_layers(bank["objects"], bank["object_index"], bank["bank_id"], name)
            require([x["action_id"] for x in layers] == [x["action_id"] for x in recipe["paths"]],
                    "live source action identities differ from original evidence")
            bank_nodes[bank_name].update(seen)
            for layer in layers:
                source_sounds = []
                for sound in layer["sounds"]:
                    info, wem = source_media(bank["objects"][sound["id"]], bank)
                    ident = str(info["source_id"])
                    require(any(x["source_id"] == info["source_id"] and x["source_sha256"] == sha(wem)
                                for x in recipe["media"]), "source media differs from pinned trace")
                    origin = dict(bank=bank_name, **info)
                    if ident not in media:
                        wem_path = staging / "sources" / f"{ident}.wem"
                        atomic_write(wem_path, wem)
                        pcm, pcm_info = decode_wem(wem_path, decoder, work)
                        require((pcm_info["rate"], pcm_info["channels"]) == (info["rate"], info["channels"]),
                                "decoded PCM disagrees with source WEM fmt")
                        pcm_path = f"pcm/{ident}.s16le.gz"
                        atomic_write(staging / pcm_path, gzip.compress(pcm, compresslevel=9, mtime=0))
                        media[ident] = dict(source_id=info["source_id"], wem=f"sources/{ident}.wem",
                                            wem_bytes=len(wem), wem_sha256=sha(wem),
                                            pcm=pcm_path, decoded=pcm_info, origins=[])
                    require(media[ident]["wem_sha256"] == sha(wem), "global media ID collision")
                    if origin not in media[ident]["origins"]:
                        media[ident]["origins"].append(origin)
                    source_sounds.append(dict(sound_id=sound["id"], source_id=info["source_id"],
                                              weight=sound["weight"]))
                layer["sounds"] = source_sounds
            events[name] = dict(event_id=fnv(name), bank=bank_name,
                                original_event_record_sha256=recipe["event_record_sha256"], layers=layers)
        bank_files = {}
        for name, bank in sorted(banks.items()):
            path = f"banks/{name}.json"
            data = {k: bank[k] for k in ["bank_id", "version", "sha256", "entry", "package", "object_index"]}
            data["objects"] = [bank["objects"][i] for i in sorted(bank_nodes[name])]
            atomic_write(staging / path, json_bytes(data))
            bank_files[name] = path
        atomic_write(staging / "README.md", README.encode())
        atomic_write(staging / ".gitattributes", b"* -text\n")
        manifest = dict(
            schema_version=1,
            origin="User-specified local NARAKA BLADEPOINT; selected original audio, not full Wwise runtime",
            source_install_historical=str(Path(install).resolve()),
            source_wwise_banks_dbbl_sha256=trace["source_dbbl_sha256"],
            banks=bank_files, events=events, media=[media[k] for k in sorted(media, key=int)],
            decoder=dict(tool="vgmstream-r2168/vgmstream-cli", sha256=DECODER_SHA, arguments=["-i"]),
            format_reference=REFERENCE,
            producer=dict(script="scripts/import-naraka-audio.py", module="scripts/naraka_audio.py",
                          python="3.11+ stdlib", decoder_required_only_for_prepare_or_redecode=True),
            files=[dict(path=f.relative_to(staging).as_posix(), bytes=f.stat().st_size, sha256=file_sha(f))
                   for f in sorted(staging.rglob("*")) if f.is_file()],
        )
        atomic_write(staging / "prepared-manifest.json", json_bytes(manifest))
        load_prepared(staging)
        output = Path(output).resolve()
        if output.exists():
            existing = {f.relative_to(output).as_posix() for f in output.rglob("*") if f.is_file()}
            wanted = {f.relative_to(staging).as_posix() for f in staging.rglob("*") if f.is_file()}
            require(existing <= wanted, "refusing to replace a prepared directory with unrelated files")
        for file in sorted(staging.rglob("*")):
            if file.is_file():
                atomic_write(output / file.relative_to(staging), file.read_bytes())
    return dict(prepared=str(output), events=len(events), media=len(media),
                source_wem_bytes=sum(m["wem_bytes"] for m in media.values()))


def load_prepared(source):
    source = Path(source).resolve()
    manifest = json.loads(safe_file(source, "prepared-manifest.json").read_bytes())
    require(manifest["schema_version"] == 1, "unknown prepared audio version")
    check_file_list(source, manifest)
    media = {row["source_id"]: row for row in manifest["media"]}
    require(len(media) == len(manifest["media"]) and len(media) <= 64, "duplicate/media count budget")
    for ident, row in media.items():
        require(type(ident) is int and 0 <= ident <= 0xFFFFFFFF, "invalid source media ID")
        wem = safe_file(source, row["wem"]).read_bytes()
        fmt = check_wem(wem)
        require(len(wem) == row["wem_bytes"] and sha(wem) == row["wem_sha256"], "frozen WEM changed")
        require((fmt["channels"], fmt["rate"]) == (row["decoded"]["channels"], row["decoded"]["rate"]),
                "frozen WEM/PCM layout differs")
        unpack_pcm(safe_file(source, row["pcm"]), row["decoded"])
    banks = {}
    for name, path in manifest["banks"].items():
        bank = json.loads(safe_file(source, path).read_bytes())
        require(bank["version"] == 135 and bank["bank_id"] == fnv(name), "frozen bank identity changed")
        bank["objects"] = {obj["id"]: obj for obj in bank["objects"]}
        banks[name] = bank
    for name, event in manifest["events"].items():
        bank = banks[event["bank"]]
        layers, _ = event_layers(bank["objects"], bank["object_index"], bank["bank_id"], name)
        for layer in layers:
            converted = []
            for sound in layer["sounds"]:
                obj = bank["objects"][sound["id"]]
                plugin, stream, ident, memory, bits = struct.unpack_from("<IBIIB", bytes.fromhex(obj["body_hex"]))
                require(plugin == 0x40001 and stream == 0 and bits & 1 == 0, "unsupported frozen sound")
                require(ident in media and media[ident]["wem_bytes"] == memory, "frozen sound/media mismatch")
                require(any(o["bank"] == event["bank"] and o["sound_id"] == sound["id"]
                            and o["source_id"] == ident for o in media[ident]["origins"]),
                        "frozen media is missing this original Sound identity")
                converted.append(dict(sound_id=sound["id"], source_id=ident, weight=sound["weight"]))
            layer["sounds"] = converted
        require(layers == event["layers"] and event["event_id"] == fnv(name), "frozen event graph changed")
    require({event for _, event, _, _ in CUES} == set(manifest["events"]), "unexpected cue event set")
    return manifest, media


def build_runtime(source, output, verify, decoder=None):
    source, output = Path(source).resolve(), Path(output).resolve()
    manifest, media = load_prepared(source)
    measurements, produced = {}, {}
    with tempfile.TemporaryDirectory(prefix="kiva-naraka-audio-build-") as temporary:
        work = Path(temporary)
        for ident, row in media.items():
            pcm = unpack_pcm(safe_file(source, row["pcm"]), row["decoded"])
            if decoder is not None:
                decoded, info = decode_wem(safe_file(source, row["wem"]), decoder, work)
                require(decoded == pcm and info["sha256"] == row["decoded"]["sha256"],
                        "fresh WEM decode differs from frozen complete PCM")
            data, encoded = encode(pcm, row["decoded"], work)
            name = f"naraka-{ident}.ogg"
            produced[name] = data
            measurements[ident] = dict(source_id=ident, file=name, **encoded)
        cues = {}
        for cue, event_name, priority, note in CUES:
            event = manifest["events"][event_name]
            layers = []
            for layer in event["layers"]:
                variants = [dict(file=measurements[s["source_id"]]["file"], weight=s["weight"],
                                 gain=measurements[s["source_id"]]["playback_gain"])
                            for s in layer["sounds"]]
                require(1 <= len(variants) <= 16 and 0 <= layer["delay"] <= 2, "cue layer budget")
                layers.append(dict(delay=layer["delay"], variants=variants))
            cues[cue] = dict(priority=priority, offset=0, layers=layers)
        produced["naraka-cues.json"] = json_bytes(dict(version=1, cues=cues))
        runtime = dict(
            schema_version=1, generator="scripts/import-naraka-audio.py",
            prepared_manifest_sha256=file_sha(source / "prepared-manifest.json"),
            source_origin=manifest["origin"],
            encoding=dict(format="Vorbis/Ogg", quality=4, natural_pitch=True,
                          crop=False, normalize=False, splice=False,
                          command=["ffmpeg", "-nostdin", "-v", "error", "-y", "-f", "s16le",
                                   "-ar", "<source rate>", "-ac", "<source channels>", "-i", "source.s16le",
                                   "-map", "0:a:0", "-map_metadata", "-1", "-c:a", "libvorbis", "-q:a", "4",
                                   "-fflags", "+bitexact", "output.ogg"],
                          ffmpeg_version=subprocess.run(["ffmpeg", "-version"], check=True,
                                                        capture_output=True, text=True).stdout.splitlines()[0],
                          decoder=manifest["decoder"]),
            playback=dict(global_effect_gain=EFFECT_GAIN, max_sources=4, priority="larger number is more important",
                          peak_limit_per_source_after_variant_gain=PEAK_LIMIT,
                          max_four_effects_peak_bound=4 * EFFECT_GAIN * PEAK_LIMIT,
                          variant_gain="min(1,0.85/measured complete Vorbis float peak); source PCM/Ogg unchanged",
                          proto_track_time="unknown units/group semantics; offset0 is explicit Kiva adaptation",
                          layer_delay="seconds converted from actual original PlayAction DelayTime milliseconds"),
            cues={cue: dict(original_event=event, note=note) for cue, event, _, note in CUES},
            media=list(measurements.values()),
            evidence=[
                dict(path=f"assets/prepared/naraka/audio/source-evidence/{name}",
                     sha256=file_sha(source / "source-evidence" / name)) for name in EVIDENCE
            ],
            limits=[
                "Original default switches selected; material/armor/strength labels remain unknown.",
                "Original random mode, RTPC, positioning, buses and reverb are not reconstructed.",
                "Kiva picks one weighted original variant per layer; land keeps two separate PlayActions.",
                "Shared clash/shock assigned to Kiva settled defense; not proof of original success rules.",
                "Optional generic male death voice is not character-specific and must not be used as a female voice.",
                "Gun/bow are original weapon-family firing events, not skin-exclusive; sniper clip audio mapping is unproven.",
                "move_run and move_sprint original default variants are the same source IDs; no two-set claim.",
                "Signal/decode checks do not replace actual browser AudioContext and listening acceptance.",
            ],
        )
        produced["naraka-manifest.json"] = json_bytes(runtime)
        for name, data in produced.items():
            require(name.startswith("naraka-") and "/" not in name, "runtime ownership violation")
            path = output / name
            if verify:
                require(path.is_file() and not path.is_symlink() and path.read_bytes() == data,
                        f"runtime rebuild mismatch: {name}")
            else:
                atomic_write(path, data)
        actual = {f.name for f in output.glob("naraka-*") if f.is_file()}
        require(actual == set(produced), "unexpected/missing naraka runtime files")
    return dict(verified=verify, events=len(manifest["events"]), cues=len(cues), media=len(media),
                ogg_bytes=sum(len(v) for k, v in produced.items() if k.endswith(".ogg")),
                prepared_bytes=sum(f.stat().st_size for f in source.rglob("*") if f.is_file()),
                max_layer_sources=max(len(v["layers"]) for v in cues.values()),
                maximum_ogg_duration=max(v["duration"] for v in measurements.values()),
                max_effect_mix_peak=4 * EFFECT_GAIN * PEAK_LIMIT, redecode=decoder is not None)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--prepare", type=Path, metavar="INSTALL")
    mode.add_argument("--verify", type=Path, metavar="PREPARED")
    parser.add_argument("--source", type=Path, default=PREPARED)
    parser.add_argument("--output", type=Path, default=OUTPUT)
    parser.add_argument("--prepared-output", type=Path, default=PREPARED)
    parser.add_argument("--evidence", type=Path)
    parser.add_argument("--decoder", type=Path)
    parser.add_argument("--redecode", action="store_true")
    args = parser.parse_args()
    if args.prepare:
        require(args.evidence is not None and args.decoder is not None,
                "--prepare requires --evidence and --decoder")
        result = prepare(args.prepare, args.evidence, args.decoder, args.prepared_output)
    else:
        require(not args.redecode or args.decoder is not None, "--redecode requires --decoder")
        result = build_runtime(args.verify or args.source, args.output, args.verify is not None,
                               args.decoder if args.redecode else None)
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
