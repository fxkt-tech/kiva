#!/usr/bin/env python3
"""Bake selected CC0 Quaternius GLB clips for the rigid stickman skeleton.

No Blender or Python packages required. Pass the official Standard ZIP files:
  python3 scripts/import-animations.py /path/to/UAL1.zip /path/to/UAL2.zip
The output keeps local bone transforms (not retargeted poses), at 30 fps.
"""
import bisect
import json
import math
from pathlib import Path
import struct
import sys
import zipfile

OUTPUT = Path(__file__).resolve().parents[1] / "assets/animations"
BONES = [
    "root", "pelvis", "spine_01", "spine_02", "spine_03", "neck_01", "Head",
    "clavicle_l", "upperarm_l", "lowerarm_l", "hand_l",
    "clavicle_r", "upperarm_r", "lowerarm_r", "hand_r",
    "thigh_l", "calf_l", "foot_l", "ball_l",
    "thigh_r", "calf_r", "foot_r", "ball_r",
]
CLIPS = [
    ["Sword_Idle", "Jog_Fwd_Loop", "Walk_Loop", "Jump_Start", "Jump_Loop",
     "Jump_Land", "Roll", "Hit_Chest", "Death01", "Sword_Attack"],
    ["Sword_Regular_A", "Sword_Regular_A_Rec", "Sword_Regular_B",
     "Sword_Regular_B_Rec", "Sword_Regular_C", "Sword_Block",
     "NinjaJump_Idle_Loop", "Sword_Heavy_Combo", "Sword_Dash"],
]


def slerp(a, b, weight):
    dot = sum(x * y for x, y in zip(a, b))
    if dot < 0:
        b, dot = [-v for v in b], -dot
    if dot > 0.9995:
        result = [x + (y - x) * weight for x, y in zip(a, b)]
    else:
        angle = math.acos(min(1, dot))
        result = [(x * math.sin((1 - weight) * angle) + y * math.sin(weight * angle))
                  / math.sin(angle) for x, y in zip(a, b)]
    length = math.sqrt(sum(v * v for v in result))
    return [v / length for v in result]


class Glb:
    def __init__(self, archive):
        self.zip = zipfile.ZipFile(archive)
        name = next(n for n in self.zip.namelist()
                    if n.endswith("_Standard.glb") and "Unreal-Godot/" in n)
        data = self.zip.read(name)
        assert data[:4] == b"glTF" and struct.unpack_from("<I", data, 4)[0] == 2
        size = struct.unpack_from("<I", data, 12)[0]
        self.doc = json.loads(data[20:20 + size])
        self.buffer = data[28 + size:]
        self.ids = {n["name"]: i for i, n in enumerate(self.doc["nodes"])}
        self.parents = {c: i for i, n in enumerate(self.doc["nodes"])
                        for c in n.get("children", [])}
        self.animations = {a["name"]: a for a in self.doc["animations"]}

    def accessor(self, index):
        a = self.doc["accessors"][index]
        assert a["componentType"] == 5126 and "sparse" not in a
        view = self.doc["bufferViews"][a["bufferView"]]
        count = {"SCALAR": 1, "VEC3": 3, "VEC4": 4}[a["type"]]
        offset = view.get("byteOffset", 0) + a.get("byteOffset", 0)
        stride = view.get("byteStride", count * 4)
        return [struct.unpack_from("<" + "f" * count, self.buffer, offset + i * stride)
                for i in range(a["count"])]

    def rest(self, name):
        n = self.doc["nodes"][self.ids[name]]
        return (n.get("translation", [0, 0, 0]), n.get("rotation", [0, 0, 0, 1]),
                n.get("scale", [1, 1, 1]))

    def tracks(self, name):
        a = self.animations[name]
        tracks = {}
        for c in a["channels"]:
            s = a["samplers"][c["sampler"]]
            assert s.get("interpolation", "LINEAR") == "LINEAR"
            tracks[c["target"]["node"], c["target"]["path"]] = (
                [v[0] for v in self.accessor(s["input"])], self.accessor(s["output"]))
        return tracks

    def pose(self, tracks, t):
        result = []
        for name in BONES:
            values = []
            for path, rest in zip(["translation", "rotation", "scale"], self.rest(name)):
                track = tracks.get((self.ids[name], path))
                if track is None:
                    values.append(rest)
                    continue
                times, keys = track
                hi = min(bisect.bisect_right(times, t), len(times) - 1)
                lo = max(0, hi - 1)
                w = max(0, min(1, (t - times[lo]) / (times[hi] - times[lo]))) if hi != lo else 0
                values.append(slerp(keys[lo], keys[hi], w) if path == "rotation" else
                              [x + (y - x) * w for x, y in zip(keys[lo], keys[hi])])
            # Rigid bones have constant scale; reject assets that need scale animation.
            assert max(abs(a - b) for a, b in zip(values[2], self.rest(name)[2])) < 0.001
            result.append(values[:2])
        return result


def main():
    sources = [Glb(p) for p in sys.argv[1:]]
    assert len(sources) == 2, "Pass UAL1 Standard ZIP, then UAL2 Standard ZIP"
    base = sources[0]
    # Verify identical rest frames and topology: do not silently retarget.
    for source in sources:
        for name in BONES:
            for a, b in zip(base.rest(name), source.rest(name)):
                assert max(abs(x - y) for x, y in zip(a, b)) < 0.001, name
            if name != "root":
                parent = source.doc["nodes"][source.parents[source.ids[name]]]["name"]
                base_parent = base.doc["nodes"][base.parents[base.ids[name]]]["name"]
                assert parent == base_parent and parent in BONES, name
    OUTPUT.mkdir(parents=True, exist_ok=True)
    data = bytearray(b"KIVAAN01")

    def pack(fmt, *values):
        data.extend(struct.pack("<" + fmt, *values))

    def string(value):
        encoded = value.encode()
        pack("H", len(encoded))
        data.extend(encoded)

    pack("HH", len(BONES), sum(map(len, CLIPS)))
    for i, name in enumerate(BONES):
        string(name)
        parent_name = base.doc["nodes"][base.parents[base.ids[name]]]["name"]
        parent = -1 if name == "root" else BONES.index(parent_name)
        assert parent < i
        pack("h", parent)
        for values in base.rest(name):
            pack("f" * len(values), *values)
    for source, names in zip(sources, CLIPS):
        for name in names:
            tracks = source.tracks(name)
            duration = max(times[-1] for times, _ in tracks.values())
            frames = math.ceil(duration * 30) + 1
            string(name)
            pack("fH", duration, frames)
            for frame in range(frames):
                pose = source.pose(tracks, duration * frame / (frames - 1))
                # Strip horizontal root motion in the source root's parent space.
                # Source root is Y-up; keep vertical motion in pelvis / limbs.
                pose[0][0] = [0, 0, 0]
                for translation, rotation in pose:
                    pack("7f", *translation, *rotation)
            print(f"{name}: {duration:.3f}s, {frames} frames")
    (OUTPUT / "stickman.anim").write_bytes(data)
    license_name = next(n for n in sources[1].zip.namelist() if n.lower().endswith("license.txt"))
    (OUTPUT / "LICENSE.txt").write_bytes(sources[1].zip.read(license_name))
    print(f"Wrote {len(data):,} bytes to {OUTPUT / 'stickman.anim'}")


if __name__ == "__main__":
    main()
