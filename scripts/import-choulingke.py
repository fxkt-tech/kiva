#!/usr/bin/env python3
"""Build Kiva's trusted embedded avatar from the verified offline extraction.

Use the extraction's .venv (numpy/Pillow), never the game executable:
  /path/to/research/.venv/bin/python scripts/import-choulingke.py --source /path/to/research

KIVAAV02 is explicitly little endian: JSON provenance, 23 structural anchors,
PNG images, material slots, then named palettes and indexed vertex primitives.
Each source vertex keeps four joint bytes/four weight bytes, with weights /255
only at runtime. The raw extraction remains the authority for original units,
shader parameters and bind matrices. Clothing/fingers use an explicit retarget
map; this does not recover the original hierarchy or secondary animation.
"""

import argparse
import hashlib
import io
import json
from pathlib import Path
import struct
import sys

import numpy as np
from PIL import Image


DRIVERS = [
    ("root", None), ("pelvis", "Bip001 Pelvis"),
    ("spine_01", "Bip001 Spine"), ("spine_02", "Bip001 Spine1"),
    ("spine_03", "Bip001 Spine2"), ("neck_01", "Bip001 Neck"),
    ("Head", "Bip001 Head"), ("clavicle_l", "Bip001 L Clavicle"),
    ("upperarm_l", "Bone L UpperArm Twist"), ("lowerarm_l", "Bone L ForeArm Twist"),
    ("hand_l", "Bip001 L Hand"), ("clavicle_r", "Bip001 R Clavicle"),
    ("upperarm_r", "Bone R UpperArm Twist"), ("lowerarm_r", "Bone R ForeArm Twist"),
    ("hand_r", "Bip001 R Hand"), ("thigh_l", "Bip001 LThighTwist"),
    ("calf_l", "Bip001 L Calf"), ("foot_l", "Bip001 L Foot"),
    ("ball_l", "Bip001 L Toe0"), ("thigh_r", "Bip001 RThighTwist"),
    ("calf_r", "Bip001 R Calf"), ("foot_r", "Bip001 R Foot"),
    ("ball_r", "Bip001 R Toe0"),
]


def numeric(parameters, key, default):
    return float(parameters.get(key, default))


def surface_pixels(pixels, shader, parameters):
    """Convert source mix to Bevy ORM using recovered DXIL family dataflow.

    Evidence: research shader_research/{basic_13,hair_37,skin_47}.ll and
    GBufferGenPixelShading_0.ll / GBufferGenPixelShadingCEDiamondS_0.ll.
    R is smoothness, G metallic, B AO. Hair/skin have separate masking rules;
    skin's source profile tag suppresses metallic, and hair uses cMetallic.
    Detail-array blending/face makeup/diamond interior effects are not ported.
    """
    source = np.asarray(pixels, dtype=np.float32) / 255.0
    if shader == "PBR_DetailArray_Adv":
        smoothness = (source[:, :, 0] * numeric(parameters, "cSmoothnessScale", 1)
                      + numeric(parameters, "cSmoothnessOffset", 0))
        metallic = (source[:, :, 1] * numeric(parameters, "cMetallicScale", 1)
                    + numeric(parameters, "cMetallicOffset", 0))
    elif shader == "PBR_Hair_Adv":
        smoothness = source[:, :, 0] + numeric(parameters, "cSmoothOffset", 0)
        metallic = np.full(source.shape[:2], numeric(parameters, "cMetallic", 0))
    else:
        smoothness = source[:, :, 0]
        metallic = source[:, :, 1].copy()
        if shader == "PBR_SkinPRE":
            metallic *= (1.0 - source[:, :, 3]) > 0.05
    roughness = np.clip(1.0 - np.clip(smoothness, 0, 1), 0.08, 1)
    ao = np.clip(source[:, :, 2] + numeric(parameters, "cAOOffset", 0), 0.1, 1)
    return np.round(np.stack((ao, roughness, np.clip(metallic, 0, 1),
                              np.ones_like(ao)), axis=-1) * 255).astype(np.uint8)


def material_response(shader, parameters):
    """Explicit approximations of source BRDF families on Bevy's GGX model."""
    hair = shader == "PBR_Hair_Adv"
    skin = shader == "PBR_SkinPRE"
    cloth = shader == "PBR_DetailArray_Adv"
    diamond = shader == "PBR_Simple_Diamond_S"
    base = [1.0, 1.0, 1.0]
    if hair:
        base = [float(value) for value in parameters["cBaseColor"].strip("()").split(",")]
    # The imported source tangent stream and its hair/fuzz frame convention are
    # not recovered. Generated Mikk tangents work for normal mapping, but using
    # them for a guessed GGX anisotropy produced hard silver highlights on dark
    # cloth in the real WebGPU frame. Keep those families isotropic until that
    # frame/lobe mapping is verified; preserve the measured ORM data instead.
    return [1.0, 1.0, 0.5 if not cloth else 0.38,
            0.0,
            0.32 if diamond else 0.0, 0.18,
            0.06 if skin else 0.0, *base,
            numeric(parameters, "cAlpha", 0.25) if hair else 0.5]


def verify_surface_conversion():
    pixels = np.array([[[255, 128, 0, 255], [0, 255, 255, 0]]], dtype=np.uint8)
    detail = surface_pixels(pixels, "PBR_DetailArray_Adv", {})
    assert detail[0, 0].tolist() == [26, 20, 128, 255]
    assert detail[0, 1].tolist() == [255, 255, 255, 255]
    skin = surface_pixels(pixels, "PBR_SkinPRE", {})
    assert skin[0, 0, 2] == 0 and skin[0, 1, 2] == 255
    hair = surface_pixels(pixels, "PBR_Hair_Adv", {"cSmoothOffset": "-0.03", "cMetallic": "0"})
    assert not hair[:, :, 2].any()
    assert surface_pixels(pixels, "PBR_DetailArray_Adv", {"cMetallicScale": "0"})[:, :, 2].max() == 0
    assert len(material_response("PBR", {})) == 11


def matrix_translation(position):
    matrix = np.eye(4)
    matrix[:3, 3] = position
    return matrix


def rotate_z(angle):
    cosine, sine = np.cos(angle), np.sin(angle)
    return np.array([[cosine, -sine, 0, 0], [sine, cosine, 0, 0],
                     [0, 0, 1, 0], [0, 0, 0, 1]])


def driver_for(name, position, anchors, suffix):
    exact = {source: index for index, (_, source) in enumerate(DRIVERS) if source}
    if name in exact:
        return exact[name]
    by_name = {name: index for index, (name, _) in enumerate(DRIVERS)}
    if suffix == "weapon1":
        return by_name["hand_r"]
    if suffix in ("weapon2", "3_weapon", "lingdang"):
        return by_name["pelvis"]
    if "faceskin" in name or "_hat_" in name or "_hair_" in name or "_ershi_" in name:
        return by_name["neck_01" if "Larynx" in name else "Head"]
    if "Neck" in name:
        return by_name["neck_01"]
    side = "l" if " L " in name or " LThigh" in name else "r"
    if "Finger" in name or " Hand" in name:
        return by_name[f"hand_{side}"]
    if "ForeArm" in name:
        return by_name[f"lowerarm_{side}"]
    if "UpperArm" in name:
        return by_name[f"upperarm_{side}"]
    if "Thigh" in name:
        return by_name[f"thigh_{side}"]
    # Cloth attachment follows the closest torso/leg structural segment. Hair
    # and fingers are handled above so spatial proximity cannot bind them to
    # shoulders or knees. This is an approximation, not source parent data.
    candidates = [1, 2, 3, 4, 15, 16, 19, 20]
    return min(candidates, key=lambda index: np.linalg.norm(position - anchors[index]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--output", type=Path,
                        default=Path(__file__).resolve().parents[1] / "assets/choulingke/character.bin")
    parser.add_argument("--max-texture-size", type=int, default=512)
    args = parser.parse_args()
    if args.self_test:
        verify_surface_conversion()
        print("surface conversion checks passed")
        return
    if args.source is None:
        parser.error("--source is required unless --self-test is used")
    source = args.source.resolve()
    sys.path.insert(0, str(source))
    from skin_to_glb import TARGET_PARTS, load_part

    parts = {suffix: load_part(source / f"extracted/full/Mesh_ylk_{suffix}.mesh",
                               source / f"extracted/metadata/Skin_ylk_{suffix}.skin")
             for suffix in TARGET_PARTS}
    materials = json.loads((source / "material_bindings.json").read_text())
    bounds = np.concatenate([part.mesh.positions for suffix, part in parts.items()
                             if suffix not in ("weapon1", "weapon2", "3_weapon")])
    floor, top = bounds[:, 1].min(), bounds[:, 1].max()
    scale = 1.73 / (top - floor)
    conversion = np.diag([-scale, scale, -scale, 1.0])  # source +Z -> gameplay -Z
    conversion[1, 3] = -floor * scale
    original_binds = {}
    for part in parts.values():
        for name, inverse_bind in zip(part.skin.names, part.skin.inverse_bind_matrices):
            original_binds.setdefault(name, np.linalg.inv(inverse_bind))
    anchors = [np.zeros(3) if name == "root" else
               (conversion @ original_binds[source_name])[:3, 3]
               for name, source_name in DRIVERS]

    # These accessory placements are authored corrections. The extraction
    # exposes their geometry/palettes, but does not establish attachment data.
    conversions = {suffix: conversion for suffix in TARGET_PARTS}
    conversions["3_weapon"] = conversion @ matrix_translation([0.26, 1.36, -0.14])
    conversions["weapon2"] = (conversion @ matrix_translation([-0.34, 1.48, -0.16])
                               @ rotate_z(-0.3)
                               @ matrix_translation(-original_binds["Bone_jian_001"][:3, 3]))
    # Source long blade points down -Y from the grip. Melee attachments expect
    # +Y geometry, rotated by the established wrist correction toward +Z.
    conversions["weapon1"] = (np.diag([scale, scale, scale, 1.0]) @ rotate_z(np.pi)
                               @ matrix_translation(-original_binds["Bone_jian_002"][:3, 3]))

    material_records, texture_records, texture_indices, material_indices = [], [], {}, {}
    material_provenance = []
    zero_guid = "00000000-0000-0000-0000-000000000000"
    for suffix, part in parts.items():
        slots = materials[f"PBR_ylk_{suffix}_mat"]["slots"]
        assert len(slots) == len(part.mesh.submeshes), f"material/submesh count mismatch: {suffix}"
        for slot_index, slot in enumerate(slots):
            textures = []
            for parameter in ("tBaseMap", "tNormalMap"):
                guid = slot["texture_guids"].get(parameter, zero_guid)
                if guid == zero_guid:
                    textures.append(-1)
                    continue
                key = (guid, parameter == "tBaseMap")
                if key not in texture_indices:
                    path = source / f"extracted/textures/{guid}.png"
                    original = path.read_bytes()  # missing textures must fail, never become placeholders
                    image = Image.open(io.BytesIO(original)).convert("RGBA")
                    original_size = image.size
                    image.thumbnail((args.max_texture_size, args.max_texture_size), Image.Resampling.LANCZOS)
                    output = io.BytesIO()
                    image.save(output, format="PNG", optimize=True)
                    texture_indices[key] = len(texture_records)
                    texture_records.append((guid, key[1], output.getvalue()))
                    material_provenance.append({"guid": guid, "parameter": parameter,
                                                "source_png_sha256": hashlib.sha256(original).hexdigest(),
                                                "source_size": original_size, "embedded_size": image.size})
                textures.append(texture_indices[key])
            guid = slot["texture_guids"]["tMixMap"]
            original = (source / f"extracted/textures/{guid}.png").read_bytes()
            mix = Image.open(io.BytesIO(original)).convert("RGBA")
            mix.thumbnail((args.max_texture_size, args.max_texture_size), Image.Resampling.LANCZOS)
            orm = Image.fromarray(surface_pixels(mix, slot["shader"], slot["parameters"]))
            encoded = io.BytesIO()
            orm.save(encoded, format="PNG", optimize=True)
            surface = len(texture_records)
            texture_records.append((guid + "_derived_orm", False, encoded.getvalue()))
            material_provenance.append({"guid": guid, "parameter": "tMixMap",
                                        "source_png_sha256": hashlib.sha256(original).hexdigest(),
                                        "conversion": "DXIL verified R=smoothness/G=metallic/B=AO; family offsets/profile masking applied",
                                        "shader_family": slot["shader"],
                                        "source_parameters": slot["parameters"]})
            material_indices[suffix, slot_index] = len(material_records)
            material_records.append((slot["name"], *textures, surface, "Hair" in slot["shader"],
                                     *material_response(slot["shader"], slot["parameters"])))

    header = {"format": "KIVAAV02", "height_m": 1.73, "source_floor": float(floor),
              "source_scale": float(scale), "source_yaw_degrees": 180,
              "source_hierarchy_recovered": False, "secondary_motion": False,
              "shader_recovery": "DX12 cache ZZZ4/LZ4 -> DXBC/DXIL -> dxc disassembly; mix R smoothness/G metallic/B AO recovered per family",
              "shader_approximation": "Bevy GGX with source ORM and hair tint; source hair/fuzz/skin/diamond BRDFs adapted, no detail-array/makeup/interior source effects",
              "accessory_placements": "authored hip placement for sheath/belt knives; source attachment transforms unresolved",
              "weapon_correction": "weapon1 centered on Bone_jian_002, local -Y blade changed to +Y; tassels rigid",
              "sources": {suffix: part.sources for suffix, part in parts.items()},
              "textures": material_provenance}
    result = bytearray(b"KIVAAV02")

    def write(fmt, *values):
        result.extend(struct.pack("<" + fmt, *values))

    def string(text):
        data = text.encode()
        write("H", len(data))
        result.extend(data)

    encoded_header = json.dumps(header, separators=(",", ":")).encode()
    write("I", len(encoded_header))
    result.extend(encoded_header)
    write("H", len(DRIVERS))
    for (name, _), anchor in zip(DRIVERS, anchors):
        string(name)
        write("3f", *anchor)
    write("H", len(texture_records))
    for guid, srgb, data in texture_records:
        string(guid)
        write("BI", srgb, len(data))
        result.extend(data)
    write("H", len(material_records))
    for name, base, normal, surface, alpha, *response in material_records:
        string(name)
        write("hhhB11f", base, normal, surface, alpha, *response)
    write("H", len(parts))
    vertices, triangles, max_rest_error = 0, 0, 0
    for suffix, part in parts.items():
        conversion = conversions[suffix]
        converted_positions = (conversion @ np.column_stack((part.mesh.positions,
                                                              np.ones(len(part.mesh.positions)))).T).T[:, :3]
        converted_normals = (np.linalg.inv(conversion[:3, :3]).T @ np.asarray(part.mesh.normals).T).T
        converted_normals /= np.linalg.norm(converted_normals, axis=1)[:, None]
        string(suffix)
        write("BH", 1 if suffix == "weapon1" else 0, len(part.skin.names))
        for name, inverse_bind in zip(part.skin.names, part.skin.inverse_bind_matrices):
            bind = conversion @ np.linalg.inv(inverse_bind)
            normalized_inverse = np.linalg.inv(bind).astype(np.float32)
            max_rest_error = max(max_rest_error, float(np.abs(bind.astype(np.float32) @ normalized_inverse - np.eye(4)).max()))
            string(name)
            write("H16f", driver_for(name, bind[:3, 3], anchors, suffix), *normalized_inverse.T.reshape(16))
        write("H", len(part.mesh.submeshes))
        for slot, submesh in enumerate(part.mesh.submeshes):
            first, count = submesh.first_vertex, submesh.vertex_count
            original = np.array(part.mesh.indices[submesh.first_index:submesh.first_index + submesh.index_count])
            indices = original.reshape(-1, 3)[:, [0, 2, 1]].reshape(-1) - first
            assert 0 <= indices.min() <= indices.max() < count
            write("HII", material_indices[suffix, slot], count, len(indices))
            for index in range(first, first + count):
                write("8f8B", *converted_positions[index], *converted_normals[index], *part.mesh.uv0[index],
                      *part.joints[index], *part.weights[index])
            result.extend(indices.astype("<u4").tobytes())
            vertices += count
            triangles += len(indices) // 3
    assert max_rest_error < 1e-6
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(result)
    print(json.dumps({"output": str(args.output), "bytes": len(result), "sha256": hashlib.sha256(result).hexdigest(),
                      "parts": len(parts), "vertices": vertices, "triangles": triangles,
                      "materials": len(material_records), "textures": len(texture_records),
                      "source_scale": scale, "max_float32_bind_cancellation_error": max_rest_error}))


if __name__ == "__main__":
    main()
