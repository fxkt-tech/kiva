"""Read FILE-backed cape image CPU data and graph tags; compare PNG bytes independently."""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

import bpy
import numpy as np


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, action='append', required=True)
    parser.add_argument('--samples', type=Path, required=True)
    args = parser.parse_args(sys.argv[sys.argv.index('--')+1:])
    reports = []
    for source in args.source:
        bpy.ops.wm.open_mainfile(filepath=str(source))
        objects = [o for o in bpy.context.scene.objects if o.type == 'MESH' and o.get('kiva_source_role') == 'cape_detail']
        assert len(objects) == 3
        materials = {slot.material for o in objects for slot in o.material_slots}
        assert len(materials) == 1
        mat = materials.pop()
        assert mat['kiva_surface_effect'] == 0 and mat['kiva_alpha_mode'] == 0
        shader = next(n for n in mat.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
        assert shader.inputs['Sheen Weight'].default_value == 0
        normal_map = next(n for n in mat.node_tree.nodes if n.type == 'NORMAL_MAP')
        assert normal_map.inputs['Strength'].default_value == 1
        images = {}
        for key in ('base', 'normal', 'orm', 'emission'):
            img = mat.node_tree.nodes['Direct_'+key].image
            assert img.source == 'FILE' and img.packed_file is not None, (key, img.source)
            assert tuple(img.size) == (2048, 2048)
            assert img.colorspace_settings.name == ('sRGB' if key in ('base', 'emission') else 'Non-Color')
            actual = source.parent/'textures'/('cape_detail_'+key+'.png')
            raw = actual.read_bytes()
            assert bytes(img.packed_file.data) == raw, 'packed data differs from actual published PNG'
            pixels = np.empty(2048*2048*4, dtype=np.float32)
            img.pixels.foreach_get(pixels)
            pixels = pixels.reshape((2048, 2048, 4))
            coords = np.linspace(0, 2047, 64, dtype=int)
            samples = [[int(x), int(y), *map(float, pixels[y, x])] for y in coords for x in coords]
            images[key] = {'file': actual.name, 'sha256': hashlib.sha256(raw).hexdigest(),
                           'source': img.source, 'colorspace': img.colorspace_settings.name,
                           'minimum_shader_rgb': float(pixels[:, :, :3].min()),
                           'maximum_shader_rgb': float(pixels[:, :, :3].max()),
                           'mean_shader_rgb': list(map(float, pixels[:, :, :3].mean(axis=(0, 1)))),
                           'samples_bottom_up': samples}
        reports.append({'source': source.name, 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
                        'sheen': 0, 'normal_strength': 1, 'images': images})
    args.samples.write_text(json.dumps(reports)+'\n')
    print('FILE cape shader sampling complete:', args.samples)


if __name__ == '__main__':
    main()
