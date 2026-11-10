#!/usr/bin/env python3
"""Rebuild only Kiva's procedural hit/deflect; preserve verified source attack.

Import attack with import-choulingke-effects.py first. An existing source attack
and its per-item provenance/hash are required, and are never regenerated here.
"""
import argparse
import array
import hashlib
import json
import math
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import wave

ROOT = Path(__file__).resolve().parents[1]
RATE = 48000


def samples(kind, duration):
    rng = random.Random(0x4B495641)
    result = []
    low = 0.0
    previous = 0.0
    for index in range(round(duration * RATE)):
        t = index / RATE
        noise = rng.uniform(-1.0, 1.0)
        low += 0.10 * (noise - low)
        if kind == 'hit':
            value = noise * math.exp(-t / 0.017) * 0.7
            value += math.sin(2 * math.pi * 140 * t) * math.exp(-t / 0.045)
            value += math.sin(2 * math.pi * 1800 * t) * math.exp(-t / 0.021) * 0.25
        else:
            value = (noise - previous) * math.exp(-t / 0.006) * 0.3
            for frequency, volume in [(1046.5, 0.4), (1677, 0.28), (2870, 0.2), (4521, 0.12)]:
                value += math.sin(2 * math.pi * frequency * t) * math.exp(-t / 0.16) * volume
        # No discontinuity at either endpoint; peak-normalized before PCM conversion.
        value *= min(1.0, t / 0.0007) * min(1.0, (duration - t) / 0.02)
        result.append(value)
        previous = noise
    peak = max(abs(value) for value in result)
    return array.array('h', (round(value / peak * 28000) for value in result))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'assets/audio')
    output = parser.parse_args().output.resolve()
    manifest = json.loads((output / 'effects-manifest.json').read_text())
    effects = {entry['name']: entry for entry in manifest['effects']}
    if len(effects) != len(manifest['effects']) or set(effects) != {'attack', 'hit', 'deflect'}:
        raise ValueError('expected exactly one attack/hit/deflect manifest entry')
    attack = effects['attack']
    if (attack.get('source', {}).get('kind') != 'extracted_game_audio'
            or attack.get('file') != 'attack.ogg'
            or not attack['source'].get('wem_sha256')
            or not attack['source'].get('event_name')):
        raise ValueError('import the verified source attack before rebuilding procedural effects')
    attack_hash = hashlib.sha256((output / 'attack.ogg').read_bytes()).hexdigest()
    if attack_hash != attack['sha256']:
        raise ValueError('source attack does not match its manifest; refusing to overwrite provenance')
    for name in ('hit', 'deflect'):
        source = effects[name].get('source', {})
        if (source.get('kind') != 'original_kiva_procedural'
                or source.get('generator') != 'scripts/generate-combat-audio.py'):
            raise ValueError(f'{name} is not owned by this procedural generator')
    prepared = []
    with tempfile.TemporaryDirectory(prefix='kiva-audio-') as temporary:
        for kind, duration in [('hit', 0.19), ('deflect', 0.65)]:
            source = Path(temporary) / f'{kind}.wav'
            with wave.open(str(source), 'wb') as stream:
                stream.setparams((1, 2, RATE, 0, 'NONE', 'not compressed'))
                pcm = samples(kind, duration)
                if sys.byteorder != 'little':
                    pcm.byteswap()
                stream.writeframes(pcm.tobytes())
            destination = Path(temporary) / f'{kind}.ogg'
            subprocess.run(['ffmpeg', '-nostdin', '-v', 'error', '-y', '-i', str(source), '-c:a', 'libvorbis',
                            '-q:a', '4', '-map_metadata', '-1', '-fflags', '+bitexact', str(destination)], check=True)
            subprocess.run(['ffmpeg', '-nostdin', '-xerror', '-v', 'error', '-i', str(destination),
                            '-f', 'null', '-'], check=True)
            data = destination.read_bytes()
            prepared.append((kind, data, {
                'name': kind, 'file': f'{kind}.ogg', 'duration': duration,
                'sha256': hashlib.sha256(data).hexdigest(),
                'source': {'kind': 'original_kiva_procedural',
                           'generator': 'scripts/generate-combat-audio.py',
                           'sample_rate': RATE, 'channels': 1,
                           'pcm16_sha256': hashlib.sha256(pcm.tobytes()).hexdigest()},
            }))
    # Only these two explicitly owned procedural assets are written. The source
    # attack entry and binary retain exactly their existing provenance/identity.
    for kind, data, entry in prepared:
        (output / f'{kind}.ogg').write_bytes(data)
        effects[kind] = entry
    manifest['effects'] = [effects[name] for name in ('attack', 'hit', 'deflect')]
    (output / 'effects-manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')


if __name__ == '__main__':
    main()
