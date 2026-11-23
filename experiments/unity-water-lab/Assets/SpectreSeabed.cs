using System;
using System.Collections.Generic;
using UnityEngine;
using Point = System.Numerics.Complex;

/// <summary>A finite crop of the Spectre substitution tiling, in world XZ metres.</summary>
// Layout adapted from Craig S. Kaplan's spectreviz (BSD-3-Clause). See the accompanying notice.
// Smith, Myers, Kaplan & Goodman-Strauss, "A chiral aperiodic monotile", Lemma 2.1 and Appendix A.
public sealed class SpectreSeabed
{
    private const double BucketSize = 4.0;
    private const double LeafRadius = 3.0;
    private const float SeamHeight = -5f;
    public const double EdgeCurvature = 0.96;
    private const int Gamma = 0, Delta = 1;
    private static readonly Point LeafCenter = new(1.5, 0.9);
    private static readonly Point[] Corners = MakeCorners();
    private static readonly Pose SecondMystic = new(Corners[8], Rotation(30), false);

    // Γ Δ Θ Λ Ξ Π Σ Φ Ψ. Each row lists the child types at the eight shared placements.
    private static readonly int[,] Children =
    {
        {5,1,-1,2,6,4,7,0}, {4,1,4,7,6,5,7,0}, {8,1,5,7,6,5,7,0},
        {8,1,4,7,6,5,7,0}, {8,1,5,7,6,8,7,0}, {8,1,4,7,6,8,7,0},
        {4,1,4,7,6,5,3,0}, {8,1,8,7,6,5,7,0}, {8,1,8,7,6,8,7,0}
    };

    private readonly struct Pose
    {
        public readonly Point Origin, Axis;
        public readonly bool Reflected;
        public Pose(Point origin, Point axis, bool reflected)
        {
            Origin = origin;
            Axis = axis;
            Reflected = reflected;
        }
        public Point Map(Point p) => Origin + Axis * (Reflected ? Point.Conjugate(p) : p);
        public Point Local(Point p)
        {
            Point local = (p - Origin) * Point.Conjugate(Axis);
            return Reflected ? Point.Conjugate(local) : local;
        }
        public Pose Compose(Pose child) => new(Map(child.Origin),
            Axis * (Reflected ? Point.Conjugate(child.Axis) : child.Axis), Reflected != child.Reflected);
    }

    private sealed class Level
    {
        public Point[] Landmarks;
        public Pose[] Placements;
        public Point Center;
        public double Radius;
    }

    private readonly struct Tile
    {
        public readonly Pose Pose;
        public readonly float Tone;
        public Tile(Pose pose, uint path)
        {
            Pose = pose;
            uint h = unchecked((path ^ (path >> 16)) * 0x7feb352du);
            h = unchecked((h ^ (h >> 15)) * 0x846ca68bu);
            Tone = ((h ^ (h >> 16)) >> 8) / 16777215f;
        }
    }

    private readonly double scale, extent;
    private readonly float halfExtent;
    private readonly int gridSize;
    private readonly List<Tile> tiles = new();
    private readonly List<int>[] buckets;
    public int TileCount => tiles.Count;

    /// <param name="tileScale">Length of a prototile edge in metres.</param>
    public SpectreSeabed(float halfExtent, float tileScale = 6f)
    {
        if (!(halfExtent > 0f) || float.IsInfinity(halfExtent))
            throw new ArgumentOutOfRangeException(nameof(halfExtent));
        if (!(tileScale > 0f) || float.IsInfinity(tileScale))
            throw new ArgumentOutOfRangeException(nameof(tileScale));
        this.halfExtent = halfExtent;
        scale = tileScale;
        extent = halfExtent / scale;
        // Bound the finite CPU crop; smaller tiles need a tiled/streaming caller.
        if (extent > 512)
            throw new ArgumentOutOfRangeException(nameof(tileScale), "The half-extent exceeds 512 tile edges.");
        gridSize = Math.Max(1, (int)Math.Ceiling(2 * extent / BucketSize));
        buckets = new List<int>[gridSize * gridSize];
        BuildPatch();
    }

    public float Height(Vector2 p) => TrySample(p, out float height, out _) ? height : SeamHeight;

    /// <summary>Returns relief in metres and a stable face tone in [0,1] inside the requested square.</summary>
    public bool TrySample(Vector2 p, out float height, out float tileTone)
    {
        height = SeamHeight;
        tileTone = 0f;
        if (!(Math.Abs(p.x) <= halfExtent && Math.Abs(p.y) <= halfExtent))
            return false;
        int index = FindTile(new Point(p.x / scale, p.y / scale), out double clearance);
        if (index < 0)
            return false;
        double s = Math.Min(1, clearance / 0.43);
        tileTone = tiles[index].Tone;
        height = SeamHeight + (1.2f + 6.2f * tileTone) * (float)(s * s * (3 - 2 * s));
        return true;
    }

    private int FindTile(Point q, out double clearance)
    {
        clearance = 0;
        var candidates = buckets[Bucket(q.Real) + gridSize * Bucket(q.Imaginary)];
        if (candidates == null)
            return -1;
        foreach (int index in candidates)
            if (Inside(tiles[index].Pose.Local(q), out clearance))
                return index;
        return -1;
    }

#if UNITY_EDITOR
    public int FindExperimentTile(Point world) => FindTile(world / scale, out _);

    // Return both owners of a shared edge. The trace resolves their coincident intersections.
    public void VisitExperimentEdges(Point origin, Point direction, double length, double sideMargin,
        Action<int, int, Point, Point> visit)
    {
        Point rayOrigin = origin / scale;
        double normalizedLength = length / scale;
        double margin = sideMargin / scale + LeafRadius;
        Point acrossRay = Point.Conjugate(direction);
        for (int tileIndex = 0; tileIndex < tiles.Count; tileIndex++)
        {
            Pose pose = tiles[tileIndex].Pose;
            Point center = (pose.Map(LeafCenter) - rayOrigin) * acrossRay;
            if (Math.Abs(center.Imaginary) > margin || center.Real < -margin || center.Real > normalizedLength + margin)
                continue;
            Point a = pose.Map(Corners[Corners.Length - 1]) * scale;
            for (int edge = 0; edge < Corners.Length; edge++)
            {
                Point b = pose.Map(Corners[edge]) * scale;
                visit(tileIndex, edge, a, b);
                a = b;
            }
        }
    }
#endif

    private static bool Inside(Point p, out double clearance, bool resolveChordTie = true)
    {
        clearance = 0;
        double x = p.Real, y = p.Imaginary;
        if (x < -1.0 || x > 4.0 || y < -1.0 || y > 2.5)
            return false;
        bool inside = false;
        double nearest = double.PositiveInfinity;
        Point a = Corners[Corners.Length - 1];
        foreach (Point b in Corners)
        {
            double dx = b.Real - a.Real, dy = b.Imaginary - a.Imaginary;
            double px = x - a.Real, py = y - a.Imaginary;
            if ((a.Imaginary > y) != (b.Imaginary > y) && px < py * dx / dy)
                inside = !inside;
            // In unit-edge coordinates the actual contour is u=f(t), not the straight chord.
            // f(1-t)=-f(t), so a neighbour tracing the edge backwards has exactly the same curve.
            double t = px * dx + py * dy;
            double square;
            if (t > 0 && t < 1)
            {
                double u = dx * py - dy * px;
                double curve = EdgeCurvature * t * (1 - t) * (1 - 2 * t);
                // An exact point on the auxiliary chord has ambiguous straight-polygon parity.
                // Move to a definite side of that chord, well inside the shared-edge tolerance.
                if (resolveChordTie && Math.Abs(u) < 1e-14)
                    return Inside(p + new Point(-dy, dx) * 4e-12, out clearance, false);
                if ((u > 0) != (u > curve))
                    inside = !inside;
                square = (u - curve) * (u - curve);
            }
            else
            {
                if (t >= 1) { px -= dx; py -= dy; }
                square = px * px + py * py;
            }
            nearest = Math.Min(nearest, square);
            a = b;
        }
        // The local normal-direction clearance vanishes exactly on the cubic. Along shared edges
        // either owner gives the same height; this tolerance absorbs rigid-transform roundoff.
        clearance = Math.Sqrt(nearest);
        return inside || clearance < 1e-10;
    }

    private static Point Rotation(int degrees) => Point.FromPolarCoordinates(1, degrees * Math.PI / 180);

    private static Point[] MakeCorners()
    {
        int[] headings = { 0,-60,30,90,0,60,150,-150,120,180,180,-120,-30,-90 };
        var points = new Point[headings.Length];
        Point current = Point.Zero;
        for (int i = 0; i < headings.Length; i++)
        {
            points[i] = current;
            current += Rotation(headings[i]);
        }
        return points;
    }

    private void BuildPatch()
    {
        var basis = new Level { Landmarks = new[] { Corners[3], Corners[5], Corners[7], Corners[11] },
            Center = LeafCenter, Radius = LeafRadius + Point.Abs(SecondMystic.Map(LeafCenter) - LeafCenter) };
        var levels = new List<Level> { basis };
        while (levels[levels.Count - 1].Radius < 4 * extent + 12)
            levels.Add(Expand(levels[levels.Count - 1]));
        for (int attempt = 0; attempt < 5; attempt++)
        {
            var root = levels[levels.Count - 1];
            int depth = levels.Count - 1;
            // Undo the odd generation's global reflection once, so every emitted tile is a
            // rotation/translation of the same explicit prototile even when the crop size changes.
            bool reflectRoot = (depth & 1) != 0;
            Point center = reflectRoot ? Point.Conjugate(root.Center) : root.Center;
            Visit(levels, depth, Delta, new Pose(-center, Point.One, reflectRoot), 2166136261u);
            if (InteriorIsCovered() && TrySample(Vector2.zero, out _, out _))
                return;
            tiles.Clear();
            Array.Clear(buckets, 0, buckets.Length);
            levels.Add(Expand(root));
        }
        throw new InvalidOperationException("Spectre patch does not enclose the requested square.");
    }

    private static Level Expand(Level previous)
    {
        Point[] quad = previous.Landmarks;
        var positions = new Pose[8];
        positions[0] = new Pose(0, 1, false);
        int[] headings = { 60, 60, 120, 180, 180, 240, 120 };
        int[] from = { 3, 2, 3, 3, 2, 3, 3 };
        int[] to = { 1, 0, 1, 1, 0, 1, 3 };
        for (int i = 0; i < headings.Length; i++)
        {
            Point axis = Rotation(headings[i]);
            Point joint = positions[i].Map(quad[from[i]]);
            positions[i + 1] = new Pose(joint - axis * quad[to[i]], axis, false);
        }
        // Each substitution reverses the ENTIRE patch. Every leaf in a given generation
        // therefore still has the same chirality, including both tiles in the Mystic.
        var reflection = new Pose(0, -1, true);
        for (int i = 0; i < positions.Length; i++)
            positions[i] = reflection.Compose(positions[i]);
        var next = new[] { positions[6].Map(quad[2]), positions[5].Map(quad[1]),
            positions[3].Map(quad[2]), positions[0].Map(quad[1]) };
        Point center = (next[0] + next[1] + next[2] + next[3]) / 4;
        double radius = 0;
        foreach (Pose position in positions)
            radius = Math.Max(radius, Point.Abs(position.Map(previous.Center) - center) + previous.Radius);
        return new Level { Landmarks = next, Placements = positions, Center = center, Radius = radius };
    }

    private void Visit(List<Level> levels, int depth, int kind, Pose pose, uint path)
    {
        Level level = levels[depth];
        Point center = pose.Map(level.Center);
        if (Math.Max(Math.Abs(center.Real), Math.Abs(center.Imaginary)) > extent + 1 + level.Radius)
            return;
        if (depth == 0)
        {
            AddTile(pose, unchecked(path * 16777619u));
            if (kind == Gamma)
                AddTile(pose.Compose(SecondMystic), unchecked(path * 16777619u + 1));
            return;
        }
        for (int i = 0; i < 8; i++)
        {
            int child = Children[kind, i];
            if (child >= 0)
                Visit(levels, depth - 1, child, pose.Compose(level.Placements[i]), unchecked(path * 16777619u + (uint)i));
        }
    }

    private void AddTile(Pose pose, uint path)
    {
        Point center = pose.Map(LeafCenter);
        if (Math.Max(Math.Abs(center.Real), Math.Abs(center.Imaginary)) > extent + LeafRadius + 1)
            return;
        int id = tiles.Count;
        tiles.Add(new Tile(pose, path));
        for (int z = Bucket(center.Imaginary - LeafRadius); z <= Bucket(center.Imaginary + LeafRadius); z++)
        for (int x = Bucket(center.Real - LeafRadius); x <= Bucket(center.Real + LeafRadius); x++)
            (buckets[z * gridSize + x] ??= new List<int>()).Add(id);
    }

    private int Bucket(double p) => Math.Max(0, Math.Min(gridSize - 1, (int)Math.Floor((p + extent) / BucketSize)));

    private bool InteriorIsCovered()
    {
        // Each unit edge that could meet the square must have a mate. Combined with an
        // interior witness this rules out crossing the patch boundary anywhere in the crop.
        var open = new HashSet<((long, long), (long, long))>();
        foreach (Tile tile in tiles)
        {
            Point a = tile.Pose.Map(Corners[Corners.Length - 1]);
            foreach (Point corner in Corners)
            {
                Point b = tile.Pose.Map(corner);
                Point mid = (a + b) * 0.5;
                if (Math.Max(Math.Abs(mid.Real), Math.Abs(mid.Imaginary)) <= extent + 0.6)
                {
                    var first = VertexKey(a);
                    var last = VertexKey(b);
                    var key = first.CompareTo(last) < 0 ? (first, last) : (last, first);
                    if (!open.Add(key))
                        open.Remove(key);
                }
                a = b;
            }
        }
        return open.Count == 0;
    }

    private static (long, long) VertexKey(Point p) =>
        ((long)Math.Round(p.Real * 1e7), (long)Math.Round(p.Imaginary * 1e7));
}
