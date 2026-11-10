#!/usr/bin/env python3
"""Import one verified complete original swing; preserve procedural hit/deflect.

python3 scripts/import-choulingke-effects.py --source /path/to/choulingke \
  --vgmstream /path/to/vgmstream-cli

Offline only. Requires extracted WEM/bank and mesh_verified_swing_media.json.
The shared polearm cue is adapted to Kiva's longblade, not Boss-exclusive audio.
"""
import argparse
import array
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import wave

ROOT = Path(__file__).resolve().parents[1]
EVENT = 'char_skill_qiang_end_swing_fast_1p'
MEDIA = 145056080
STEM = 'mesh_swing_145056080_p278_109633301'
WEM_SHA = 'a1db4110a426cb0c88134cefc1d6a5a3963b1f9401fc1556ae678027e4165fd5'
WAV_SHA = '07151daa87f3007a94b9102fc3a9d63d7c2f62e07217cf11f0ce081413e1f522'
BANK_SHA = '5f8a35ea6736577cbd00109ba65fc355aeec97232c19a6f606ad003f6154b800'
DECODER_SHA = 'e736da315053bf91796d01eca18b54cbcc2d5b328eecdaf15786e7838251f581'
GENERATOR = 'scripts/generate-combat-audio.py'
FRAMES, RATE, CHANNELS = 34332, 48000, 2
TARGET_PEAK = 0.85


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def preserved_procedural(output):
    manifest = json.loads((output / 'effects-manifest.json').read_text())
    entries = {item['name']: item for item in manifest['effects']}
    require(len(entries) == len(manifest['effects']) and set(entries) == {'attack', 'hit', 'deflect'},
            'expected exactly one attack/hit/deflect manifest entry')
    for name, duration in [('hit', 0.19), ('deflect', 0.65)]:
        item = entries[name]
        require(item['sha256'] == digest(output / f'{name}.ogg'), f'{name} no longer matches its manifest')
        require(item['duration'] == duration, f'{name} procedural duration mismatch')
        if 'source' not in item:
            require(manifest.get('generator') == GENERATOR
                    and manifest.get('effects_origin') == 'original Kiva procedural synthesis; not extracted game audio',
                    'legacy procedural provenance is unverified')
            item['source'] = {'kind': 'original_kiva_procedural', 'generator': GENERATOR,
                              'sample_rate': 48000, 'channels': 1}
        require(item['source']['kind'] == 'original_kiva_procedural'
                and item['source']['generator'] == GENERATOR, f'{name} is not owned by the procedural generator')
        item['file'] = f'{name}.ogg'
    return [entries['hit'], entries['deflect']]


def verified_chain(source):
    evidence = source / 'audio_followup/mesh_verified_swing_media.json'
    report = json.loads(evidence.read_text())
    matches = [row for row in report['chains'] if row['source_id'] == MEDIA and row['event_name'] == EVENT]
    require(len(matches) == 1, 'source event/media chain is missing or ambiguous')
    chain = matches[0]
    require((chain['event_id'], chain['action_id'], chain['action_type'],
             chain['random_container_id'], chain['sound_id']) ==
            (1897049250, 814896891, 1027, 893021932, 1041100646), 'source HIRC identity mismatch')
    require(chain['source_media_path'] == f'Sounds/Media/{MEDIA}.wem'
            and chain['source_language_specific'] is False, 'source media path/language mismatch')
    tail = chain['container_tail']
    require(set(tail['children']) == {53244466, 58426737, 1041100646}
            and {item['id'] for item in tail['playlist']} == set(tail['children'])
            and all(item['weight'] == 50000 for item in tail['playlist']), 'source random playlist mismatch')
    bank = source / f'audio_followup/mesh_{EVENT}.bnk'
    require(digest(bank) == BANK_SHA and bank.stat().st_size == 917, 'source bank identity mismatch')
    require((chain['bank']['pack'], chain['bank']['offset'], chain['bank']['sha256']) ==
            (18010, 4685489, BANK_SHA), 'bank physical extraction evidence mismatch')
    require(len(chain['media_extractions']) == 1, 'ambiguous physical media extraction')
    media = chain['media_extractions'][0]
    physical = media['physical']
    require((physical['pack'], physical['offset'], physical['decoded_bytes'], physical['sha256']) ==
            (278, 109633301, 6472, WEM_SHA), 'media physical extraction evidence mismatch')
    require(physical['paths'] == [f'Sounds/Media/{MEDIA}.wem'], 'media VFS identity mismatch')
    require(media['expected_samples'] == FRAMES and media['codec'] == 0xffff
            and media['measurement']['wav_sha256'] == WAV_SHA, 'decoded source evidence mismatch')
    cues = []
    expected_data = f'Sounds/;char_skill_qiang.bnk;{EVENT};0;1;0;0;0;0'
    for use in chain['cue_uses']:
        cue = use['cue']
        require(cue['raw_data'] == expected_data and EVENT in cue['events']
                and cue['bank'] == 'char_skill_qiang.bnk' and 0 <= cue['time_per'] <= 1,
                'original animation cue does not name the source event')
        require(use['res'].startswith('Char/b_01_char_male/11_qiang/8_skill/'),
                'unexpected original source animation family')
        cues.append({'clip': use['clip'], 'res': use['res'],
                     'time_per': cue['time_per'], 'raw_data': cue['raw_data']})
    require(bool(cues), 'the original animation cue is required')
    wem = source / f'audio_followup/{STEM}.wem'
    require(digest(wem) == WEM_SHA and wem.stat().st_size == 6472, 'source WEM SHA/size mismatch')
    raw = wem.read_bytes()
    require(raw[:4] == b'RIFF' and raw[8:12] == b'WAVE'
            and struct.unpack_from('<I', raw, 4)[0] + 8 == len(raw), 'source is not a complete RIFF WEM')
    return wem, chain, cues, evidence


def probe(path):
    result = subprocess.run(['ffprobe', '-v', 'error', '-show_entries',
                             'format=duration:stream=codec_name,sample_rate,channels',
                             '-of', 'json', str(path)], check=True, capture_output=True, text=True)
    return json.loads(result.stdout)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--vgmstream', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=ROOT / 'assets/audio')
    args = parser.parse_args()
    output, decoder = args.output.resolve(), args.vgmstream.resolve()
    preserved = preserved_procedural(output)
    wem, chain, cues, evidence = verified_chain(args.source.resolve())
    require(digest(decoder) == DECODER_SHA, 'expected verified offline vgmstream decoder')
    with tempfile.TemporaryDirectory(prefix='kiva-original-swing-') as temporary:
        wav = Path(temporary) / 'source.wav'
        decoded = subprocess.run([str(decoder), '-i', '-o', str(wav), str(wem)], check=True,
                                 capture_output=True, text=True)
        require(digest(wav) == WAV_SHA, 'fresh ignoreloop decode differs from the verified full source PCM')
        with wave.open(str(wav), 'rb') as stream:
            require((stream.getnframes(), stream.getframerate(), stream.getnchannels(), stream.getsampwidth()) ==
                    (FRAMES, RATE, CHANNELS, 2), 'full decoded source samples/rate/channels mismatch')
            pcm = array.array('h', stream.readframes(FRAMES))
        if sys.byteorder != 'little':
            pcm.byteswap()
        original_peak = max(abs(value) for value in pcm) / 32768
        require(0 < original_peak < 1, 'invalid original peak')
        gain = TARGET_PEAK / original_peak
        volume = f'volume={gain:.17g}:precision=double'
        destination = Path(temporary) / 'attack.ogg'
        command = ['ffmpeg', '-nostdin', '-v', 'error', '-y', '-i', str(wav),
                   '-map', '0:a:0', '-af', volume, '-c:a', 'libvorbis', '-q:a', '4',
                   '-map_metadata', '-1', '-fflags', '+bitexact', str(destination)]
        subprocess.run(command, check=True)
        encoded = probe(destination)
        require(len(encoded['streams']) == 1 and encoded['streams'][0] ==
                {'codec_name': 'vorbis', 'sample_rate': '48000', 'channels': 2}, 'output stream identity mismatch')
        require(abs(float(encoded['format']['duration']) - FRAMES / RATE) < 0.000001,
                'transcoding changed the full source duration')
        float_decode = subprocess.run(['ffmpeg', '-nostdin', '-xerror', '-v', 'error', '-i', str(destination),
                                       '-map', '0:a:0', '-c:a', 'pcm_f32le', '-f', 'f32le', '-'],
                                      check=True, capture_output=True)
        samples = array.array('f', float_decode.stdout)
        if sys.byteorder != 'little':
            samples.byteswap()
        require(samples and all(math.isfinite(value) for value in samples), 'invalid complete float decode')
        encoded_peak = max(abs(value) for value in samples)
        require(encoded_peak < 0.95, 'Vorbis float overshoot exceeds the Kiva headroom bound')
        media = chain['media_extractions'][0]
        entry = {
            'name': 'attack', 'file': 'attack.ogg', 'duration': FRAMES / RATE,
            'sha256': digest(destination),
            'source': {
                'kind': 'extracted_game_audio', 'event_name': EVENT, 'event_id': chain['event_id'],
                'animation_cues': cues, 'bank': chain['bank'], 'bank_sha256': BANK_SHA,
                'action_id': chain['action_id'], 'action_type': chain['action_type'],
                'random_container_id': chain['random_container_id'], 'playlist': chain['container_tail'],
                'sound_id': chain['sound_id'], 'media_id': MEDIA,
                'media_vfs_path': chain['source_media_path'], 'physical': media['physical'],
                'wem_sha256': WEM_SHA, 'full_decoded_wav_sha256': WAV_SHA,
                'evidence_file': evidence.name, 'evidence_sha256': digest(evidence),
                'selection': 'One verified fast end-swing variant selected from the equal-weight source playlist by source identity, complete duration and signal measurements; not a subjective audition claim.',
            },
            'conversion': {
                'importer': 'scripts/import-choulingke-effects.py',
                'decoder': {'tool': 'vgmstream-r2168/vgmstream-cli', 'sha256': DECODER_SHA,
                            'arguments': ['-i', '-o', 'source.wav', wem.name],
                            'stdout': decoded.stdout.replace(str(wav), 'source.wav').replace(str(wem), wem.name)},
                'command': ['ffmpeg', '-nostdin', '-v', 'error', '-y', '-i', 'source.wav',
                            '-map', '0:a:0', '-af', volume, '-c:a', 'libvorbis', '-q:a', '4',
                            '-map_metadata', '-1', '-fflags', '+bitexact', 'attack.ogg'],
                'source_frames': FRAMES, 'sample_rate': RATE, 'channels': CHANNELS,
                'original_peak_linear': original_peak, 'original_peak_dbfs': 20 * math.log10(original_peak),
                'target_pcm_peak_linear': TARGET_PEAK, 'constant_linear_gain': gain,
                'vorbis_decoded_float_peak_linear': encoded_peak,
                'vorbis_decoded_float_frames': len(samples) // CHANNELS,
                'priming_observation': {
                    'source_and_ogg_granule_frames': FRAMES,
                    'ffmpeg_float_decode_frames': len(samples) // CHANNELS,
                    'difference_frames': FRAMES - len(samples) // CHANNELS,
                    'note': 'FFmpeg observes 128 fewer float frames for this Vorbis encode/decode priming; the complete source PCM was supplied to the encoder, with no manual excerpt or truncation. Ogg granule duration and decoded frame count are reported separately.',
                },
                'full_float_decode_verified': True,
            },
            'adaptation': [
                'Shared original polearm/dual-polearm end-swing adapted to Kiva longblade attacks; not Choulingke-exclusive blade audio or a successful-deflection impact.',
                'Complete 0.71525-second source retained, ignore source loop flags; one constant Kiva gain targets PCM peak 0.85. No cropping, dynamic-range compression, splicing, pitch/tempo changes or fades. Vorbis remains lossy encoding.',
                'Triggered by the existing authoritative Kiva attack feedback; original animation Cue timing and weighted random variation are not reconstructed.',
                'Signal statistics and original event provenance are verified; subjective listening was not available in this environment.',
            ],
        }
        manifest = {'schema_version': 2, 'effects': [entry, *preserved]}
        # All source/evidence/decoding checks finish before shipped files change.
        # The procedural binary files are deliberately never written here.
        data = destination.read_bytes()
        target = output / 'attack.ogg'
        if not target.exists() or target.read_bytes() != data:
            target.write_bytes(data)
        (output / 'effects-manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
        print(json.dumps({'file': 'attack.ogg', 'sha256': entry['sha256'], 'duration': entry['duration'],
                          'constant_gain': gain, 'float_peak': encoded_peak, 'float_frames': len(samples) // CHANNELS}))


if __name__ == '__main__':
    main()
