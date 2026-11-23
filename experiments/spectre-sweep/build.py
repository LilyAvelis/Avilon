"""Build an offline Spectre ray sweep. Python 3.10+, standard library only."""

import argparse
import cmath
import csv
import hashlib
import json
import math
import sys
from itertools import pairwise
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "spectre-continuation"))
import check as geometry

RADIUS = 12.0
SAMPLES = 1440
K = geometry.CURVATURE
ORIGIN = geometry.ORIGIN
OUTLINE = list(map(geometry.xy, geometry.CORNERS))


def segment(a, b, t):
    f = K * t * (1 - t) * (1 - 2 * t)
    fp = K * (1 - 6 * t + 6 * t * t)
    return a + (b - a) * complex(t, f), (b - a) * complex(1, fp)


def prototype_centroid():
    # All integrands have degree <= 8. Five Gauss-Legendre points integrate through degree 9.
    quadrature = [
        (0.0, 0.5688888888888889),
        (0.5384693101056831, 0.4786286704993665),
        (-0.5384693101056831, 0.4786286704993665),
        (0.9061798459386640, 0.2369268850561891),
        (-0.9061798459386640, 0.2369268850561891),
    ]
    area = mx = my = 0.0
    for a, b in zip(OUTLINE, OUTLINE[1:] + OUTLINE[:1]):
        for node, weight in quadrature:
            p, v = segment(a, b, (node + 1) / 2)
            area += weight * (p.real * v.imag - p.imag * v.real) / 4
            mx += weight * p.real**2 * v.imag / 4
            my -= weight * p.imag**2 * v.real / 4
    return area, complex(mx, my) / area


def polygon_reference(subdivisions):
    points = [
        segment(a, b, j / subdivisions)[0]
        for a, b in zip(OUTLINE, OUTLINE[1:] + OUTLINE[:1])
        for j in range(subdivisions)
    ]
    twice_area = 0.0
    moment = 0j
    for a, b in zip(points, points[1:] + points[:1]):
        cross = (a.conjugate() * b).imag
        twice_area += cross
        moment += (a + b) * cross
    return points, twice_area / 2, moment / (3 * twice_area)


def winding(point, polygon):
    x, y = point.real, point.imag
    inside = False
    a = polygon[-1]
    for b in polygon:
        if (a.imag > y) != (b.imag > y) and x < a.real + (y - a.imag) * (
            b.real - a.real
        ) / (b.imag - a.imag):
            inside = not inside
        a = b
    return inside


def setup():
    root = geometry.extend(2, geometry.extend(0, geometry.Pose()))
    faces, counts = geometry.patch(4, root)
    assert geometry.safe_radius(counts) > RADIUS + 1
    area, centroid = prototype_centroid()
    _, linear_area, linear_center = polygon_reference(1024)
    assert abs(area - linear_area) < 1e-8 and abs(centroid - linear_center) < 1e-8
    tiles = []
    boundaries = {}
    reference_tile = None
    for pose in sorted(faces, key=lambda p: (p.origin, p.turn)):
        centre = geometry.floating_map(pose, centroid) - ORIGIN
        if abs(centre) > RADIUS * 1.65 + 3:
            continue
        index = len(tiles)
        anchor = geometry.xy(pose.origin) - ORIGIN
        unit = geometry.xy(geometry.orient(geometry.ONE, pose.turn, False))
        tiles.append(
            [
                anchor.real,
                anchor.imag,
                unit.real,
                unit.imag,
                centre.real,
                centre.imag,
                *pose.origin,
                pose.turn,
            ]
        )
        if pose == geometry.Pose():
            reference_tile = index
        for e in geometry.edges(pose):
            boundaries.setdefault(e, []).append(index)
    assert reference_tile is not None
    edges = []
    for e, sides in sorted(boundaries.items()):
        a, b = (geometry.xy(q) - ORIGIN for q in e)
        if abs((a + b) / 2) <= RADIUS + 0.6:
            assert len(sides) == 2
            edges.append((a, b, sides))
    return tiles, edges, reference_tile, area, centroid


def trace(angle, edges, anchor):
    rot = cmath.exp(-1j * math.radians(angle))
    hits = []
    for index, (a, b, sides) in enumerate(edges):
        mid = (a + b) * rot / 2
        if abs(mid.imag) > 0.6 or mid.real < -0.6 or mid.real > RADIUS + 0.6:
            continue
        for r in geometry.roots(a * rot, b * rot):
            if 1e-10 < r < RADIUS:
                hits.append((r, index, sides))
    hits.sort()
    row = []
    tile = anchor
    for r, edge, sides in hits:
        # This fixed sweep excludes exact vertex/tangent angles; detect any future change.
        assert tile in sides and (not row or r - row[-1][0] > 1e-8), (
            angle,
            tile,
            r,
            edge,
            sides,
        )
        row.append((r, edge, tile))
        tile = sides[1] if sides[0] == tile else sides[0]
    return row


def lengths(row):
    return [b - a for a, b in pairwise([0.0] + [r for r, *_ in row])]


def critical_angle(a, b, angle1, angle2):
    # Tangency means the position and derivative vectors are parallel. Bisection
    # between stationary points of the resulting cubic finds all interior roots.
    d = b - a
    x, y = (a / d).real, (a / d).imag
    c0, c1, c2, c3 = K * x - y, -6 * K * x, K * (6 * x - 3), 4 * K

    def f(t):
        return ((c3 * t + c2) * t + c1) * t + c0

    disc = 4 * c2 * c2 - 12 * c3 * c1
    cuts = [0.0, 1.0]
    if disc > 0:
        for sign in (-1, 1):
            t = (-2 * c2 + sign * math.sqrt(disc)) / (6 * c3)
            if 0 < t < 1:
                cuts.append(t)
    for lo, hi in pairwise(sorted(cuts)):
        if f(lo) * f(hi) >= 0:
            continue
        for _ in range(55):
            mid = (lo + hi) / 2
            if (f(lo) > 0) == (f(mid) > 0):
                lo = mid
            else:
                hi = mid
        t = (lo + hi) / 2
        p, v = segment(a, b, t)
        angle = math.degrees(cmath.phase(p)) % 360
        if angle1 < angle < angle2 and 0 < abs(p) < RADIUS:
            assert abs((p.conjugate() * v).imag) < 1e-9
            return angle
    return None


def find_example(rows, edges):
    options = []
    for i, (l, r) in enumerate(pairwise(rows)):
        small, big = (l, r) if len(l) < len(r) else (r, l)
        if len(big) != len(small) + 2:
            continue
        s = [e for _, e, _ in small]
        b = [e for _, e, _ in big]
        for k in range(1, min(5, len(s) - 1)):
            if b[k] != b[k + 1] or b[:k] + b[k + 2 :] != s:
                continue
            angle = critical_angle(
                *edges[b[k]][:2], i * 360 / SAMPLES, (i + 1) * 360 / SAMPLES
            )
            if angle is None:
                continue
            a_len, b_len = lengths(l), lengths(r)
            score = abs(a_len[k + 1] - b_len[k + 1])
            options.append(
                (
                    score,
                    {
                        "left": i,
                        "right": i + 1,
                        "critical": angle,
                        "order": k + 2,
                        "edge": b[k],
                        "split": k + 1,
                        "before": a_len[k + 1],
                        "after": b_len[k + 1],
                        "firstBefore": a_len[0],
                        "firstAfter": b_len[0],
                    },
                )
            )
    return max(options, key=lambda v: v[0])[1]


def write_csv(path, header, records):
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(header)
        writer.writerows(records)


def reference_crossings(angle, polylines):
    """Intersect straight chords of dense independent outlines, ignoring seam ownership."""
    rotate = cmath.exp(-1j * math.radians(angle))
    hits = []
    for edge, vertices in enumerate(polylines):
        a = vertices[0] * rotate
        for vertex in vertices[1:]:
            b = vertex * rotate
            if (a.imag > 0) != (b.imag > 0):
                distance = a.real - a.imag * (b.real - a.real) / (b.imag - a.imag)
                if 0 < distance < RADIUS:
                    hits.append((distance, edge))
            a = b
    return sorted(hits)


def build(out):
    out.mkdir(exist_ok=True, parents=True)
    tiles, edges, anchor, area, centroid = setup()
    rows = [trace(i * 360 / SAMPLES, edges, anchor) for i in range(SAMPLES)]
    assert rows[0] == trace(360, edges, anchor) or all(
        a[1:] == b[1:] and abs(a[0] - b[0]) < 1e-10
        for a, b in zip(rows[0], trace(360, edges, anchor), strict=True)
    )
    sample_polygon, _, _ = polygon_reference(96)
    divisions = 256
    polylines = [
        [segment(a, b, j / divisions)[0] for j in range(divisions + 1)]
        for a, b, _ in edges
    ]
    max_error = 0
    checks = 0
    for i in range(0, SAMPLES, 11):
        direction = cmath.exp(2j * math.pi * i / SAMPLES)
        prev = 0.0
        alt = reference_crossings(i * 360 / SAMPLES, polylines)
        for (distance, edge, owner), (chord_distance, chord_edge) in zip(
            rows[i], alt, strict=True
        ):
            assert edge == chord_edge
            max_error = max(max_error, abs(chord_distance - distance))
            anchor_x, anchor_y, c, s, *_ = tiles[owner]
            sample = (
                direction * ((distance + prev) / 2) - complex(anchor_x, anchor_y)
            ) * complex(c, -s)
            assert winding(sample, sample_polygon), (i, distance, owner)
            checks += 1
            prev = distance
    assert max_error < 0.001, max_error
    example = find_example(rows, edges)
    max_order = max(map(len, rows))
    jump_index = max(
        range(SAMPLES), key=lambda i: abs(rows[(i + 1) % SAMPLES][0][0] - rows[i][0][0])
    )
    first_before, first_after = rows[jump_index], rows[(jump_index + 1) % SAMPLES]
    assert (
        first_before[0][1] == first_before[1][1]
        and first_before[2][1] == first_after[0][1]
    )
    jump_angle = critical_angle(
        *edges[first_before[0][1]][:2],
        jump_index * 360 / SAMPLES,
        (jump_index + 1) * 360 / SAMPLES,
    )
    assert jump_angle is not None
    data = {
        "radius": RADIUS,
        "step": 360 / SAMPLES,
        "curvature": K,
        "origin": [ORIGIN.real, ORIGIN.imag],
        "outline": [[p.real, p.imag] for p in OUTLINE],
        "faces": tiles,
        "anchor": anchor,
        "rows": rows,
        "example": example,
        "maxOrder": max_order,
        "area": area,
        "centroid": [centroid.real, centroid.imag],
    }
    source = (
        HERE.parent
        / "unity-water-lab"
        / "Assets"
        / "StreamingAssets"
        / "Spectre-NOTICE.txt"
    )
    legal = source.read_text(encoding="utf-8")
    html = (
        (HERE / "viewer.html")
        .read_text(encoding="utf-8")
        .replace("/*__SWEEP__*/", json.dumps(data, separators=(",", ":")))
        .replace("__SPECTRE_NOTICE__", legal)
    )
    (out / "index.html").write_text(html, encoding="utf-8")
    write_csv(
        out / "segments.csv",
        ["angle_deg", "order", "start", "end", "length", "tile_id", "end_edge_id"],
        (
            (
                i * 360 / SAMPLES,
                j + 1,
                prev[0],
                hit[0],
                hit[0] - prev[0],
                hit[2],
                hit[1],
            )
            for i, row in enumerate(rows)
            for j, (prev, hit) in enumerate(pairwise([(0.0, -1, -1)] + row))
        ),
    )
    write_csv(
        out / "centroids.csv",
        [
            "tile_id",
            "a",
            "b",
            "c",
            "d",
            "turn_30deg",
            "centroid_x_from_origin",
            "centroid_y_from_origin",
            "area",
        ],
        ((i, *t[6:], t[4], t[5], area) for i, t in enumerate(tiles)),
    )
    report = {
        "samples": SAMPLES,
        "display_tiles": len(tiles),
        "shared_edges": len(edges),
        "intervals": sum(map(len, rows)),
        "min_intervals": min(map(len, rows)),
        "max_intervals": max_order,
        "midpoint_checks": checks,
        "chord_check_angles": len(range(0, SAMPLES, 11)),
        "chords_per_edge": divisions,
        "largest_chord_distance_error": max_error,
        "prototype_area": area,
        "prototype_centroid": [centroid.real, centroid.imag],
        "max_adjacent_first_jump": abs(first_after[0][0] - first_before[0][0]),
        "first_exit_example": {
            "before_deg": jump_index * 360 / SAMPLES,
            "after_deg": (jump_index + 1) * 360 / SAMPLES,
            "tangent_deg": jump_angle,
            "before": first_before[0][0],
            "after": first_after[0][0],
            "disappearing_edge": first_before[0][1],
            "next_edge": first_after[0][1],
        },
        "example": example,
        "html_sha256": hashlib.sha256((out / "index.html").read_bytes()).hexdigest(),
    }
    (out / "verification.json").write_text(
        json.dumps(report, indent=2), encoding="utf-8"
    )
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output", type=Path, default=HERE.parents[1] / "target" / "spectre-sweep"
    )
    build(parser.parse_args().output)
