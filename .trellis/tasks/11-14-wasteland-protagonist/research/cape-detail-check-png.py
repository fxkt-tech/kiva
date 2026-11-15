"""Decode actual PNG values and identify Blender FILE CPU pixel representation."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
from PIL import Image

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--samples', type=Path, required=True)
p.add_argument('--textures', type=Path, required=True)
p.add_argument('--report', type=Path, required=True)
a = p.parse_args()
reports = json.loads(a.samples.read_text())
for report in reports:
    for key, image in report['images'].items():
        raw = np.asarray(Image.open(a.textures/image['file']).convert('RGBA'), dtype=np.float64)/255
        assert raw.shape == (2048, 2048, 4)
        expected = raw.copy()
        if key in ('base', 'emission'):
            rgb = expected[:, :, :3]
            expected[:, :, :3] = np.where(rgb <= .04045, rgb/12.92, ((rgb+.055)/1.055)**2.4)
        maximum_encoded = maximum_linear = 0.0
        samples = image.pop('samples_bottom_up')
        for sample in samples:
            x, y, *actual = sample
            maximum_encoded = max(maximum_encoded, float(np.max(np.abs(raw[2047-y, x]-actual))))
            maximum_linear = max(maximum_linear, float(np.max(np.abs(expected[2047-y, x]-actual))))
        # Byte-backed FILE Image.pixels may expose encoded byte floats even
        # though the Texture node applies its declared sRGB transform on GPU.
        # Never mistake the CPU accessor representation for shader radiance.
        assert min(maximum_encoded, maximum_linear) < 2e-6, (report['source'], key, maximum_encoded, maximum_linear)
        image['CPU_accessor_representation'] = 'encoded PNG byte floats' if maximum_encoded <= maximum_linear else 'linear color floats'
        image['maximum_CPU_pixels_vs_matching_PNG_representation_error'] = min(maximum_encoded, maximum_linear)
        image['expected_linear_Texture_node_rgb_range'] = [float(expected[:, :, :3].min()), float(expected[:, :, :3].max())]
        for field in ('minimum_shader_rgb', 'maximum_shader_rgb', 'mean_shader_rgb'):
            if field in image:
                image[field.replace('shader', 'CPU_pixel')] = image.pop(field)
        if key == 'orm':
            assert np.all(raw[:, :, 0] == 1) and np.all(raw[:, :, 2] == 0)
            image['AO_red_explicit_unoccluded'] = True
            image['metal_blue_zero'] = True
        image['png_encoded_rgb_range'] = [float(raw[:, :, :3].min()), float(raw[:, :, :3].max())]
a.report.write_text(json.dumps({'FILE_source_PNG_bytes_and_declared_color_space_contract_match': True, 'artifacts': reports}, indent=2)+'\n')
print(json.dumps({'report': str(a.report), 'artifacts': [r['source'] for r in reports],
                  'maximum_error': max(i['maximum_CPU_pixels_vs_matching_PNG_representation_error'] for r in reports for i in r['images'].values())}, indent=2))
