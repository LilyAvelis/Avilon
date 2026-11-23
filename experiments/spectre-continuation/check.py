"""Run with Python 3.10+. Test nested Spectre patches in a fixed world frame.

Substitution data from Craig S. Kaplan's BSD-3-Clause spectreviz. The full
notice is included in ../unity-water-lab/Assets/StreamingAssets/Spectre-NOTICE.txt.
The coordinate ring and address experiment need only Python's standard library.
"""

import argparse
import cmath
import csv
import functools
import json
import math
import struct
from collections import Counter
from dataclasses import dataclass
from itertools import pairwise
from pathlib import Path

# a + b ζ + c ζ² + d ζ³, with ζ = exp(iπ/6) and ζ⁴ = ζ² - 1.
ZERO = (0, 0, 0, 0)
ONE = (1, 0, 0, 0)


def add(p, q):
    return (p[0] + q[0], p[1] + q[1], p[2] + q[2], p[3] + q[3])


def sub(p, q):
    return (p[0] - q[0], p[1] - q[1], p[2] - q[2], p[3] - q[3])


@functools.lru_cache(maxsize=16384)
def orient(p, turn, mirrored):
    a, b, c, d = p
    if mirrored:
        a, c, d = a + c, -c, -b - d
    for _ in range(turn % 12):
        a, b, c, d = -d, a, b + d, c
    return (a, b, c, d)


def xy(p):
    a, b, c, d = p
    return complex(a + math.sqrt(3) / 2 * b + c / 2, b / 2 + math.sqrt(3) / 2 * c + d)


@dataclass(frozen=True)
class Pose:
    origin: tuple = ZERO
    turn: int = 0
    mirrored: bool = False

    def map(self, p):
        return add(self.origin, orient(p, self.turn, self.mirrored))

    def compose(self, child):
        turn = self.turn + (-child.turn if self.mirrored else child.turn)
        return Pose(self.map(child.origin), turn % 12, self.mirrored != child.mirrored)

    def inverse(self):
        turn = self.turn if self.mirrored else (-self.turn) % 12
        return Pose(
            orient(sub(ZERO, self.origin), turn, self.mirrored), turn, self.mirrored
        )


CORNERS = []
tip = ZERO
for direction in (0, -2, 1, 3, 0, 2, 5, -5, 4, 6, 6, -4, -1, -3):
    CORNERS.append(tip)
    tip = add(tip, orient(ONE, direction, False))
assert tip == ZERO
MYSTIC = Pose(CORNERS[8], 1)
RULES = (
    (5, 1, -1, 2, 6, 4, 7, 0),
    (4, 1, 4, 7, 6, 5, 7, 0),
    (8, 1, 5, 7, 6, 5, 7, 0),
    (8, 1, 4, 7, 6, 5, 7, 0),
    (8, 1, 5, 7, 6, 8, 7, 0),
    (8, 1, 4, 7, 6, 8, 7, 0),
    (4, 1, 4, 7, 6, 5, 3, 0),
    (8, 1, 8, 7, 6, 5, 7, 0),
    (8, 1, 8, 7, 6, 8, 7, 0),
)


@functools.lru_cache(None)
def level(depth):
    if depth == 0:
        return (), tuple(CORNERS[i] for i in (3, 5, 7, 11))
    _, q = level(depth - 1)
    placements = [Pose()]
    for turn, previous, current in zip(
        (2, 2, 4, 6, 6, 8, 4), (3, 2, 3, 3, 2, 3, 3), (1, 0, 1, 1, 0, 1, 3)
    ):
        joint = placements[-1].map(q[previous])
        placements.append(Pose(sub(joint, orient(q[current], turn, False)), turn))
    flip = Pose(ZERO, 6, True)
    placements = tuple(flip.compose(child) for child in placements)
    return placements, tuple(
        placements[i].map(q[j]) for i, j in ((6, 2), (5, 1), (3, 2), (0, 1))
    )


def faces(depth, kind, pose):
    if depth == 0:
        yield pose
        if kind == 0:
            yield pose.compose(MYSTIC)
        return
    for typ, child in zip(RULES[kind], level(depth)[0]):
        if typ >= 0:
            yield from faces(depth - 1, typ, pose.compose(child))


def edge(a, b):
    return (a, b) if a < b else (b, a)


def edges(face):
    prev = face.map(CORNERS[-1])
    for p in CORNERS:
        nxt = face.map(p)
        yield edge(prev, nxt)
        prev = nxt


def patch(depth, root):
    emitted = list(faces(depth, 1, root))
    placements = set(emitted)
    assert len(emitted) == len(placements)
    assert all(not tile.mirrored for tile in placements)
    counts = Counter(e for tile in placements for e in edges(tile))
    assert all(1 <= n <= 2 for n in counts.values())
    return placements, counts


def extend(old_depth, world, first_child=4):
    # The second digit 1 is a Delta inside every parent type, including the
    # type in the chosen outer slot of Delta. Sandwich exact inverse transforms.
    outer = level(old_depth + 2)[0][first_child]
    inner = level(old_depth + 1)[0][1]
    return world.compose(outer.compose(inner).inverse())


ORIGIN = complex(1.41, 0.83)  # Strictly inside the initial Delta tile.
CURVATURE = 0.96


def safe_radius(counts):
    # Every point of the world-space cubic lies within .6 edge units of the
    # chord midpoint (|t-.5| <= .5 and |f(t)| <= .96*sqrt(3)/18 < .093).
    return (
        min(abs((xy(a) + xy(b)) / 2 - ORIGIN) for (a, b), n in counts.items() if n == 1)
        - 0.6
    )


def roots(a, b):
    # Intersections with the real axis in ray coordinates. Reversing an edge
    # preserves its S-curve because f(1-t) = -f(t).
    d = b - a
    k = CURVATURE * d.real
    c1, c2, c3 = d.imag + k, -3 * k, 2 * k

    def value(t):
        return ((c3 * t + c2) * t + c1) * t + a.imag

    cuts = [0.0, 1.0]
    disc = 4 * c2 * c2 - 12 * c3 * c1
    if abs(c3) > 1e-16 and disc > 0:
        mid, span = -c2 / (3 * c3), math.sqrt(disc) / abs(6 * c3)
        cuts += [r for r in (mid - span, mid + span) if 0 < r < 1]
    cuts.sort()
    ts = [t for t in cuts if abs(value(t)) < 1e-12]
    for left, right in pairwise(cuts):
        vl, vr = value(left), value(right)
        if (vl > 0) == (vr > 0) or abs(vl) < 1e-12 or abs(vr) < 1e-12:
            continue
        for _ in range(48):
            mid = (left + right) / 2
            if (value(mid) > 0) == (vl > 0):
                left = mid
            else:
                right = mid
        ts.append((left + right) / 2)
    return [
        a.real + d.real * t - d.imag * CURVATURE * t * (1 - t) * (1 - 2 * t) for t in ts
    ]


def trace(counts, angle, length, origin=ORIGIN, after=0.0):
    towards_ray = cmath.exp(-1j * math.radians(angle))
    result = []
    for e, multiplicity in counts.items():
        a, b = ((xy(p) - origin) * towards_ray for p in e)
        mid = (a + b) / 2
        if abs(mid.imag) > 0.6 or not after - 0.6 < mid.real < length + 0.6:
            continue
        for distance in roots(a, b):
            if after < distance <= length:
                assert multiplicity == 2
                result.append((distance, e))
    result.sort()
    # These deliberately generic rays have no tangent or multi-edge vertex hits.
    for left, right in pairwise(result):
        assert right[0] - left[0] > 1e-8
    return result


@functools.lru_cache(None)
def enclosure(depth):
    if depth == 0:
        center = complex(1.5, 0.9)
        return center, 3 + abs(floating_map(MYSTIC, center) - center)
    positions, landmarks = level(depth)
    center = sum(map(xy, landmarks)) / 4
    previous, radius = enclosure(depth - 1)
    return center, radius + max(
        abs(floating_map(child, previous) - center) for child in positions
    )


def floating_map(pose, value):
    return xy(pose.origin) + xy(orient(ONE, pose.turn, False)) * (
        value.conjugate() if pose.mirrored else value
    )


def strip_faces(depth, kind, pose, origin, to_ray, length):
    center, radius = enclosure(depth)
    local = (floating_map(pose, center) - origin) * to_ray
    if abs(local.imag) > radius or local.real < -radius or local.real > length + radius:
        return
    if depth == 0:
        yield from faces(0, kind, pose)
        return
    for typ, child in zip(RULES[kind], level(depth)[0]):
        if typ >= 0:
            yield from strip_faces(
                depth - 1, typ, pose.compose(child), origin, to_ray, length
            )


def strip_trace(depth, root, origin, angle, length, after=0.0):
    to_ray = cmath.exp(-1j * math.radians(angle))
    segment_start = origin + after * to_ray.conjugate()
    kept = list(strip_faces(depth, 1, root, segment_start, to_ray, length - after))
    counts = Counter(e for tile in kept for e in edges(tile))
    return trace(counts, angle, length, origin, after), len(kept)


def covers_water_crop(depth, root, center, extent):
    # Reproduce the Unity crop's boundary certificate before looking at its recorded ray.
    # The circumscribed disk is a conservative traversal region.
    rad = math.sqrt(2) * (extent + 1)

    def candidates(d, kind, pose):
        pivot, radius = enclosure(d)
        if abs(floating_map(pose, pivot) - center) > rad + radius:
            return
        if d == 0:
            yield from faces(0, kind, pose)
            return
        for typ, child in zip(RULES[kind], level(d)[0]):
            if typ >= 0:
                yield from candidates(d - 1, typ, pose.compose(child))

    counts = Counter()
    for tile in candidates(depth, 1, root):
        for e in edges(tile):
            m = (xy(e[0]) + xy(e[1])) / 2 - center
            if max(abs(m.real), abs(m.imag)) <= extent + 0.6:
                counts[e] += 1
    return bool(counts) and all(v == 2 for v in counts.values())


def check_water_recording(recording):
    with recording.open(encoding="utf-8-sig", newline="") as source:
        if recording.suffix.lower() == ".csv":
            rows = [r for r in csv.DictReader(source) if r["run"] == "water"]
            assert rows, "water trace missing from CSV"
            fixture = rows[0]
            expected = [float(r["distance"]) for r in rows]
        else:
            fixture = json.load(source)
            expected = fixture["crossings_m"]
    scale = struct.unpack("f", struct.pack("f", float(fixture["tile_scale"])))[0]
    extent = float(fixture["half_extent"]) / scale
    start = complex(float(fixture["x0"]), float(fixture["z0"])) / scale
    angle = float(fixture["angle_deg"])
    direction = cmath.exp(1j * math.radians(angle))
    end = min(
        ((extent if d > 0 else -extent) - a) / d
        for a, d in ((start.real, direction.real), (start.imag, direction.imag))
        if d != 0
    )
    depth = 0
    while enclosure(depth)[1] < 4 * extent + 12:
        depth += 1
    while True:
        root = Pose(ZERO, 0, bool(depth % 2))
        center = floating_map(root, enclosure(depth)[0])
        if covers_water_crop(depth, root, center, extent):
            break
        depth += 1
    origin = start + center
    original, _ = strip_trace(depth, root, origin, angle, end)
    assert len(original) == len(expected)
    err = max(
        abs(distance - hit[0] * scale) for distance, hit in zip(expected, original)
    )
    assert err < 1e-8
    print(
        f"Unity seed 23 depth={depth}: {len(original)} canonical crossings; "
        f"max error vs recorded Unity metres={err:.3g}",
        flush=True,
    )
    before = original
    for length_metres in (1500, 10000, 50000):
        root = extend(depth, root)
        depth += 2
        ray, visited = strip_trace(depth, root, origin, angle, length_metres / scale)
        assert ray[: len(before)] == before
        print(
            f"continued depth={depth} length_m={length_metres} "
            f"crossings={len(ray)} inherited_prefix={len(before)} strip_faces={visited}",
            flush=True,
        )
        before = ray
    # An arbitrary checkpoint and an exact seam checkpoint must each emit the shared event once.
    # Intersections retain the original ray parameter even when traversal restarts from a strip.
    cuts = (0.0, 153.25 / scale, ray[813][0], ray[4120][0], length_metres / scale)
    resumed = []
    for lo, hi in pairwise(cuts):
        chunk, _ = strip_trace(depth, root, origin, angle, hi, lo)
        resumed.extend(chunk)
    assert resumed == ray
    lengths = [b[0] - a[0] for a, b in pairwise(ray)]
    digits = [math.floor(10 * length) % 10 for length in lengths]

    def periods(series, tolerance):
        return [
            lag
            for lag in range(1, len(series) // 2 + 1)
            if all(abs(a - b) <= tolerance for a, b in zip(series, series[lag:]))
        ]

    print(
        f"resumed {len(resumed)} events across {len(cuts) - 1} windows, including two exact seam cuts; "
        f"periods up to {len(lengths) // 2}: at .01 edge {periods(lengths, 0.01)}, "
        f"for first decimal digits {periods(digits, 0)}; digit counts={sorted(Counter(digits).items())}"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--full",
        action="store_true",
        help="also check every face and edge of the 272,791-tile patch",
    )
    parser.add_argument(
        "--recording",
        type=Path,
        default=Path(__file__).with_name("water-seed23.json"),
        help="the recorded Unity crossings.csv or the bundled seed 23 fixture",
    )
    args = parser.parse_args()
    root = Pose()
    old_faces = set()
    old_edges = {}
    old_radius = 0
    for depth in range(0, 7 if args.full else 5, 2):
        if depth:
            root = extend(depth - 2, root)
        tiles, counts = patch(depth, root)
        radius = safe_radius(counts)
        assert old_faces <= tiles and old_edges.keys() <= counts.keys()
        assert all(counts[e] == 2 for e, n in old_edges.items() if n == 2)
        checks = 0
        if old_radius > 0:
            for angle in (7.31, 41.67, 109.23, 203.17, 311.43):
                prefix = trace(old_edges, angle, old_radius)
                long = trace(counts, angle, radius)
                assert prefix == [hit for hit in long if hit[0] <= old_radius]
                checks += len(prefix)
        print(
            f"depth={depth} faces={len(tiles)} unique_edges={len(counts)} "
            f"certified_radius={radius:.5f} inherited_faces={len(old_faces)} matching_crossings={checks}",
            flush=True,
        )
        old_faces, old_edges, old_radius = tiles, counts, radius
    check_water_recording(args.recording)


if __name__ == "__main__":
    main()
