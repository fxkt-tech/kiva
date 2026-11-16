"""Final flame pigment adjustment only; geometry, UV and opaque maps frozen."""
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
        raise RuntimeError('Background only')
    source = Path(sys.argv[sys.argv.index('--') + 1])
    output = source.parent
    directory = output / 'textures'
    bpy.ops.wm.open_mainfile(filepath=str(source))
    obj = bpy.data.objects['Character_fire_fringe']
    mat = obj.data.materials[0]
    for key in ('base', 'emission'):
        node = mat.node_tree.nodes['Direct_' + key]
        bpy.data.images.remove(node.image, do_unlink=True)
    images = author.fire_maps(2048, directory, only_colors=True)
    for key, image in images.items():
        mat.node_tree.nodes['Direct_' + key].image = image
    mat.node_tree.nodes.get('Principled BSDF').inputs['Emission Strength'].default_value = 5.5
    for filename in ('uv-report.json', 'texture-report.json'):
        path = output / filename
        report = json.loads(path.read_text(encoding='utf-8'))
        report['fire_revision']['emission_strength'] = 5.5
        report['fire_revision']['pigment'] = 'Broad low-frequency red-orange body, limited gold cores; no narrow parallel vein pattern.'
        for component, value in report['source_components'].items():
            if value['role'] == 'fire_fringe':
                value['vertices'] = len(obj.data.vertices)
                attribute = obj.data.attributes.get('kiva_component_id') or obj.data.attributes.new('kiva_component_id', 'INT', 'FACE')
                attribute.data.foreach_set('value', [int(component)] * len(obj.data.polygons))
        if 'textures' in report:
            report['textures'] = [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(directory.glob('*.png'))]
        path.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    rig = bpy.data.objects['Zhuque_v11_CompatibleRig']
    builder.RIG = rig
    builder.MESHES = [o for o in bpy.data.collections['Zhuque textured character - four surface roles'].objects if o.type == 'MESH']
    metadata = json.loads(bpy.data.texts['reference-rig.json'].as_string())
    builder.NODES = {node['name']: node for node in metadata['nodes']}
    builder.reset_pose()
    bpy.ops.wm.save_as_mainfile(filepath=str(source), check_existing=False)
    print('FROZEN_SOURCE_SHA256=' + hashlib.sha256(source.read_bytes()).hexdigest(), flush=True)
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


if __name__ == '__main__':
    main()
