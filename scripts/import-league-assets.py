#!/usr/bin/env python3
"""Rebuild pinned local League models without a runtime conversion dependency.

Normal invocation verifies packaged hashes. --download reimports verified sources.
The manifest deliberately stores source and prepared hashes separately.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "assets/league-models.json"
COMPONENTS = {5120: ("b", 1), 5121: ("B", 1), 5122: ("h", 2), 5123: ("H", 2),
              5125: ("I", 4), 5126: ("f", 4)}
WIDTHS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}
USER_AGENT = ("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
              "(KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def unpack_glb(data):
    magic, version, length = struct.unpack_from("<4sII", data)
    if magic != b"glTF" or version != 2 or length != len(data):
        raise ValueError("not a complete glTF 2.0 binary")
    json_size, json_type = struct.unpack_from("<II", data, 12)
    if json_type != 0x4E4F534A:
        raise ValueError("GLB must start with its JSON chunk")
    document = json.loads(data[20:20 + json_size])
    binary_offset = 20 + json_size
    binary_size, binary_type = struct.unpack_from("<II", data, binary_offset)
    if binary_type != 0x004E4942:
        raise ValueError("GLB must embed its binary data")
    return document, bytearray(data[binary_offset + 8:binary_offset + 8 + binary_size])


def pack_glb(document, binary):
    document["buffers"] = [{"byteLength": len(binary)}]
    payload = json.dumps(document, separators=(",", ":"), ensure_ascii=False).encode()
    payload += b" " * (-len(payload) % 4)
    binary += b"\0" * (-len(binary) % 4)
    size = 12 + 8 + len(payload) + 8 + len(binary)
    return (struct.pack("<4sII", b"glTF", 2, size)
            + struct.pack("<II", len(payload), 0x4E4F534A) + payload
            + struct.pack("<II", len(binary), 0x004E4942) + binary)


def accessor_bytes(document, binary, index):
    accessor = document["accessors"][index]
    if "sparse" in accessor:
        raise ValueError("sparse accessors are not supported by this importer")
    view = document["bufferViews"][accessor["bufferView"]]
    width = WIDTHS[accessor["type"]] * COMPONENTS[accessor["componentType"]][1]
    stride = view.get("byteStride", width)
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    return [bytes(binary[start + i * stride:start + i * stride + width])
            for i in range(accessor["count"])]


def append_accessor(document, binary, template, values):
    binary.extend(b"\0" * (-len(binary) % 4))
    offset = len(binary)
    binary.extend(b"".join(values))
    view = len(document["bufferViews"])
    document["bufferViews"].append({"buffer": 0, "byteOffset": offset,
                                   "byteLength": len(binary) - offset})
    accessor = copy.deepcopy(template)
    accessor.update(bufferView=view, byteOffset=0, count=len(values))
    accessor.pop("min", None)
    accessor.pop("max", None)
    fmt = "<" + COMPONENTS[accessor["componentType"]][0] * WIDTHS[accessor["type"]]
    if accessor["type"] in ("SCALAR", "VEC3"):
        decoded = [struct.unpack(fmt, item) for item in values]
        accessor["min"] = list(map(min, zip(*decoded)))
        accessor["max"] = list(map(max, zip(*decoded)))
    result = len(document["accessors"])
    document["accessors"].append(accessor)
    return result


def accessor_references(document):
    """Yield mutable locations holding an accessor index."""
    for mesh in document["meshes"]:
        for primitive in mesh["primitives"]:
            if "indices" in primitive:
                yield primitive, "indices"
            for attribute in primitive["attributes"]:
                yield primitive["attributes"], attribute
            for target in primitive.get("targets", []):
                for attribute in target:
                    yield target, attribute
    for skin in document.get("skins", []):
        if "inverseBindMatrices" in skin:
            yield skin, "inverseBindMatrices"
    for animation in document.get("animations", []):
        for sampler in animation["samplers"]:
            yield sampler, "input"
            yield sampler, "output"


def compact_storage(document, binary):
    references = list(accessor_references(document))
    used_accessors = sorted({owner[key] for owner, key in references})
    remap = {old: new for new, old in enumerate(used_accessors)}
    for owner, key in references:
        owner[key] = remap[owner[key]]
    document["accessors"] = [document["accessors"][i] for i in used_accessors]
    owners = document["accessors"] + [i for i in document.get("images", []) if "bufferView" in i]
    used_views = sorted({owner["bufferView"] for owner in owners})
    remap = {old: new for new, old in enumerate(used_views)}
    output = bytearray()
    views = []
    for old in used_views:
        view = copy.deepcopy(document["bufferViews"][old])
        start = view.get("byteOffset", 0)
        output.extend(b"\0" * (-len(output) % 4))
        view["byteOffset"] = len(output)
        output.extend(binary[start:start + view["byteLength"]])
        views.append(view)
    for owner in owners:
        owner["bufferView"] = remap[owner["bufferView"]]
    document["bufferViews"] = views
    return output


def prepare_model(data):
    document, binary = unpack_glb(data)
    materials = document.get("materials", [])
    for mesh in document["meshes"]:
        mesh["primitives"] = [p for p in mesh["primitives"]
                              if materials[p["material"]].get("extras", {}).get("visible", True)]
        for primitive in mesh["primitives"]:
            if primitive.get("targets"):
                raise ValueError("morph targets require explicit importer support")
            indices = accessor_bytes(document, binary, primitive["indices"])
            component = document["accessors"][primitive["indices"]]["componentType"]
            decoded = [struct.unpack("<" + COMPONENTS[component][0], i)[0] for i in indices]
            used = list(dict.fromkeys(decoded))
            mapping = {old: new for new, old in enumerate(used)}
            for name, old in list(primitive["attributes"].items()):
                values = accessor_bytes(document, binary, old)
                primitive["attributes"][name] = append_accessor(
                    document, binary, document["accessors"][old], [values[i] for i in used])
            index_type = 5123 if len(used) <= 65535 else 5125
            index_format = "<" + COMPONENTS[index_type][0]
            primitive["indices"] = append_accessor(
                document, binary, {"componentType": index_type, "type": "SCALAR"},
                [struct.pack(index_format, mapping[i]) for i in decoded])

    # Root planar translation belongs to UnitVisual. Preserve source vertical
    # body motion (including falling), but never apply a second run/dash path.
    roots = {i: node.get("translation", [0, 0, 0])
             for i, node in enumerate(document["nodes"]) if node.get("name") == "Root"}
    for animation in document.get("animations", []):
        for channel in animation["channels"]:
            target = channel["target"]
            if target.get("node") not in roots or target["path"] != "translation":
                continue
            sampler = animation["samplers"][channel["sampler"]]
            if sampler.get("interpolation", "LINEAR") != "LINEAR":
                raise ValueError("root motion normalization expects linear tracks")
            old = sampler["output"]
            base = roots[target["node"]]
            values = [struct.unpack("<fff", item) for item in accessor_bytes(document, binary, old)]
            sampler["output"] = append_accessor(
                document, binary, document["accessors"][old],
                [struct.pack("<fff", base[0], value[1], base[2]) for value in values])
    document["asset"]["generator"] = "Kiva pinned League importer v1"
    return pack_glb(document, compact_storage(document, binary))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--download", action="store_true", help="rebuild from pinned public sources")
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text())
    for item in manifest["files"]:
        path = ROOT / "assets" / item["path"]
        if args.download:
            request = urllib.request.Request(item["url"], headers={
                "User-Agent": USER_AGENT, "Referer": "https://raw.communitydragon.org/"})
            with urllib.request.urlopen(request, timeout=45) as response:
                source = response.read()
            if digest(source) != item["source_sha256"]:
                raise ValueError(f"source changed: {item['path']}")
            data = prepare_model(source) if item["path"].endswith(".glb") else source
            if digest(data) != item["sha256"]:
                raise ValueError(f"prepared output changed: {item['path']}")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        if not path.exists() or digest(path.read_bytes()) != item["sha256"]:
            raise ValueError(f"packaged asset missing or changed: {item['path']}")
        print(f"verified {item['path']}")


if __name__ == "__main__":
    main()
