"""Extract a compact CC0 anatomy base from Blender's official demo bundle.

blender --background --factory-startup --python prepare_head_reference.py -- \
  --source human_base_meshes_bundle.blend --output head-source.blend
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

import bpy
from mathutils import Matrix


def main():
    if not bpy.app.background:
        raise RuntimeError('Use an independent background process')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archive', type=Path, help='Optional original downloaded archive for provenance hash')
    options = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    bpy.ops.wm.read_factory_settings(use_empty=True)
    with bpy.data.libraries.load(str(options.source), link=False) as (available, loaded):
        loaded.objects = [name for name in available.objects if name.startswith('GEO-head_animation_realistic')]
    for obj in loaded.objects:
        bpy.context.scene.collection.objects.link(obj)
    bpy.context.view_layer.update()
    head = bpy.data.objects['GEO-head_animation_realistic']
    to_local = head.matrix_world.inverted()
    transforms = {obj.name: to_local @ obj.matrix_world for obj in loaded.objects}
    for obj in loaded.objects:
        obj.parent = None
        obj.modifiers.clear()
        obj.data.materials.clear()
        obj.data.transform(transforms[obj.name])
        obj.matrix_world = Matrix.Identity(4)
    provenance = {'source': 'Blender official Human Base Meshes bundle v1.4.1',
                  'source_url': 'https://www.blender.org/download/demo-files/', 'license': 'CC0',
                  'source_blend_sha256': hashlib.sha256(options.source.read_bytes()).hexdigest(),
                  'objects': [obj.name for obj in loaded.objects],
                  'preparation': 'Only base topology, no Multires modifier/materials. Objects baked into source head local coordinates, no shape retargeting here.',
                  'source_coordinates': '+Z up, -Y face forward; later character adapter explicitly converts into Kiva game coordinates.'}
    if options.archive:
        provenance['source_archive_sha256'] = hashlib.sha256(options.archive.read_bytes()).hexdigest()
    text = bpy.data.texts.new('CC0-head-provenance.json')
    text.write(json.dumps(provenance, indent=2))
    options.output.parent.mkdir(parents=True, exist_ok=True)
    bpy.data.libraries.write(str(options.output), set(loaded.objects) | {text}, fake_user=True)
    options.output.with_suffix('.provenance.json').write_text(json.dumps(provenance, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(provenance, indent=2))


if __name__ == '__main__':
    main()
