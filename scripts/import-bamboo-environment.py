#!/usr/bin/env python3
"""Import exact extracted source terrain/undergrowth maps, without touching game files.

Run with the extraction's existing numpy/Pillow venv:
  research/.venv/bin/python scripts/import-bamboo-environment.py --source research

Inputs: repository.sqlite (read only), extracted/environment_surface/*.texture,
and the bounded probe manifests produced by environment_surface_probe.py and
environment_grass_final_probe.py / environment_forest_floor_probe.py. Re-decodes BC7 top mips, verifies raw SHA-256,
retains source RGBA/alpha, limits the longest side to 1024, and merges only these
eight named images into the existing environment manifest. No packed mix channel
is assigned a meaning, and no source material/shader equivalence is asserted.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import sys
import tempfile

import numpy as np
from PIL import Image


# Exact Repository identity; names alone are never used to select a payload.
SELECTED = [
    ('ground_base', 'a5e476fd-935a-4c23-8cb2-f4c55be10f14', 'A_weight007_soil_d', 'sRGB'),
    ('ground_normal', '811f0c33-723b-43a7-a4f8-2722b1a97a5f', 'A_weight007_soil_n', 'linear'),
    ('grass_base', '7b18bb71-f914-4a5c-86e7-231cf82d0d3a', 'grass_caocong_01_d', 'sRGB'),
    ('grass_normal', '52a61f8f-3c88-44e1-b29d-19832c3ae3fa', 'grass_caocong_01_n', 'linear'),
    ('grass_blades_base', '57bc5ac2-15df-47ec-ac3d-00bac2481f86', 'grass_gancao_d', 'sRGB'),
    ('grass_blades_normal', '5938ef88-4352-43b9-9dbc-fee98455abea', 'grass_gancao_n', 'linear'),
    ('forest_floor_base', '02a7f065-e9c0-4d1d-b7f9-c456042a79f7', 'ue_HG_jn_duoren_A_weight010_grass_d', 'sRGB'),
    ('forest_floor_normal', '6ce8b466-5fe3-4690-bee9-3c0440fe7076', 'ue_HG_jn_duoren_A_weight010_grass_n', 'linear'),
]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def image_stats(pixels):
    return dict(rgba_min=pixels.min(axis=(0, 1)).tolist(),
                rgba_max=pixels.max(axis=(0, 1)).tolist(),
                rgba_mean=pixels.mean(axis=(0, 1)).tolist(),
                alpha_below_128_ratio=float((pixels[:, :, 3] < 128).mean()))


def verify_normal(pixels, visible):
    # RGB is the observed three-component tangent normal; alpha varies on the
    # terrain map and has an unresolved role, so it is not interpreted as ORM.
    xyz = pixels[:, :, :3].astype(np.float32) / 127.5 - 1
    normal = xyz[visible]
    if not len(normal):
        raise ValueError('normal has no visible base-map region')
    lengths = np.linalg.norm(normal, axis=1)
    median = float(np.median(lengths))
    positive_z = float((normal[:, 2] > 0).mean())
    if not .85 < median < 1.1 or positive_z < .99:
        raise ValueError('selected normal fails observed XYZ/positive-Z validation')
    return dict(encoding='source RGB XYZ, copied without swizzle/G flip; linear sampling',
                visible_length_quantiles=np.quantile(lengths, [.01, .5, .95, .99]).tolist(),
                visible_positive_z_ratio=positive_z,
                vector_length_note='source RGB vectors are not all unit length; runtime normal direction must be normalized, especially forest-floor layer',
                alpha='preserved for provenance, ignored by runtime normal sampling; role unresolved')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parents[1] / 'assets/choulingke/environment')
    args = parser.parse_args()
    source, output = args.source.resolve(), args.output.resolve()
    sys.path.insert(0, str(source))
    from extract_assets import save_texture

    directory = source / 'extracted/environment_surface'
    records = {}
    for filename in ['probe_manifest.json', 'grass_final_probe_manifest.json', 'forest_floor_probe_manifest.json']:
        for record in json.loads((directory / filename).read_text()):
            if 'error' not in record:
                records[record['repository']['guid']] = record
    connection = sqlite3.connect(f'file:{source / "repository.sqlite"}?mode=ro', uri=True)
    connection.row_factory = sqlite3.Row
    prepared, manifest_records = {}, []
    with tempfile.TemporaryDirectory(prefix='kiva-bamboo-import-') as scratch:
        for name, guid, expected_name, space in SELECTED:
            row = connection.execute('SELECT * FROM resources WHERE guid=?', (guid,)).fetchone()
            if row is None or row['kind'] != 5 or row['name'] != expected_name:
                raise ValueError(f'Repository identity mismatch: {name}')
            record = records[guid]
            raw_path = directory / Path(record['texture']['png']).with_suffix('.texture').name
            raw = raw_path.read_bytes()
            if digest(raw) != record['sha256']:
                raise ValueError(f'raw texture SHA mismatch: {raw_path.name}')
            texture = save_texture(raw, Path(scratch) / guid)
            image = Image.open(texture['png']).convert('RGBA')
            pixels = np.asarray(image)
            source_size = list(image.size)
            if texture['pixel_format'] != 25 or source_size != [texture['width'], texture['height']]:
                raise ValueError('unverified source pixel layout/dimension')
            # Selected maps are already <=1024. Retain exact source pixels; the
            # general bound prevents a future oversized substitution being shipped.
            if max(image.size) > 1024:
                image.thumbnail((1024, 1024), Image.Resampling.LANCZOS)
            path = Path(scratch) / (name + '.png')
            image.save(path)
            prepared[name] = path.read_bytes()
            evidence = dict(repository_name=row['name'], repository_folder=row['folder'],
                            repository_block=row['block'], repository_offset=row['offset'],
                            role='base RGB plus original alpha' if space == 'sRGB' else 'normal RGB XYZ; alpha unresolved',
                            source_channel_role_evidence=(
                                'original G/terrain/TRN/TEX weight010_grass _d/_n pair; inspected opaque ground cover/alpha and measured normal XYZ; no original Boss material/tiling claim'
                                if name.startswith('forest_floor_') else
                                'paired Repository _d/_n identity and shared Material references, inspected base/alpha and measured normal XYZ; standalone Material .4 payload not found'))
            metadata = dict(name=name, guid=guid, physical_vfs=guid[:2] + '/' + guid + '.6',
                            source_size=source_size, source_declared_size=[texture['declared_width'], texture['declared_height']],
                            source_sha256=record['sha256'], source_png_sha256=record['png_sha256'],
                            embedded_sha256=digest(prepared[name]), embedded_size=list(image.size), color_space=space,
                            source_pixel_format='25 / BC7_UNORM', source_mip_count=texture['mip_count'],
                            source_header_byte10=raw[10],
                            source_header_observation='selected base pairs have byte10=2, normals=0; byte retained as evidence, original enum definition not asserted',
                            physical={key: record[key] for key in ['key', 'pack', 'offset', 'stored_size', 'decoded_bytes', 'stored_sha256', 'index_sources']},
                            statistics=image_stats(pixels), **evidence)
            if space == 'sRGB':
                if name in ['ground_base', 'forest_floor_base']:
                    if metadata['statistics']['alpha_below_128_ratio'] > .01:
                        raise ValueError('ground is unexpectedly transparent')
                    metadata['sampling'] = dict(address='Repeat', tile_world_meters=2.5, alpha='opaque material; source alpha retained',
                                                scale_origin='Kiva visual adaptation; source terrain tiling not recovered')
                    if name == 'forest_floor_base':
                        rgb = pixels[:, :, :3].astype(np.float32)
                        metadata['appearance'] = 'opaque fine green grass/moss-like forest-floor cover over dirt; original name is weight010_grass, not an alpha vegetation card'
                        metadata['tile_edge_validation'] = dict(
                            horizontal_seam_rgb_rms=float(np.sqrt(np.mean((rgb[:, 0] - rgb[:, -1]) ** 2))),
                            vertical_seam_rgb_rms=float(np.sqrt(np.mean((rgb[0] - rgb[-1]) ** 2))),
                            interior_neighbor_rgb_rms=float(np.sqrt(np.mean((rgb[:, 1:] - rgb[:, :-1]) ** 2))),
                            scope='measured source edges; runtime scale/mask and original terrain blending remain Kiva adaptation')
                elif name == 'grass_base':
                    if not .75 < metadata['statistics']['alpha_below_128_ratio'] < .9:
                        raise ValueError('source herb atlas alpha changed')
                    metadata['sampling'] = dict(address='ClampToEdge', alpha_cutoff=.46,
                                                image_uv_rectangle=[0, .02, 1, .92],
                                                image_root_uv=[.52, .91], card_height_meters=[.26, .50],
                                                appearance='green multi-stem broadleaf understory herb/shrub, not an isolated blade',
                                                scale_origin='Kiva card placement/UV crop; original mesh/pivot painter not ported')
                else:
                    metadata['sampling'] = dict(address='ClampToEdge', usage='fine grass clumps using five cropped blade strips',
                                                appearance='source grey-yellow blades; restrained green/dry-grass runtime tint is a Kiva adaptation')
            manifest_records.append(metadata)
        by_name = {record['name']: record for record in manifest_records}
        for prefix in ['ground', 'grass', 'grass_blades', 'forest_floor']:
            base = np.asarray(Image.open(Path(scratch) / (prefix + '_base.png')).convert('RGBA'))
            normal = np.asarray(Image.open(Path(scratch) / (prefix + '_normal.png')).convert('RGBA'))
            if base.shape != normal.shape:
                raise ValueError('paired normal/base dimension mismatch')
            by_name[prefix + '_normal']['normal_validation'] = verify_normal(normal, base[:, :, 3] >= 128)
        # Both source terrain aliases decoded to exactly the same PNG pixels.
        original = records[SELECTED[0][1]]
        original_normal = records[SELECTED[1][1]]
        alias_records = json.loads((directory / 'probe_manifest.json').read_text())
        aliases = [dict(name=entry['repository']['name'], guid=entry['repository']['guid'], png_sha256=entry['png_sha256'])
                   for entry in alias_records if entry.get('png_sha256') in [original['png_sha256'], original_normal['png_sha256']]
                   and entry['repository']['name'].startswith('ue_HG_jn_duoren_')]
        role_proofs = json.loads((directory / 'material_role_proof.json').read_text())
        proof = [dict(guid=item['repository']['guid'], name=item['repository']['name'], folder=item['repository']['folder'],
                      texture_refs=json.loads(item['repository']['refs']), standalone_payload_found=bool(item['payloads']))
                 for item in role_proofs]
        output.mkdir(parents=True, exist_ok=True)
        manifest_path = output / 'manifest.json'
        manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {'textures': []}
        selected_names = set(prepared)
        manifest['textures'] = [record for record in manifest['textures'] if record['name'] not in selected_names] + manifest_records
        manifest['reference_surface_pass'] = dict(
            scope='original shared dirt/undergrowth textures selected for Kiva reference matching; exact Boss map instances/material/tiling are not restored',
            original_materials=proof, pixel_identical_terrain_aliases=aliases,
            shader='base/normal only; source mix, normal alpha, pivot-position/axis and wind noise are not assigned new meanings',
            adaptation='dry/wet roughness, world wet mask, reflection, vegetation placement and scale remain Kiva material/scene choices',
            raw_extractor='research/environment_surface_probe.py, environment_grass_final_probe.py and environment_forest_floor_probe.py, exact GUID .6 VFS, bounded indexed archive reads',
            forest_floor_addition=dict(candidate_names=16, extracted_pairs=1,
                                       source_category='G/terrain/TRN/TEX original ground cover; source base alpha is 254..255 everywhere',
                                       runtime_usage='additional soil/green forest-floor blend layer; placement, wetness and tiling are Kiva choices'),
            texture_decoder_sha256=digest((source / 'extract_assets.py').read_bytes()),
            reproduce='research/.venv/bin/python scripts/import-bamboo-environment.py --source research')
        for name, data in prepared.items():
            (output / (name + '.png')).write_bytes(data)
        manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
    connection.close()
    print(f'imported {len(prepared)} maps ({sum(map(len, prepared.values())):,} PNG bytes); original RGBA retained, max 1024, no packed-channel guesses')


if __name__ == '__main__':
    main()
