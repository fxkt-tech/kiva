"""Create or verify an independent reference .blend with Windows Blender.

blender --background --factory-startup --python import_reference.py -- \
    --reference reference-rig.json --output reference-rig.blend
blender --background --factory-startup --python import_reference.py -- \
    --reference reference-rig.json --output reference-rig.blend --verify

This intentionally refuses to run in an interactive Blender session.
"""
import argparse
import json
import sys
from pathlib import Path

import bpy
from mathutils import Matrix, Vector

RIG_NAME = 'Kiva_DefaultMale_CompatibilityReference'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def flatten(value):
    return [float(v) for row in value for v in row]


def maximum_difference(a, b):
    return max(abs(x - y) for x, y in zip(flatten(a), flatten(b)))


def transformed_rest(node, basis):
    return basis @ Matrix(node['source_rest_game']) @ basis.inverted()


def create(data, output):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.unit_settings.system = 'METRIC'
    scene.unit_settings.scale_length = 1.0
    basis = Matrix(data['coordinates']['game_to_blender'])
    rig_collection = bpy.data.collections.new('Compatibility reference - no character mesh')
    scene.collection.children.link(rig_collection)
    armature = bpy.data.armatures.new(RIG_NAME)
    rig = bpy.data.objects.new(RIG_NAME, armature)
    rig_collection.objects.link(rig)
    rig.show_in_front = True
    armature.display_type = 'STICK'
    rig['purpose'] = 'Extracted reference skeleton only; no finished model or animation validation.'
    rig['height_m'] = 1.73
    rig['original_metadata_text'] = 'reference-rig.json'
    rig['matrix_contract'] = 'source_rest_blender = bone.matrix_local @ kiva_source_correction_json; original IBM is separate metadata, not inverse current rest.'
    bpy.context.view_layer.objects.active = rig
    rig.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT')
    nodes = {n['name']: n for n in data['nodes']}
    for node in data['nodes']:
        name = node['name']
        bone = armature.edit_bones.new(name)
        raw = transformed_rest(node, basis)
        location, rotation, _scale = raw.decompose()
        # Blender edit-bone matrices have unit scale. Retain that difference
        # explicitly instead of replacing the source rest or original IBM.
        rigid = Matrix.LocRotScale(location, rotation, Vector((1, 1, 1)))
        bone.head = location
        bone.tail = location + Vector((0, 0.03, 0))
        bone.matrix = rigid
        children = [n for n in data['nodes'] if n['parent'] == name]
        extent = min(((transformed_rest(n, basis).translation - location).length for n in children), default=0.025)
        bone.length = max(0.015, min(extent, 0.2))
        bone.use_deform = name != 'root'
        if node['parent']:
            bone.parent = armature.edit_bones[node['parent']]
            bone.use_connect = False
    bpy.ops.object.mode_set(mode='OBJECT')
    for bone in armature.bones:
        node = nodes[bone.name]
        correction = bone.matrix_local.inverted() @ transformed_rest(node, basis)
        bone['kiva_native_name'] = bone.name
        bone['kiva_driver_index'] = node['driver_index']
        bone['kiva_source_rest_game_json'] = json.dumps(node['source_rest_game'])
        bone['kiva_source_correction_json'] = json.dumps([list(row) for row in correction])
    anchors = bpy.data.collections.new('Runtime structural anchors - distinct from skin joints')
    scene.collection.children.link(anchors)
    for driver in data['drivers']:
        item = bpy.data.objects.new('anchor::' + driver['name'], None)
        item.empty_display_type = 'SPHERE'
        item.empty_display_size = 0.009
        item.location = basis @ Vector(driver['position_game'])
        item['kiva_driver_name'] = driver['name']
        anchors.objects.link(item)
    metadata = bpy.data.texts.new('reference-rig.json')
    metadata.write(json.dumps(data, ensure_ascii=False, indent=2))
    notes = bpy.data.texts.new('READ_ME')
    notes.write('Reference skeleton only. No original character geometry copied and no new mesh built.\n'
                '52 gMan deform joints + 1 reference root; 30 finger joints.\n'
                'Actual skin rest and runtime structural anchors are both retained; calves differ ~3.1cm in the existing avatar.\n'
                'Bone tail lengths are display aids along native bone Y axes, not claims of anatomical segment endpoints.\n'
                'Do not copy old IBM to a new mesh. Bind the new mesh in its own reference pose.\n'
                'A later exporter must explicitly consume the rest-frame correction; plain GLB export does not preserve this custom contract automatically.\n')
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type == 'VIEW_3D':
                area.spaces.active.region_3d.view_distance = 2.5
                area.spaces.active.region_3d.view_location = Vector((0, 0, 0.85))
    output.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(output), check_existing=False)


def verify(data, output):
    # Re-open the on-disk artifact, so this validates Blender serialization too.
    bpy.ops.wm.open_mainfile(filepath=str(output))
    rig = bpy.data.objects[RIG_NAME]
    basis = Matrix(data['coordinates']['game_to_blender'])
    require(len(rig.data.bones) == len(data['nodes']) == 53, 'wrong reference bone count')
    require(json.loads(bpy.data.texts['reference-rig.json'].as_string()) == data, 'embedded reference changed')
    matrix_error, position_error = 0.0, 0.0
    for node in data['nodes']:
        bone = rig.data.bones[node['name']]
        require((bone.parent.name if bone.parent else None) == node['parent'], 'wrong native parent')
        require(json.loads(bone['kiva_source_rest_game_json']) == node['source_rest_game'], 'original rest metadata changed')
        expected = transformed_rest(node, basis)
        actual = bone.matrix_local @ Matrix(json.loads(bone['kiva_source_correction_json']))
        matrix_error = max(matrix_error, maximum_difference(actual, expected))
        position_error = max(position_error, (bone.head_local - expected.translation).length)
        require(bone.use_deform == (node['name'] != 'root'), 'incorrect deform flag')
    require(matrix_error < 1e-5 and position_error < 1e-6, 'Blender reference round trip exceeded tolerance')
    anchor_error = 0.0
    for driver in data['drivers']:
        item = bpy.data.objects['anchor::' + driver['name']]
        anchor_error = max(anchor_error, (item.location - basis @ Vector(driver['position_game'])).length)
    require(anchor_error < 1e-6, 'structural anchor changed')
    require(not any(obj.type == 'MESH' for obj in bpy.data.objects), 'reference unexpectedly contains mesh')
    report = {'artifact': output.name, 'blender_version': bpy.app.version_string,
              'reference_bones': len(rig.data.bones), 'deform_bones': 52,
              'structural_anchors': len(data['drivers']), 'meshes': 0,
              'rest_reconstruction_max_matrix_error': matrix_error,
              'native_joint_head_max_position_error_m': position_error,
              'structural_anchor_max_position_error_m': anchor_error,
              'source_rest_and_ibm_kept_separate': True,
              'limits': 'Reference skeleton round-trip only. No new mesh, skinning, attack playback or in-game acceptance yet.'}
    output.with_suffix('.report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))


def main():
    require(bpy.app.background, 'Run only with --background on a separate Blender process')
    args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--verify', action='store_true')
    options = parser.parse_args(args)
    data = json.loads(options.reference.read_text(encoding='utf-8'))
    require(data['schema'] == 'kiva-compatibility-reference-v1', 'unsupported reference schema')
    if not options.verify:
        create(data, options.output)
    verify(data, options.output)


if __name__ == '__main__':
    main()
