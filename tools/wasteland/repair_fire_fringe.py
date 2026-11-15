"""Replace only the failed fire fringe, keeping all opaque atlases unchanged."""
import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

import bpy

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import build_greybox as builder
import texture_character as author


def main():
    if not bpy.app.background:
        raise RuntimeError('Use an independent background process')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--textured', type=Path, required=True)
    parser.add_argument('--refined', type=Path, required=True)
    options = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    output = options.textured.parent
    directory = output / 'textures'
    opaque = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in directory.glob('*.png') if not p.name.startswith('fire_fringe_')}
    bpy.ops.wm.open_mainfile(filepath=str(options.textured))
    rig = bpy.data.objects['Zhuque_v11_CompatibleRig']
    character = bpy.data.collections['Zhuque textured character - four surface roles']
    old = bpy.data.objects['Character_fire_fringe']
    bpy.data.objects.remove(old, do_unlink=True)
    for img in list(bpy.data.images):
        if img.name.startswith('fire_fringe_'):
            bpy.data.images.remove(img, do_unlink=True)
    with bpy.data.libraries.load(str(options.refined), link=False) as (_, data):
        data.objects = ['Broad asymmetric cape with tapered feather-fire edge']
    cape = data.objects[0]
    fringe = author.fire_fringe(character, rig, cape)
    fringe.name = 'Character_fire_fringe'
    fringe.data.materials.append(author.final_material(3, author.fire_maps(2048, directory), 5.5))
    bpy.data.objects.remove(cape, do_unlink=True)
    objects = [obj for obj in character.objects if obj.type == 'MESH']
    triangles = sum(sum(len(face.vertices) - 2 for face in obj.data.polygons) for obj in objects)
    if triangles > 40000:
        raise RuntimeError(f'Fire repair exceeds total budget: {triangles}')
    for path in directory.glob('*.png'):
        if path.name in opaque and opaque[path.name] != hashlib.sha256(path.read_bytes()).hexdigest():
            raise RuntimeError('Opaque atlas unexpectedly changed')
    for name in ('uv-report.json', 'texture-report.json'):
        path = output / name
        report = json.loads(path.read_text(encoding='utf-8'))
        report['character_triangles'] = triangles
        report['uv']['fire_fringe'] = author.uv_report(fringe, True)
        report['fire_revision'] = {'layers': 2, 'ribbons_per_layer': 15, 'fire_triangles': 1260,
                                   'surface_offset_m': 0.010, 'emission_strength': 5.5,
                                   'fix': 'Each vertex sampled on actual cloth surface; broad alpha/gold core; front and back visible. Opaque atlases unchanged.'}
        for component, source in report.get('source_components', {}).items():
            if source['role'] == 'fire_fringe':
                source['vertices'] = len(fringe.data.vertices)
                source['revision'] = 'surface sampled front/back layers'
                attribute = fringe.data.attributes.get('kiva_component_id') or fringe.data.attributes.new('kiva_component_id', 'INT', 'FACE')
                attribute.data.foreach_set('value', [int(component)] * len(fringe.data.polygons))
        if 'textures' in report:
            report['textures'] = [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(directory.glob('*.png'))]
        path.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    builder.RIG = rig
    builder.MESHES = objects
    metadata = json.loads(bpy.data.texts['reference-rig.json'].as_string())
    builder.NODES = {node['name']: node for node in metadata['nodes']}
    builder.reset_pose()
    bpy.ops.wm.save_as_mainfile(filepath=str(options.textured), check_existing=False)
    bpy.context.scene.cycles.samples = 16
    builder.render(output, 'textured-back', math.pi)
    builder.render(output, 'textured-back-three-quarter', math.pi * 0.82)
    builder.render(output, 'textured-front')
    builder.relaxed_review_pose()
    builder.render(output, 'textured-relaxed-review', -math.pi / 7)
    builder.reset_pose()
    scene = bpy.context.scene
    scene.camera.location = (-0.7, -3, 1.0)
    target = builder.Vector((-0.12, -0.24, 0.58))
    scene.camera.rotation_euler = (target - scene.camera.location).to_track_quat('-Z', 'Y').to_euler()
    scene.camera.data.ortho_scale = 0.95
    scene.render.resolution_x, scene.render.resolution_y = 1100, 1000
    scene.render.filepath = str(output / 'textured-fire-detail.png')
    bpy.ops.render.render(write_still=True)
    print(json.dumps({'triangles': triangles, 'fringe_triangles': 1260, 'opaque_atlas_hashes_unchanged': True}), flush=True)


if __name__ == '__main__':
    main()
