"""Render final textured views without saving or changing the frozen source."""
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


def main():
    if not bpy.app.background:
        raise RuntimeError('Background only')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--poses', type=Path, required=True)
    parser.add_argument('--all', action='store_true', help='Also recreate neutral front/back/threequarter')
    options = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    original = hashlib.sha256(options.source.read_bytes()).hexdigest()
    output = options.source.parent
    bpy.ops.wm.open_mainfile(filepath=str(options.source))
    builder.RIG = bpy.data.objects['Zhuque_v11_CompatibleRig']
    builder.MESHES = [o for o in bpy.data.collections['Zhuque textured character - four surface roles'].objects if o.type == 'MESH']
    data = json.loads(bpy.data.texts['reference-rig.json'].as_string())
    builder.NODES = {node['name']: node for node in data['nodes']}
    builder.reset_pose()
    bpy.context.scene.cycles.samples = 16
    if options.all:
        builder.render(output, 'textured-front')
        builder.render(output, 'textured-back', math.pi)
        builder.render(output, 'textured-back-three-quarter', math.pi * 0.82)
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
    poses = json.loads(options.poses.read_text(encoding='utf-8'))
    for pose in poses['poses']:
        if pose['name'] == 'HammerMusket:PlayerMelee { pair: HammerMusket, stage: One }:tick024':
            builder.apply_runtime_pose(poses, pose)
            builder.render(output, 'textured-runtime-hammer-attack', -math.pi / 7)
            break
    if hashlib.sha256(options.source.read_bytes()).hexdigest() != original:
        raise RuntimeError('Read-only render unexpectedly changed source file')
    print('Frozen source unchanged: ' + original, flush=True)


if __name__ == '__main__':
    main()
