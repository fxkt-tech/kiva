#!/usr/bin/env python3
"""Import the verified official release locally; never download or read game files.

python3 scripts/import-choulingke-music.py --source /path/to/audio_reference

Requires the selected MP3 plus the saved official detail.json and urls.json,
Python's standard library, FFmpeg/libvorbis and ffprobe. Only one music track is
packaged. This is an official album recording, not a recovered WEM/bank mapping.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
from urllib.parse import urlparse


ROOT = Path(__file__).resolve().parents[1]
SONG_ID = 3429659434
TITLE = '镇守_大雾幽篁_愁铃客_惊惧一夜'
ARTIST = '燕云十六声'
ARTIST_ID = 53926051
ALBUM = '江南问情 | 《燕云十六声》游戏原声音乐辑（江南卷）'
ALBUM_ID = 389835995
SOURCE_SHA256 = '8a4d16cd45864403e882615b154b69dc5b5ff12c4e56c0bc2d58e410357236aa'
SOURCE_MD5 = 'f56a6234007e7b436813c80b113a2725'
SOURCE_SIZE = 1259773
DURATION_MS = 78666
OUTPUT = 'choulingke-bgm.ogg'


def digest(path, algorithm='sha256'):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, algorithm).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def selected(rows):
    matches = [row for row in rows if row.get('id') == SONG_ID]
    require(len(matches) == 1, 'metadata must identify the selected song exactly once')
    return matches[0]


def verify_metadata(source):
    detail_path, urls_path = source / 'detail.json', source / 'urls.json'
    detail, urls = json.loads(detail_path.read_text()), json.loads(urls_path.read_text())
    require(detail['code'] == 200 and urls['code'] == 200, 'official API response failed')
    song, playback = selected(detail['songs']), selected(urls['data'])
    require(song['name'] == TITLE, 'song title does not match the selected official release')
    require([(a['id'], a['name']) for a in song['artists']] == [(ARTIST_ID, ARTIST)],
            'official artist identity mismatch')
    album = song['album']
    require((album['id'], album['name'], album['company']) == (ALBUM_ID, ALBUM, '网易游戏'),
            'official album/publisher identity mismatch')
    require(song['duration'] == DURATION_MS and song['fee'] == 0,
            'song duration/free-play metadata mismatch')
    for key, expected in {'code': 200, 'fee': 0, 'time': DURATION_MS, 'type': 'mp3',
                          'encodeType': 'mp3', 'br': 128000, 'sr': 44100,
                          'size': SOURCE_SIZE, 'md5': SOURCE_MD5}.items():
        require(playback.get(key) == expected, f'playback metadata mismatch: {key}')
    require(playback.get('freeTrialInfo') is None, 'trial snippets are not an accepted source')
    url = urlparse(playback['url'])
    require(url.scheme in ('http', 'https') and bool(url.hostname)
            and url.hostname.endswith('.music.126.net'), 'unverified official media host')
    return {
        'detail': {'file': detail_path.name, 'sha256': digest(detail_path)},
        'playback': {'file': urls_path.name, 'sha256': digest(urls_path)},
        'response_code': 200, 'fee': 0, 'free_trial_info': None,
        'media_host': url.hostname, 'duration_ms': DURATION_MS,
        'media_size': SOURCE_SIZE, 'media_md5': SOURCE_MD5,
        'note': 'Saved official full-play response verified offline; expiring media URL is not a runtime dependency.',
    }


def probe(path):
    result = subprocess.run([
        'ffprobe', '-v', 'error', '-show_entries',
        'format=duration,size:stream=codec_type,codec_name,sample_rate,channels,channel_layout,duration',
        '-of', 'json', str(path),
    ], check=True, capture_output=True, text=True)
    info = json.loads(result.stdout)
    require(len(info['streams']) == 1 and info['streams'][0]['codec_type'] == 'audio',
            'expected exactly one audio stream')
    return info


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True, help='local MP3 and official JSON response directory')
    parser.add_argument('--output', type=Path, default=ROOT / 'assets/audio')
    args = parser.parse_args()
    source = args.source.resolve()
    source_file = source / f'{SONG_ID}.mp3'
    require(source_file.stat().st_size == SOURCE_SIZE, 'source MP3 byte length mismatch')
    require(digest(source_file) == SOURCE_SHA256, 'source MP3 SHA-256 mismatch')
    require(digest(source_file, 'md5') == SOURCE_MD5, 'source MP3 official MD5 mismatch')
    verification = verify_metadata(source)
    original = probe(source_file)
    stream = original['streams'][0]
    require((stream['codec_name'], stream['sample_rate'], stream['channels']) == ('mp3', '44100', 2),
            'source MP3 stream identity mismatch')
    # MP3 container duration includes encoder delay/padding. FFmpeg honours the
    # gapless tags; the resulting Vorbis duration is checked against the release.
    require(abs(float(original['format']['duration']) - DURATION_MS / 1000) < 0.05,
            'source container duration mismatch')
    with tempfile.TemporaryDirectory(prefix='kiva-choulingke-music-') as directory:
        converted = Path(directory) / OUTPUT
        command = ['ffmpeg', '-nostdin', '-v', 'error', '-y', '-i', str(source_file),
                   '-map', '0:a:0', '-vn', '-c:a', 'libvorbis', '-q:a', '4',
                   '-map_metadata', '-1', '-fflags', '+bitexact', str(converted)]
        subprocess.run(command, check=True)
        encoded = probe(converted)
        audio = encoded['streams'][0]
        require((audio['codec_name'], audio['sample_rate'], audio['channels']) == ('vorbis', '44100', 2),
                'output must preserve source sample rate and stereo layout')
        require(abs(float(encoded['format']['duration']) - DURATION_MS / 1000) < 0.001,
                'transcoding changed the decoded musical duration')
        decode_command = ['ffmpeg', '-nostdin', '-xerror', '-v', 'error', '-i', str(converted),
                          '-map', '0:a:0', '-f', 'null', '-']
        subprocess.run(decode_command, check=True)
        version = subprocess.run(['ffmpeg', '-version'], check=True, capture_output=True,
                                 text=True).stdout.splitlines()[0]
        canonical_command = command.copy()
        canonical_command[canonical_command.index(str(source_file))] = source_file.name
        canonical_command[-1] = OUTPUT
        manifest = {
            'origin': 'official published soundtrack recording; not an extracted game WEM',
            'song': {'id': SONG_ID, 'title': TITLE, 'url': f'https://music.163.com/song?id={SONG_ID}'},
            'artist': {'id': ARTIST_ID, 'name': ARTIST, 'url': f'https://music.163.com/artist?id={ARTIST_ID}'},
            'album': {'id': ALBUM_ID, 'title': ALBUM, 'publisher': '网易游戏',
                      'url': f'https://music.163.com/album?id={ALBUM_ID}'},
            'source': {'file': source_file.name, 'sha256': SOURCE_SHA256, 'md5': SOURCE_MD5,
                       'bytes': SOURCE_SIZE, 'probe': original, 'official_metadata_verification': verification},
            'output': {'file': OUTPUT, 'sha256': digest(converted), 'bytes': converted.stat().st_size,
                       'probe': encoded, 'full_decode_verified': True},
            'conversion': {'importer': 'scripts/import-choulingke-music.py', 'ffmpeg': version,
                           'command': canonical_command,
                           'processing': 'Vorbis quality 4; source sample rate/channel count; no gain normalization, compression, pitch, tempo, trimming or fades.'},
            'boundaries': [
                'The official title/artist/album establish this release identity; local WEM, bank/event and sound_no mappings remain unresolved.',
                'Kiva repeats the complete album recording. Original game loop boundaries, stems and phase-specific switching are not recovered.',
                'Official recording retains its original rights; it is not original Kiva synthesis or a CC0 asset.',
                'Only this selected recording is packaged; playback uses a same-origin bundled Ogg and performs no source-site download.',
            ],
        }
        # All source identity checks, conversion and full decode finish before
        # touching shipped assets. No source JSON, MP3 or game file is changed.
        output = args.output.resolve()
        output.mkdir(parents=True, exist_ok=True)
        (output / OUTPUT).write_bytes(converted.read_bytes())
        (output / 'music-manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
        (output / 'music.json').write_text(json.dumps({'file': OUTPUT}, indent=2) + '\n')
        print(json.dumps({'file': OUTPUT, 'sha256': manifest['output']['sha256'],
                          'bytes': manifest['output']['bytes'], 'duration': encoded['format']['duration']}, ensure_ascii=False))


if __name__ == '__main__':
    main()
