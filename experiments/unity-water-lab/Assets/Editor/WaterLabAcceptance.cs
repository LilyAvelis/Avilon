using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using UnityEditor;
using UnityEngine;
using UnityEngine.Rendering;

public static class WaterLabAcceptance
{
    [MenuItem("Mira/Check Water Laboratory")]
    public static void Run()
    {
        CheckGpuWater();
        CheckWaveQualityChange();
        CheckSeabed();
        CheckRain();
        CheckRainGrid();
        CheckNodeEventSchedule();
        CheckStormPhysics();
        CheckDropSizes();
        CheckGpuStorm();
        CheckRainRendering();
        CheckRefraction();
        Debug.Log("MIRA WATER LAB CHECKS OK: surface, rain, refraction");
    }

    public static void BuildAndVerify()
    {
        Run();
        const string scene = "Assets/Water Laboratory.unity";
        string[] paths = AssetDatabase.GetDependencies(scene)
            .Where(path => path == scene || path.EndsWith(".mat", StringComparison.Ordinal)).ToArray();
        Require(paths.Contains(scene), "saved scene missing from build check");
        using var sha = SHA256.Create();
        byte[][] before = paths.Select(path => sha.ComputeHash(File.ReadAllBytes(path))).ToArray();
        WaterLabBuild.Build();
        for (int i = 0; i < paths.Length; i++)
            Require(before[i].SequenceEqual(sha.ComputeHash(File.ReadAllBytes(paths[i]))), "build changed " + paths[i]);
        Debug.Log("MIRA WATER LAB BUILD PRESERVED SCENE AND MATERIALS");
    }

    private static void CheckGpuWater()
    {
        using var waves = new WaterLabGpuRain(118f);
        Vector2 near = Vector2.zero;
        Vector2 beyondOldBox = new(27.4f, 0f);
        Vector2 nearFog = new(-81f, 61f);
        Vector2 outsideFog = new(119f, 0f);
        Require(!waves.IsActive, "still water dispatched an active simulation");
        waves.Add(outsideFog, 1f, 4f);
        Require(!waves.IsActive, "impulse beyond fog activated the field");
        foreach (Vector2 point in new[] { near, beyondOldBox, nearFog })
            waves.Add(point, 1.1f, 4f);
        waves.Step(0.02f, 4.1f, 0.65f);
        foreach (Vector2 point in new[] { near, beyondOldBox, nearFog })
        {
            Vector2[] region = ReadWaveRegion(waves, point, 25);
            Require(region.Min(value => value.x) < -0.02f && region.Max(value => value.x) > 0.004f,
                "missing GPU depression or rim at " + point);
            Require(Mathf.Abs(region.Sum(value => value.x)) < 0.002f,
                "GPU impulse added water across the impacted region at " + point);
        }

        for (int step = 0; step < 19; step++)
            waves.Step(0.02f, 4.1f, 0.65f);
        Vector2[] traveled = ReadWaveRegion(waves, beyondOldBox, 33);
        Require(traveled.Where((_, i) => Mathf.Abs(i % 33 - 16) >= 10)
            .Any(value => Mathf.Abs(value.x) > 0.0003f), "far click did not propagate through the GPU field");

        waves.Clear();
        Require(!waves.IsActive && ReadWaveRegion(waves, nearFog, 5).All(value => value == Vector2.zero),
            "reset left a wave in the distant field");
        Debug.Log("GPU wave checks: center, 27 m, 101 m, propagation and reset");
    }

    private static Vector2[] ReadWaveRegion(WaterLabGpuRain waves, Vector2 position, int size)
    {
        float factor = (waves.GridResolution - 1) / (waves.Radius * 2f);
        int x = Mathf.RoundToInt((position.x + waves.Radius) * factor) - size / 2;
        int z = Mathf.RoundToInt((position.y + waves.Radius) * factor) - size / 2;
        var request = AsyncGPUReadback.Request(waves.HeightTexture, 0, x, size, z, size, 0, 1);
        request.WaitForCompletion();
        Require(!request.hasError, "GPU wave readback failed");
        return request.GetData<Vector2>().ToArray();
    }

    private static void CheckSeabed()
    {
        var shader = Resources.Load<Shader>("Mira/WaterLabSeabed");
        Require(shader != null && shader.isSupported, "procedural seabed shader is missing");
        var parent = new GameObject("Seabed test root");
        var material = new Material(shader);
        try
        {
            using var seabed = new WaterLabSeabed(parent.transform, material);
            var surface = parent.GetComponentInChildren<MeshFilter>();
            Mesh mesh = surface.sharedMesh;
            Require(mesh.bounds.extents.x >= 118f && mesh.bounds.extents.z >= 118f,
                "procedural seabed did not reach the fog");
            Require(mesh.bounds.size.y > 12f && mesh.bounds.max.y < 4.0f,
                $"seabed relief/bounds: min {mesh.bounds.min.y:F2}, max {mesh.bounds.max.y:F2}");
            foreach (int depth in new[] { 5, 10, 20 })
            {
                seabed.SetView(true, depth);
                Near(surface.transform.position.y, -depth, "selected seabed depth");
                Require(surface.sharedMesh == mesh, "changing depth rebuilt the procedural mesh");
            }
            seabed.SetView(true, 5f, SeabedProfile.Pool, 0.65f);
            Mesh tile = surface.sharedMesh;
            Require(tile != mesh && tile.bounds.size.y == 0f, "pool tile did not use a flat profile");
            seabed.SetView(true, 5f, SeabedProfile.Rock, 0.65f);
            Require(surface.sharedMesh != mesh && surface.sharedMesh.bounds.size.y > 10f,
                "rock profile did not have a separate relief");
            seabed.SetView(true, 5f, SeabedProfile.Mud, 0.65f);
            Mesh mud = surface.sharedMesh;
            Require(mud.bounds.size.y > mesh.bounds.size.y + 5f && mud.bounds.max.y < 4f,
                "mud did not keep its deeper depressions below water");
            seabed.SetView(true, 20f, SeabedProfile.Mud, 1f);
            Require(surface.sharedMesh == mud, "mud noise or depth rebuilt the relief");
            Near(surface.transform.position.y, -20f, "mud selected depth");
            seabed.SetView(true, 5f, SeabedProfile.Pool, 0f);
            Require(surface.sharedMesh == tile, "returning to the tile did not reuse the original mesh");
            seabed.SetView(true, 5f, SeabedProfile.Poincare, 0.65f);
            Mesh parquet = surface.sharedMesh;
            Require(parquet.bounds.size.y == 0f && parquet.bounds.extents.x > WaterLabSeabed.HalfSize
                && parquet.colors32.All(color => color.a == 255), "Poincare floor fades before the surrounding water");
            var nearbyGlazes = new HashSet<Color32>();
            Vector3[] parquetVertices = parquet.vertices;
            Color32[] parquetColors = parquet.colors32;
            for (int i = 0; i < parquetVertices.Length; i++)
                if (parquetVertices[i].sqrMagnitude < 100f)
                    nearbyGlazes.Add(parquetColors[i]);
            Require(nearbyGlazes.Count >= 5, "Poincare floor tiles are too large for the regular orbit");
            var floorProbe = surface.gameObject.AddComponent<MeshCollider>();
            try
            {
                foreach (float x in new[] { -170f, 0f, 170f })
                foreach (float z in new[] { -170f, 0f, 170f })
                    Require(floorProbe.Raycast(new Ray(new Vector3(x, 1f, z), Vector3.down), out _, 10f),
                        $"Poincare floor is missing under ({x}, {z})");
            }
            finally
            {
                UnityEngine.Object.DestroyImmediate(floorProbe);
            }
            seabed.SetView(true, 5f, SeabedProfile.Poincare, 0.65f, poincareDiagram: true);
            Mesh disc = surface.sharedMesh;
            Require(disc != parquet && disc.vertexCount > 1000 && disc.bounds.size.y < 0.05f,
                "Poincare diagram did not use its own flat tessellation");
            Color32[] ceramic = disc.colors32;
            Require(ceramic.Any(color => color.a == 0) && ceramic.Where(color => color.a == 255)
                .Select(color => (color.r, color.g, color.b)).Distinct().Count() >= 5,
                "Poincare ceramic palette or radial fade is missing");
            seabed.SetView(true, 10f, SeabedProfile.Poincare, 0.65f);
            Require(surface.sharedMesh == parquet, "returning from the disk rebuilt the Poincare floor");
            seabed.SetView(true, 5f, SeabedProfile.Spectre, 0.65f);
            Mesh fine = surface.sharedMesh;
            Require(fine.bounds.size.y > 3f && fine.bounds.max.y < 4f && fine.bounds.extents.x >= 118f,
                "fine Spectre relief does not cover the pond safely");
            seabed.SetView(true, 5f, SeabedProfile.Spectre, 0.65f, coarseSpectre: true);
            Mesh coarse = surface.sharedMesh;
            Require(coarse != fine && coarse.bounds.size.y > 3f && coarse.bounds.max.y < 4f,
                "coarse Spectre relief is missing");
            seabed.SetView(true, 20f, SeabedProfile.Spectre, 0.9f);
            Require(surface.sharedMesh == fine, "changing Spectre depth or grain rebuilt the fine profile");
            Debug.Log($"Experimental seabeds: {disc.vertexCount} Poincare vertices; " +
                $"Spectre relief {fine.bounds.size.y:F2} m fine, {coarse.bounds.size.y:F2} m coarse");
            seabed.SetView(false, 5);
            Require(!surface.gameObject.activeSelf, "procedural seabed remained visible in control mode");
            Debug.Log($"Procedural seabed: {mesh.vertexCount} vertices, {mesh.bounds.size.y:F2} m relief");
        }
        finally
        {
            UnityEngine.Object.DestroyImmediate(parent);
            UnityEngine.Object.DestroyImmediate(material);
        }
    }

    private static void CheckWaveQualityChange()
    {
        using var original = new WaterLabGpuRain(118f, 1024);
        using var upgraded = new WaterLabGpuRain(118f, 2048);
        Vector2 point = new(61.25f, 34.75f);
        Vector2 queuedPoint = new(-74f, -28f);
        original.Add(point, 1.5f, 4f);
        original.Step(0.02f, 4.1f, 0.65f);
        original.Add(queuedPoint, 1.1f, 4f);
        float before = ReadWaveRegion(original, point, 13).Min(sample => sample.x);
        upgraded.RestoreFrom(original);
        float after = ReadWaveRegion(upgraded, point, 25).Min(sample => sample.x);
        Require(upgraded.IsActive && before < -0.02f && Mathf.Abs(before - after) < 0.02f,
            "quality change reset or moved the existing GPU wave");
        upgraded.Step(0.02f, 4.1f, 0.65f);
        Require(ReadWaveRegion(upgraded, queuedPoint, 25).Min(sample => sample.x) < -0.02f,
            "quality change dropped a queued impact");
    }

    private static void CheckDropSizes()
    {
        var rain = new WaterLabRain(12f, seed: 27);
        rain.Reset(0, 80f, 0.24f, 0.055f);
        rain.Step(30, 80f, 0.24f, 0.055f);
        var sizes = new List<float>();
        while (rain.TryLand(30, out var drop))
            sizes.Add(drop.Diameter);
        Require(sizes.Count > 1000 && sizes.All(diameter => diameter >= 0.006f && diameter <= 0.40f),
            "hail exceeded the permitted diameter range");
        Require(Mathf.Abs(sizes.Average() - 0.24f) < 0.006f, "hail no longer follows the selected mean diameter");
        Require(sizes.Any(value => value < 0.18f) && sizes.Any(value => value > 0.30f), "hail no longer has size variation");
        Require(WaterLabRain.ImpactImpulse(0.40f) > 5f * WaterLabRain.ImpactImpulse(0.015f),
            "diameter no longer affects the physical impulse");
    }

    private static void CheckRain()
    {
        int frequent = CountLandings(90);
        int rare = CountLandings(10);
        Require(frequent == rare, $"polling frequency changed landings: {frequent} versus {rare}");
        Require(Mathf.Abs(frequent / 180f - 8f) < 0.6f, $"rain missed its 8/s rate: {frequent} in 180 s");

        var rain = new WaterLabRain(Mathf.Sqrt(150f / Mathf.PI), seed: 42);
        rain.Reset(0, 8f);
        rain.Step(3, 8f);
        int landed = 0;
        while (rain.TryLand(3, out _))
            landed++;
        Require(landed > 8, "catch-up lost physical impacts beyond the eight visual slots");
        rain.Step(10, 0f);
        Require(rain.Falling.Count == 0 && !rain.TryLand(10, out _), "rain off left pending drops");
        rain.Step(100, 8f);
        Require(!rain.TryLand(100, out _), "turning rain back on replayed the disabled interval");
        Debug.Log($"Rain at 8/s: {frequent} landings in 180 seconds with both polling rates");
    }

    private static int CountLandings(int ticksPerSecond)
    {
        var rain = new WaterLabRain(Mathf.Sqrt(150f / Mathf.PI), seed: 42);
        rain.Reset(0, 8f);
        int count = 0;
        for (int tick = 1; tick <= 180 * ticksPerSecond; tick++)
        {
            double now = (double)tick / ticksPerSecond;
            rain.Step(now, 8f);
            while (rain.TryLand(now, out _))
                count++;
        }
        return count;
    }

    private static void CheckRainGrid()
    {
        const float radius = 118f;
        float light = WaterLabRain.GridStep(0.5f);
        float heavy = WaterLabRain.GridStep(8f);
        Require(light > heavy, "the grid must tighten as rain intensifies");

        foreach (float intensity in new[] { 0.5f, 8f })
        {
            var rain = new WaterLabRain(radius, seed: 42);
            rain.Reset(0, intensity);
            rain.Step(10, intensity);
            float step = WaterLabRain.GridStep(intensity);
            Vector2[] nodes = WaterLabRain.CacheRainNodes(radius, step);
            var cached = new HashSet<Vector2>(nodes);
            Require(nodes.Any(p => Mathf.Abs(p.x / step - Mathf.Round(p.x / step)) > 0.05f),
                "transformation left the whole lattice axis-aligned");
            int outerQuadrants = 0;
            int nearCenter = 0;
            int count = 0;
            while (rain.TryLand(10, out var drop))
            {
                Vector2 p = drop.Point;
                Require(p.sqrMagnitude <= radius * radius, "rain landed past the fog boundary");
                Require(cached.Contains(p), "rain missed a cached transformed node");
                if (p.sqrMagnitude > 100f * 100f)
                    outerQuadrants |= 1 << ((p.x < 0 ? 1 : 0) + (p.y < 0 ? 2 : 0));
                if (Mathf.Abs(p.x) < 6f && Mathf.Abs(p.y) < 6f)
                    nearCenter++;
                count++;
            }
            Require(outerQuadrants == 15, "rain did not reach the fog in all four directions");
            Require(nearCenter >= 2 * intensity, "widening rain starved the central water");
            Debug.Log($"Rain grid at {intensity}/150 m²/s: {count} drops in 10 s, {nearCenter} at the center, {step:F2} m spacing");
        }
    }

    private static void CheckNodeEventSchedule()
    {
        int calls = 0;
        float[] points = NodeEventSchedule.CacheNodes(new[] { 0, 1, 2 }, node =>
        {
            calls++;
            return 3f + node * node * 0.5f;
        });
        Require(calls == 3 && points.SequenceEqual(new[] { 3f, 3.5f, 5f }), "node transformation was not cached");

        var regular = new WaterLabRain(12f, seed: 97);
        var catchUp = new WaterLabRain(12f, seed: 97);
        regular.Reset(0, 4f);
        catchUp.Reset(0, 4f);
        var expected = new List<WaterLabRain.Drop>();
        for (int tick = 0; tick <= 30; tick++)
        {
            double now = tick / 6.0;
            regular.Step(now, 4f);
            while (regular.TryLand(now, out var drop))
                expected.Add(drop);
        }
        Require(expected.Count > 10, "node window test did not schedule enough impacts");
        catchUp.Step(5, 4f);
        foreach (var drop in expected)
            Require(catchUp.TryLand(5, out var replay) && replay.Point.Equals(drop.Point) &&
                replay.ImpactTime == drop.ImpactTime,
                "catch-up changed the seeded sequence of transformed nodes and times");
        Require(!catchUp.TryLand(5, out _), "catch-up emitted an extra impact at a window boundary");

        var plan = new List<NodeEventSchedule.Event>();
        NodeEventSchedule.PlanWindow(97, 0, 1, 8, 0, plan);
        Require(plan.Count == 0, "the scheduler chose a node from an empty cache");
        CheckWindowContract(plan);
        CheckRainSettings();
    }

    private static void CheckWindowContract(List<NodeEventSchedule.Event> plan)
    {
        NodeEventSchedule.PlanWindow(42, 3, 0.25, 100, 17, plan);
        var original = plan.ToArray();
        int[] nodes = { 10, 1, 12, 9 };
        ulong[] keys = { 0x8cd164277c74fe46UL, 0xeb19a18e8d1c5f06UL, 0x0159421fdd265430UL, 0x3b14d76f02035f22UL };
        double[] times = { 0.7711873370621616, 0.7798945259812462, 0.7958801263777859, 0.7977931797318631 };
        Require(plan.Count == 21, "portable schedule count changed");
        for (int i = 0; i < keys.Length; i++)
            Require(plan[i].Key == keys[i] && plan[i].Node == nodes[i] && Math.Abs(plan[i].Time - times[i]) < 1e-14,
                $"portable schedule mismatch at event {i}");
        Require(NodeEventSchedule.Unit(keys[0], 0) == 0.12768396438538765, "portable payload channel changed");
        NodeEventSchedule.PlanWindow(42, -2, 0.25, 100, 17, plan);
        Require(plan.All(e => e.Time >= -0.5 && e.Time < -0.25), "event outside negative window");
        NodeEventSchedule.PlanWindow(42, 3, 0.25, 100, 17, plan, original.Length);
        Require(plan.SequenceEqual(original), "requesting another window changed random access");

        RequireRejected<InvalidOperationException>(() => NodeEventSchedule.PlanWindow(42, 3, 0.25, 100, 17, plan, 20));
        Require(plan.Count == 0, "failed event limit returned a partial window");
        foreach (double invalid in new[] { double.NaN, double.PositiveInfinity, -1.0 })
        {
            plan.Add(original[0]);
            RequireRejected<ArgumentOutOfRangeException>(() => NodeEventSchedule.PlanWindow(42, 3, invalid, 100, 17, plan));
            Require(plan.Count == 0, "invalid duration left stale events");
            RequireRejected<ArgumentOutOfRangeException>(() => NodeEventSchedule.PlanWindow(42, 3, 0.25, invalid, 17, plan));
        }
        RequireRejected<ArgumentOutOfRangeException>(() => NodeEventSchedule.PlanWindow(42, long.MaxValue, 1, 8, 17, plan));
    }

    private static void RequireRejected<T>(Action action) where T : Exception
    {
        try { action(); }
        catch (T) { return; }
        throw new InvalidOperationException($"Expected {typeof(T).Name}");
    }

    private static void CheckRainSettings()
    {
        var regular = new WaterLabRain(12, seed: 97);
        var catchUp = new WaterLabRain(12, seed: 97);
        foreach (var rain in new[] { regular, catchUp })
        {
            rain.Reset(0, 8, 0.02f);
            rain.Step(0.25, 24, 0.22f, 0.01f);
        }
        for (int tick = 2; tick <= 9; tick++)
            regular.Step(tick / 4.0, 24, 0.22f, 0.01f);
        foreach (var rain in new[] { regular, catchUp })
            rain.Step(2.5, 10, 0.36f);
        var expected = regular.Falling.ToArray();
        Require(expected.Length > 30 && expected.SequenceEqual(catchUp.Falling),
            "polling changed the density, grid or diameters of previously started windows");
        const double fall = 0.42;
        var first = expected.Where(d => d.ImpactTime < 1 + fall).ToArray();
        var later = expected.Where(d => d.ImpactTime >= 1 + fall).ToArray();
        Require(first.Length > 0 && first.All(d => d.Diameter == 0.02f) &&
            later.Length > 0 && later.All(d => d.Diameter > 0.15f && d.Diameter < 0.30f),
            "an incoming setting changed diameters in an earlier window");
        var restarted = new WaterLabRain(12, seed: 97);
        restarted.Reset(1.25, 24, 0.22f, 0.01f);
        restarted.Step(2.5, 10, 0.36f);
        Require(restarted.Falling.SequenceEqual(expected.Where(d => d.ImpactTime >= 1.25 + fall)),
            "starting mid-window changed its remaining events or their payloads");
    }

    private static void CheckStormPhysics()
    {
        var field = new WaveField(105, 9f);
        int center = (field.Size / 2) * (field.Size + 1);
        field.StrikeDrop(Vector2.zero, 1f);
        field.Step(0.01f, 4.1f, 0.65f);
        Require(field.AtVertex(center) < -0.005f && field.AtVertex(center + 1) > 0.001f,
            "drop impact did not dent the surface and push water outward");
        float netHeight = 0f;
        for (int i = 0; i < field.Size * field.Size; i++)
            netHeight += field.AtVertex(i);
        Near(netHeight, 0f, "drop impulse added or removed water");

        float before = field.AtVertex(center + 2);
        for (int step = 0; step < 3; step++)
            field.Step(0.01f, 4.1f, 0.65f);
        Require(field.AtVertex(center + 2) > before + 0.0001f, "the physical wave did not propagate beyond the impact stencil");

        field.Clear();
        var storm = new WaterLabRain(118f, seed: 42);
        float intensity = WaterLabRain.MaximumIntensity;
        storm.Reset(0, intensity);
        int hits = 0;
        int peakFalling = 0;
        float heightPeak = 0f;
        for (int tick = 0; tick <= 500; tick++)
        {
            double now = tick * 0.02;
            storm.Step(now, intensity);
            while (storm.TryLand(now, out var drop))
            {
                field.StrikeDrop(drop.Point, 1.05f);
                if (Mathf.Max(Mathf.Abs(drop.Point.x), Mathf.Abs(drop.Point.y)) < 8f)
                    hits++;
            }
            peakFalling = Mathf.Max(peakFalling, storm.Falling.Count);
            field.Step(0.01f, 4.1f, 0.65f);
            field.Step(0.01f, 4.1f, 0.65f);
        }

        netHeight = 0f;
        for (int i = 0; i < field.Size * field.Size; i++)
        {
            float height = field.AtVertex(i);
            Require(!float.IsNaN(height) && !float.IsInfinity(height), "the storm destabilized the wave solver");
            netHeight += height;
            heightPeak = Mathf.Max(heightPeak, Mathf.Abs(height));
        }
        Require(hits > 1000 && peakFalling > 5000, "the maximum setting did not sustain a storm");
        Require(heightPeak > 0.005f && heightPeak < 0.15f && Mathf.Abs(netHeight) < field.Size * field.Size * 0.002f,
            "the storm did not yield bounded micro-waves with a stable mean water level");
        Debug.Log($"CPU stencil reference: {hits} central impacts in 10 s, {peakFalling} drops in flight, {heightPeak:F3} m peak displacement");
    }

    private static void CheckGpuStorm()
    {
        using var waves = new WaterLabGpuRain(118f);
        var rain = new WaterLabRain(waves.Radius, seed: 42);
        float rate = WaterLabRain.MaximumIntensity;
        rain.Reset(0, rate);
        for (int tick = 0; tick <= 115; tick++)
        {
            double now = tick * 0.02;
            rain.Step(now, rate, 0.38f, 0.05f);
            while (rain.TryLand(now, out var drop))
                waves.Add(drop.Point, drop.Diameter, WaterLabRain.ImpactImpulse(drop.Diameter));
            waves.Step(0.02f, 4.1f, 0.65f);
        }

        var request = AsyncGPUReadback.Request(waves.HeightTexture, 0);
        request.WaitForCompletion();
        Require(!request.hasError, "GPU storm readback failed");
        var state = request.GetData<Vector2>();
        double sum = 0;
        float peak = 0f;
        for (int i = 0; i < state.Length; i++)
        {
            float h = state[i].x;
            if (float.IsNaN(h) || float.IsInfinity(h))
                throw new InvalidOperationException("GPU storm water became non-finite.");
            sum += h;
            peak = Mathf.Max(peak, Mathf.Abs(h));
        }
        Require(peak > 0.04f && peak < 1.5f, "hail did not yield bounded physical waves");
        Require(Math.Abs(sum / state.Length) < 0.001, "hail moved the mean water level");
        Debug.Log($"GPU hail checks: peak {peak:F3} m, mean {sum / state.Length:F5} m");
    }

    private static void CheckRainRendering()
    {
        var shader = AssetDatabase.LoadAssetAtPath<Shader>("Assets/Editor/WaterLabRainProbe.shader");
        Require(shader != null && shader.isSupported, "rain probe shader unavailable");
        var material = new Material(shader);
        var target = RenderTexture.GetTemporary(1, 1, 0, RenderTextureFormat.ARGBFloat, RenderTextureReadWrite.Linear);
        var readback = new Texture2D(1, 1, TextureFormat.RGBAFloat, false, true);
        var previous = RenderTexture.active;
        try
        {
            using var surface = new WaterLabRainSurface(material, 118f);
            Color At(float x, float z, float time, float footprint = 0.01f)
            {
                material.SetVector("_Probe", new Vector4(x, z, time, footprint));
                Graphics.Blit(Texture2D.blackTexture, target, material);
                RenderTexture.active = target;
                readback.ReadPixels(new Rect(0, 0, 1, 1), 0, 0);
                return readback.GetPixel(0, 0);
            }

            surface.Add(new WaterLabRain.Drop(new Vector2(0f, 0f), 3));
            surface.Add(new WaterLabRain.Drop(new Vector2(74.9f, -9f), 3));
            surface.Add(new WaterLabRain.Drop(new Vector2(74.4f, -9f), 3.1));
            surface.Publish();
            Require(At(0f, 0f, 3.01f).r > 0.5f, "rain touchdown missing at center");
            Require(At(75.1f, -9f, 3.20f, 0.16f).r > 0.1f, "far rain ring missing across a bucket boundary");
            Require(At(75.1f, -9f, 3.08f, 1.2f).r > 0.1f, "far rain vanished at a grazing angle");
            Require(At(74.4f, -9f, 3.11f).r > 0.5f, "second impact in the same bucket missing");
            Near(At(75.1f, -9f, 4.4f).r, 0f, "expired rain ring");
            surface.Clear();
            Near(At(0f, 0f, 3.01f).r, 0f, "rain clear left a GPU ring");

        }
        finally
        {
            RenderTexture.active = previous;
            RenderTexture.ReleaseTemporary(target);
            UnityEngine.Object.DestroyImmediate(readback);
            UnityEngine.Object.DestroyImmediate(material);
        }
    }

    private static void CheckRefraction()
    {
        Require(SystemInfo.graphicsDeviceType != GraphicsDeviceType.Null, "refraction check needs a graphics device");
        var shader = AssetDatabase.LoadAssetAtPath<Shader>("Assets/Editor/WaterLabRefractionProbe.shader");
        Require(shader != null && shader.isSupported, "refraction probe shader unavailable");
        var material = new Material(shader);
        var target = RenderTexture.GetTemporary(16, 16, 16, RenderTextureFormat.ARGBFloat, RenderTextureReadWrite.Linear);
        var readback = new Texture2D(1, 1, TextureFormat.RGBAFloat, false, true);
        var cameraObject = new GameObject("Refraction test camera");
        var quad = GameObject.CreatePrimitive(PrimitiveType.Quad);
        var previous = RenderTexture.active;
        try
        {
            Camera camera = cameraObject.AddComponent<Camera>();
            camera.enabled = false;
            camera.cullingMask = 1 << 31;
            camera.clearFlags = CameraClearFlags.SolidColor;
            camera.targetTexture = target;
            quad.layer = 31;
            quad.GetComponent<Renderer>().sharedMaterial = material;

            Color RenderAt(float yaw)
            {
                Quaternion rotation = Quaternion.Euler(0f, yaw, 0f);
                camera.transform.SetPositionAndRotation(quad.transform.position - rotation * Vector3.forward * 2f, rotation);
                camera.Render();
                RenderTexture.active = target;
                readback.ReadPixels(new Rect(8, 8, 1, 1), 0, 0);
                return readback.GetPixel(0, 0);
            }

            Color front = RenderAt(0f);
            Color back = RenderAt(180f);
            float displacement = 0.25f / Mathf.Sqrt(2f);
            Near(front.r, 0.5f + displacement, "front depth X");
            Near(back.r, 0.5f - displacement, "opposed depth X");
            Near(front.b, front.r, "front grab X");
            Near(back.b, back.r, "opposed grab X");
            Near(Mathf.Abs(front.g - 0.5f), displacement, "depth Y");
            Near(front.a, 0.5f + (SystemInfo.graphicsUVStartsAtTop ? -displacement : displacement), "grab Y");
            Near(back.g, front.g, "opposed depth Y");
            Near(back.a, front.a, "opposed grab Y");
        }
        finally
        {
            RenderTexture.active = previous;
            UnityEngine.Object.DestroyImmediate(cameraObject);
            UnityEngine.Object.DestroyImmediate(quad);
            UnityEngine.Object.DestroyImmediate(readback);
            UnityEngine.Object.DestroyImmediate(material);
            RenderTexture.ReleaseTemporary(target);
        }
    }

    private static void Near(float actual, float expected, string check)
        => Require(Mathf.Abs(actual - expected) < 0.0001f, $"{check}: expected {expected}, got {actual}");

    private static void Require(bool condition, string check)
    {
        if (!condition)
            throw new InvalidOperationException("Water laboratory regression: " + check);
    }
}

