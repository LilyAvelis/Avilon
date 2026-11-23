using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Rendering;

public static class PoincarePool
{
    private const float DiskCut = 0.976f;
    private const float FadeStart = 0.918f;
    private const int DiagramSteps = 10;
    private const int FloorSteps = 24;
    private const int TileLimit = 1400;
    private static readonly Vector3 Up = Vector3.up;
    private static readonly Color32[] Ceramic =
    {
        new(93, 164, 157, 255), new(77, 136, 156, 255), new(101, 176, 181, 255),
        new(222, 156, 80, 255), new(232, 101, 72, 255), new(158, 139, 210, 255),
        new(151, 179, 139, 255), new(211, 191, 151, 255), new(107, 140, 172, 255),
    };
    private static readonly int[] CentralColors = { 3, 0, 5, 4, 2 };

    /// <summary>Builds the {4,5} tiling in XZ, optionally expanding the outer rings across the pool floor.</summary>
    public static Mesh Build(float radius, bool spreadAcrossFloor)
    {
        if (!(radius > 0f) || float.IsInfinity(radius))
            throw new ArgumentOutOfRangeException(nameof(radius));

        // The center–corner–edge midpoint right triangle has angles pi/4, pi/5, pi/2.
        double coshCorner = 1.0 / (Math.Tan(Math.PI / 4.0) * Math.Tan(Math.PI / 5.0));
        double sinhCorner = Math.Sqrt(coshCorner * coshCorner - 1.0);
        double cornerRadius = sinhCorner / (coshCorner + 1.0);
        // Include any neighboring cell whose circumscribed hyperbolic circle can touch the cut.
        double searchRadius = (DiskCut + cornerRadius) / (1.0 + DiskCut * cornerRadius);
        double searchRadiusSquared = searchRadius * searchRadius;

        var first = new Tile(ToCornerOrigin(new HyperPoint(0.0, 0.0, 1.0), coshCorner, sinhCorner),
            new[]
            {
                ToCornerOrigin(new HyperPoint(sinhCorner, 0.0, coshCorner), coshCorner, sinhCorner),
                ToCornerOrigin(new HyperPoint(0.0, sinhCorner, coshCorner), coshCorner, sinhCorner),
                ToCornerOrigin(new HyperPoint(-sinhCorner, 0.0, coshCorner), coshCorner, sinhCorner),
                ToCornerOrigin(new HyperPoint(0.0, -sinhCorner, coshCorner), coshCorner, sinhCorner),
            });

        var queue = new Queue<Tile>();
        var centers = new HashSet<long> { CenterKey(first.Center) };
        queue.Enqueue(first);
        var builder = new PoolMesh(radius, spreadAcrossFloor);
        while (queue.TryDequeue(out Tile tile))
        {
            builder.Add(tile);
            for (int side = 0; side < 4; side++)
            {
                HyperPoint a = tile.Corners[side];
                HyperPoint b = tile.Corners[(side + 1) % 4];
                HyperPoint normal = HyperPoint.SideNormal(a, b);
                double reflectionScale = 2.0 / HyperPoint.Inner(normal, normal);
                HyperPoint center = Reflect(tile.Center, normal, reflectionScale);
                if (center.DiskLengthSquared > searchRadiusSquared || !centers.Add(CenterKey(center)))
                    continue;
                if (centers.Count > TileLimit)
                    throw new InvalidOperationException("The Poincare tiling exceeded its construction budget.");

                // A reflection reverses orientation. Swap corners 1 and 3 to keep all faces upward.
                queue.Enqueue(new Tile(center, new[]
                {
                    Reflect(tile.Corners[0], normal, reflectionScale), Reflect(tile.Corners[3], normal, reflectionScale),
                    Reflect(tile.Corners[2], normal, reflectionScale), Reflect(tile.Corners[1], normal, reflectionScale),
                }));
            }
        }

        return builder.Create();
    }

    private static HyperPoint ToCornerOrigin(HyperPoint p, double cosh, double sinh)
        => new(cosh * p.X - sinh * p.T, p.Z, cosh * p.T - sinh * p.X);

    private static HyperPoint Reflect(HyperPoint p, HyperPoint normal, double scale)
        => p - normal * (scale * HyperPoint.Inner(p, normal));

    private static long CenterKey(HyperPoint p)
    {
        double scale = 1e6 / (p.T + 1.0);
        int x = (int)Math.Round(p.X * scale);
        int z = (int)Math.Round(p.Z * scale);
        return ((long)x << 32) | (uint)z;
    }

    private static Color32 TileColor(HyperPoint p)
    {
        if (p.DiskLengthSquared < 0.19)
        {
            // Give the five tiles meeting at the origin distinct colors.
            double angle = Math.Atan2(p.Z, p.X) + Math.PI + 0.025;
            int sector = (int)(angle / (2.0 * Math.PI / 5.0)) % CentralColors.Length;
            return Ceramic[CentralColors[sector]];
        }
        long key = CenterKey(p);
        unchecked
        {
            uint hash = (uint)(key >> 32) * 0x8da6b343u ^ (uint)key * 0xd8163841u;
            hash = (hash ^ (hash >> 16)) * 0x7feb352du;
            hash ^= hash >> 15;
            return Ceramic[(int)(hash % Ceramic.Length)];
        }
    }

    private readonly struct HyperPoint
    {
        public readonly double X, Z, T;

        public HyperPoint(double x, double z, double t) { X = x; Z = z; T = t; }

        public Vector2 Disk => new((float)(X / (T + 1.0)), (float)(Z / (T + 1.0)));
        public double DiskLengthSquared => (X * X + Z * Z) / ((T + 1.0) * (T + 1.0));

        public static double Inner(HyperPoint a, HyperPoint b) => a.X * b.X + a.Z * b.Z - a.T * b.T;

        public static HyperPoint SideNormal(HyperPoint a, HyperPoint b)
            => new(a.Z * b.T - a.T * b.Z, a.T * b.X - a.X * b.T, a.Z * b.X - a.X * b.Z);

        public static HyperPoint Geodesic(HyperPoint a, HyperPoint b, double fraction)
        {
            HyperPoint p = a * (1.0 - fraction) + b * fraction;
            return p * (1.0 / Math.Sqrt(-Inner(p, p)));
        }

        public static HyperPoint operator +(HyperPoint a, HyperPoint b) => new(a.X + b.X, a.Z + b.Z, a.T + b.T);
        public static HyperPoint operator -(HyperPoint a, HyperPoint b) => new(a.X - b.X, a.Z - b.Z, a.T - b.T);
        public static HyperPoint operator *(HyperPoint a, double scale) => new(a.X * scale, a.Z * scale, a.T * scale);
    }

    private readonly struct Tile
    {
        public readonly HyperPoint Center;
        public readonly HyperPoint[] Corners;

        public Tile(HyperPoint center, HyperPoint[] corners) { Center = center; Corners = corners; }
    }

    private readonly struct PoolVertex
    {
        public readonly Vector2 Point, Uv;

        public PoolVertex(Vector2 point, Vector2 uv) { Point = point; Uv = uv; }
        public bool Inside => Point.sqrMagnitude <= DiskCut * DiskCut;
    }

    private sealed class PoolMesh
    {
        private readonly int steps, rowSize;
        private readonly float scale;
        private readonly bool spreadAcrossFloor;
        private readonly List<Vector3> positions = new(45000);
        private readonly List<Vector3> normals = new(45000);
        private readonly List<Vector2> uv = new(45000);
        private readonly List<Color32> colors = new(45000);
        private readonly List<int> indices = new(210000);
        private readonly PoolVertex[] grid;
        private readonly int[] gridIds;
        private readonly PoolVertex[] clip = new PoolVertex[4];

        public PoolMesh(float radius, bool spreadAcrossFloor)
        {
            this.spreadAcrossFloor = spreadAcrossFloor;
            scale = radius / (DiskCut * (spreadAcrossFloor ? Expansion(DiskCut * DiskCut) : 1f));
            steps = spreadAcrossFloor ? FloorSteps : DiagramSteps;
            rowSize = steps + 1;
            grid = new PoolVertex[rowSize * rowSize];
            gridIds = new int[grid.Length];
        }

        // Keep the curved tiles readable around the center while extending the outer rings into the fog.
        private static float Expansion(float radiusSquared)
            => 1f + radiusSquared * radiusSquared / (1f - radiusSquared);

        public void Add(Tile tile)
        {
            HyperPoint[] corners = tile.Corners;
            Color32 color = TileColor(tile.Center);
            for (int z = 0; z <= steps; z++)
            {
                float v = (float)z / steps;
                HyperPoint left = HyperPoint.Geodesic(corners[0], corners[3], v);
                HyperPoint right = HyperPoint.Geodesic(corners[1], corners[2], v);
                for (int x = 0; x <= steps; x++)
                {
                    float u = (float)x / steps;
                    var vertex = new PoolVertex(HyperPoint.Geodesic(left, right, u).Disk, new Vector2(u, v));
                    int i = z * rowSize + x;
                    grid[i] = vertex;
                    gridIds[i] = vertex.Inside ? AddVertex(vertex, color) : -1;
                }
            }

            for (int z = 0; z < steps; z++)
            for (int x = 0; x < steps; x++)
            {
                int a = z * rowSize + x;
                int b = a + rowSize;
                AddTriangle(a, b, a + 1, color);
                AddTriangle(a + 1, b, b + 1, color);
            }
        }

        private int AddVertex(PoolVertex vertex, Color32 color)
        {
            int index = positions.Count;
            float radialScale = spreadAcrossFloor ? Expansion(vertex.Point.sqrMagnitude) * scale : scale;
            positions.Add(new Vector3(vertex.Point.x * radialScale, 0f, vertex.Point.y * radialScale));
            normals.Add(Up);
            uv.Add(vertex.Uv);
            if (!spreadAcrossFloor)
            {
                float rim = Mathf.InverseLerp(FadeStart, DiskCut, vertex.Point.magnitude);
                color.a = (byte)Mathf.RoundToInt(255f * (1f - Mathf.SmoothStep(0f, 1f, rim)));
            }
            colors.Add(color);
            return index;
        }

        private void AddTriangle(int a, int b, int c, Color32 color)
        {
            int first = gridIds[a], second = gridIds[b], third = gridIds[c];
            if (first >= 0 && second >= 0 && third >= 0)
            {
                AddIndices(first, second, third);
                return;
            }
            if (first < 0 && second < 0 && third < 0)
                return;

            int count = 0;
            AddClipEdge(grid[c], grid[a], ref count);
            AddClipEdge(grid[a], grid[b], ref count);
            AddClipEdge(grid[b], grid[c], ref count);
            int start = positions.Count;
            for (int i = 0; i < count; i++)
                AddVertex(clip[i], color);
            for (int i = 2; i < count; i++)
                AddIndices(start, start + i - 1, start + i);
        }

        private void AddClipEdge(PoolVertex from, PoolVertex to, ref int count)
        {
            bool fromInside = from.Inside, toInside = to.Inside;
            if (fromInside != toInside)
            {
                Vector2 delta = to.Point - from.Point;
                float a = delta.sqrMagnitude;
                float b = Vector2.Dot(from.Point, delta);
                float c = from.Point.sqrMagnitude - DiskCut * DiskCut;
                float root = Mathf.Sqrt(Mathf.Max(0f, b * b - a * c));
                float t = Mathf.Clamp01((-b + (fromInside ? root : -root)) / a);
                clip[count++] = new PoolVertex(Vector2.Lerp(from.Point, to.Point, t), Vector2.Lerp(from.Uv, to.Uv, t));
            }
            if (toInside)
                clip[count++] = to;
        }

        private void AddIndices(int a, int b, int c)
        {
            indices.Add(a);
            indices.Add(b);
            indices.Add(c);
        }

        public Mesh Create()
        {
            if (!spreadAcrossFloor)
                AddDiagramBackground();
            var mesh = new Mesh { name = spreadAcrossFloor ? "Poincare pool floor {4,5}" : "Poincare disk {4,5}" };
            if (positions.Count > ushort.MaxValue)
                mesh.indexFormat = IndexFormat.UInt32;
            mesh.SetVertices(positions);
            mesh.SetNormals(normals);
            mesh.SetUVs(0, uv);
            mesh.SetColors(colors);
            mesh.SetTriangles(indices, 0);
            return mesh;
        }

        private void AddDiagramBackground()
        {
            float halfSize = Mathf.Max(174f, scale * DiskCut);
            int background = positions.Count;
            positions.Add(new Vector3(-halfSize, -0.035f, -halfSize));
            positions.Add(new Vector3(halfSize, -0.035f, -halfSize));
            positions.Add(new Vector3(-halfSize, -0.035f, halfSize));
            positions.Add(new Vector3(halfSize, -0.035f, halfSize));
            for (int i = 0; i < 4; i++)
            {
                normals.Add(Up);
                uv.Add(new Vector2(-1f, -1f));
                colors.Add(new Color32(0, 0, 0, 0));
            }
            AddIndices(background, background + 2, background + 1);
            AddIndices(background + 1, background + 2, background + 3);
        }
    }
}
