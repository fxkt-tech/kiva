"""Bounded readers and deterministic conversion for the pinned Naraka audio set.

Only the observed Wwise 135 Play/Random/default-Switch/embedded-Vorbis subset is
supported. Unknown action properties and ambiguous containers fail explicitly.
"""
from __future__ import annotations

import array
from fractions import Fraction
import gzip
import hashlib
import json
import math
from pathlib import Path, PurePosixPath
import struct
import subprocess
import sys
import tempfile
import wave

MAX_BANK = 32 * 1024 * 1024
MAX_PCM = 16 * 1024 * 1024
MAX_SOURCE = 2 * 1024 * 1024
MAX_HEADER = 4 * 1024 * 1024
PEAK_LIMIT = 0.85
DECODER_SHA = "e736da315053bf91796d01eca18b54cbcc2d5b328eecdaf15786e7838251f581"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n").encode()


class Reader:
    def __init__(self, data, at=0):
        self.data, self.at = data, at

    def take(self, count):
        require(0 <= count <= len(self.data) - self.at, "binary read outside declared range")
        result = self.data[self.at:self.at + count]
        self.at += count
        return result

    def u8(self):
        return self.take(1)[0]

    def u16(self):
        return struct.unpack("<H", self.take(2))[0]

    def u32(self):
        return struct.unpack("<I", self.take(4))[0]

    def s32(self):
        return struct.unpack("<i", self.take(4))[0]

    def finish(self):
        require(self.at == len(self.data), "unexpected trailing binary fields")


def fnv(name):
    value = 2166136261
    for byte in name.lower().encode():
        value = ((value * 16777619) ^ byte) & 0xFFFFFFFF
    return value


def chunks(data, start=0):
    reader = Reader(data, start)
    while reader.at < len(data):
        at = reader.at
        tag = reader.take(4)
        size = reader.u32()
        yield tag, at, reader.take(size)
    reader.finish()


def read_pck(path):
    path = Path(path)
    with path.open("rb") as stream:
        first = stream.read(28)
        require(len(first) == 28, "truncated PCK header")
        magic, size, version, language, banks, media, external = struct.unpack("<4s6I", first)
        require(magic == b"AKPK" and version == 1, "unsupported PCK version")
        require(20 <= size <= MAX_HEADER, "PCK header size exceeds budget")
        rest = stream.read(size + 8 - 28)
    raw = first + rest
    require(len(raw) == size + 8, "truncated PCK directory")
    require(28 + language + banks + media + external == len(raw), "PCK directory extent mismatch")
    entries = []
    at = 28 + language
    for section, length in [("banks", banks), ("media", media), ("external", external)]:
        reader = Reader(raw[at:at + length])
        count = reader.u32()
        require(count <= 100000 and length == 4 + count * 20, "PCK entry count/length mismatch")
        for _ in range(count):
            ident, alignment, count_bytes, offset, language_id = (reader.u32() for _ in range(5))
            require(alignment > 0, "PCK alignment must be positive")
            physical = offset * alignment
            require(count_bytes <= path.stat().st_size - physical, "PCK entry exceeds file")
            entries.append(dict(section=section, id=ident, alignment=alignment,
                                bytes=count_bytes, offset=physical, language=language_id))
        reader.finish()
        at += length
    return dict(file=path.name, file_bytes=path.stat().st_size, header_bytes=len(raw),
                header_sha256=sha(raw), entries=entries)


def read_bank(path, package, bank_id, expected_sha):
    entries = [row for row in package["entries"]
               if row["section"] == "banks" and row["id"] == bank_id and row["language"] == 0]
    require(len(entries) == 1, "expected one unlocalized PCK bank")
    entry = entries[0]
    require(entry["bytes"] <= MAX_BANK, "selected bank exceeds read budget")
    with Path(path).open("rb") as stream:
        stream.seek(entry["offset"])
        data = stream.read(entry["bytes"])
    require(len(data) == entry["bytes"] and sha(data) == expected_sha, "pinned source bank changed")
    parts = list(chunks(data))
    tags = [tag for tag, _, _ in parts]
    require(all(tags.count(tag) == 1 for tag in [b"BKHD", b"HIRC", b"DIDX", b"DATA"]),
            "missing/duplicate source bank chunks")
    header = next(body for tag, _, body in parts if tag == b"BKHD")
    require(struct.unpack_from("<II", header) == (135, bank_id), "expected pinned Wwise 135 bank")
    hirc = Reader(next(body for tag, _, body in parts if tag == b"HIRC"))
    count = hirc.u32()
    require(count <= 50000, "HIRC object budget exceeded")
    objects = {}
    for _ in range(count):
        offset = hirc.at
        kind, length = hirc.u8(), hirc.u32()
        require(4 <= length <= MAX_BANK, "invalid HIRC object size")
        value = Reader(hirc.take(length))
        ident = value.u32()
        require(ident not in objects, "duplicate HIRC object ID")
        objects[ident] = dict(id=ident, type=kind, offset=offset,
                              bytes=length, body_hex=value.take(length - 4).hex())
    hirc.finish()
    media = {}
    didx = next(body for tag, _, body in parts if tag == b"DIDX")
    require(len(didx) % 12 == 0, "invalid DIDX stride")
    data_at, data_body = next((at + 8, body) for tag, at, body in parts if tag == b"DATA")
    for ident, offset, size in struct.iter_unpack("<III", didx):
        require(ident not in media and 0 < size <= MAX_SOURCE, "invalid/duplicate DIDX media")
        require(offset + size <= len(data_body), "DIDX media exceeds DATA chunk")
        media[ident] = dict(id=ident, offset=offset, bytes=size)
    return dict(bank_id=bank_id, version=135, sha256=sha(data), entry=entry,
                object_index=[dict(id=o["id"], type=o["type"]) for o in objects.values()],
                objects=objects, media=media, data_offset=data_at, data=data_body)


def play_action(obj, bank_id):
    require(obj["type"] == 3, "expected PlayAction")
    reader = Reader(bytes.fromhex(obj["body_hex"]))
    require(reader.u16() == 0x403, "unsupported action type")
    target = reader.u32()
    require(reader.u8() == 0, "bus action is not a supported sound target")
    count = reader.u8()
    require(count <= 1, "unsupported PlayAction property count")
    ids = list(reader.take(count))
    values = [reader.s32() for _ in ids]
    require(len(set(ids)) == len(ids) and all(i == 0x0F for i in ids),
            "unsupported PlayAction property")
    props = dict(zip(ids, values))
    delay_ms = props.get(0x0F, 0)
    require(0 <= delay_ms <= 2000, "PlayAction delay exceeds two seconds")
    require(reader.u8() == 0, "ranged PlayAction properties are not implemented")
    fade_curve = reader.u8()
    require(fade_curve <= 31 and reader.u32() == bank_id, "PlayAction bank/curve mismatch")
    reader.finish()
    return dict(target=target, delay=delay_ms / 1000,
                original_delay_ms=delay_ms, fade_curve=fade_curve)


def random_tail(obj, object_ids):
    data = bytes.fromhex(obj["body_hex"])
    found = []
    for count in range(1, min(512, len(object_ids)) + 1):
        at = len(data) - (4 + count * 4 + 2 + count * 8)
        if at < 4 or struct.unpack_from("<I", data, at)[0] != count:
            continue
        children = list(struct.unpack_from("<" + "I" * count, data, at + 4))
        playlist_at = at + 4 + count * 4
        if struct.unpack_from("<H", data, playlist_at)[0] != count:
            continue
        playlist = [dict(id=i, weight=w) for i, w in struct.iter_unpack("<Ii", data[playlist_at + 2:])]
        if (len(set(children)) == count and set(children) <= object_ids
                and set(children) == {p["id"] for p in playlist}
                and all(p["weight"] > 0 for p in playlist)):
            found.append(dict(offset=at, children=children, playlist=playlist))
    require(len(found) == 1, "random container tail is not uniquely bounded")
    return found[0]


def switch_tail(obj, object_ids):
    data = bytes.fromhex(obj["body_hex"])
    found = []
    for at in range(4, len(data) - 14):
        try:
            r = Reader(data, at)
            kind, group, default, continuous = r.u8(), r.u32(), r.u32(), r.u8()
            require(kind in [0, 1] and continuous in [0, 1], "invalid switch header")
            count = r.u32()
            require(1 <= count <= 512, "switch child budget")
            children = [r.u32() for _ in range(count)]
            require(len(set(children)) == count and set(children) <= object_ids, "switch child IDs")
            group_count = r.u32()
            require(1 <= group_count <= 512, "switch group budget")
            groups = []
            for _ in range(group_count):
                ident, size = r.u32(), r.u32()
                require(size <= count, "switch branch budget")
                ids = [r.u32() for _ in range(size)]
                require(set(ids) <= set(children), "switch branch child")
                groups.append(dict(switch=ident, children=ids))
            require(len({g["switch"] for g in groups}) == group_count, "duplicate switch branch")
            param_count = r.u32()
            require(param_count <= count, "switch parameter budget")
            params = []
            for _ in range(param_count):
                ident, flags, on_switch = r.u32(), r.u8(), r.u8()
                out_ms, in_ms = r.s32(), r.s32()
                require(ident in children and flags <= 3 and on_switch <= 7, "switch parameters")
                params.append(dict(node=ident, flags=flags, on_switch=on_switch,
                                   fade_out=out_ms, fade_in=in_ms))
            r.finish()
            require(default in {g["switch"] for g in groups}, "missing default switch")
            found.append(dict(offset=at, group_type=kind, group_id=group,
                              default_switch=default, continuous=continuous,
                              children=children, groups=groups, params=params))
        except (ValueError, struct.error):
            continue
    require(len(found) == 1, "switch container tail is not uniquely bounded")
    return found[0]


def event_layers(objects, object_index, bank_id, event_name):
    event_id = fnv(event_name)
    require(event_id in objects and objects[event_id]["type"] == 4, "event identity/type mismatch")
    reader = Reader(bytes.fromhex(objects[event_id]["body_hex"]))
    count = reader.u8()
    require(1 <= count <= 4, "event action budget")
    actions = [reader.u32() for _ in range(count)]
    reader.finish()
    require(len(set(actions)) == count, "duplicate PlayAction")
    seen = {event_id}
    index_ids = {o["id"] for o in object_index}

    def visit(ident, path):
        require(len(path) <= 16 and ident not in path and ident in objects,
                "container cycle/depth/missing selected node")
        obj = objects[ident]
        seen.add(ident)
        if obj["type"] == 2:
            return [(ident, Fraction(1))], []
        if obj["type"] == 5:
            tail = random_tail(obj, index_ids)
            weights = {p["id"]: p["weight"] for p in tail["playlist"]}
            total = sum(weights.values())
            chosen = [(i, Fraction(weights[i], total)) for i in tail["children"]]
        elif obj["type"] == 6:
            tail = switch_tail(obj, index_ids)
            chosen_ids = next(g["children"] for g in tail["groups"]
                              if g["switch"] == tail["default_switch"])
            require(len(chosen_ids) == 1, "multi-child default switch needs explicit layering")
            chosen = [(chosen_ids[0], Fraction(1))]
        else:
            raise ValueError(f"unsupported source container type {obj['type']}")
        sounds, branches = [], [dict(id=ident, type=obj["type"], tail=tail)]
        for child, weight in chosen:
            leaves, more = visit(child, path + [ident])
            sounds.extend((i, w * weight) for i, w in leaves)
            branches.extend(more)
        require(len(sounds) <= 128, "event variant budget exceeded")
        return sounds, branches

    layers = []
    for action_id in actions:
        require(action_id in objects, "missing PlayAction")
        seen.add(action_id)
        action = play_action(objects[action_id], bank_id)
        sounds, branches = visit(action["target"], [])
        require(len({i for i, _ in sounds}) == len(sounds), "ambiguous repeated terminal sound")
        layers.append(dict(action_id=action_id, **action, containers=branches,
                           sounds=[dict(id=i, weight=float(weight)) for i, weight in sounds]))
    return layers, seen


def source_media(obj, bank):
    require(obj["type"] == 2, "expected Sound")
    reader = Reader(bytes.fromhex(obj["body_hex"]))
    plugin, stream, ident, memory, bits = reader.u32(), reader.u8(), reader.u32(), reader.u32(), reader.u8()
    require(plugin == 0x40001 and stream == 0 and bits & 1 == 0,
            "expected embedded Wwise Vorbis Sound")
    require(ident in bank["media"], "Sound has no DIDX entry")
    slot = bank["media"][ident]
    require(slot["bytes"] == memory, "Sound memory size differs from DIDX")
    data = bank["data"][slot["offset"]:slot["offset"] + slot["bytes"]]
    fmt = check_wem(data)
    return dict(source_id=ident, sound_id=obj["id"], bytes=len(data), sha256=sha(data),
                bank_data_offset=bank["data_offset"] + slot["offset"],
                physical_offset=bank["entry"]["offset"] + bank["data_offset"] + slot["offset"],
                plugin_id=plugin, stream_type=stream, **fmt), data


def check_wem(data):
    require(12 <= len(data) <= MAX_SOURCE and data[:4] == b"RIFF" and data[8:12] == b"WAVE",
            "source is not a complete bounded WEM RIFF")
    require(struct.unpack_from("<I", data, 4)[0] + 8 == len(data), "WEM RIFF size mismatch")
    parts = list(chunks(data, 12))
    require(sum(tag == b"fmt " for tag, _, _ in parts) == 1, "WEM fmt missing/duplicate")
    fmt = next(body for tag, _, body in parts if tag == b"fmt ")
    require(len(fmt) >= 8, "truncated WEM fmt")
    codec, channels, rate = struct.unpack_from("<HHI", fmt)
    require(codec == 0xFFFF and channels in [1, 2] and 8000 <= rate <= 192000,
            "unsupported source codec/channel/rate")
    return dict(codec=codec, channels=channels, rate=rate)


def run(command):
    return subprocess.run([str(x) for x in command], check=True, capture_output=True)


def decode_wem(wem, decoder, directory):
    require(file_sha(decoder) == DECODER_SHA, "expected pinned vgmstream-r2168 decoder")
    wav = Path(directory) / "decoded.wav"
    result = run([decoder, "-i", "-o", wav, wem])
    with wave.open(str(wav), "rb") as stream:
        require(stream.getsampwidth() == 2 and stream.getnchannels() in [1, 2],
                "decoder must output mono/stereo PCM16")
        frames, rate, channels = stream.getnframes(), stream.getframerate(), stream.getnchannels()
        require(0 < frames <= rate * 15, "decoded media duration exceeds budget")
        pcm = stream.readframes(frames)
    require(len(pcm) == frames * channels * 2 <= MAX_PCM, "decoded PCM size mismatch")
    stats = pcm_measurement(pcm)
    return pcm, dict(frames=frames, rate=rate, channels=channels, sample_width=2,
                     sha256=sha(pcm), wav_sha256=file_sha(wav), **stats,
                     decoder_output=result.stdout.decode(errors="replace")
                     .replace(str(wav), "decoded.wav").replace(str(wem), "source.wem"))


def pcm_measurement(pcm):
    values = array.array("h", pcm)
    if sys.byteorder != "little":
        values.byteswap()
    require(bool(values), "empty PCM")
    peak = max(abs(x) for x in values) / 32768
    rms = math.sqrt(sum(x * x for x in values) / len(values)) / 32768
    require(peak > 0 and math.isfinite(rms), "silent/nonfinite media")
    return dict(peak=peak, rms_dbfs=20 * math.log10(rms))


def unpack_pcm(path, metadata):
    require(type(metadata["frames"]) is int and type(metadata["rate"]) is int
            and type(metadata["channels"]) is int, "PCM dimensions must be integers")
    require(8000 <= metadata["rate"] <= 192000 and metadata["channels"] in [1, 2]
            and 0 < metadata["frames"] <= metadata["rate"] * 15
            and metadata["sample_width"] == 2, "invalid frozen PCM dimensions")
    size = metadata["frames"] * metadata["channels"] * 2
    require(size <= MAX_PCM, "frozen PCM exceeds memory budget")
    with gzip.open(path, "rb") as stream:
        pcm = stream.read(size + 1)
    require(len(pcm) == size and sha(pcm) == metadata["sha256"], "frozen PCM SHA/length mismatch")
    return pcm


def ogg_command(pcm_path, output, metadata):
    return ["ffmpeg", "-nostdin", "-v", "error", "-y", "-f", "s16le",
            "-ar", str(metadata["rate"]), "-ac", str(metadata["channels"]), "-i", str(pcm_path),
            "-map", "0:a:0", "-map_metadata", "-1", "-c:a", "libvorbis", "-q:a", "4",
            "-fflags", "+bitexact", str(output)]


def encode(pcm, metadata, directory):
    source = Path(directory) / "source.s16le"
    destination = Path(directory) / "output.ogg"
    source.write_bytes(pcm)
    run(ogg_command(source, destination, metadata))
    info = json.loads(run(["ffprobe", "-v", "error", "-show_entries",
                          "format=duration:stream=codec_name,sample_rate,channels",
                          "-of", "json", destination]).stdout)
    require(len(info["streams"]) == 1 and info["streams"][0] ==
            dict(codec_name="vorbis", sample_rate=str(metadata["rate"]), channels=metadata["channels"]),
            "Ogg stream does not match complete source PCM")
    duration = metadata["frames"] / metadata["rate"]
    require(abs(float(info["format"]["duration"]) - duration) <= 1 / metadata["rate"],
            "Ogg granule duration differs from source")
    decoded = run(["ffmpeg", "-nostdin", "-xerror", "-v", "error", "-i", destination,
                   "-map", "0:a:0", "-c:a", "pcm_f32le", "-f", "f32le", "-"]).stdout
    floats = array.array("f", decoded)
    if sys.byteorder != "little":
        floats.byteswap()
    require(bool(floats) and all(math.isfinite(v) for v in floats), "nonfinite/empty full Ogg decode")
    frames = len(floats) // metadata["channels"]
    require(abs(frames - metadata["frames"]) <= 1024, "unexpected Vorbis priming/frame difference")
    peak = max(abs(v) for v in floats)
    require(0 < peak < 2, "Vorbis signal exceeds supported peak range")
    data = destination.read_bytes()
    require(data.startswith(b"OggS"), "encoder did not produce Ogg")
    return data, dict(bytes=len(data), sha256=sha(data), duration=duration,
                      channels=metadata["channels"], rate=metadata["rate"],
                      decoded_frames=frames, decoded_peak=peak,
                      playback_gain=min(1, PEAK_LIMIT / peak))


def safe_file(root, relative):
    require(isinstance(relative, str) and "\\" not in relative, "invalid portable relative path")
    part = PurePosixPath(relative)
    require(not part.is_absolute() and part.parts and all(x not in [".", "..", ""] for x in part.parts)
            and str(part) == relative, "path must be canonical and relative")
    root = Path(root).resolve()
    file = root.joinpath(*part.parts)
    require(not any(root.joinpath(*part.parts[:i]).is_symlink()
                    for i in range(1, len(part.parts) + 1)), "symlink in frozen artifact path")
    require(file.is_file() and file.resolve().is_relative_to(root), "missing/outside frozen artifact")
    return file


def check_file_list(root, manifest):
    rows = manifest["files"]
    require(isinstance(rows, list) and 0 < len(rows) <= 512, "prepared file count budget")
    names = set()
    for row in rows:
        name = row["path"]
        require(name not in names, "duplicate frozen file")
        names.add(name)
        file = safe_file(root, name)
        require(type(row["bytes"]) is int and 0 <= row["bytes"] <= MAX_BANK, "file size budget")
        require(file.stat().st_size == row["bytes"] and file_sha(file) == row["sha256"],
                f"frozen file changed: {name}")
    require(not any(p.is_symlink() for p in Path(root).rglob("*")), "symlink in frozen pack")
    actual = {p.relative_to(root).as_posix() for p in Path(root).rglob("*") if p.is_file()}
    require(actual == names | {"prepared-manifest.json"}, "unlisted/missing frozen files")


def atomic_write(path, data):
    path = Path(path)
    if path.exists():
        require(path.is_file() and not path.is_symlink(), "destination must be regular file")
        if path.read_bytes() == data:
            return
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(prefix=".naraka-audio-", dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        stream.write(data)
    try:
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)
