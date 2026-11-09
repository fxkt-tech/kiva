#!/usr/bin/env python3
"""Decode observed CHAR::ANIM v15 sparse T/R(/S) channels offline.

The legacy Messiah track layout is documented in the primary template at
https://github.com/CucFlavius/Zee-010-Templates/blob/main/DiabloImmortal_Animation.bt
Current v15 differences are checked against actual channel-directory lengths.
Numeric 0xfc/0xfd bone-name references are retained, never guessed as names.
"""
from dataclasses import dataclass, field
from pathlib import Path
import math
import struct


class AnimationError(ValueError):
    pass


@dataclass
class Keys:
    times_ms: list[int] = field(default_factory=list)
    values: list[list[float]] = field(default_factory=list)
    encoding: int | None = None


@dataclass
class Bone:
    name: str | None
    encoded_name_reference: tuple[int, int] | None
    parent: int
    translation: Keys
    rotation: Keys
    scale: Keys


@dataclass
class Animation:
    duration_ms: int
    flags: int
    track_count: int
    source_frequency: int
    bones: list[Bone]
    channel_start: int
    channel_end: int
    sections: dict


class Reader:
    def __init__(self, data, begin=0, end=None):
        self.data, self.at = data, begin
        self.end = len(data) if end is None else end

    def take(self, n):
        if n < 0 or self.at + n > self.end:
            raise AnimationError(f"read exceeds channel extent at {self.at}")
        result = self.data[self.at:self.at + n]
        self.at += n
        return result

    def unpack(self, fmt):
        return struct.unpack(fmt, self.take(struct.calcsize(fmt)))

    def var(self):
        result = 0
        for shift in range(0, 35, 7):
            value = self.take(1)[0]
            result |= (value & 127) << shift
            if value < 128:
                return result
        raise AnimationError("oversized name reference")


def directory(data):
    if len(data) < 8:
        raise AnimationError("truncated animation")
    wrapped = data[:4] == b"\x4e\xa1\x42\x41"
    if wrapped:
        start, end = struct.unpack_from("<I", data, 4)[0], len(data)
        entry_format, alignment = "<IIII", 16
    else:
        count_bytes = struct.unpack_from("<I", data, len(data) - 4)[0]
        start, end = len(data) - 4 - count_bytes, len(data) - 4
        entry_format, alignment = "<QQII", 4
    if end - start > 16384 or not 32 <= start < end:
        raise AnimationError("invalid section directory size")
    reader = Reader(data, start, end)
    sections = {}
    while reader.at < end:
        if wrapped and not any(data[reader.at:end]):
            reader.at = end
            break
        size, reserved, flags, length = reader.unpack(entry_format)
        if length > 128 or reserved or flags:
            raise AnimationError("unsupported section-directory entry")
        try:
            name = reader.take(length).decode("ascii")
        except UnicodeDecodeError as exc:
            raise AnimationError("invalid section name") from exc
        if name in sections:
            # Wrapped v15 files can have several independent TRACKS_DATA
            # sections. Preserve every declared size; core channels still have
            # exactly one bounded section. Never accept duplicate core data.
            if name != "TRACKS_DATA":
                raise AnimationError("duplicate section name")
            previous = sections[name]
            sections[name] = ([previous] if isinstance(previous, int) else previous) + [size]
        else:
            sections[name] = size
        padding = (-reader.at) % alignment
        if reader.take(padding) != b"\0" * padding:
            raise AnimationError("nonzero section name padding")
    required = {"HEADER", "CHANNEL_DATA", "CURVE_DATA", "MOTION_DATA"}
    if not required <= set(sections) <= required | {"TRACKS_DATA"}:
        raise AnimationError("unsupported animation section set")
    return sections, start


def smallest_three(data, bits):
    packed = int.from_bytes(data, "little")
    count_bits = 3 * bits + 2
    if packed >> count_bits:
        raise AnimationError("nonzero unused quaternion bits")
    mask = (1 << bits) - 1
    half = math.sqrt(0.5)
    a, b, c = [((packed >> (shift * bits)) & mask) / (mask * half) - half
               for shift in (2, 1, 0)]
    square = 1.0 - a * a - b * b - c * c
    if square < -0.00002:
        raise AnimationError("invalid smallest-three quaternion")
    d = math.sqrt(max(0.0, square))
    slot = (packed >> (3 * bits)) & 3
    result = ([d, a, b, c], [c, d, a, b], [b, c, d, a], [a, b, c, d])[slot]
    if d + 0.00002 < max(abs(a), abs(b), abs(c)):
        raise AnimationError("reconstructed quaternion component is not largest")
    return result


def read_keys(reader, duration, last, semantic):
    count = reader.unpack("<H")[0]
    keys = Keys()
    if not count:
        if not last and reader.take(2) != b"\0\0":
            raise AnimationError("nonzero empty-track padding")
        return keys
    code = reader.take(1)[0]
    keys.encoding = code
    if code == 0:
        if semantic == "rotation":
            raise AnimationError("unverified raw rotation layout")
        if reader.take(1) != b"\0":
            raise AnimationError("unsupported uncompressed vector flag")
        for _ in range(count):
            time, *value = reader.unpack("<H3f")
            keys.times_ms.append(time)
            keys.values.append(value)
    elif code & 3 == 2 and semantic == "rotation":
        bits = code >> 2
        if not 4 <= bits <= 20:
            raise AnimationError("unsupported quaternion precision")
        encoded_bytes = (3 * bits + 2 + 7) // 8
        for _ in range(count):
            keys.times_ms.append(reader.unpack("<H")[0])
            value = smallest_three(reader.take(encoded_bytes), bits)
            if keys.values and sum(a * b for a, b in zip(value, keys.values[-1])) < 0:
                value = [-v for v in value]
            keys.values.append(value)
    else:
        raise AnimationError(f"unsupported {semantic} track encoding {code}")
    if any(a >= b for a, b in zip(keys.times_ms, keys.times_ms[1:])):
        raise AnimationError("non-increasing key timestamps")
    if keys.times_ms[-1] > duration:
        raise AnimationError("key exceeds declared clip duration")
    if not all(math.isfinite(v) for key in keys.values for v in key):
        raise AnimationError("non-finite animation value")
    return keys


def parse_animation(data):
    sections, directory_start = directory(data)
    if data[:4] == b"\x4e\xa1\x42\x41":
        base = 16
        if struct.unpack_from("<I", data, 4)[0] != directory_start:
            raise AnimationError("wrapped directory pointer mismatch")
        if data[8:16] != b"\0" * 8:
            raise AnimationError("unsupported wrapped header")
    elif data[:4] == b"\x65\x4e\xa1\x42":
        base = 4
    else:
        raise AnimationError("unsupported animation wrapper")
    reader = Reader(data, base)
    if reader.take(10) != b"CHAR::ANIM":
        raise AnimationError("missing CHAR::ANIM signature")
    version, created, duration, tracks, flags, reserved, frequency, unknown, bones, precision = reader.unpack("<H I I BB H H H H H")
    if version != 15 or flags != 54 or reserved or tracks not in (2, 3):
        raise AnimationError("unsupported animation version/flags/track count")
    if not 0 < duration <= 65535 or not 1 <= bones <= 4096:
        raise AnimationError("invalid animation duration or bone count")
    if frequency != 200 or unknown != 2 or precision != 20 or sections["HEADER"] != 32:
        raise AnimationError("unverified v15 channel header")
    channel_start = reader.at
    channel_end = channel_start + sections["CHANNEL_DATA"]
    if channel_end > directory_start:
        raise AnimationError("channel section exceeds directory")
    reader.end = channel_end
    result = []
    for index in range(bones):
        marker = reader.take(1)[0]
        reference, name = None, None
        if marker >= 252:
            reference = (marker, reader.take(1)[0])
        elif 1 <= marker <= 251:
            try:
                name = reader.take(marker).decode("utf-8")
            except UnicodeDecodeError as exc:
                raise AnimationError("invalid UTF-8 bone name") from exc
            if "\0" in name:
                raise AnimationError("NUL in bone name")
        else:
            raise AnimationError(f"unsupported bone name marker {marker}")
        parent = reader.unpack("<h")[0]
        if parent < -1 or parent >= index:
            raise AnimationError(f"non-ancestral parent index bone={index} name={name} ref={reference} parent={parent} at={reader.at - 2}")
        # Three-track files store scale before translation and rotation.
        # Full section consumption and finite positive scale values validate
        # this observed ordering; no new track layout is silently accepted.
        scale = read_keys(reader, duration, False, "scale") if tracks == 3 else Keys()
        translation = read_keys(reader, duration, False, "translation")
        rotation = read_keys(reader, duration, True, "rotation")
        if any(v <= 0 for key in scale.values for v in key):
            raise AnimationError("non-positive scale in observed three-track layout")
        result.append(Bone(name, reference, parent, translation, rotation, scale))
    if reader.at != channel_end:
        raise AnimationError(f"channels consumed {reader.at - channel_start}, directory declares {sections['CHANNEL_DATA']}")
    names = [b.name for b in result if b.name is not None]
    if len(set(names)) != len(names):
        raise AnimationError("duplicate explicit bone names")
    return Animation(duration, flags, tracks, frequency, result,
                     channel_start, channel_end, sections)


if __name__ == "__main__":
    import argparse
    import json
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("file", type=Path)
    args = parser.parse_args()
    anim = parse_animation(args.file.read_bytes())
    print(json.dumps(dict(duration_ms=anim.duration_ms, bones=len(anim.bones),
                         explicit_names=sum(b.name is not None for b in anim.bones),
                         encoded_names=sum(b.encoded_name_reference is not None for b in anim.bones),
                         track_count=anim.track_count, sections=anim.sections)))
