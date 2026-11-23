using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using UnityEditor;
using Point = System.Numerics.Complex;

public static class SpectreRayExperiment
{
    private const double Epsilon = 1e-8;
    private const double MaxViewLength = 23;

    private readonly struct Hit
    {
        public readonly double S;
        public readonly int Tile, Side;
        public Hit(double s, int tile, int side) { S = s; Tile = tile; Side = side; }
    }

    [MenuItem("Mira/Run Spectre Ray Experiment")]
    public static void Run()
    {
        string output = Environment.GetEnvironmentVariable("MIRA_SPECTRE_RAY_OUTPUT")
            ?? Path.GetFullPath("../../target/spectre-ray");
        Directory.CreateDirectory(output);
        CheckSquareGrid();
        const float gameScale = 3.6f;
        float gameExtent = WaterLabSeabed.HalfSize;
        var game = Build(gameExtent, gameScale, out double buildSmall);
        var large = Build(512, 1, out double buildLarge);
        using (var csv = Writer(Path.Combine(output, "crossings.csv")))
        using (var timings = Writer(Path.Combine(output, "timings.csv")))
        {
            csv.WriteLine("run,seed,half_extent,tile_scale,x0,z0,angle_deg,index,distance,interval,x,z,edge,from_tile,to_tile");
            timings.WriteLine("run,crossings,trace_ms,patch_ms,tiles");
            WriteTrace("water", 23, game, gameExtent, gameScale, buildSmall, csv, timings);
            WriteTrace("water-repeat", 23, game, gameExtent, gameScale, buildSmall, csv, timings);
            foreach (uint seed in new uint[] { 1, 7, 23, 77, 137, 2026, 10001, 4294967295 })
                WriteTrace("seed-" + seed, seed, large, 512, 1, buildLarge, csv, timings);
            WriteTrace("axis-0", 23, large, 512, 1, buildLarge, csv, timings, 0);
            WriteTrace("axis-90", 23, large, 512, 1, buildLarge, csv, timings, 90);
            WriteTrace("angle-30", 23, large, 512, 1, buildLarge, csv, timings, 30);
            WriteTrace("angle-30-near", 23, large, 512, 1, buildLarge, csv, timings, 30.00001);
        }
        WriteDiagram(output, large);
        UnityEngine.Debug.Log($"MIRA SPECTRE RAY OK · {output}");
    }

    private static StreamWriter Writer(string path) => new(path, false, new System.Text.UTF8Encoding(false));
    private static string F(FormattableString line) => FormattableString.Invariant(line);

    private static SpectreSeabed Build(float halfExtent, float scale, out double elapsed)
    {
        var timer = Stopwatch.StartNew();
        var result = new SpectreSeabed(halfExtent, scale);
        elapsed = timer.Elapsed.TotalMilliseconds;
        return result;
    }

    private static double Unit(ref uint seed)
    {
        unchecked
        {
            uint bits = seed += 0x9e3779b9;
            bits = (bits ^ (bits >> 16)) * 0x21f0aaad;
            bits = (bits ^ (bits >> 15)) * 0x735a2d97;
            return ((bits ^ (bits >> 15)) + 0.5) / 4294967296.0;
        }
    }

    private static (Point, double) Start(uint seed, double scale)
    {
        var origin = new Point((Unit(ref seed) - 0.5) * 15 * scale, (Unit(ref seed) - 0.5) * 15 * scale);
        return (origin, Unit(ref seed) * 360);
    }

    private static Point Direction(double degrees) => Point.FromPolarCoordinates(1, degrees * Math.PI / 180);
    private static double Exit(Point start, Point direction, double extent)
        => Math.Min(Slab(start.Real, direction.Real, extent), Slab(start.Imaginary, direction.Imaginary, extent));
    private static double Slab(double x, double dx, double extent)
        => dx == 0 ? double.PositiveInfinity : ((dx > 0 ? extent : -extent) - x) / dx;

    private static void WriteTrace(string name, uint seed, SpectreSeabed tiling, double extent, float scale,
        double buildMs, TextWriter csv, TextWriter timings, double? fixedAngle = null)
    {
        (Point start, double angle) = Start(seed, scale);
        angle = fixedAngle ?? angle;
        Point direction = Direction(angle);
        double length = Exit(start, direction, extent);
        var timer = Stopwatch.StartNew();
        var trace = Trace(tiling, start, direction, length);
        timings.WriteLine(F($"{name},{trace.Count},{timer.Elapsed.TotalMilliseconds:F3},{buildMs:F3},{tiling.TileCount}"));
        for (int i = 0; i < trace.Count; i++)
        {
            var (hit, before, after) = trace[i];
            Point point = start + direction * hit.S;
            string interval = i == 0 ? "" : F($"{hit.S - trace[i - 1].Hit.S:R}");
            csv.WriteLine(F($"{name},{seed},{extent:R},{scale:R},{start.Real:R},{start.Imaginary:R},{angle:R},{i},{hit.S:R},{interval},{point.Real:R},{point.Imaginary:R},{hit.Tile}:{hit.Side},{before},{after}"));
        }
    }

    private static List<(Hit Hit, int Before, int After)> Trace(SpectreSeabed tiling, Point start, Point direction, double length)
    {
        var hits = new List<Hit>();
        Point toRay = Point.Conjugate(direction);
        tiling.VisitExperimentEdges(start, direction, length, 0, (tile, edge, a, b) =>
            Intersect((a - start) * toRay, (b - start) * toRay, SpectreSeabed.EdgeCurvature, length, tile, edge, hits));
        hits = MergeHits(hits);
        var result = new List<(Hit, int, int)>();
        int before = OwnerBetween(tiling, start, direction, 0, hits.Count > 0 ? hits[0].S : length);
        for (int i = 0; i < hits.Count; i++)
        {
            Hit hit = hits[i];
            int after = OwnerBetween(tiling, start, direction, hit.S, i + 1 == hits.Count ? length : hits[i + 1].S);
            if (after != before)
                result.Add((hit, before, after));
            before = after;
        }
        return result;
    }

    private static int OwnerBetween(SpectreSeabed tiling, Point start, Point direction, double a, double b)
    {
        int middle = tiling.FindExperimentTile(start + direction * (a + (b - a) * 0.5));
        if (middle < 0 ||
            tiling.FindExperimentTile(start + direction * (a + (b - a) * 0.25)) != middle ||
            tiling.FindExperimentTile(start + direction * (a + (b - a) * 0.75)) != middle)
            throw new InvalidOperationException(F($"Suspected missed crossing between {a:R} and {b:R}."));
        return middle;
    }

    private static List<Hit> MergeHits(List<Hit> hits)
    {
        hits.Sort((a, b) => a.S.CompareTo(b.S));
        var merged = new List<Hit>(hits.Count);
        for (int first = 0; first < hits.Count;)
        {
            Hit representative = hits[first];
            int next = first + 1;
            while (next < hits.Count && hits[next].S - hits[first].S <= Epsilon)
            {
                Hit hit = hits[next++];
                if (hit.Tile < representative.Tile || (hit.Tile == representative.Tile && hit.Side < representative.Side))
                    representative = hit;
            }
            merged.Add(representative);
            first = next;
        }
        return merged;
    }

    // Local ray coordinates. y(t) is cubic; each interval between derivative roots is monotone.
    private static void Intersect(Point a, Point b, double curvature, double length,
        int tile, int edge, List<Hit> hits)
    {
        Point delta = b - a;
        double k = delta.Real * curvature;
        double c1 = delta.Imaginary + k;
        double c2 = -3 * k;
        double c3 = 2 * k;
        double Y(double t) => ((c3 * t + c2) * t + c1) * t + a.Imaginary;
        void Add(double t)
        {
            double s = a.Real + delta.Real * t - delta.Imaginary * (curvature * t * (1 - t) * (1 - 2 * t));
            if (s > Epsilon && s < length - Epsilon)
                hits.Add(new Hit(s, tile, edge));
        }
        void Search(double left, double right)
        {
            double yl = Y(left), yr = Y(right);
            if (Math.Abs(yl) < 1e-12) Add(left);
            if (Math.Abs(yr) < 1e-12) Add(right);
            if ((yl < 0) == (yr < 0)) return;
            for (int i = 0; i < 46; i++)
            {
                double middle = (left + right) * 0.5;
                if ((Y(middle) < 0) == (yl < 0)) left = middle;
                else right = middle;
            }
            Add((left + right) * 0.5);
        }
        double start = 0;
        double discriminant = 4 * c2 * c2 - 12 * c3 * c1;
        if (Math.Abs(c3) > 1e-18 && discriminant > 0)
        {
            double mid = -c2 / (3 * c3);
            double span = Math.Sqrt(discriminant) / Math.Abs(6 * c3);
            double left = mid - span, right = mid + span;
            if (left > 0 && left < 1) { Search(start, left); start = left; }
            if (right > 0 && right < 1) { Search(start, right); start = right; }
        }
        Search(start, 1);
    }

    private static void CheckSquareGrid()
    {
        // Same solver on straight unit edges. The diagonal passes through shared grid vertices.
        foreach (double angle in new[] { 0.0, 45.0 })
        {
            const double length = 5;
            Point origin = new(0.125, 0.125);
            Point toRay = Point.Conjugate(Direction(angle));
            var hits = new List<Hit>();
            for (int x = 0; x <= 6; x++)
            for (int z = 0; z <= 6; z++)
            {
                Point a = (new Point(x, z) - origin) * toRay;
                Intersect(a, a + toRay, 0, length, 0, 0, hits);
                Intersect(a, a + Point.ImaginaryOne * toRay, 0, length, 0, 1, hits);
            }
            hits = MergeHits(hits);
            double expected = angle == 0 ? 1 : Math.Sqrt(2);
            if (hits.Count < 3 || hits.Zip(hits.Skip(1), (a, b) => b.S - a.S).Any(value => Math.Abs(value - expected) > Epsilon))
                throw new InvalidOperationException("Square grid did not recover its known period.");
        }
    }

    private static void WriteDiagram(string output, SpectreSeabed tiling)
    {
        (Point origin, double angle) = Start(23, 1);
        Point direction = Direction(angle), toRay = Point.Conjugate(direction);
        var trace = Trace(tiling, origin, direction, MaxViewLength);
        using var svg = Writer(Path.Combine(output, "beam.svg"));
        svg.WriteLine("<svg xmlns='http://www.w3.org/2000/svg' width='1120' height='470' viewBox='0 0 1120 470'>");
        svg.WriteLine("<rect width='1120' height='470' fill='#101a2d'/>");
        svg.WriteLine("<clipPath id='field'><rect x='0' y='62' width='1120' height='353'/></clipPath>");
        svg.WriteLine("<text x='38' y='40' font-family='Arial' font-size='22' fill='#e1edff'>Spectre · луч seed 23</text>");
        svg.WriteLine("<g clip-path='url(#field)'><g transform='translate(42 236) scale(44 -44)' fill='none' stroke='#607ca6' stroke-width='.022'>");
        tiling.VisitExperimentEdges(origin, direction, MaxViewLength, 5, (_, _, a, b) =>
        {
            a = (a - origin) * toRay;
            b = (b - origin) * toRay;
            Point d = b - a;
            Point bend = d * Point.ImaginaryOne * (SpectreSeabed.EdgeCurvature / 3);
            Point first = a + d / 3 + bend, second = b - d / 3 - bend;
            svg.WriteLine(F($"<path d='M {a.Real:F5} {a.Imaginary:F5} C {first.Real:F5} {first.Imaginary:F5} {second.Real:F5} {second.Imaginary:F5} {b.Real:F5} {b.Imaginary:F5}'/>"));
        });
        svg.WriteLine("<path d='M 0 0 L 23 0' stroke='#ffc47c' stroke-width='.045'/>");
        foreach (var mark in trace)
            svg.WriteLine(F($"<circle cx='{mark.Hit.S:F6}' cy='0' r='.067' fill='#dfff95' stroke='#19243a'/>"));
        svg.WriteLine("</g></g><text x='38' y='446' font-family='Arial' font-size='17' fill='#e1edff'>Зелёные точки — переходы между плитками. Расстояние между точками — одно число.</text></svg>");
    }
}
