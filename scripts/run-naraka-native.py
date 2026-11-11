#!/usr/bin/env python3
"""Run the task-owned native Unity fixture on frozen original clip/Avatar inputs.

The environment is provisioned separately. This runner never launches the user
Naraka executable, modifies its installation, or installs/activates an Editor.
"""
import argparse
import json
from pathlib import Path
import shutil
import hashlib
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--environment', type=Path, required=True)
    p.add_argument('--bundle-manifest', type=Path, required=True)
    p.add_argument('--prepared', type=Path, default=ROOT / 'assets/prepared/naraka/animations')
    p.add_argument('--output-dir', type=Path,
                   help='copy successful native capture/report to an external task cache')
    p.add_argument('--clip', action='append', help='repeat exact source name; omit for all frozen clips')
    p.add_argument('--with-real-skin', action='store_true')
    p.add_argument('--fps', type=int, choices=(60, 120, 137), default=60,
                   help='base capture rate; 137 with an offset is an independent interpolation reference')
    p.add_argument('--time-offset', type=float, default=0.,
                   help='offset interior reference samples in seconds; endpoints remain exact')
    p.add_argument('--adaptive', action='store_true',
                   help='refine native midpoint angular/FK error on top of the base time grid')
    args = p.parse_args()
    location = json.loads((args.environment / 'fixture-location.json').read_text())
    fixture = Path(location['linux'])
    data = fixture / 'NarakaOracle_Data'
    inputs = data / 'Inputs'
    inputs.mkdir(exist_ok=True)
    manifest = json.loads(args.bundle_manifest.read_text())
    selection = json.loads((args.prepared / 'selection.json').read_text())
    by_name = {c['name']: c for c in selection['clips']}
    wanted = set(args.clip) if args.clip else set(by_name)
    if not wanted <= set(by_name):
        raise ValueError('Unknown source clip names: ' + str(wanted - set(by_name)))
    if args.clip:
        # Only include hand defaults if a requested clip actually needs them.
        for name in list(wanted):
            raw = json.loads((args.prepared / 'source/clips' / (name + '.json')).read_text())['tree']
            count = sum(x['typeID'] == 95 and 42 <= x['attribute'] <= 136
                        for x in raw['m_ClipBindingConstant']['genericBindings'])
            if count < 95 and 'finger' not in name:
                prefix = ('female_unarmed_finger_default' if name.startswith('female_sniper_') else
                          'male_unarmed_finger_default' if name.startswith(('male_sniper_', 'male_fk_')) else
                          'male_bow_finger_default' if name.startswith('male_bow_') else 'male_hengdao_finger')
                wanted |= {prefix + '_left_01', prefix + '_right_01'}
    def win(path):
        return subprocess.check_output(['wslpath', '-w', str(path)], text=True).strip()
    def copy(source):
        source = Path(source)
        target = inputs / source.name
        if not target.exists() or hashlib.sha256(target.read_bytes()).digest() != hashlib.sha256(source.read_bytes()).digest():
            shutil.copy2(source, target)
        return win(target)
    clips = []
    for bundle in manifest['animation_bundles']:
        matches = [c for c in bundle['selected_clips'] if c['name'] in wanted]
        if not matches:
            continue
        if bundle.get('canonical_sha256') and hashlib.sha256(Path(bundle['path']).read_bytes()).hexdigest() != bundle['canonical_sha256']:
            raise ValueError('Canonical source bundle hash changed: ' + bundle['path'])
        path = copy(bundle['path'])
        for item in matches:
            if item['raw_sha256'] != by_name[item['name']]['raw_sha256'] or item['path'] != by_name[item['name']]['path']:
                raise ValueError('Native input bundle source selection changed: ' + item['name'])
            raw = json.loads((args.prepared / 'source/clips' / (item['name'] + '.json')).read_text())['tree']
            attrs = {x['attribute'] for x in raw['m_ClipBindingConstant']['genericBindings'] if x['typeID'] == 95}
            prefix = ('female_unarmed_finger_default' if item['name'].startswith('female_sniper_') else
                      'male_unarmed_finger_default' if item['name'].startswith(('male_sniper_', 'male_fk_')) else
                      'male_bow_finger_default' if item['name'].startswith('male_bow_') else 'male_hengdao_finger')
            clips.append({'default_left': prefix + '_left_01', 'default_right': prefix + '_right_01', 'name': item['name'], 'path': item['path'], 'bundle': path,
                          'source_sha256': item['raw_sha256'], 'duration': by_name[item['name']]['duration'],
                          'authored_muscle_mask': [42 + i in attrs for i in range(95)]})
    clips.sort(key=lambda c: (c['name'] != 'male_hengdao_attack_light_01', c['name']))
    if {c['name'] for c in clips} != wanted:
        raise ValueError('Native manifest does not cover the complete requested clip set')
    hand_input = args.prepared / 'source/real-hand-lbs.json'
    if not hand_input.exists():
        hand_input = args.bundle_manifest.parent / 'real-hand-lbs.json'
    config = {'rig_bundle': copy(manifest['rig_bundle']), 'rig_path': manifest['rig_path'],
              'avatar_sha256': selection['avatar']['raw_sha256'],
              'rig_json': copy(args.prepared / 'source/rig.json'),
              'hand_mesh_json': copy(hand_input) if args.with_real_skin else '',
              'output': win(data / 'native-poses.bin'), 'report': win(data / 'native-report.json'),
              'clips': clips, 'fps': args.fps, 'adaptive': args.adaptive, 'time_offset': args.time_offset}
    (data / 'KivaOracle.input.json').write_text(json.dumps(config, ensure_ascii=False, indent=2) + '\n')
    subprocess.run(['python3', str(args.environment / 'compile-sampler.py'),
                    str(ROOT / 'scripts/naraka-native/NativeSampler.cs')], check=True)
    subprocess.run(['python3', str(args.environment / 'run-fixture.py')], check=True)
    report = json.loads((data / 'native-report.json').read_text())
    print(json.dumps({'passed': report.get('passed'), 'clips': len(report.get('clips', [])),
                      'fps': args.fps, 'report': str(data / 'native-report.json'),
                      'capture': str(data / 'native-poses.bin')}, ensure_ascii=False, indent=2))
    if not report.get('passed'):
        raise SystemExit(2)
    if args.output_dir:
        output = args.output_dir.resolve()
        if output.is_relative_to(ROOT):
            raise ValueError('Keep intermediate native captures in an external task cache')
        output.mkdir(parents=True, exist_ok=True)
        for name in ('native-poses.bin', 'native-report.json'):
            shutil.copy2(data / name, output / name)


if __name__ == '__main__':
    main()
