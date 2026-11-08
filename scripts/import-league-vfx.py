#!/usr/bin/env python3
"""Verify local VFX assets, or rebuild reviewed PNG/SCB sources offline.

--source-dir rebuilds from already downloaded originals. --download needs the
optional pyzstd importer dependency for pinned Riot bundle ranges. Neither the
game nor normal builds run this tool or contact an external resource service.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import tempfile
import urllib.request
import zlib


ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets/vfx"
USER_AGENT = (
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def json_bytes(value: object) -> bytes:
    return (json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode()


def output_path(relative: str) -> Path:
    path = Path(relative)
    if path.is_absolute() or any(part in (".", "..") for part in path.parts):
        raise ValueError(f"invalid asset path: {relative}")
    return ASSETS / path


def decode_png(data: bytes) -> tuple[int, int, bytes]:
    """Decode the exact RGBA pixels despite Cloudflare lossless PNG Polish."""
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("source is not a PNG")
    at, compressed, size, ended = 8, bytearray(), None, False
    palette, transparency = b"", b""
    while at < len(data):
        length, = struct.unpack_from(">I", data, at)
        kind, payload = data[at + 4:at + 8], data[at + 8:at + 8 + length]
        checksum, = struct.unpack_from(">I", data, at + 8 + length)
        if len(payload) != length or zlib.crc32(kind + payload) & 0xFFFFFFFF != checksum:
            raise ValueError("invalid PNG chunk checksum")
        if kind == b"IHDR":
            width, height, bits, color, _, _, interlace = struct.unpack(">IIBBBBB", payload)
            if (color not in (0, 2, 3, 4, 6) or interlace != 0
                    or bits not in ((1, 2, 4, 8) if color in (0, 3) else (8,))):
                raise ValueError("unsupported PNG transport pixel format")
            if not width or not height or width * height > 16_777_216:
                raise ValueError("PNG dimensions outside the importer limit")
            size = width, height
        elif kind == b"PLTE":
            palette = payload
        elif kind == b"tRNS":
            transparency = payload
        elif kind == b"IDAT":
            compressed.extend(payload)
        elif kind == b"IEND":
            ended = True
        at += length + 12
    if size is None or not ended or at != len(data):
        raise ValueError("incomplete PNG")
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color]
    stride = (size[0] * channels * bits + 7) // 8
    pixel_bytes = max(1, (channels * bits + 7) // 8)
    decoded = zlib.decompress(compressed)
    if len(decoded) != size[1] * (stride + 1):
        raise ValueError("invalid PNG scanline length")
    if any(decoded[row * (stride + 1)] > 4 for row in range(size[1])):
        raise ValueError("invalid PNG row filter")
    previous, pixels = bytearray(stride), bytearray()
    for row_index in range(size[1]):
        mode = decoded[row_index * (stride + 1)]
        row = bytearray(decoded[row_index * (stride + 1) + 1:(row_index + 1) * (stride + 1)])
        for i in range(stride):
            left, above = row[i - pixel_bytes] if i >= pixel_bytes else 0, previous[i]
            corner = previous[i - pixel_bytes] if i >= pixel_bytes else 0
            if mode == 1:
                predictor = left
            elif mode == 2:
                predictor = above
            elif mode == 3:
                predictor = (left + above) // 2
            elif mode == 4:
                estimate = left + above - corner
                distances = [abs(estimate - left), abs(estimate - above), abs(estimate - corner)]
                predictor = [left, above, corner][distances.index(min(distances))]
            else:
                predictor = 0
            row[i] = (row[i] + predictor) & 255
        for x in range(size[0]):
            if color in (0, 3):
                sample = (row[x * bits // 8] >> (8 - bits - (x * bits % 8))) & ((1 << bits) - 1)
                if color == 3:
                    if sample * 3 + 3 > len(palette):
                        raise ValueError("PNG palette index out of range")
                    rgb = palette[sample * 3:sample * 3 + 3]
                    alpha = transparency[sample] if sample < len(transparency) else 255
                else:
                    gray = sample * 255 // ((1 << bits) - 1)
                    rgb = bytes((gray, gray, gray))
                    alpha = 0 if len(transparency) == 2 and sample == int.from_bytes(transparency, "big") else 255
            elif color == 2:
                rgb = row[x * 3:x * 3 + 3]
                alpha = 0 if len(transparency) == 6 and tuple(rgb) == struct.unpack(">3H", transparency) else 255
            elif color == 4:
                gray, alpha = row[x * 2:x * 2 + 2]
                rgb = bytes((gray, gray, gray))
            else:
                rgb, alpha = row[x * 4:x * 4 + 3], row[x * 4 + 3]
            pixels.extend(rgb)
            pixels.append(alpha)
        previous = row
    return *size, bytes(pixels)


def canonical_png(width: int, height: int, pixels: bytes) -> bytes:
    def chunk(kind: bytes, payload: bytes) -> bytes:
        return (struct.pack(">I", len(payload)) + kind + payload
                + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF))
    stride = width * 4
    rows = b"".join(b"\0" + pixels[row * stride:(row + 1) * stride] for row in range(height))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b""))


def canonical_pixels(pixels: bytes) -> bytes:
    """Polish also clears unused RGB beneath fully transparent texels."""
    result = bytearray(pixels)
    for at in range(0, len(result), 4):
        if result[at + 3] == 0:
            result[at:at + 3] = b"\0\0\0"
    return bytes(result)


def validate_source(source: dict, data: bytes) -> None:
    if "pixel_sha256" in source:
        width, height, pixels = decode_png(data)
        if [width, height] != source["pixel_size"] or sha256(canonical_pixels(pixels)) != source["pixel_sha256"]:
            raise ValueError(f"source pixels changed: {source['file']}")
        # The original container hash remains recorded, but Cloudflare can
        # losslessly re-encode the transport. Pixel identity is strictly pinned.
        return
    if len(data) != source["size"] or sha256(data) != source["sha256"]:
        raise ValueError(f"source missing or changed: {source['file']}")


def parse_scb(data: bytes) -> tuple[list, list, list]:
    if data[:8] != b"r3d2Mesh" or struct.unpack_from("<HH", data, 8) != (3, 2):
        raise ValueError("only reviewed SCB 3.2 is supported")
    vertex_count, face_count, flags = struct.unpack_from("<III", data, 140)
    has_colors, = struct.unpack_from("<I", data, 176)
    if flags & ~3 or has_colors not in (0, 1):
        raise ValueError("unsupported SCB flags")
    at = 180
    vertices = [struct.unpack_from("<3f", data, at + i * 12) for i in range(vertex_count)]
    at += vertex_count * 12
    colors = [(1.0, 1.0, 1.0, 1.0)] * vertex_count
    if has_colors:
        colors = [tuple(data[at + i * 4 + j] / 255 for j in (2, 1, 0, 3))
                  for i in range(vertex_count)]
        at += vertex_count * 4
    at += 12  # Original central point; bounds are recomputed from real vertices.
    faces = []
    for _ in range(face_count):
        indices = struct.unpack_from("<3I", data, at)
        if any(index >= vertex_count for index in indices):
            raise ValueError("SCB face references a missing vertex")
        # Three indices + 64-byte material name + separate U[3]/V[3].
        uv = struct.unpack_from("<6f", data, at + 76)
        faces.append({"indices": indices, "uv": [(uv[i], uv[i + 3]) for i in range(3)],
                      "colors": [(1.0, 1.0, 1.0)] * 3})
        at += 100
    if flags & 1:
        for face in faces:
            face["colors"] = [tuple(data[at + i * 3 + j] / 255 for j in range(3))
                              for i in range(3)]
            at += 9
    if at != len(data) or not vertices or not faces:
        raise ValueError("incomplete or unsupported SCB payload")
    numbers = [value for point in vertices for value in point]
    numbers += [value for face in faces for point in face["uv"] for value in point]
    if not all(math.isfinite(value) for value in numbers):
        raise ValueError("nonfinite SCB geometry")
    return vertices, colors, faces


def bounds(points: list) -> dict:
    return {"min": [min(point[i] for point in points) for i in range(3)],
            "max": [max(point[i] for point in points) for i in range(3)]}


def prepare_mesh(data: bytes, recipe: dict) -> tuple[bytes, dict]:
    vertices, vertex_colors, faces = parse_scb(data)
    face_rgb_policy = recipe["face_rgb_policy"]
    if face_rgb_policy == "ignore_zero_legacy_vcp":
        # The original E Weapon mesh has legacy VCP RGB=0 on every face.
        # Its selected particle uses the colored hero atlas; treating this
        # unused legacy lighting channel as PBR COLOR_0 blackens the spear.
        # Scope this adaptation to the explicitly reviewed all-zero source.
        if any(any(channel != 0.0 for channel in color) for face in faces for color in face["colors"]):
            raise ValueError("reviewed zero-VCP mesh acquired nonzero face colors")
    elif face_rgb_policy != "preserve":
        raise ValueError("unsupported SCB face RGB policy")
    rotation, scale = recipe["source_to_game"], recipe["source_to_world"]
    positions, normals, uvs, colors = [], [], [], []
    for face in faces:
        triangle = []
        for index in face["indices"]:
            point = vertices[index]
            converted = [sum(rotation[i][j] * point[j] for j in range(3)) * scale for i in range(3)]
            triangle.append(struct.unpack("<3f", struct.pack("<3f", *converted)))
        a, b = ([triangle[1][i] - triangle[0][i] for i in range(3)],
                [triangle[2][i] - triangle[0][i] for i in range(3)])
        normal = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
        magnitude = math.hypot(*normal)
        normal = [v / magnitude for v in normal] if magnitude > 1e-12 else [0.0, 1.0, 0.0]
        for corner, index in enumerate(face["indices"]):
            positions.append(triangle[corner])
            normals.append(normal)
            uvs.append(face["uv"][corner])
            vc = vertex_colors[index]
            fc = (1.0, 1.0, 1.0) if face_rgb_policy == "ignore_zero_legacy_vcp" else face["colors"][corner]
            colors.append(tuple(vc[i] * fc[i] for i in range(3)) + (vc[3],))
    blob, views, accessors = bytearray(), [], []

    def attribute(values: list, kind: str, fmt: str) -> int:
        blob.extend(b"\0" * (-len(blob) % 4))
        start = len(blob)
        blob.extend(b"".join(struct.pack(fmt, *value) for value in values))
        views.append({"buffer": 0, "byteOffset": start, "byteLength": len(blob) - start, "target": 34962})
        accessor = {"bufferView": len(views) - 1, "componentType": 5126, "count": len(values), "type": kind}
        if values is positions:
            accessor.update(bounds(values))
        accessors.append(accessor)
        return len(accessors) - 1

    attrs = {"POSITION": attribute(positions, "VEC3", "<3f"),
             "NORMAL": attribute(normals, "VEC3", "<3f"),
             "TEXCOORD_0": attribute(uvs, "VEC2", "<2f"),
             "COLOR_0": attribute(colors, "VEC4", "<4f")}
    document = {"asset": {"version": "2.0", "generator": "Kiva pinned SCB importer"},
                "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
                "meshes": [{"name": recipe["key"], "primitives": [{"attributes": attrs}]}],
                "buffers": [{"byteLength": len(blob)}], "bufferViews": views, "accessors": accessors}
    encoded = json.dumps(document, separators=(",", ":")).encode()
    encoded += b" " * (-len(encoded) % 4)
    blob.extend(b"\0" * (-len(blob) % 4))
    result = (struct.pack("<4sII", b"glTF", 2, 28 + len(encoded) + len(blob))
              + struct.pack("<I4s", len(encoded), b"JSON") + encoded
              + struct.pack("<I4s", len(blob), b"BIN\0") + blob)
    metadata = {key: recipe[key] for key in ("key", "texture", "source_to_world", "source_to_game", "forward", "up")}
    metadata.update(path="vfx/" + recipe["output"], source_bounds=bounds(vertices), bounds=bounds(positions))
    return result, metadata


def build_outputs(manifest: dict, sources: dict[str, bytes]) -> dict[str, bytes]:
    outputs, textures, meshes = {}, [], []
    for recipe in manifest["textures"]:
        width, height, pixels = decode_png(sources[recipe["source"]])
        data = canonical_png(width, height, canonical_pixels(pixels))
        spec = {key: recipe[key] for key in ("key", "blend", "atlas", "sprite_forward", "source_system", "source_emitter", "source_blend")}
        spec.update(path="vfx/" + recipe["output"], size=[width, height])
        atlas = spec["atlas"]
        if not (0 < atlas["frames"] <= atlas["columns"] * atlas["rows"] and 0 <= atlas["default_frame"] < atlas["frames"]):
            raise ValueError("invalid reviewed atlas recipe")
        outputs[recipe["output"]] = data
        textures.append(spec)
    for recipe in manifest["meshes"]:
        data, spec = prepare_mesh(sources[recipe["source"]], recipe)
        outputs[recipe["output"]] = data
        meshes.append(spec)
    outputs["catalog.json"] = json_bytes({"format": 1, "textures": textures, "meshes": meshes})
    outputs["parameters.json"] = json_bytes(manifest["reviewed_parameters"])
    return outputs


def download_source(source: dict, cache: Path) -> bytes:
    path = cache / source["file"]
    if path.exists():
        data = path.read_bytes()
        try:
            validate_source(source, data)
            return data
        except ValueError:
            pass
    if "url" in source:
        request = urllib.request.Request(source["url"], headers={"User-Agent": USER_AGENT, "Referer": "https://raw.communitydragon.org/"})
        with urllib.request.urlopen(request, timeout=30) as response:
            data = response.read()
    else:
        try:
            import pyzstd
        except ImportError as error:
            raise ValueError("--download requires optional importer dependency: pip install pyzstd") from error
        pieces = []
        for segment in source["segments"]:
            segment_cache = cache / (segment["decoded_sha256"] + ".segment")
            decoded = segment_cache.read_bytes() if segment_cache.exists() else b""
            if len(decoded) != segment["decoded_size"] or sha256(decoded) != segment["decoded_sha256"]:
                start, length = segment["start"], segment["length"]
                request = urllib.request.Request(segment["url"], headers={"User-Agent": USER_AGENT, "Range": f"bytes={start}-{start + length - 1}"})
                with urllib.request.urlopen(request, timeout=30) as response:
                    if response.status != 206:
                        raise ValueError("Riot CDN did not honor the required range")
                    compressed = response.read(length + 1)
                if len(compressed) != length:
                    raise ValueError("invalid compressed segment length")
                decoded = pyzstd.decompress(compressed)
                if len(decoded) != segment["decoded_size"] or sha256(decoded) != segment["decoded_sha256"]:
                    raise ValueError("Riot segment checksum mismatch")
                segment_cache.write_bytes(decoded)
            pieces.append(decoded)
        combined = b"".join(pieces)
        raw = combined[source["offset"]:source["offset"] + source["length"]]
        if len(raw) != source["length"] or source["encoding"] != "zstd":
            raise ValueError("invalid reviewed WAD source range/encoding")
        data = pyzstd.decompress(raw)
    validate_source(source, data)
    path.write_bytes(data)
    return data


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--source-dir", type=Path, help="rebuild from hash-verified downloaded originals")
    mode.add_argument("--download", action="store_true", help="rebuild from pinned public sources")
    parser.add_argument("--cache", type=Path, help="optional download cache directory")
    args = parser.parse_args()
    manifest = json.loads((ASSETS / "sources.json").read_text())
    if manifest["format"] != 1:
        raise ValueError("unsupported VFX manifest")
    if args.source_dir or args.download:
        with tempfile.TemporaryDirectory(prefix="kiva-vfx-import-") as temporary:
            cache = args.cache or Path(temporary)
            cache.mkdir(parents=True, exist_ok=True)
            sources = {}
            for source in manifest["files"] + manifest.get("parameter_sources", []):
                if Path(source["file"]).name != source["file"]:
                    raise ValueError("a source filename must not contain directories")
                data = ((args.source_dir / source["file"]).read_bytes() if args.source_dir
                        else download_source(source, cache))
                validate_source(source, data)
                sources[source["file"]] = data
            outputs = build_outputs(manifest, sources)
            expected = {item["path"]: item for item in manifest["outputs"]}
            if outputs.keys() != expected.keys():
                raise ValueError("prepared file set changed")
            for path, data in outputs.items():
                if len(data) != expected[path]["size"] or sha256(data) != expected[path]["sha256"]:
                    raise ValueError(f"prepared output changed: {path}")
            # Verify the complete output set before mutating any product files.
            for path, data in outputs.items():
                target = output_path(path)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
    for item in manifest["outputs"]:
        path = output_path(item["path"])
        if not path.is_file() or len(path.read_bytes()) != item["size"] or sha256(path.read_bytes()) != item["sha256"]:
            raise ValueError(f"packaged asset missing or changed: {item['path']}")
    print(f"Verified {len(manifest['outputs'])} packaged VFX files")


if __name__ == "__main__":
    main()
