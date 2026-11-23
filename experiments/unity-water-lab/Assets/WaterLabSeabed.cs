using System;
using UnityEngine;
using UnityEngine.Rendering;

public enum SeabedProfile { Pool, Sand, Rock, Mud, Poincare, Spectre }

public sealed class WaterLabSeabed : IDisposable
{
    public const float HalfSize = 174f;
    public const int Resolution = 225;
    public const float MinimumRelief = -13.5f;
    public const float MaximumRelief = 3.7f;
    public const float MudMinimumRelief = -26f;
    public const float PoincareRadius = 142f;
    private const int RockResolution = 289;
    private const int MudResolution = 321;
    private const float SurfaceClearance = 0.70f;
    private static readonly int SurfaceNoise = Shader.PropertyToID("_SeabedNoise");
    private readonly Mesh[] meshes = new Mesh[8];
    private readonly MaterialPropertyBlock properties = new();

    private GameObject floor;
    private MeshFilter filter;
    private MeshRenderer surface;
    private float appliedNoise = -1f;

    public WaterLabSeabed(Transform parent, Material material)
    {
        if (material == null)
            throw new ArgumentNullException(nameof(material));

        floor = new GameObject("Sand and clay seabed", typeof(MeshFilter), typeof(MeshRenderer));
        floor.transform.SetParent(parent, true);
        filter = floor.GetComponent<MeshFilter>();
        surface = floor.GetComponent<MeshRenderer>();
        surface.sharedMaterial = material;
        surface.shadowCastingMode = ShadowCastingMode.Off;
        SetView(true, 5f);
    }

    public void SetView(bool visible, float depth)
        => SetView(visible, depth, SeabedProfile.Sand, 0.65f);

    public void SetView(bool visible, float depth, SeabedProfile profile, float noise,
        bool coarseSpectre = false, bool poincareDiagram = false)
    {
        if (floor == null)
            return;

        float belowWater = Mathf.Max(MaximumRelief + SurfaceClearance, depth);
        floor.transform.position = new Vector3(0f, -belowWater, 0f);
        if (visible)
        {
            int index = (int)profile;
            if ((uint)index > (int)SeabedProfile.Spectre)
                throw new ArgumentOutOfRangeException(nameof(profile));
            if (profile == SeabedProfile.Spectre && coarseSpectre)
                index = 6;
            if (profile == SeabedProfile.Poincare && poincareDiagram)
                index = 7;
            Mesh selected = meshes[index] ??= BuildMesh(profile, coarseSpectre, poincareDiagram);
            if (filter.sharedMesh != selected)
                filter.sharedMesh = selected;

            float amount = Mathf.Clamp01(noise);
            if (amount != appliedNoise)
            {
                surface.GetPropertyBlock(properties);
                properties.SetFloat(SurfaceNoise, amount);
                surface.SetPropertyBlock(properties);
                appliedNoise = amount;
            }
        }
        floor.SetActive(visible);
    }

    /// <summary>Local relief at a world XZ position, before subtracting the selected depth.</summary>
    public static float SampleHeight(Vector2 position)
        => SampleHeight(position, SeabedProfile.Sand);

    public static float SampleHeight(Vector2 position, SeabedProfile profile)
        => profile switch
        {
            SeabedProfile.Pool => 0f,
            SeabedProfile.Sand => SandHeight(position),
            SeabedProfile.Rock => RockHeight(position),
            SeabedProfile.Mud => MudHeight(position),
            _ => throw new ArgumentOutOfRangeException(nameof(profile)),
        };

    private static float SandHeight(Vector2 position)
    {
        // Fixed offsets identify each layer and keep the relief independent of Unity's random state.
        float warpX = Noise(position, 0.014f, 31.7f, 82.1f);
        float warpZ = Noise(position, 0.018f, 118.4f, 51.8f);
        Vector2 curved = position + 31f * new Vector2(warpX - 0.5f, warpZ - 0.5f);
        float basin = Noise(curved, 0.011f, 203.6f, 94.7f);
        float bend = Noise(curved, 0.031f, 67.7f, 147.9f) - 0.5f;
        float phase = 0.225f * Vector2.Dot(curved, new Vector2(0.81f, 0.59f)) + 2.4f * bend + 0.8f;
        float ridge = 0.5f + 0.5f * Mathf.Cos(phase);

        float shelf = Mathf.Lerp(-4.2f, -0.6f, Noise(curved, 0.015f, 184.9f, 17.2f));
        float crest = 4.7f * Mathf.Pow(ridge, 1.35f);
        float channel = -8.4f * Mathf.Pow(1f - ridge, 2.05f) * Mathf.Lerp(0.80f, 1.32f, basin);
        float hummocks = 2.2f * (Noise(position, 0.061f, 10.2f, 74.3f) - 0.5f);
        float smallRelief = 0.55f * (Noise(position, 0.134f, 91.5f, 50.4f) - 0.5f);
        float ridgeTop = Mathf.SmoothStep(0f, 1f, Mathf.InverseLerp(0.66f, 0.95f, ridge));
        float sandFlat = 3.25f + hummocks * 0.33f;
        float relief = Mathf.Lerp(shelf + crest + channel + hummocks, sandFlat, ridgeTop * 0.82f);
        return Mathf.Clamp(relief + smallRelief, MinimumRelief, MaximumRelief);
    }

    private static float RockHeight(Vector2 position)
    {
        var warp = new Vector2(Noise(position, 0.019f, 15.7f, 133.2f), Noise(position, 0.016f, 89.2f, 61.4f));
        Vector2 curved = position + (warp - Vector2.one * 0.5f) * 24f;
        float foundation = 0.74f * Noise(curved, 0.025f, 93.1f, 37.8f)
            + 0.26f * Noise(curved, 0.065f, 181.6f, 51.7f);
        float levels = foundation * 5f;
        float plateau = Mathf.Floor(levels);
        float terrace = (plateau + Ease(0.30f, 0.62f, levels - plateau)) / 5f;

        float outcrops = BoulderField(curved, 19f, 0x77a41c32u);
        float boulders = BoulderField(curved, 8f, 0x5367ab2fu);
        float fragments = BoulderField(position, 4.5f, 0x2953d1a9u);
        float cracks = 1.2f * (Noise(position, 0.123f, 140.1f, 22.7f) - 0.5f);
        float height = -8.9f + terrace * 7.7f + outcrops * 5.2f + boulders * 3.1f + fragments * 1.4f + cracks;
        return Mathf.Clamp(height, MinimumRelief, MaximumRelief);
    }

    private static float MudHeight(Vector2 position)
    {
        var warp = new Vector2(Noise(position, 0.017f, 177.2f, 105.1f), Noise(position, 0.012f, 227.4f, 31.4f));
        Vector2 curved = position + (warp - Vector2.one * 0.5f) * 39f;
        float sediment = 0.61f * Noise(curved, 0.025f, 112.1f, 191.4f)
            + 0.28f * Noise(curved, 0.053f, 63.8f, 25.6f)
            + 0.11f * Noise(position, 0.112f, 17.7f, 207.9f);

        float pools = BoulderField(curved, 29f, 0xf2b6c953u);
        float pockets = BoulderField(curved, 13f, 0xca798c2bu);
        // A skewed local hollow remains visible from the laboratory's default orbit.
        Vector2 nearby = position + (warp - Vector2.one * 0.5f) * 6f - new Vector2(5f, 1f);
        float hollowRadius = new Vector2((nearby.x + nearby.y * 0.34f) / 12f, nearby.y / 7f).magnitude;
        float hollow = 1f - Ease(0.18f, 1f, hollowRadius);
        float low = Mathf.Max(20f * pools, 21f * hollow) + 5f * pockets;
        return Mathf.Clamp(1.6f + 8f * (sediment - 0.5f) - low, MudMinimumRelief, MaximumRelief);
    }

    private static float BoulderField(Vector2 point, float cellSize, uint seed)
    {
        Vector2 cells = point / cellSize;
        int cellX = Mathf.FloorToInt(cells.x);
        int cellZ = Mathf.FloorToInt(cells.y);
        float height = 0f;
        for (int z = cellZ - 1; z <= cellZ + 1; z++)
        for (int x = cellX - 1; x <= cellX + 1; x++)
        {
            uint bits = HashCell(x, z, seed);
            var center = new Vector2(x + 0.11f + 0.78f * ByteUnit(bits), z + 0.11f + 0.78f * ByteUnit(bits >> 8));
            float radius = 0.20f + 0.37f * ByteUnit(bits >> 16);
            float weight = 0.52f + 0.48f * ByteUnit(bits >> 24);
            float distance = (cells - center).sqrMagnitude / (radius * radius);
            height = Mathf.Max(height, weight * (1f - Ease(0.24f, 1f, distance)));
        }
        return height;
    }

    private static uint HashCell(int x, int z, uint seed)
    {
        unchecked
        {
            uint bits = (uint)x * 0x8da6b343u ^ (uint)z * 0xd8163841u ^ seed;
            bits = (bits ^ (bits >> 16)) * 0x7feb352du;
            bits = (bits ^ (bits >> 15)) * 0x846ca68bu;
            return bits ^ (bits >> 16);
        }
    }

    private static float ByteUnit(uint bits) => (bits & 255u) * (1f / 255f);

    private static float Ease(float start, float end, float value)
    {
        float progress = Mathf.InverseLerp(start, end, value);
        return progress * progress * (3f - 2f * progress);
    }

    private static float Noise(Vector2 point, float frequency, float offsetX, float offsetZ)
        => Mathf.Clamp01(Mathf.PerlinNoise(point.x * frequency + offsetX, point.y * frequency + offsetZ));

    private static Mesh BuildMesh(SeabedProfile profile, bool coarseSpectre, bool poincareDiagram)
    {
        if (profile == SeabedProfile.Poincare)
        {
            // The expanded pattern covers even the corners of the square seabed.
            float radius = poincareDiagram ? PoincareRadius : HalfSize * 1.52f;
            return PoincarePool.Build(radius, spreadAcrossFloor: !poincareDiagram);
        }

        var spectre = profile == SeabedProfile.Spectre ? new SpectreSeabed(HalfSize, coarseSpectre ? 9f : 3.6f) : null;
        int resolution = profile switch
        {
            SeabedProfile.Pool => 2,
            SeabedProfile.Rock => RockResolution,
            SeabedProfile.Mud => MudResolution,
            SeabedProfile.Spectre => coarseSpectre ? 385 : 641,
            _ => Resolution
        };
        var vertices = new Vector3[resolution * resolution];
        var colors = spectre != null ? new Color32[vertices.Length] : null;
        int cells = resolution - 1;
        float step = HalfSize * 2f / cells;
        var triangles = new int[cells * cells * 6];
        for (int z = 0; z < resolution; z++)
        for (int x = 0; x < resolution; x++)
        {
            var worldXZ = new Vector2(x == cells ? HalfSize : x * step - HalfSize,
                z == cells ? HalfSize : z * step - HalfSize);
            int i = z * resolution + x;
            float height;
            if (spectre != null)
            {
                if (!spectre.TrySample(worldXZ, out height, out float tone))
                    throw new InvalidOperationException($"The Spectre tiling did not cover {worldXZ}.");
                height = Mathf.Clamp(height, MudMinimumRelief, MaximumRelief);
                colors[i] = new Color32((byte)Mathf.RoundToInt(Mathf.Clamp01(tone) * 255f), 0, 0, 255);
            }
            else
                height = SampleHeight(worldXZ, profile);
            vertices[i] = new Vector3(worldXZ.x, height, worldXZ.y);
        }

        for (int z = 0; z < cells; z++)
        for (int x = 0; x < cells; x++)
        {
            int a = z * resolution + x;
            int b = a + resolution;
            int t = (z * cells + x) * 6;
            triangles[t] = a;
            triangles[t + 1] = b;
            triangles[t + 2] = a + 1;
            triangles[t + 3] = a + 1;
            triangles[t + 4] = b;
            triangles[t + 5] = b + 1;
        }

        var relief = new Mesh
        {
            name = $"Water Lab {profile} seabed",
            indexFormat = vertices.Length > ushort.MaxValue ? IndexFormat.UInt32 : IndexFormat.UInt16,
            vertices = vertices,
            triangles = triangles,
        };
        if (colors != null)
            relief.colors32 = colors;
        relief.RecalculateNormals();
        relief.RecalculateBounds();
        return relief;
    }

    public void Dispose()
    {
        Destroy(floor);
        foreach (Mesh profileMesh in meshes)
            Destroy(profileMesh);
        Array.Clear(meshes, 0, meshes.Length);
        floor = null;
    }

    private static void Destroy(UnityEngine.Object asset)
    {
        if (asset == null)
            return;
        if (Application.isPlaying)
            UnityEngine.Object.Destroy(asset);
        else
            UnityEngine.Object.DestroyImmediate(asset);
    }
}
