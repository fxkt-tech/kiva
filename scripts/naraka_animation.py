"""Bounded Naraka scalar-curve decoding and verified-local-pose packing.

This module deliberately does not approximate Unity's native Humanoid solver.
Original muscle values and authored masks remain separate from baked bone TRS.
Only the extraction CLI imports UnityPy or reads a game installation.
"""
from __future__ import annotations

from bisect import bisect_right
from dataclasses import dataclass
import hashlib
import json
import math
import struct
import zlib
from typing import Any

MAX_SCALARS = 4096
MAX_FRAMES = 36_001
MAX_VALUES = 4_000_000
MAX_JOINTS = 4096
MAX_CLIPS = 256
MAX_SECONDS = 600.0
MAGIC = b'KIVASR02'
RIG_ID = 'naraka/l22_male_skeleton'


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def unwrap(value: Any) -> Any:
    while isinstance(value, dict) and set(value) == {'data'}:
        value = value['data']
    return value


def finite(value: Any, label: str = 'number') -> float:
    require(isinstance(value, (int, float)) and not isinstance(value, bool), f'invalid {label}')
    result = float(value)
    require(math.isfinite(result), f'nonfinite {label}')
    return result


def integer(value: Any, low: int, high: int, label: str) -> int:
    require(isinstance(value, int) and not isinstance(value, bool) and low <= value <= high,
            f'invalid {label}')
    return value


def vector(value: Any, width: int, label: str) -> list[float]:
    if isinstance(value, dict):
        keys = 'xyzw'[:width]
        require(all(k in value for k in keys), f'incomplete {label}')
        value = [value[k] for k in keys]
    require(isinstance(value, list) and len(value) == width, f'invalid {label} width')
    return [finite(x, label) for x in value]


def quaternion(value: Any, label: str = 'rotation') -> list[float]:
    q = vector(value, 4, label)
    norm = math.sqrt(sum(x * x for x in q))
    require(abs(norm - 1.0) < 0.002, f'nonunit {label}')
    return [x / norm for x in q]


def local_trs(value: dict) -> list[float]:
    t = vector(value.get('t', value.get('position')), 3, 'translation')
    q = quaternion(value.get('q', value.get('rotation')))
    s = vector(value.get('s', value.get('scale')), 3, 'scale')
    require(min(abs(x) for x in s) > 1e-8, 'singular scale')
    return t + q + s


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True,
                       separators=(',', ':')) + '\n').encode('utf-8')


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


@dataclass(frozen=True)
class Binding:
    type_id: int
    custom_type: int
    path_hash: int
    attribute: int
    offset: int
    width: int

    @property
    def muscle(self) -> int | None:
        if self.type_id == 95 and self.custom_type == 8 and 42 <= self.attribute <= 136:
            return self.attribute - 42
        return None


class ScalarClip:
    """Decode Unity streamed polynomial, dense and constant scalar sections.

    Bindings expand to widths before indexing. Missing muscle attributes are
    represented by a mask, never confused with an explicitly authored zero.
    """
    def __init__(self, tree: dict):
        require(isinstance(tree, dict), 'clip tree is not an object')
        self.name = tree['m_Name']
        require(isinstance(self.name, str) and 0 < len(self.name.encode()) <= 1024, 'invalid clip name')
        require(not tree.get('m_Legacy') and not tree.get('m_Compressed'),
                'legacy/packed rotation clip requires a separate verified decoder')
        muscle = tree['m_MuscleClip']
        clip = unwrap(muscle['m_Clip'])
        self.start = finite(muscle['m_StartTime'], 'clip start')
        self.stop = finite(muscle['m_StopTime'], 'clip stop')
        self.duration = self.stop - self.start
        require(0 <= self.start < self.stop and self.duration <= MAX_SECONDS, 'invalid clip time range')
        self.rate = finite(tree['m_SampleRate'], 'sample rate')
        require(0 < self.rate <= 1000, 'invalid sample rate')
        self.looping = bool(muscle['m_LoopTime'])
        stream = clip['m_StreamedClip']
        self.stream_count = integer(stream['curveCount'], 0, MAX_SCALARS, 'stream curve count')
        words = stream['data']
        require(isinstance(words, list) and len(words) <= MAX_VALUES, 'oversized streamed payload')
        for word in words:
            integer(word, 0, 0xffffffff, 'stream word')
        raw = struct.pack('<' + 'I' * len(words), *words)
        self.stream_keys: list[list[tuple[float, tuple[float, ...]]]] = [[] for _ in range(self.stream_count)]
        at, previous_time = 0, -math.inf
        while at < len(raw):
            require(at + 8 <= len(raw), 'truncated streamed frame')
            time, count = struct.unpack_from('<fi', raw, at)
            at += 8
            require(not math.isnan(time) and time >= previous_time, 'unordered streamed frame time')
            require(time != -math.inf, 'invalid negative-infinity streamed sentinel')
            require(time != math.inf or at + count * 20 == len(raw), 'nonfinal infinity sentinel')
            require(0 <= count <= self.stream_count and at + count * 20 <= len(raw), 'invalid streamed key count')
            seen = set()
            for _ in range(count):
                index, *coeff = struct.unpack_from('<i4f', raw, at)
                at += 20
                require(0 <= index < self.stream_count and index not in seen, 'invalid/duplicate streamed index')
                require(all(math.isfinite(x) for x in coeff), 'nonfinite streamed coefficient')
                seen.add(index)
                if time == math.inf:
                    continue  # End marker is not an evaluable key.
                keys = self.stream_keys[index]
                require(not keys or time > keys[-1][0], 'duplicate streamed key time')
                keys.append((time, tuple(coeff)))
            previous_time = time
        for keys in self.stream_keys:
            require(bool(keys) and keys[0][0] <= self.start, 'stream curve lacks an initial value')
        self.stream_times = [[t for t, _ in keys] for keys in self.stream_keys]
        dense = clip['m_DenseClip']
        self.dense_count = integer(dense['m_CurveCount'], 0, MAX_SCALARS, 'dense curve count')
        self.frame_count = integer(dense['m_FrameCount'], 0, MAX_FRAMES, 'dense frame count')
        self.dense_rate = finite(dense['m_SampleRate'], 'dense sample rate')
        self.dense_begin = finite(dense['m_BeginTime'], 'dense begin')
        self.dense = dense['m_SampleArray']
        require(self.frame_count * self.dense_count <= MAX_VALUES, 'oversized dense payload')
        require(len(self.dense) == self.frame_count * self.dense_count, 'dense sample count mismatch')
        if self.dense_count:
            require(self.frame_count > 0 and 0 < self.dense_rate <= 1000, 'invalid dense layout')
        self.dense = [finite(x, 'dense sample') for x in self.dense]
        self.constant = [finite(x, 'constant sample') for x in unwrap(clip['m_ConstantClip'])]
        self.scalar_count = self.stream_count + self.dense_count + len(self.constant)
        require(0 < self.scalar_count <= MAX_SCALARS, 'invalid total scalar count')
        self.bindings = []
        cursor = 0
        raw_bindings = tree['m_ClipBindingConstant']['genericBindings']
        require(isinstance(raw_bindings, list) and len(raw_bindings) <= MAX_SCALARS, 'oversized binding table')
        for raw_binding in raw_bindings:
            require(not raw_binding.get('isPPtrCurve'), 'PPtr curve is not a scalar animation curve')
            kind = integer(raw_binding['typeID'], 0, 65535, 'binding type')
            attr = integer(raw_binding['attribute'], 0, 0xffffffff, 'binding attribute')
            custom = integer(raw_binding['customType'], 0, 255, 'binding custom type')
            path = integer(raw_binding['path'], 0, 0xffffffff, 'binding path')
            if kind == 4:
                require(attr in (1, 2, 3, 4), 'unknown Transform property width')
                width = 4 if attr == 2 else 3
            else:
                require(kind == 95 and custom == 8 and 0 <= attr < MAX_SCALARS and path == 0,
                        'unsupported non-Transform scalar binding')
                width = 1
            self.bindings.append(Binding(kind, custom, path, attr, cursor, width))
            cursor += width
        require(cursor == self.scalar_count, 'expanded binding width does not match scalar payload')
        self.animator_bindings = {}
        self.transform_bindings = []
        for binding in self.bindings:
            if binding.type_id == 95:
                require(binding.attribute not in self.animator_bindings, 'duplicate Animator attribute')
                self.animator_bindings[binding.attribute] = binding
            else:
                self.transform_bindings.append(binding)
        self.muscle_mask = [42 + i in self.animator_bindings for i in range(95)]
        indices = muscle['m_IndexArray']
        require(isinstance(indices, list) and len(indices) <= MAX_SCALARS, 'oversized Animator index table')
        for attr, binding in self.animator_bindings.items():
            require(attr < len(indices) and indices[attr] == binding.offset, 'Animator index-array/binding mismatch')
        self.endpoint_reference = muscle['m_ValueArrayDelta']
        require(len(self.endpoint_reference) == self.scalar_count, 'endpoint reference width mismatch')

    def sample_scalars(self, seconds: float) -> list[float]:
        time = self.start + min(self.duration, max(0.0, finite(seconds, 'sample time')))
        result = []
        for keys, times in zip(self.stream_keys, self.stream_times, strict=True):
            index = bisect_right(times, time) - 1
            require(index >= 0, 'stream curve lacks a value at sample time')
            key_time, (a, b, c, d) = keys[index]
            dt = 0.0 if key_time < -1e20 else time - key_time
            result.append(((a * dt + b) * dt + c) * dt + d)
        if self.dense_count:
            frame = (time - self.dense_begin) * self.dense_rate
            low = min(self.frame_count - 1, max(0, math.floor(frame)))
            high = min(self.frame_count - 1, low + 1)
            blend = min(1.0, max(0.0, frame - low))
            result.extend(self.dense[low * self.dense_count + i] * (1 - blend)
                          + self.dense[high * self.dense_count + i] * blend
                          for i in range(self.dense_count))
        result.extend(self.constant)
        require(all(math.isfinite(v) for v in result), 'nonfinite evaluated scalar')
        return result

    def sample(self, seconds: float) -> dict:
        scalars = self.sample_scalars(seconds)
        width = max(137, max(self.animator_bindings, default=136) + 1)
        values = [0.0] * width
        mask = [False] * width
        for attr, binding in self.animator_bindings.items():
            values[attr], mask[attr] = scalars[binding.offset], True
        return {'time': min(self.duration, max(0.0, seconds)), 'animator_values': values,
                'animator_mask': mask,
                'transform_values': [x for binding in self.transform_bindings
                                     for x in scalars[binding.offset:binding.offset + binding.width]]}

    def endpoint_errors(self) -> dict:
        values = [self.sample_scalars(0.0), self.sample_scalars(self.duration)]
        return {label: max(abs(v - finite(ref[field])) for v, ref in zip(actual, self.endpoint_reference, strict=True))
                for label, field, actual in [('start', 'm_Start', values[0]), ('stop', 'm_Stop', values[1])]}

    def oracle_clip(self, fps: int = 60) -> dict:
        require(fps in (30, 60, 120), 'unsupported oracle sample rate')
        count = math.ceil(self.duration * fps)
        stop = struct.unpack('<f', struct.pack('<f', self.duration))[0]
        times = [struct.unpack('<f', struct.pack('<f', i / fps))[0] for i in range(count)
                 if struct.unpack('<f', struct.pack('<f', i / fps))[0] < stop]
        times.append(stop)
        return {'name': self.name, 'start': self.start, 'duration': self.duration,
                'looping': self.looping, 'authored_muscle_mask': self.muscle_mask,
                'generic_bindings': [dict(path_hash=b.path_hash, attribute=b.attribute, width=b.width)
                                     for b in self.transform_bindings],
                'samples': [self.sample(t) for t in times]}


def avatar_rig(tree: dict, root_name: str = 'l22_male_skeleton') -> dict:
    """Keep original DefaultPose, Avatar pose and human neutral axes distinct."""
    avatar = tree['m_Avatar']
    skeleton = unwrap(avatar['m_AvatarSkeleton'])
    raw_nodes = skeleton['m_Node']
    hashes = skeleton['m_ID']
    rest = unwrap(avatar['m_DefaultPose'])['m_X']
    require(0 < len(raw_nodes) <= MAX_JOINTS and len(hashes) == len(raw_nodes) == len(rest), 'Avatar skeleton shape mismatch')
    paths = dict(tree['m_TOS'])
    nodes, names = [], set()
    for i, (node, path_hash, pose) in enumerate(zip(raw_nodes, hashes, rest, strict=True)):
        parent = node['m_ParentId']
        require(parent == -1 if i == 0 else 0 <= parent < i, 'Avatar parent is not ancestral')
        require(path_hash in paths, 'Avatar path hash has no TOS entry')
        path = paths[path_hash]
        name = path.rsplit('/', 1)[-1] if path else root_name
        require(name and name not in names, 'ambiguous Avatar bone name')
        names.add(name)
        leaf_hash = avatar['m_SkeletonNameIDArray'][i]
        require(leaf_hash == 0 if i == 0 else zlib.crc32(name.encode()) == leaf_hash, 'Avatar leaf-name hash mismatch')
        nodes.append({'name': name, 'path': path, 'path_hash': path_hash, 'parent': parent,
                      'rest': local_trs(pose)})
    motion_root = next((i for i, n in enumerate(nodes) if n['name'] == 'MotionRoot'), None)
    require(motion_root is not None, 'original MotionRoot missing')
    human = unwrap(avatar['m_Human'])
    h_skeleton = unwrap(human['m_Skeleton'])
    h_indices = avatar['m_HumanSkeletonIndexArray']
    reverse = avatar['m_HumanSkeletonReverseIndexArray']
    require(len(h_indices) == len(h_skeleton['m_Node']), 'human/full skeleton index size mismatch')
    for i, full in enumerate(h_indices):
        require(0 <= full < len(nodes) and reverse[full] == i, 'human/full reverse index mismatch')
    return {'rig_id': RIG_ID, 'coordinate_system': 'raw Unity; metres; quaternion xyzw',
            'motion_root': motion_root, 'nodes': nodes,
            'human_indices': h_indices, 'human': human,
            'human_description': tree['m_HumanDescription'],
            'avatar_skeleton_pose': unwrap(avatar['m_AvatarSkeletonPose'])['m_X']}


def _string(value: str) -> bytes:
    require(isinstance(value, str), 'invalid packed string')
    data = value.encode('utf-8')
    require(0 < len(data) <= 1024, 'oversized/empty packed string')
    return struct.pack('<H', len(data)) + data


def reflect_z_trs(values: list[float], scale: float = 1.0, root_y: float = 0.0) -> list[float]:
    require(len(values) == 10, 'TRS width mismatch')
    t, q, s = values[:3], quaternion(values[3:7]), values[7:]
    return [t[0] * scale, t[1] * scale + root_y, -t[2] * scale, -q[0], -q[1], q[2], q[3], *s]


def pack_verified_locals(document: dict) -> bytes:
    """Pack native-verified local TRS. No muscle-to-quaternion guessing here."""
    require(document.get('format') == 'KIVA_NATIVE_POSES_1', 'unexpected native pose document')
    require(document.get('h0_verified') is True, 'native H0 validation has not passed')
    normalization = document['runtime_normalization']
    factor = finite(normalization['scale'], 'normalization scale')
    floor_y = finite(normalization['floor_offset_y'], 'normalization floor offset')
    require(0 < factor <= 100 and abs(floor_y) <= 100 and normalization['reflection_axis'] == 'Z', 'invalid normalization')
    rig = document['rig']
    nodes = rig['nodes']
    clips = document['clips']
    require(0 < len(nodes) <= MAX_JOINTS and 0 < len(clips) <= MAX_CLIPS, 'invalid packed skeleton/clip count')
    motion_root = integer(rig['motion_root'], 0, len(nodes) - 1, 'packed motion root')
    out = bytearray(MAGIC + _string(rig['rig_id']) + struct.pack('<4H', len(nodes), len(clips), motion_root, 0))
    names = set()
    for i, node in enumerate(nodes):
        require(node['name'] not in names, 'duplicate packed bone name')
        names.add(node['name'])
        parent = integer(node['parent'], -1, len(nodes) - 1, 'packed parent')
        require(parent == -1 if i == 0 else 0 <= parent < i, 'packed parent not ancestral')
        rest = [finite(x) for x in node['rest']]
        require(len(rest) == 10, 'packed rest width mismatch')
        rest = local_trs({'position': rest[:3], 'rotation': rest[3:7], 'scale': rest[7:]})
        out += _string(node['name']) + struct.pack('<h10f', parent, *reflect_z_trs(rest, factor, floor_y if i == 0 else 0.0))
    clip_names = set()
    for clip in clips:
        require(clip['name'] not in clip_names, 'duplicate packed clip name')
        clip_names.add(clip['name'])
        duration = finite(clip['duration'], 'packed clip duration')
        require(0 < duration <= MAX_SECONDS, 'invalid packed duration')
        frames = clip['frames']
        require(2 <= len(frames) <= MAX_FRAMES, 'invalid baked frame count')
        times = [finite(f['time'], 'baked time') for f in frames]
        packed_times = [struct.unpack('<f', struct.pack('<f', t))[0] for t in times]
        require(all(a < b for a, b in zip(packed_times, packed_times[1:])), 'baked times collide at f32 precision')
        require(times[0] == 0 and abs(times[-1] - duration) < 1e-6
                and all(a < b for a, b in zip(times, times[1:])), 'nonmonotonic/incomplete baked timeline')
        require(all(len(f['locals']) == len(nodes) for f in frames), 'baked frame joint count mismatch')
        values = []
        for frame in frames:
            poses = []
            for pose in frame['locals']:
                require(isinstance(pose, list) and len(pose) == 10, 'baked TRS width mismatch')
                poses.append(reflect_z_trs(local_trs({'position': pose[:3], 'rotation': pose[3:7], 'scale': pose[7:]}), factor, floor_y if len(poses) == 0 else 0.0))
            values.append(poses)
        out += _string(clip['name']) + struct.pack('<fH', duration, len(nodes))
        for joint in range(len(nodes)):
            out += struct.pack('<H', joint)
            for start, width in [(0, 3), (3, 4), (7, 3)]:
                channels = [pose[joint][start:start + width] for pose in values]
                if start == 3:
                    for i in range(1, len(channels)):
                        if sum(a * b for a, b in zip(channels[i - 1], channels[i])) < 0:
                            channels[i] = [-x for x in channels[i]]
                indices = [0] if all(channel == channels[0] for channel in channels) else range(len(times))
                out += struct.pack('<I', len(indices))
                for i in indices:
                    out += struct.pack('<f' + 'f' * width, times[i], *channels[i])
    require(len(out) <= 64 * 1024 * 1024, 'oversized packed animation asset')
    return bytes(out)


class BinaryReader:
    def __init__(self, data: bytes):
        require(isinstance(data, bytes) and len(data) <= 256 * 1024**2, 'oversized binary input')
        self.data, self.at = data, 0

    def take(self, size: int) -> bytes:
        require(isinstance(size, int) and 0 <= size <= len(self.data) - self.at, 'truncated binary extent')
        value = self.data[self.at:self.at + size]
        self.at += size
        return value

    def number(self, fmt: str):
        return struct.unpack('<' + fmt, self.take(struct.calcsize('<' + fmt)))[0]

    def string(self, fmt: str = 'H') -> str:
        length = self.number(fmt)
        require(0 < length <= 1024, 'invalid binary string length')
        try:
            return self.take(length).decode('utf-8')
        except UnicodeDecodeError as exc:
            raise ValueError('invalid binary UTF-8') from exc

    def floats(self, count: int) -> list[float]:
        require(0 <= count <= MAX_VALUES * 32, 'oversized float array')
        values = list(struct.unpack('<' + 'f' * count, self.take(count * 4)))
        require(all(math.isfinite(x) for x in values), 'nonfinite binary float')
        return values

    def finish(self):
        require(self.at == len(self.data), 'trailing binary bytes')


def read_native_capture(data: bytes) -> dict:
    """Read independent native TRS plus world matrices, preserving raw floats."""
    reader = BinaryReader(data)
    require(reader.take(8) == b'KIVANAT2', 'unsupported native capture version')
    unity_version, rig_id = reader.string('i'), reader.string('i')
    count = integer(reader.number('i'), 1, MAX_JOINTS, 'native joint count')
    motion_root = integer(reader.number('i'), 0, count - 1, 'native motion root')
    nodes, names = [], set()
    for i in range(count):
        name, parent, rest = reader.string('i'), reader.number('i'), reader.floats(10)
        require(name not in names and (parent == -1 if i == 0 else 0 <= parent < i), 'invalid native hierarchy')
        names.add(name)
        local_trs({'position': rest[:3], 'rotation': rest[3:7], 'scale': rest[7:]})
        nodes.append({'name': name, 'parent': parent, 'rest': rest})
    clip_count = integer(reader.number('i'), 1, MAX_CLIPS, 'native clip count')
    clips, names = [], set()
    total_frames = 0
    for _ in range(clip_count):
        name, digest = reader.string('i'), reader.string('i')
        require(name not in names and len(digest) == 64 and set(digest) <= set('0123456789abcdef'), 'invalid native clip identity')
        names.add(name)
        duration = finite(reader.number('f'), 'native duration')
        require(0 < duration <= MAX_SECONDS, 'invalid native duration')
        frames = integer(reader.number('i'), 2, MAX_FRAMES, 'native frame count')
        total_frames += frames
        require(total_frames * count * 26 <= MAX_VALUES * 32, 'oversized native frame data')
        values = []
        previous = -1.0
        for _ in range(frames):
            time = finite(reader.number('f'), 'native frame time')
            require(previous < time <= duration and time >= 0, 'unordered native frame time')
            previous = time
            raw = reader.floats(count * 26)
            locals_ = [raw[i * 26:i * 26 + 10] for i in range(count)]
            for pose in locals_:
                local_trs({'position': pose[:3], 'rotation': pose[3:7], 'scale': pose[7:]})
            values.append({'time': time, 'locals': locals_,
                           'native_world_column_major': [raw[i * 26 + 10:(i + 1) * 26] for i in range(count)]})
        require(values[0]['time'] == 0 and values[-1]['time'] == duration, 'native timeline endpoints missing')
        clips.append({'name': name, 'source_sha256': digest, 'duration': duration, 'frames': values})
    reader.finish()
    return {'format': 'KIVA_NATIVE_POSES_1', 'evaluator': {'unity_version': unity_version},
            'rig': {'rig_id': rig_id, 'motion_root': motion_root, 'nodes': nodes}, 'clips': clips}


def unpack_runtime(data: bytes) -> dict:
    reader = BinaryReader(data)
    require(reader.take(8) == MAGIC, 'unsupported runtime animation version')
    rig_id = reader.string()
    count = integer(reader.number('H'), 1, MAX_JOINTS, 'runtime joint count')
    clip_count = integer(reader.number('H'), 1, MAX_CLIPS, 'runtime clip count')
    root = integer(reader.number('H'), 0, count - 1, 'runtime motion root')
    require(reader.number('H') == 0, 'unknown runtime flags')
    nodes, names = [], set()
    for i in range(count):
        name, parent, rest = reader.string(), reader.number('h'), reader.floats(10)
        require(name not in names and (parent == -1 if i == 0 else 0 <= parent < i), 'invalid runtime hierarchy')
        names.add(name)
        local_trs({'position': rest[:3], 'rotation': rest[3:7], 'scale': rest[7:]})
        nodes.append({'name': name, 'parent': parent, 'rest': rest})
    clips, names = [], set()
    for _ in range(clip_count):
        name, duration = reader.string(), finite(reader.number('f'), 'runtime duration')
        require(name not in names and 0 < duration <= MAX_SECONDS, 'invalid runtime clip')
        names.add(name)
        track_count = integer(reader.number('H'), 1, count, 'runtime track count')
        tracks = {}
        for _ in range(track_count):
            index = integer(reader.number('H'), 0, count - 1, 'runtime track joint')
            require(index not in tracks, 'duplicate runtime track')
            channels = []
            for width in (3, 4, 3):
                key_count = integer(reader.number('I'), 0, MAX_FRAMES, 'runtime key count')
                require(key_count * (width + 1) * 4 <= len(reader.data) - reader.at, 'runtime key range exceeds bytes')
                keys, previous = [], -1.0
                for _ in range(key_count):
                    time = finite(reader.number('f'), 'runtime key time')
                    require(0 <= time <= duration and time > previous, 'unordered runtime keys')
                    previous = time
                    values = reader.floats(width)
                    if width == 4:
                        quaternion(values)
                    keys.append((time, values))
                channels.append(keys)
            tracks[index] = channels
        clips.append({'name': name, 'duration': duration, 'tracks': tracks})
    reader.finish()
    return {'rig_id': rig_id, 'motion_root': root, 'nodes': nodes, 'clips': clips}
