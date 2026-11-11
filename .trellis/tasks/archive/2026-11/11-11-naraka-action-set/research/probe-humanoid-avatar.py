#!/usr/bin/env python3
"""Planning probe: read the original Avatar, without changing client assets."""
from pathlib import Path
import hashlib
import json
import sys

BASE = Path('/home/asuka/kiva-asset-research/naraka')
SOURCE = Path('/mnt/c/Program Files (x86)/Steam/steamapps/common/NARAKA BLADEPOINT/NarakaBladepoint_Data/StreamingAssets')
OUT = Path(__file__).resolve().parent
sys.path.insert(0, str(BASE / 'containers'))
from naraka_sparse import SparseBundle
import UnityPy


def unwrap(value):
    while isinstance(value, dict) and set(value) == {'data'}:
        value = value['data']
    return value


def run():
    key = '5/8/589302f25ec7b3dd'
    with SparseBundle(SOURCE / key) as bundle:
        cab = next(n['name'] for n in bundle.nodes if n['flags'] & 4)
        env = UnityPy.Environment()
        sf = env.load_file(bundle.open_node(cab), name=cab)
        obj = sf.objects[-5575226590203376231]
        assert obj.type.name == 'Avatar'
        tree = obj.read_typetree(check_read=True)
        assert tree['m_Name'] == 'l22_male_skeletonAvatar'
        raw = obj.get_raw_data()
        report = {
            'bundle_key': key, 'cab': cab, 'path_id': obj.path_id,
            'raw_bytes': len(raw), 'raw_sha256': hashlib.sha256(raw).hexdigest(),
            'unity_version': sf.unity_version, 'tree': tree, 'proof': bundle.report(),
        }
        target = OUT / 'humanoid-avatar.probe.json'
        target.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
        avatar = tree['m_Avatar']
        human = unwrap(avatar['m_Human'])
        skeleton = unwrap(human['m_Skeleton'])
        print(json.dumps({
            'name': tree['m_Name'], 'path_id': obj.path_id,
            'human_skeleton_keys': list(skeleton),
            'human_indices': human['m_HumanBoneIndex'],
            'human_skeleton': {k: len(v) for k, v in skeleton.items()},
            'human_scale': human['m_Scale'],
            'human_twist': {k: v for k, v in human.items() if 'Twist' in k},
            'human_description_bones': len(tree['m_HumanDescription']['m_Human']),
            'output': str(target), 'raw_sha256': report['raw_sha256'],
        }, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    run()
