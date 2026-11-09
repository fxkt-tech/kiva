#!/usr/bin/env python3
"""Focused bounded-format/quantization checks, no game installation access."""
import math
import random
import struct
import unittest
from messiah_anim_v15 import AnimationError, parse_animation, smallest_three


def encode_quaternion(value, bits=20):
    slot = max(range(4), key=lambda i: abs(value[i]))
    if value[slot] < 0:
        value = [-x for x in value]
    order = ((1, 2, 3), (2, 3, 0), (3, 0, 1), (0, 1, 2))[slot]
    half = math.sqrt(.5)
    mask = (1 << bits) - 1
    packed = slot << (bits * 3)
    for index, shift in zip(order, (2, 1, 0)):
        packed |= round((value[index] + half) * mask * half) << (shift * bits)
    return packed.to_bytes((bits * 3 + 9) // 8, 'little')


def sample_file():
    root = b'\x06Bip001' + struct.pack('<h', -1)
    root += struct.pack('<HBBH3fH3f', 2, 0, 0, 0, 0, 1, 0, 1000, 2, .5, 3)
    root += struct.pack('<HBH', 1, 82, 0) + encode_quaternion([0, 0, 0, 1])
    child = b'\x04Hand' + struct.pack('<h', 0) + b'\0' * 6
    channels = root + child
    header = b'CHAR::ANIM' + struct.pack('<H I I BB H H H H H', 15, 0, 1000, 2, 54, 0, 200, 2, 2, 20)
    result = bytearray(b'\x65\x4e\xa1\x42' + header + channels)
    start = len(result)
    for name, size in [('HEADER', 32), ('CHANNEL_DATA', len(channels)), ('CURVE_DATA', 0), ('MOTION_DATA', 0)]:
        result += struct.pack('<QQII', size, 0, 0, len(name)) + name.encode('ascii')
        result += b'\0' * ((-len(result)) % 4)
    result += struct.pack('<I', len(result) - start)
    return bytes(result)


class FormatTests(unittest.TestCase):
    def test_all_largest_component_slots_quantization_error(self):
        rng = random.Random(1501)
        slots = set()
        for _ in range(1000):
            q = [rng.uniform(-1, 1) for _ in range(4)]
            norm = math.sqrt(sum(x * x for x in q))
            q = [x / norm for x in q]
            slots.add(max(range(4), key=lambda i: abs(q[i])))
            actual = smallest_three(encode_quaternion(q), 20)
            dot = abs(sum(a * b for a, b in zip(q, actual)))
            angle = 2 * math.acos(min(1.0, dot))
            self.assertLess(angle, .000006)
        self.assertEqual(slots, {0, 1, 2, 3})

    def test_true_sparse_channels_and_empty_tracks(self):
        animation = parse_animation(sample_file())
        self.assertEqual(animation.duration_ms, 1000)
        self.assertEqual(animation.bones[0].translation.times_ms, [0, 1000])
        self.assertEqual(animation.bones[0].translation.values[-1], [2, .5, 3])
        self.assertEqual(animation.bones[1].parent, 0)
        self.assertEqual(animation.bones[1].rotation.values, [])

    def test_all_truncations_and_invalid_quaternion_bits_rejected(self):
        data = sample_file()
        for length in range(len(data)):
            with self.assertRaises(AnimationError):
                parse_animation(data[:length])
        invalid = bytearray(encode_quaternion([0, 0, 0, 1]))
        invalid[-1] |= 128
        with self.assertRaises(AnimationError):
            smallest_three(invalid, 20)

    def test_oversized_key_time_rejected(self):
        data = bytearray(sample_file())
        # duration field precedes channels; setting it shorter makes a real key
        # invalid while leaving every extent and byte count unchanged.
        struct.pack_into('<I', data, 20, 999)
        with self.assertRaisesRegex(AnimationError, 'key exceeds'):
            parse_animation(data)


if __name__ == '__main__':
    unittest.main()
