#!/usr/bin/env python3
"""Rebuild Kiva's original procedural fallback effects (Python stdlib + ffmpeg)."""
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
        if kind == 'attack':
            phase = t / duration
            envelope = math.sin(math.pi * phase) ** 2
            value = envelope * ((noise - low) * 0.6 + low * 1.8)
        elif kind == 'hit':
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
    output = ROOT / 'assets/audio'
    output.mkdir(exist_ok=True)
    manifest = {'effects_origin': 'original Kiva procedural synthesis; not extracted game audio',
                'generator': 'scripts/generate-combat-audio.py', 'effects': []}
    with tempfile.TemporaryDirectory(prefix='kiva-audio-') as temporary:
        for kind, duration in [('attack', 0.36), ('hit', 0.19), ('deflect', 0.65)]:
            source = Path(temporary) / f'{kind}.wav'
            with wave.open(str(source), 'wb') as stream:
                stream.setparams((1, 2, RATE, 0, 'NONE', 'not compressed'))
                pcm = samples(kind, duration)
                if sys.byteorder != 'little':
                    pcm.byteswap()
                stream.writeframes(pcm.tobytes())
            destination = output / f'{kind}.ogg'
            subprocess.run(['ffmpeg', '-v', 'error', '-y', '-i', str(source), '-c:a', 'libvorbis',
                            '-q:a', '4', '-map_metadata', '-1', str(destination)], check=True)
            manifest['effects'].append({'name': kind, 'duration': duration,
                                        'sha256': hashlib.sha256(destination.read_bytes()).hexdigest()})
    (output / 'effects-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


if __name__ == '__main__':
    main()
