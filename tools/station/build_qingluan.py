#!/usr/bin/env python3
"""Make the station sister from the pinned female AV03; requires NumPy and Pillow."""

import argparse
import hashlib
import io
import math
import runpy
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
AV = runpy.run_path(str(ROOT / "scripts/import-prepared-models.py"))
BASE = ROOT / "assets/characters/bailingmiao.bin"
OUTPUT = ROOT / "assets/characters/qingluan.bin"
BASE_SHA = "adaeb80895ade1234677fd3b024dba459621b395e8d478ff222b74f28402408b"
TAU = math.tau


def png(pixels):
    data = io.BytesIO()
    Image.fromarray(np.uint8(np.clip(pixels, 0, 255))).save(data, "PNG")
    return data.getvalue()


def mix(a, b, amount):
    return a * (1 - amount[..., None]) + np.array(b) * amount[..., None]


def dye(raw, trim):
    src = np.asarray(Image.open(io.BytesIO(raw)).convert("RGBA")).astype(float)
    rgb = src[:, :, :3]
    light = rgb @ [0.28, 0.55, 0.17] / 255
    # Keep the high-frequency weave and the original embroidered lotus designs.
    # The saturated fastenings become antique gold and the luminous panels jade.
    stops = [0, 0.25, 0.58, 0.84, 1]
    colors = np.array(
        [[10, 26, 33], [21, 47, 59], [33, 89, 97], [78, 146, 143], [178, 210, 184]]
        if not trim
        else [
            [14, 36, 45],
            [27, 60, 69],
            [44, 102, 112],
            [81, 142, 151],
            [128, 179, 175],
        ]
    )
    dyed = np.stack([np.interp(light, stops, c) for c in colors.T], axis=-1)
    warm = np.clip((rgb[:, :, 0] - rgb[:, :, 2] - 12) / 90, 0, 1)
    gold = mix(np.array([75, 45, 23]), [223, 192, 125], light)
    src[:, :, :3] = mix(dyed, gold, warm)
    return png(src)


def hair_dye(raw, jewelry):
    src = np.asarray(Image.open(io.BytesIO(raw)).convert("RGBA")).astype(float)
    rgb = src[:, :, :3]
    light = rgb @ [0.28, 0.55, 0.17] / 255
    if jewelry:
        # Copper/red cords, the forehead jewel and the beadwork share this atlas.
        red = np.clip((rgb[:, :, 0] - rgb[:, :, 1] - 12) / 38, 0, 1)
        blue = np.clip((rgb[:, :, 2] - rgb[:, :, 0] - 10) / 45, 0, 1)
        src[:, :, :3] = mix(
            mix(rgb, mix(np.array([12, 42, 51]), [95, 173, 148], light), red),
            mix(np.array([84, 55, 28]), [222, 188, 120], light),
            blue,
        )
    else:
        v, u = np.mgrid[: src.shape[0], : src.shape[1]]
        u, v = u / src.shape[1], v / src.shape[0]
        streak = (0.5 + 0.5 * np.sin(TAU * (72 * u + 0.48 * np.sin(11 * v)))) ** 5
        src[:, :, :3] = mix(
            np.array([21, 29, 35]), [62, 76, 78], light * (0.22 + 0.7 * streak)
        )
    return png(src)


def lash_mask():
    y, x = np.mgrid[0:512, 0:512]
    a = np.abs((x / 511 - 0.5) * 0.16)
    y = 1.64 - (y / 511) * 0.20
    top = np.interp(
        a,
        [0.015, 0.017, 0.021, 0.025, 0.030, 0.033, 0.036, 0.039, 0.044],
        [1.539, 1.541, 1.5443, 1.5464, 1.547, 1.5467, 1.5457, 1.5441, 1.5464],
    )
    width = np.clip(1 - ((a - 0.030) / 0.014) ** 2, 0, 1)
    barb = np.maximum(0, np.sin(3100 * a - 790 * (y - top))) ** 8
    alpha = np.abs(y - top - 0.0007 * width) < (0.0009 + 0.0025 * barb) * width
    img = np.empty((*a.shape, 4))
    img[:, :, :3], img[:, :, 3] = [48, 37, 36], alpha * 255
    return png(img)


def plume_texture():
    y, x = np.mgrid[0:768, 0:256]
    t = y / 767
    x = (x - 127.5) / 127.5
    v = np.abs(x)
    barbs = np.sin(202 * t - 18 * v - 3 * x * x + 1.4 * np.sin(23 * t))
    pigment = 0.50 + 0.14 * barbs + 0.09 * np.sin(68 * t - 11 * v) + 0.27 * (1 - v)
    color = mix(np.array([16, 52, 65]), [114, 190, 174], pigment)
    eye = np.sqrt((x / 0.65) ** 2 + ((t - 0.77) / 0.13) ** 2)
    color = mix(color, [183, 151, 83], np.exp(-(((eye - 1) / 0.105) ** 2)) * 0.86)
    color = mix(color, [18, 60, 73], np.exp(-((eye / 0.70) ** 4)) * 0.88)
    color = mix(
        color, [111, 200, 179], np.exp(-((x / 0.14) ** 2 + ((t - 0.80) / 0.045) ** 2))
    )
    shaft = np.exp(-((x / (0.022 + 0.022 * (1 - t))) ** 2))
    return png(mix(color, [205, 189, 128], shaft * (1 - t) ** 0.65))


def face_map(eyes):
    # Planar UVs are cut to the original eyes and face in their evaluated rest.
    # Their separate meshes allow a real pupil, iris and brows at portrait distance.
    y, x = np.mgrid[0:512, 0:512]
    x = (x / 511 - 0.5) * 0.16
    y = 1.64 - (y / 511) * 0.20
    a = np.abs(x)
    if eyes:
        r = np.hypot((a - 0.0262) / 0.0063, (y - 1.5438) / 0.0063)
        angle = np.arctan2(y - 1.5438, a - 0.0262)
        iris = mix(
            np.array([29, 45, 45]),
            [120, 140, 99],
            (0.48 + 0.18 * np.sin(angle * 37) + 0.18 * np.cos(angle * 17))
            * np.clip(r, 0, 1),
        )
        c = mix(np.array([221, 214, 197]), [45, 52, 47], np.clip((1.07 - r) * 10, 0, 1))
        c = mix(c, iris, np.clip((0.95 - r) * 10, 0, 1))
        c = mix(c, [19, 27, 28], np.clip((0.51 - r) * 9, 0, 1))
        glint = np.exp(-(((a - 0.0248) / 0.0009) ** 2) - ((y - 1.546) / 0.0011) ** 2)
        return png(mix(c, [242, 240, 223], glint))
    c = np.broadcast_to([206.0, 151.0, 133.0], (*x.shape, 3)).copy()
    c = mix(c, [220, 173, 148], np.exp(-((x / 0.057) ** 2)) * 0.80)
    c = mix(
        c,
        [184, 102, 98],
        np.exp(-(((a - 0.044) / 0.024) ** 2) - ((y - 1.52) / 0.015) ** 2) * 0.45,
    )
    lip = np.exp(
        -((x / 0.019) ** 4) - ((y - 1.4813 + 0.0016 * np.cos(x * 220)) / 0.0060) ** 2
    )
    c = mix(c, [153, 69, 70], 0.85 * lip)
    brow_arc = 1.559 + 0.014 * np.sin((a - 0.009) / 0.046 * 2.4)
    brow = np.exp(-(((a - 0.031) / 0.021) ** 4) - ((y - brow_arc) / 0.0028) ** 2)
    c = mix(c, [56, 43, 39], brow * 0.93)
    # Small jade teardrop between the brows.
    mark = np.exp(-((x / 0.0018) ** 2) - ((y - 1.572) / 0.0048) ** 2)
    return png(mix(c, [38, 108, 104], mark))


class Ornament:
    def __init__(self, name, anchors):
        self.name = name
        self.anchors = anchors
        self.vertices = []
        self.triangles = []

    def patch(self, rows, cols, point, flip=False):
        start = len(self.vertices)
        for row in range(rows + 1):
            for col in range(cols + 1):
                t, u = row / rows, col / cols
                position, bindings = point(t, u)
                self.vertices.append((position, (u, t), bindings))
        for row in range(rows):
            for col in range(cols):
                a = start + row * (cols + 1) + col
                b, c = a + 1, a + cols + 1
                faces = [(a, b, c), (b, c + 1, c)]
                self.triangles.extend([f[::-1] if flip else f for f in faces])

    def feather(self, root, end, width_axis, outward, width, lift, binds, flip=False):
        root, end = np.array(root), np.array(end)
        width_axis, outward = np.array(width_axis), np.array(outward)

        def sample(t, u):
            cross = 2 * u - 1
            swell = max(math.sin(math.pi * t**0.85), 0) ** 0.72
            # The vanes are arched on both axes, with an individually raised shaft.
            vane = (0.06 * (1 - t) + 0.012 * t + 0.988 * swell) * (
                1 - 0.055 * (0.5 + 0.5 * math.cos(61 * t + 2 * cross)) ** 3
            )
            p = root * (1 - t) + end * t + width_axis * width * cross * vane
            p += outward * (lift * swell + width * 0.25 * (1 - abs(cross)) * vane)
            return p, binds(t)

        self.patch(18, 4, sample, flip)

    def stone(self, center, radii, bone):
        def sample(t, u):
            phi = 0.004 + t * (math.pi - 0.008)
            ring = math.sin(phi)
            n = np.array(
                [ring * math.cos(TAU * u), math.cos(phi), ring * math.sin(TAU * u)]
            )
            return np.array(center) + n * radii, {bone: 1}

        self.patch(8, 12, sample)

    def write(self, writer, material):
        bones = sorted({bone for _, _, weights in self.vertices for bone in weights})
        writer.string("qingluan::" + self.name)
        writer.write("BH", 0, len(bones))
        for name in bones:
            driver, pivot = self.anchors[name]
            rest = np.eye(4)
            rest[:3, 3] = pivot
            writer.string("qingluan::" + self.name + "::" + name)
            writer.write(
                "H16f16f", driver, *np.linalg.inv(rest).T.ravel(), *rest.T.ravel()
            )
        vertices = np.zeros(len(self.vertices), dtype=AV["SKIN_VERTEX"])
        for index, (p, uv, weights) in enumerate(self.vertices):
            vertices[index]["p"], vertices[index]["uv"] = p, uv
            quantized = [round(w * 255) for w in weights.values()]
            quantized[np.argmax(quantized)] += 255 - sum(quantized)
            assert min(quantized) >= 0 and len(weights) <= 4
            for slot, (bone, weight) in enumerate(zip(weights, quantized, strict=True)):
                vertices[index]["j"][slot], vertices[index]["w"][slot] = (
                    bones.index(bone),
                    weight,
                )
        faces = np.array(self.triangles, dtype="<u4")
        a, b, c = vertices["p"][faces].transpose(1, 0, 2)
        normals = np.cross(b - a, c - a)
        for slot in faces.T:
            np.add.at(vertices["n"], slot, normals)
        lengths = np.linalg.norm(vertices["n"], axis=1)
        assert lengths.min() > 0
        vertices["n"] /= lengths[:, None]
        writer.write("HHII", 1, material, len(vertices), faces.size)
        writer.data.extend(vertices.tobytes())
        writer.data.extend(faces.tobytes())


def ornaments(anchors):
    mantle = Ornament("kingfisher_mantle", anchors)
    tails = Ornament("folded_flight_feathers", anchors)
    crest = Ornament("temple_crest", anchors)
    gold = Ornament("gilt_filaments", anchors)
    jade = Ornament("carved_jade", anchors)
    for side in [-1, 1]:
        suffix = "l" if side < 0 else "r"
        shoulder, arm = "clavicle_" + suffix, "upperarm_" + suffix
        # Three shingled shoulder rows, wrapping to the front of each lapel.
        for row in range(3):
            for i in range(8):
                angle = -1.50 + i * 0.445 + row * 0.034
                radial = np.array([side * math.cos(angle), 0, math.sin(angle)])
                tangent = [side * -math.sin(angle), 0, math.cos(angle)]
                root = np.array([side * 0.115, 1.421 - row * 0.018, 0.008]) + radial * (
                    0.045 + row * 0.011
                )
                spread = 0.20 - row * 0.045
                length = 0.33 - row * 0.084
                end = (
                    root
                    + radial * spread
                    + [side * (0.04 + 0.02 * math.cos(angle)), -length, 0.04]
                )

                arc = max(0.0, math.cos(angle))

                def binds(t, arc=arc, shoulder=shoulder, arm=arm):
                    influence = (0.14 + 0.68 * t) * arc
                    return {
                        "spine_03": (1 - influence) * 0.35,
                        shoulder: (1 - influence) * 0.65,
                        arm: influence,
                    }

                mantle.feather(
                    root,
                    end,
                    tangent,
                    radial,
                    0.048 - row * 0.005,
                    0.032,
                    binds,
                    side < 0,
                )
        # Folded wings overlap the trailing rear panels. Their tips follow the
        # corresponding thigh while their shafts graduate through spine to pelvis.
        for i in range(7):
            s = i / 6
            root = (side * (0.038 + 0.134 * s), 1.36 - 0.15 * s, 0.155 + 0.025 * s)
            end = (side * (0.09 + 0.31 * s), 0.24 + 0.54 * s, 0.48 - 0.23 * s)

            def back_weights(t, suffix=suffix):
                if t < 0.50:
                    f = t / 0.50
                    return {"spine_03": 1 - f, "pelvis": f}
                f = (t - 0.50) / 0.50 * 0.45
                return {"pelvis": 1 - f, "thigh_" + suffix: f}

            tails.feather(
                root,
                end,
                [side, 0, -0.19],
                [0, 0, 1],
                0.052,
                0.063,
                back_weights,
                side > 0,
            )
        # Brass settings and translucent-looking polished jade cabochons.
        for i in range(4):
            t = i / 3
            center = np.array(
                [side * (0.075 + 0.096 * t), 1.394 - 0.023 * t, -0.074 + 0.011 * t]
            )
            gold.stone(center, [0.016, 0.014, 0.010], shoulder)
            jade.stone(center + [0, 0, -0.009], [0.010, 0.010, 0.006], shoulder)
    # A single upswept comb gives the head a readable asymmetry in profile.
    for i in range(7):
        t = i / 6
        crest.feather(
            [-0.068, 1.592 + 0.011 * t, -0.018],
            [-0.17 - 0.09 * t, 1.62 + 0.137 * t, 0.21 - 0.1 * t],
            [0.24, 0.92, -0.08],
            [-0.7, 0, 0.3],
            0.019 + 0.006 * t,
            0.018,
            lambda _: {"Head": 1},
            True,
        )
    for i in range(8):
        t = i / 7
        center = [
            -0.075 - 0.012 * math.sin(2 * t),
            1.603 - 0.17 * t,
            -0.031 + 0.012 * t,
        ]
        gold.stone(center, [0.0035, 0.014, 0.0035], "Head")
        if i in [0, 3, 7]:
            jade.stone(center, [0.010, 0.015, 0.009], "Head")
    return [mantle, tails, crest, gold, jade]


def build():
    raw = BASE.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == BASE_SHA
    r = AV["N"].Reader(raw)
    assert r.bytes(8) == b"KIVAAV03"
    r.bytes(r.read("I")[0])
    w = AV["N"].Writer()
    AV["write_header"](
        w,
        {
            "format": "KIVAAV03",
            "identity": "Qingluan / station costume study",
            "base": "characters/bailingmiao.bin",
            "base_sha256": BASE_SHA,
            "authoring": "tools/station/build_qingluan.py",
            "forward": "-Z",
            "up": "+Y",
        },
    )
    anchor_start = r.at
    anchors = {r.string(): (i, r.read("3f")) for i in range(r.read("H")[0])}
    w.data.extend(raw[anchor_start : r.at])
    textures = []
    for _ in range(r.read("H")[0]):
        name = r.string()
        srgb, length = r.read("BI")
        pixels = r.bytes(length)
        if name.startswith("naraka::outfit::ch_"):
            pixels = dye(pixels, "body04" in name)
        elif "::lianmu::" in name:
            pixels = hair_dye(pixels, "hair02" in name)
        textures.append((name, srgb, pixels))
    plume, face, eyes, lash = range(len(textures), len(textures) + 4)
    textures.extend(
        [
            ("qingluan::ocellated_vane", 1, plume_texture()),
            ("qingluan::face_pigment", 1, face_map(False)),
            ("qingluan::jade_eyes", 1, face_map(True)),
            ("qingluan::lash_strokes", 1, lash_mask()),
        ]
    )
    w.write("H", len(textures))
    for name, srgb, data in textures:
        w.string(name)
        w.write("BI", srgb, len(data))
        w.data.extend(data)
    materials = []
    for _ in range(r.read("H")[0]):
        name, values = r.string(), list(r.read("hhhB11fB"))
        if "::lianmu::" in name:
            values[11:14] = [1, 1, 1]
        if name.endswith(" female eyelash"):
            values[0], values[3] = lash, 1
            values[11:14] = [1, 1, 1]
        if name.endswith((" female eyes", " female face")):
            values[0] = eyes if name.endswith("eyes") else face
            values[11:14] = [1, 1, 1]
        materials.append((name, values))
    base = len(materials)
    for name, texture, tint, rough, metal in [
        ("jade_vanes", plume, [1, 1, 1], 0.72, 0.04),
        ("aged_brass", -1, [0.60, 0.37, 0.14], 0.44, 0.50),
        ("polished_jade", -1, [0.11, 0.43, 0.34], 0.28, 0.12),
    ]:
        materials.append(
            (
                "qingluan::" + name,
                [texture, -1, -1, 0, rough, metal, 0.42, 0, 0, 0.20, 0, *tint, 0.5, 1],
            )
        )
    w.write("H", len(materials))
    for name, values in materials:
        w.string(name)
        w.write("hhhB11fB", *values)
    count = r.read("H")[0]
    additions = ornaments(anchors)
    w.write("H", count + len(additions))
    for _ in range(count):
        start = r.at
        name = r.string()
        _, bones = r.read("BH")
        matrices = []
        for _ in range(bones):
            r.string()
            r.read("H")
            inv = np.array(r.read("16f")).reshape(4, 4).T
            rest = np.array(r.read("16f")).reshape(4, 4).T
            matrices.append(rest @ inv)
        primitives = r.read("H")[0]
        w.data.extend(raw[start : r.at])
        for _ in range(primitives):
            mat, vc, ic = r.read("HII")
            w.write("HII", mat, vc, ic)
            data = r.bytes(vc * 40)
            if name.endswith(
                ("eyes_runtime_mesh", "face_runtime_mesh", "eyelash_runtime_mesh")
            ):
                vert = np.frombuffer(data, dtype=AV["SKIN_VERTEX"]).copy()
                posed = AV["skin_pose"](
                    np.array(matrices), vert["p"], vert["j"], vert["w"] / 255
                )
                vert["uv"] = np.stack(
                    [0.5 + posed[:, 0] / 0.16, (1.64 - posed[:, 1]) / 0.20], axis=1
                )
                data = vert.tobytes()
            w.data.extend(data)
            w.data.extend(r.bytes(ic * 4))
    assert r.at == len(raw)
    for item, material in zip(
        additions, [base, base, base, base + 1, base + 2], strict=True
    ):
        item.write(w, material)
        print(
            item.name, len(item.vertices), "vertices,", len(item.triangles), "triangles"
        )
    return bytes(w.data)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    data = build()
    if args.verify:
        if OUTPUT.read_bytes() != data:
            raise SystemExit("Qingluan does not match its authoring source")
    else:
        OUTPUT.write_bytes(data)
    print(hashlib.sha256(data).hexdigest(), OUTPUT.name, len(data), "bytes")
