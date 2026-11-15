"""Original surface work for the v11 model-refinement milestone.

No texture projection or image cutouts: clumps, folds, emblem and shoe panels
are actual editable surfaces bound by the calling authoring tool.
"""
import math

from mathutils import Vector

PI = math.pi


def smooth_path(points, samples=10):
    controls = [Vector(p) for p in points]
    result = []
    for i in range(samples):
        s = i / (samples - 1) * (len(controls) - 1)
        index = min(int(s), len(controls) - 2)
        t = s - index
        a, b = controls[max(0, index - 1)], controls[index]
        c, d = controls[index + 1], controls[min(index + 2, len(controls) - 1)]
        result.append(0.5 * (2 * b + (-a + c) * t + (2 * a - 5 * b + 4 * c - d) * t * t + (-a + 3 * b - 3 * c + d) * t ** 3))
    return result


def hair_clump(api, name, controls, width, depth, rows=9):
    path = smooth_path(controls, rows)
    verts, faces = [], []
    for j, center in enumerate(path):
        t = j / (rows - 1)
        tangent = (path[min(rows - 1, j + 1)] - path[max(0, j - 1)]).normalized()
        normal = (center - Vector((0, 1.60, 0.005))).normalized()
        across = tangent.cross(normal).normalized()
        normal = across.cross(tangent).normalized()
        if normal.dot(center - Vector((0, 1.60, 0.005))) < 0:
            normal = -normal
        taper = max(0.025, math.sin(PI * (0.10 + t * 0.87)) ** 0.65 * (1 - t) ** 0.45)
        if j == rows - 1:
            taper = 0.02
        for k in range(6):
            a = k / 6 * 2 * PI
            raised = depth * math.sin(a) * (1 if math.sin(a) > 0 else 0.28)
            point = center + across * math.cos(a) * width * taper + normal * raised * taper
            point.y = min(point.y, 1.73)
            verts.append(point)
    for j in range(rows - 1):
        for k in range(6):
            a, b = j * 6 + k, j * 6 + (k + 1) % 6
            faces.append((a, b, b + 6, a + 6))
    faces += [tuple(reversed(range(6))), tuple((rows - 1) * 6 + i for i in range(6))]
    api.mesh(name, verts, faces, [{'gMan Head': 1}] * len(verts), 2)


def hair(api):
    verts, faces = [], []
    segments, rows = 32, 8
    for row in range(rows):
        t = row / (rows - 1)
        for i in range(segments):
            a = i / segments * 2 * PI
            front = max(0, -math.sin(a))
            start = 1.575 + 0.074 * front ** 2 + 0.004 * math.sin(a * 7)
            y = start * (1 - t) + 1.714 * t
            radial = math.sqrt(max(0.005, 1 - ((y - 1.602) / 0.117) ** 2))
            verts.append((0.086 * radial * math.cos(a), y, 0.004 + 0.085 * radial * math.sin(a)))
    for row in range(rows - 1):
        for i in range(segments):
            a, b = row * segments + i, row * segments + (i + 1) % segments
            faces.append((a, b, b + segments, a + segments))
    api.mesh('Refined hair underlayer with irregular hairline', verts, faces, [{'gMan Head': 1}] * len(verts), 2)
    # Broad S-shaped bangs have different sweep, taper and ends, rather than
    # repeating round strings or a line of evenly clipped fringe tips.
    for side in (-1, 1):
        for i in range(6):
            t = i / 5
            asym = 0.004 * math.sin(i * 1.8 + side)
            tip = (side * (0.026 + 0.060 * t), 1.633 - 0.064 * t + asym, -0.094 + 0.044 * t)
            hair_clump(api, f'Refined hair {side} swept front mass {i}',
                       [(side * (0.002 + 0.016 * t), 1.702 + 0.006 * math.sin(i), -0.010 + 0.035 * t),
                        (side * (0.010 + 0.028 * t), 1.713 - 0.013 * t, -0.056 + 0.028 * t),
                        (side * (0.018 + 0.063 * t), 1.677 - 0.025 * t, -0.091 + 0.032 * t), tip],
                       0.023 - 0.006 * t, 0.008 - 0.002 * t, 10)
        for i in range(6):
            t = i / 5
            hair_clump(api, f'Refined hair {side} temple and side layer {i}',
                       [(side * 0.024, 1.704 - 0.020 * t, 0.003 + 0.037 * t),
                        (side * (0.068 + 0.012 * t), 1.691 - 0.028 * t, -0.018 + 0.069 * t),
                        (side * (0.095 + 0.008 * math.sin(i)), 1.634 - 0.022 * t, -0.010 + 0.077 * t),
                        (side * (0.083 + 0.020 * math.sin(i * 1.1)), 1.562 + 0.020 * math.sin(i * 1.8), 0.005 + 0.079 * t)],
                       0.018 + 0.003 * math.sin(i), 0.007, 9)
        for i in range(5):
            t = i / 4
            hair_clump(api, f'Refined hair {side} broken nape fan {i}',
                       [(side * (0.006 + 0.033 * t), 1.695 - 0.022 * t, 0.043),
                        (side * (0.034 + 0.045 * t), 1.661 - 0.025 * t, 0.089),
                        (side * (0.020 + 0.060 * t), 1.584, 0.098),
                        (side * (0.016 + 0.078 * t), 1.541 + 0.022 * math.sin(i * 1.5), 0.082 + 0.008 * math.cos(i))],
                       0.018, 0.007, 8)
        for i in range(3):
            hair_clump(api, f'Refined hair {side} crown lift {i}',
                       [(side * 0.008, 1.698, -0.003 + i * 0.021),
                        (side * (0.025 + i * 0.009), 1.726, -0.012 + i * 0.026),
                        (side * (0.066 + i * 0.008), 1.708 - i * 0.013, -0.005 + i * 0.035)],
                       0.015, 0.006, 7)
        # A few overlapping cross-swept locks interrupt the parallel fall of
        # the underlying masses. These are closed volumetric clumps, not rods.
        for i in range(3):
            hair_clump(api, f'Refined hair {side} cross-swept shingled layer {i}',
                       [(side * (0.006 + 0.010 * i), 1.705 - i * 0.004, -0.012 + i * 0.020),
                        (side * (0.038 + 0.012 * i), 1.723 - i * 0.009, -0.048 + i * 0.024),
                        (side * (0.077 + 0.010 * i), 1.686 - i * 0.020, -0.074 + i * 0.034),
                        (side * (0.065 + 0.016 * i), 1.621 + 0.011 * math.sin(i * 1.5), -0.087 + i * 0.040)],
                       0.017, 0.006, 8)
        for i in (0, 2):
            api.strand(f'Refined hair {side} subdued ember tip {i}',
                       [(side * (0.087 + i * 0.004), 1.565 + i * 0.007, 0.031 + i * 0.018),
                        (side * (0.099 + i * 0.003), 1.557 + i * 0.007, 0.034 + i * 0.018)],
                       0.0017, 'gMan Head', 3, 5)


TOP = [(0.995, 0.137, 0.095), (1.015, 0.139, 0.096), (1.07, 0.135, 0.093),
       (1.14, 0.15, 0.101), (1.22, 0.17, 0.104), (1.30, 0.177, 0.096),
       (1.36, 0.153, 0.082), (1.408, 0.066, 0.056)]


def shirt_profile(y):
    if y <= TOP[0][0]:
        return TOP[0][1:]
    for i in range(len(TOP) - 1):
        ya, xa, za = TOP[i]
        yb, xb, zb = TOP[i + 1]
        if y <= yb:
            t = (y - ya) / (yb - ya)
            return xa * (1 - t) + xb * t, za * (1 - t) + zb * t
    return TOP[-1][1:]


def shirt_displacement(x, y, a):
    front = max(0, -math.sin(a))
    hem = math.exp(-((y - 1.015) / 0.045) ** 2)
    low = 0.0030 * math.sin(6 * a + 0.8) + 0.0017 * math.sin(11 * a - 0.4)
    fold = low * (0.4 + 0.6 * hem)
    for height, slope, amplitude in ((1.056, 0.43, 0.004), (1.087, -0.30, -0.0035), (1.17, -0.38, 0.0025)):
        fold += front * amplitude * math.exp(-((y - height - slope * x) / 0.014) ** 2)
    return fold


def shirt_front(x, y):
    rx, rz = shirt_profile(y)
    theta = -math.acos(max(-0.99, min(0.99, x / rx)))
    base = 0.003 - rz * math.sqrt(max(0.02, 1 - (x / rx) ** 2))
    return base - shirt_displacement(x, y, theta)


def outer_clothes(api):
    api.remove('Waist length outer top')
    for side in ('L', 'R'):
        for tier in (1, 2):
            api.remove(f'{side} outer short sleeve tier {tier}')
    verts, faces, weights = [], [], []
    rows, segments = 22, 44
    for row in range(rows):
        y = 0.995 + row / (rows - 1) * 0.413
        rx, rz = shirt_profile(y)
        for i in range(segments):
            a = i / segments * 2 * PI
            x = rx * math.cos(a)
            d = shirt_displacement(x, y, a)
            yy = y + 0.0015 * math.sin(5 * a + 0.3) * math.exp(-((y - 0.995) / 0.018) ** 2)
            verts.append((x + math.cos(a) * d, yy, 0.003 + math.sin(a) * (rz + d)))
            weights.append(api.torso_weight(y))
    for row in range(rows - 1):
        for i in range(segments):
            a, b = row * segments + i, row * segments + (i + 1) % segments
            faces.append((a, b, b + segments, a + segments))
    api.mesh('Refined cropped outer top with tension folds', verts, faces, weights, 1, 0.002)
    for side in ('L', 'R'):
        shoulder, elbow = api.p(f'gMan {side} UpperArm'), api.p(f'gMan {side} Forearm')
        upper = f'gMan {side} UpperArm'
        for tier, end, radius in ((1, 0.55, 0.069), (2, 0.80, 0.060)):
            ts = [-0.10, 0, 0.10, 0.22, 0.36, end - 0.022, end - 0.007, end]
            if tier == 2:
                ts = [0.02, 0.10, 0.22, 0.36, 0.51, end - 0.022, end - 0.007, end]
            points, widths, depths, skin = [], [], [], []
            for j, t in enumerate(ts):
                points.append(shoulder.lerp(elbow, t))
                shape = [0.32, 0.72, 0.95, 1.03, 1.02, 0.99, 1.01, 1.01][j]
                if tier == 2:
                    shape = [0.78, 0.92, 1.00, 1.03, 1.01, 1.0, 1.02, 1.02][j]
                widths.append(radius * shape)
                depths.append(radius * shape * 0.90)
                skin.append(api.blend_weights({f'gMan {side} Clavicle': 1}, {upper: 1}, max(0.3, min(1, (t + 0.08) / 0.3))))
            api.tube(f'Refined {side} soft outer sleeve tier {tier}', points, widths, depths, skin,
                     24, 1, 'start', 0.002, 0.027)


def scarf(api):
    verts, faces, skin = [], [], []
    segments, rows = 48, 10
    for row in range(rows):
        t = row / (rows - 1)
        for i in range(segments):
            a = i / segments * 2 * PI
            # A broad continuous wrap folds over itself: height/radius vary
            # around the neck, and the front hangs lower than the back.
            gather = 0.009 * math.cos(t * 5.5 * PI + a * 0.7) * math.sin(PI * t)
            rx = 0.083 + 0.021 * math.sin(PI * t) + gather
            rz = 0.067 + 0.023 * math.sin(PI * t) + gather
            y = 1.385 + 0.075 * t + 0.018 * math.sin(a + 0.5) + 0.006 * math.sin(2 * a - 0.6)
            y += 0.004 * math.sin(a * 5 + t * 2)
            verts.append((rx * math.cos(a), y, rz * math.sin(a)))
            skin.append({'gMan Spine2': 0.75, 'gMan Neck': 0.25})
    for row in range(rows - 1):
        for i in range(segments):
            a, b = row * segments + i, row * segments + (i + 1) % segments
            faces.append((a, b, b + segments, a + segments))
    api.mesh('Refined continuous scarf draped and gathered wrap', verts, faces, skin, 2, 0.0025)
    for fold in range(2):
        points, widths, depths = [], [], []
        count = 45 if fold == 0 else 29
        for i in range(count):
            a = i / (count - 1) * (2 * PI if fold == 0 else PI) + (0 if fold == 0 else PI)
            rx = 0.095 + 0.007 * math.sin(a * 2 + fold)
            rz = 0.079 + 0.009 * math.sin(a + 0.4) + 0.005 * fold
            yy = 1.388 + 0.026 * fold + 0.011 * math.sin(a + 0.6) + 0.004 * math.sin(a * 3)
            points.append((rx * math.cos(a), yy, rz * math.sin(a)))
            widths.append(0.0065 + 0.002 * math.sin(a + fold))
            depths.append(0.011 + 0.002 * math.sin(a * 2))
        api.tube(f'Refined scarf soft irregular rolled fold {fold}', points, widths, depths,
                 [{'gMan Spine2': 0.85, 'gMan Neck': 0.15}] * count, 8, 2)


def relief_strip(api, name, controls, width, mat=3, rows=10):
    # Genuine shallow solid relief, broad enough to read as feather/body forms.
    path = smooth_path([(x, y, 0) for x, y in controls], rows)
    verts, faces, skin = [], [], []
    for i, center in enumerate(path):
        t = i / (rows - 1)
        tangent = (path[min(rows - 1, i + 1)] - path[max(0, i - 1)]).normalized()
        across = Vector((-tangent.y, tangent.x, 0))
        taper = 0.10 + 0.9 * math.sin(PI * t) ** 0.65
        if i == rows - 1:
            taper = 0.03
        for side, offset in ((-1, 0.0006), (0, 0.0020), (1, 0.0006)):
            q = center + across * width * side * taper
            verts.append((q.x, q.y, shirt_front(q.x, q.y) - offset))
            skin.append(api.torso_weight(q.y))
    for i in range(rows - 1):
        for k in range(2):
            a = i * 3 + k
            faces.append((a, a + 1, a + 4, a + 3))
    api.mesh(name, verts, faces, skin, mat, 0.0007)


def bird(api):
    relief_strip(api, 'Refined Zhuque rising S neck and breast',
                 [(-0.016, 1.247), (0.009, 1.277), (0.007, 1.308), (-0.003, 1.329), (0.013, 1.348)], 0.006, 3, 15)
    relief_strip(api, 'Refined Zhuque head and pointed beak',
                 [(0.003, 1.342), (0.013, 1.35), (0.029, 1.345), (0.041, 1.347)], 0.005, 3, 9)
    for crest, lift in ((0, 0), (1, 0.006)):
        relief_strip(api, f'Refined Zhuque head crest {crest}',
                     [(0.011, 1.346), (0.001, 1.358 + lift), (-0.013 - lift, 1.355 + lift)], 0.002, 3, 7)
    for side in (-1, 1):
        for feather in range(4):
            y = 1.303 - feather * 0.011
            x = 0.106 + feather * 0.004
            relief_strip(api, f'Refined Zhuque {side} spread wing feather {feather}',
                         [(side * 0.009, y - 0.014), (side * 0.046, y), (side * 0.080, y + 0.012),
                          (side * x, y + 0.048 - feather * 0.006)], 0.0045 - feather * 0.0004, 3, 10)
    for i in range(3):
        relief_strip(api, f'Refined Zhuque flowing tail plume {i}',
                     [(-0.008, 1.262), (-0.011 + i * 0.011, 1.222), (-0.048 + i * 0.026, 1.175),
                      (-0.066 + i * 0.036, 1.118 + i * 0.016)], 0.004 - i * 0.0005, 3, 12)


def block(api, name, center, scale, bone, mat):
    verts = [(center[0] + x * scale[0], center[1] + y * scale[1], center[2] + z * scale[2])
             for x, y, z in ((-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1),
                             (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1))]
    faces = [(0, 1, 2, 3), (4, 7, 6, 5), (0, 4, 5, 1), (1, 5, 6, 2), (2, 6, 7, 3), (3, 7, 4, 0)]
    return api.mesh(name, verts, faces, [{bone: 1}] * 8, mat)


def shoes(api):
    for side in ('L', 'R'):
        foot = f'gMan {side} Foot'
        x = api.p(foot).x
        # Seven physical sole layers give an articulated rocker and heel stack.
        rows = [(0.008, 0.062, 0.134), (0.018, 0.073, 0.147), (0.026, 0.074, 0.149),
                (0.033, 0.069, 0.144), (0.041, 0.071, 0.145), (0.050, 0.064, 0.136)]
        verts, faces = [], []
        seg = 32
        for row, (y, rx, rz) in enumerate(rows):
            for i in range(seg):
                a = i / seg * 2 * PI
                front = max(0, -math.sin(a))
                yy = y + 0.019 * front ** 3 + 0.004 * max(0, math.sin(a))
                verts.append((x + rx * math.cos(a), yy, -0.026 + rz * math.sin(a)))
        for row in range(len(rows) - 1):
            for i in range(seg):
                a, b = row * seg + i, row * seg + (i + 1) % seg
                faces.append((a, b, b + seg, a + seg))
        faces += [tuple(reversed(range(seg))), tuple((len(rows) - 1) * seg + i for i in range(seg))]
        api.mesh(f'Refined {side} segmented trail rocker sole', verts, faces, [{foot: 1}] * len(verts), 1)
        verts, faces = [], []
        for row in range(7):
            t = row / 6
            for i in range(seg):
                a = i / seg * 2 * PI
                rx = 0.064 * (1 - 0.36 * t)
                rz = 0.136 * (1 - 0.64 * t)
                z = -0.026 * (1 - t) + 0.046 * t + rz * math.sin(a)
                y = 0.052 + t * (0.045 + 0.030 * max(0, math.sin(a)))
                y += 0.019 * max(0, -math.sin(a)) ** 3 * (1 - t) ** 2
                verts.append((x + rx * math.cos(a), y, z))
        for row in range(6):
            for i in range(seg):
                a, b = row * seg + i, row * seg + (i + 1) % seg
                faces.append((a, b, b + seg, a + seg))
        api.mesh(f'Refined {side} low-cut technical upper', verts, faces, [{foot: 1}] * len(verts), 2, 0.0015)
        # Tongue and toe protection are shaped patches, not flat shoe cutouts.
        for feature in ('tongue', 'toe', 'heel'):
            verts, faces = [], []
            rows, cols = 7, 7
            for j in range(rows):
                t = j / (rows - 1)
                for k in range(cols):
                    s = k / (cols - 1) * 2 - 1
                    if feature == 'tongue':
                        xx, yy, zz = x + s * (0.029 - 0.005 * t), 0.099 + 0.04 * t - 0.006 * s * s, -0.074 + 0.13 * t
                    elif feature == 'toe':
                        q = t * 0.42
                        a = -PI / 2 + s * PI * 0.34
                        xx = x + 0.064 * (1 - 0.36 * q) * math.cos(a)
                        yy = 0.054 + 0.045 * q + 0.019 * max(0, -math.sin(a)) ** 3 * (1 - q) ** 2
                        zz = -0.026 * (1 - q) + 0.046 * q + 0.136 * (1 - 0.64 * q) * math.sin(a) - 0.001
                    else:
                        a = PI / 2 + s * PI * 0.64
                        xx = x + (0.061 - 0.012 * t) * math.cos(a)
                        yy = 0.052 + 0.072 * t - 0.012 * abs(s) * t
                        zz = 0.052 + (0.049 - 0.010 * t) * math.sin(a)
                    verts.append((xx, yy, zz))
            for j in range(rows - 1):
                for k in range(cols - 1):
                    a = j * cols + k
                    faces.append((a, a + 1, a + 1 + cols, a + cols))
            api.mesh(f'Refined {side} shoe {feature} panel', verts, faces, [{foot: 1}] * len(verts), 1 if feature != 'tongue' else 2, 0.0015)
        for i in range(5):
            z = -0.069 + i * 0.024
            y = 0.104 + i * 0.005
            for sign in (-1, 1):
                api.tube(f'Refined {side} crossing lace {i} {sign}',
                         [(x - sign * 0.027, y, z), (x, y + 0.004, z + 0.011), (x + sign * 0.027, y + 0.007, z + 0.023)],
                         [0.0017] * 3, [0.0017] * 3, [{foot: 1}] * 3, 5, 1)
        for sign in (-1, 1):
            for rail in range(3):
                api.tube(f'Refined {side} side chassis rail {sign} {rail}',
                         [(x + sign * 0.064, 0.046, -0.105 + rail * 0.041),
                          (x + sign * 0.059, 0.068 + rail * 0.004, -0.033 + rail * 0.033),
                          (x + sign * 0.046, 0.096, 0.035 + rail * 0.019)],
                         [0.0033, 0.004, 0.0025], [0.002] * 3, [{foot: 1}] * 3, 5, 1)
        api.tube(f'Refined {side} heel pull loop', [(x - 0.009, 0.111, 0.10), (x - 0.009, 0.138, 0.11),
                                                (x + 0.009, 0.138, 0.11), (x + 0.009, 0.111, 0.10)],
                 [0.003] * 4, [0.002] * 4, [{foot: 1}] * 4, 5, 1)
        for row in range(6):
            z = -0.135 + row * 0.045
            for sign in (-1, 1):
                block(api, f'Refined {side} angular tread {row} {sign}', (x + sign * 0.044, 0.008 + 0.012 * max(0, -z / 0.15) ** 3, z),
                      (0.016, 0.007, 0.014), foot, 2)
