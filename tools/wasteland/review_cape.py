"""Reopen, clean only unused cape work data, and render matched cape reviews."""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import bpy

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import build_greybox as builder
import refine_cape as cape


def normalize_cape_materials():
    """Use the common exported PBR subset and discard only our unused work data."""
    for mat in bpy.data.materials:
        if mat.get('kiva_source_role') == 'cape_detail':
            shader = mat.node_tree.nodes.get('Principled BSDF')
            shader.inputs['Sheen Weight'].default_value = 0
            for node in mat.node_tree.nodes:
                if node.type == 'NORMAL_MAP':
                    node.inputs['Strength'].default_value = 1
    for mat in list(bpy.data.materials):
        belongs = mat.name.startswith(('Cape 02 - neutral', 'Zhuque_cape_detail_PBR', 'Zhuque_cape_fire_PBR', 'Zhuque_fire_fringe_PBR'))
        if belongs and mat.users - int(mat.use_fake_user) == 0:
            mat.use_fake_user = False
            bpy.data.materials.remove(mat, do_unlink=True)
    for image in list(bpy.data.images):
        belongs = image.name.lower().startswith(('cape_detail_', 'capedetail_file_', 'fire_fringe_'))
        if belongs and image.users == 0:
            bpy.data.images.remove(image)


def old_review(source, output):
    bpy.ops.wm.open_mainfile(filepath=str(source))
    component_map = json.loads((source.parent/'texture-report.json').read_text())['source_components']
    mat = cape.material('Cape before same neutral shape review', (.19, .18, .17))
    for obj in list(bpy.context.scene.objects):
        if obj.type != 'MESH':
            continue
        attr = obj.data.attributes.get('kiva_component_id')
        if not attr:
            continue
        if obj.get('kiva_source_role') == 'fire_fringe':
            bpy.data.objects.remove(obj, do_unlink=True)
            continue
        index = len(obj.data.materials)
        obj.data.materials.append(mat)
        for polygon in obj.data.polygons:
            if component_map[str(attr.data[polygon.index].value)]['object'] in cape.REMOVED:
                polygon.material_index = index
    cape.lighting()
    cape.render(output/'cape-before-clay-back.png',cape.PI)
    cape.render(output/'cape-before-clay-neck.png',-.28,(0,0,1.39),.56,(1100,1000))


def main():
    cape.require(bpy.app.background,'Independent Blender process required')
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--original',type=Path)
    parser.add_argument('--finalize',action='store_true')
    parser.add_argument('--extra-renders',action='store_true')
    parser.add_argument('--poses',type=Path)
    opts=parser.parse_args(sys.argv[sys.argv.index('--')+1:])
    if opts.finalize:
        report=json.loads((opts.output/'cape-report.json').read_text())
        for name,key in [('zhuque-cape-runtime-source.blend','runtime_source_sha256'),('zhuque-cape-detail.blend','author_source_sha256')]:
            path=opts.output/name
            bpy.ops.wm.open_mainfile(filepath=str(path))
            normalize_cape_materials()
            bpy.ops.wm.save_as_mainfile(filepath=str(path),check_existing=False)
            report[key]=cape.sha(path)
        report['material_notes']='Independent actual FILE/packed 2K PNG cape base/normal/ORM/emission. sRGB base explicitly encoded from linear ~0.015–0.025 cloth, normal Strength1, no unsupported sheen preview. ORM.R=1 unoccluded AO; no differential AO bake. Existing body3 materials/12PNGs untouched; dedicated new fire uses exact prior4PNG bytes with new coverage.'
        report['fire_notes']='Contour-following front/back flame bands span last23.5% of cloth length plus4 wider irregular tongues. Dedicated AlphaBlend/Ember material only; opaque cloth UV fixed.'
        report['cowl_connection']='Lower cowl is embedded into existing collar under opaque right shoulder/main cape at a few seam points; no claim of global no-intersection. Parent/checker inspect visibility in actual side and neck views.'
        (opts.output/'cape-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
        print('NORMALIZED FINAL SOURCE HASHES',json.dumps({k:report[k] for k in ('author_source_sha256','runtime_source_sha256')}),flush=True)
        cape.render(opts.output/'cape-final-back.png',cape.PI)
        cape.render(opts.output/'cape-final-back-three-quarter.png',cape.PI*.78)
        cape.render(opts.output/'cape-final-neck.png',-.28,(0,0,1.39),.56,(1100,1000))
        cape.render(opts.output/'cape-final-side.png',cape.PI/2)
    if opts.extra_renders:
        bpy.ops.wm.open_mainfile(filepath=str(opts.output/'zhuque-cape-detail.blend'))
        builder.RIG=bpy.data.objects['Zhuque_v11_CompatibleRig']
        metadata=json.loads(bpy.data.texts['reference-rig.json'].as_string())
        builder.NODES={node['name']:node for node in metadata['nodes']}
        builder.reset_pose()
        cape.render(opts.output/'cape-final-front.png',0)
        cape.render(opts.output/'cape-final-hem.png',cape.PI*.92,(.10,-.22,.46),.75,(1100,1000))
        poses=json.loads(opts.poses.read_text())
        builder.apply_runtime_pose(poses,next(p for p in poses['poses'] if 'HammerMusket:PlayerMelee' in p['name'] and 'tick024' in p['name']))
        cape.render(opts.output/'cape-runtime-hammer-attack.png',-cape.PI/7)
        builder.apply_runtime_pose(poses,next(p for p in poses['poses'] if p['name']=='ChainAxeBow:game:crouch:0.50'))
        cape.render(opts.output/'cape-runtime-crouch.png',-cape.PI/7)
    if opts.original:
        old_review(opts.original,opts.output)


if __name__=='__main__':
    main()
