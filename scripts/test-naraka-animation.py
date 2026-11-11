#!/usr/bin/env python3
"""Real-source curve/mask checks and malformed prepared/packed data regressions."""
import importlib.util
import gzip
import json
import math
from pathlib import Path
import shutil
import struct
import tempfile
import unittest

from naraka_animation import (ScalarClip, pack_verified_locals,
                              read_native_capture, reflect_z_trs, unpack_runtime)
from naraka_animation_validation import index_clip, sample_runtime, verify_cuts, verify_sampling

ROOT = Path(__file__).resolve().parents[1]
PREPARED = ROOT / 'assets/prepared/naraka/animations'


def tree(name):
    return json.loads((PREPARED / 'source/clips' / (name + '.json')).read_text())['tree']


class ScalarSourceTests(unittest.TestCase):
    def test_full_clip_matches_independent_serialized_endpoint_values(self):
        clip = ScalarClip(tree('male_hengdao_attack_light_01'))
        self.assertEqual(clip.scalar_count, 179)
        self.assertEqual(sum(clip.muscle_mask), 95)
        self.assertEqual(clip.endpoint_errors()['start'], 0)
        self.assertLess(clip.endpoint_errors()['stop'], 1e-9)

    def test_missing_hand_attributes_do_not_become_zero_authored_curves(self):
        idle = ScalarClip(tree('male_hengdao_idle_01'))
        falling = ScalarClip(tree('male_hengdao_jump_down_loop_01'))
        self.assertEqual((sum(idle.muscle_mask[55:75]), sum(idle.muscle_mask[75:95])), (0, 20))
        self.assertEqual(sum(falling.muscle_mask[55:]), 0)
        self.assertFalse(falling.sample(0)['animator_mask'][97])
        attack = ScalarClip(tree('male_hengdao_attack_light_01'))
        first = attack.sample(0)
        actual_zero = next(i for i, (v, authored) in enumerate(zip(first['animator_values'], first['animator_mask']))
                           if authored and v == 0)
        self.assertTrue(first['animator_mask'][actual_zero])

    def test_original_finger_limits_are_not_clamped(self):
        clip = ScalarClip(tree('male_bow_finger_default_left_01'))
        values = clip.sample(0)
        fingers = [values['animator_values'][97 + i] for i in range(20)]
        self.assertLess(min(fingers), -2)
        self.assertEqual(sum(clip.muscle_mask[55:75]), 20)
        self.assertEqual(sum(clip.muscle_mask[75:95]), 0)

    def test_each_original_clip_has_bounded_consistent_curves(self):
        selection = json.loads((PREPARED / 'selection.json').read_text())
        for item in selection['clips']:
            with self.subTest(item=item['name']):
                clip = ScalarClip(tree(item['name']))
                self.assertEqual(sum(clip.muscle_mask), item['muscle_binding_count'])
                self.assertLess(max(clip.endpoint_errors().values()), 0.002)
                for at in (0, clip.duration / 2, clip.duration):
                    self.assertEqual(len(clip.sample_scalars(at)), clip.scalar_count)

    def test_dense_count_corruption_is_rejected(self):
        data = tree('male_hengdao_attack_light_01')
        data['m_MuscleClip']['m_Clip']['data']['m_DenseClip']['m_SampleArray'].pop()
        with self.assertRaisesRegex(ValueError, 'dense sample count'):
            ScalarClip(data)

    def test_stream_truncation_and_nonfinite_coefficients_are_rejected(self):
        data = tree('male_hengdao_attack_light_01')
        data['m_MuscleClip']['m_Clip']['data']['m_StreamedClip']['data'].pop()
        with self.assertRaises(ValueError):
            ScalarClip(data)
        data = tree('male_hengdao_attack_light_01')
        data['m_MuscleClip']['m_Clip']['data']['m_StreamedClip']['data'][3] = 0x7fc00000
        with self.assertRaisesRegex(ValueError, 'nonfinite streamed coefficient'):
            ScalarClip(data)

    def test_unknown_binding_width_and_pointer_curve_are_rejected(self):
        data = tree('male_hengdao_attack_light_01')
        data['m_ClipBindingConstant']['genericBindings'][0]['attribute'] = 99
        with self.assertRaisesRegex(ValueError, 'unknown Transform'):
            ScalarClip(data)
        data = tree('male_hengdao_attack_light_01')
        data['m_ClipBindingConstant']['genericBindings'][0]['isPPtrCurve'] = True
        with self.assertRaisesRegex(ValueError, 'PPtr'):
            ScalarClip(data)

    def test_sample_time_nonfinite_and_outside_range(self):
        clip = ScalarClip(tree('male_hengdao_attack_light_01'))
        with self.assertRaises(ValueError):
            clip.sample_scalars(float('nan'))
        self.assertEqual(clip.sample_scalars(-5), clip.sample_scalars(0))
        self.assertEqual(clip.sample_scalars(99), clip.sample_scalars(clip.duration))

    def test_oracle_grid_has_no_duplicate_f32_endpoint(self):
        for name in ['male_hengdao_finger_left_01', 'male_hengdao_attack_light_01', 'male_hengdao_idle_01']:
            clip = ScalarClip(tree(name))
            times = [x['time'] for x in clip.oracle_clip()['samples']]
            encoded = [struct.pack('<f', x) for x in times]
            self.assertEqual(len(set(encoded)), len(encoded))
            self.assertEqual(times[0], 0)
            self.assertAlmostEqual(times[-1], clip.duration)

    def test_original_fk_fire_is_a_partial_body_layer(self):
        clip = ScalarClip(tree('male_fk_attack_01'))
        self.assertEqual(sum(clip.muscle_mask), 39)
        self.assertEqual(sum(clip.muscle_mask[55:]), 0)
        self.assertFalse(all(clip.muscle_mask[:55]))


class PackedDataTests(unittest.TestCase):
    # Synthetic here tests binary validation only; it is never H0/source proof.
    def document(self):
        rest = [0., 1., 0., 0., 0., 0., 1., 1., 1., 1.]
        return {'format': 'KIVA_NATIVE_POSES_1', 'h0_verified': True,
                'runtime_normalization': {'scale': 1., 'floor_offset_y': 0., 'reflection_axis': 'Z'},
                'rig': {'rig_id': 'test/validation-only', 'motion_root': 0,
                        'nodes': [{'name': 'Root', 'parent': -1, 'rest': rest}]},
                'clips': [{'name': 'test', 'duration': 0.5,
                           'frames': [{'time': 0., 'locals': [rest]}, {'time': .5, 'locals': [rest]}]}]}

    def test_no_binary_without_explicit_native_verification_gate(self):
        doc = self.document()
        doc['h0_verified'] = False
        with self.assertRaisesRegex(ValueError, 'H0'):
            pack_verified_locals(doc)

    def test_reflection_scale_and_floor_are_applied_once(self):
        p = reflect_z_trs([1., 2., 3., 0., 0., 0., 1., 1., 1., 1.], .5, .2)
        self.assertEqual(p[:3], [.5, 1.2, -1.5])
        self.assertEqual(p[3:], [-0., -0., 0., 1., 1., 1., 1.])
        self.assertTrue(pack_verified_locals(self.document()).startswith(b'KIVASR02'))

    def test_parent_cycle_and_invalid_quaternion_are_rejected(self):
        doc = self.document()
        doc['rig']['nodes'][0]['parent'] = 0
        with self.assertRaisesRegex(ValueError, 'ancestral'):
            pack_verified_locals(doc)
        doc = self.document()
        doc['clips'][0]['frames'][1]['locals'][0] = [0.] * 10
        with self.assertRaisesRegex(ValueError, 'nonunit'):
            pack_verified_locals(doc)

    def test_duplicate_baked_times_at_storage_precision_are_rejected(self):
        doc = self.document()
        doc['clips'][0]['duration'] = .3333333432674408
        rest = doc['rig']['nodes'][0]['rest']
        doc['clips'][0]['frames'] = [{'time': 0., 'locals': [rest]},
                                    {'time': 1 / 3, 'locals': [rest]},
                                    {'time': .3333333432674408, 'locals': [rest]}]
        with self.assertRaisesRegex(ValueError, 'f32 precision'):
            pack_verified_locals(doc)

    def test_runtime_decoder_rejects_bad_flags_and_truncation(self):
        binary = pack_verified_locals(self.document())
        for length in (0, 8, len(binary)-1):
            with self.assertRaises(ValueError):
                unpack_runtime(binary[:length])
        bad = bytearray(binary)
        flags = 10 + len(self.document()['rig']['rig_id'].encode()) + 6
        struct.pack_into('<H', bad, flags, 1)
        with self.assertRaisesRegex(ValueError, 'flags'):
            unpack_runtime(bytes(bad))

    def test_adjacent_float_cut_retains_both_native_values(self):
        doc = self.document()
        a = .25
        b = struct.unpack('<f', struct.pack('<I', struct.unpack('<I', struct.pack('<f', a))[0]+1))[0]
        rest = doc['rig']['nodes'][0]['rest'][:]
        rotated = rest[:]
        rotated[3:7] = [1., 0., 0., 0.]
        doc['clips'][0]['frames'] = [{'time': 0., 'locals': [rest]}, {'time': a, 'locals': [rest]},
                                     {'time': b, 'locals': [rotated]}, {'time': .5, 'locals': [rotated]}]
        packed = unpack_runtime(pack_verified_locals(doc))
        index = index_clip(packed['clips'][0])
        self.assertEqual(sample_runtime(index, packed['nodes'], a)[0][3:7], [0., 0., 0., 1.])
        self.assertEqual(sample_runtime(index, packed['nodes'], b)[0][3:7], [-1., 0., 0., 0.])

    def test_real_native_cut_report_survives_runtime_pack(self):
        report = json.loads((PREPARED / 'baked/native-report.json').read_text())
        selection = json.loads((PREPARED / 'selection.json').read_text())
        count = verify_cuts({'runtime_normalization': selection['runtime_normalization']}, report,
                            (ROOT / 'assets/naraka/animations.bin').read_bytes())
        self.assertGreater(count, 0)
        self.assertTrue(any(c['precision_cuts'] for c in report['clips'] if c['name'] == 'male_hengdao_attack_light_02'))

    def test_native_capture_reader_rejects_invalid_lengths(self):
        with self.assertRaisesRegex(ValueError, 'string length'):
            read_native_capture(b'KIVANAT2' + struct.pack('<I', 0x7fffffff))

    def test_actual_uniform60_pack_fails_independent_native_sampling_gate(self):
        # Real original native poses with only the old uniform grid retained.
        # Their keys are valid; the intermediate high-speed motion is not.
        document = json.loads(gzip.decompress((PREPARED / 'baked/locals.json.gz').read_bytes()))
        for clip in document['clips']:
            times = {struct.unpack('<f', struct.pack('<f', k/60))[0]
                     for k in range(math.ceil(clip['duration']*60))}
            times.add(clip['duration'])
            clip['frames'] = [frame for frame in clip['frames'] if frame['time'] in times]
        coarse = pack_verified_locals(document)
        reference = gzip.decompress((PREPARED / 'baked/reference-capture.bin.gz').read_bytes())
        proof = json.loads((PREPARED / 'baked/interpolation-report.json').read_text())
        mesh = json.loads((PREPARED / 'source/real-hand-lbs.json').read_text())
        with self.assertRaisesRegex(ValueError, 'native angular threshold'):
            verify_sampling(coarse, document, reference, proof, mesh)

    def test_prepared_hash_corruption_fails_portable_verify(self):
        spec = importlib.util.spec_from_file_location('prepare', ROOT / 'scripts/prepare-naraka-animations.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'prepared'
            shutil.copytree(PREPARED, root)
            selected = json.loads((root / 'selection.json').read_text())
            victim = root / 'source/clips' / (selected['clips'][0]['name'] + '.json')
            victim.write_bytes(victim.read_bytes() + b' ')
            with self.assertRaisesRegex(ValueError, 'size mismatch'):
                module.verify(root)


if __name__ == '__main__':
    unittest.main()
