using System;
using System.Collections.Generic;
using UnityEngine;

public sealed class WaterLabRain
{
    public const float ReferenceArea = 150f;
    public const float MaximumIntensity = 80f;
    public const float DropSpeed = 12f;
    public const float DefaultDiameter = 0.018f;
    public const float MaximumDiameter = 0.40f;

    public readonly struct Drop
    {
        public readonly Vector2 Point;
        public readonly double ImpactTime;
        public readonly float Diameter;

        public Drop(Vector2 point, double impactTime, float diameter = DefaultDiameter)
        {
            Point = point;
            ImpactTime = impactTime;
            Diameter = diameter;
        }
    }

    private const double FallDuration = 0.42;
    private const double WindowDuration = 1.0;
    private readonly struct Settings
    {
        public readonly float Rate, Diameter, Deviation;

        public Settings(float rate, float diameter, float deviation)
        {
            if (!(rate >= 0 && rate <= MaximumIntensity))
                throw new ArgumentOutOfRangeException(nameof(rate));
            if (!(diameter > 0) || float.IsInfinity(diameter))
                throw new ArgumentOutOfRangeException(nameof(diameter));
            if (!(deviation >= 0) || float.IsInfinity(deviation))
                throw new ArgumentOutOfRangeException(nameof(deviation));
            Rate = rate;
            Diameter = diameter;
            Deviation = deviation;
        }
    }

    private readonly ulong seed;
    private readonly float radius;
    private readonly double coverage;
    private readonly Queue<Drop> falling = new();
    private readonly List<NodeEventSchedule.Event> plan = new();
    private Vector2[] nodes = Array.Empty<Vector2>();
    private float cachedStep;
    private double enabledAt;
    private long nextWindow;
    private Settings requested, active;
    private int cursor;
    private bool running;

    public WaterLabRain(float radius, int? seed = null)
        : this(radius, seed.HasValue ? unchecked((uint)seed.Value) : BitConverter.ToUInt64(Guid.NewGuid().ToByteArray(), 0)) { }

    public WaterLabRain(float radius, ulong seed)
    {
        if (!(radius > 0) || float.IsInfinity(radius))
            throw new ArgumentOutOfRangeException(nameof(radius));
        this.radius = radius;
        coverage = Math.PI * radius * radius / ReferenceArea;
        this.seed = seed;
    }

    public IReadOnlyCollection<Drop> Falling => falling;
    public float Intensity { get; private set; }

    public void Reset(double now, float dropsPerSecond, float diameter = DefaultDiameter, float deviation = 0f)
    {
        ValidateTime(now);
        Restart(now, new Settings(dropsPerSecond, diameter, deviation));
    }

    private void Restart(double now, Settings settings)
    {
        falling.Clear();
        plan.Clear();
        cursor = 0;
        enabledAt = now;
        nextWindow = (long)Math.Floor(now / WindowDuration);
        requested = settings;
        Intensity = settings.Rate;
        running = settings.Rate > 0f;
    }

    public void Step(double now, float dropsPerSecond, float diameter = DefaultDiameter, float deviation = 0f)
    {
        ValidateTime(now);
        var incoming = new Settings(dropsPerSecond, diameter, deviation);
        if (dropsPerSecond == 0f)
        {
            if (running)
                Restart(now, incoming);
            return;
        }

        if (!running)
            Restart(now, incoming);
        if (now < enabledAt)
            return;

        while (true)
        {
            while (cursor < plan.Count && plan[cursor].Time <= now)
            {
                NodeEventSchedule.Event entry = plan[cursor++];
                falling.Enqueue(new Drop(nodes[entry.Node], entry.Time + FallDuration, SampleDiameter(entry.Key, active)));
            }
            double start = nextWindow * WindowDuration;
            if (now < start)
            {
                requested = incoming;
                return;
            }

            // An update observed after a boundary cannot change the window that already began.
            // A partial first window starts admitting events at the instant rain was enabled.
            active = start == now || now == enabledAt ? incoming : requested;
            float step = GridStep(active.Rate);
            if (cachedStep != step)
            {
                nodes = CacheRainNodes(radius, step);
                cachedStep = step;
            }
            NodeEventSchedule.PlanWindow(seed, nextWindow, WindowDuration, active.Rate * coverage, nodes.Length, plan);
            nextWindow++;
            Intensity = active.Rate;
            cursor = 0;
            while (cursor < plan.Count && plan[cursor].Time < enabledAt)
                cursor++;
        }
    }

    private static void ValidateTime(double now)
    {
        const double exactIndexLimit = 9007199254740992.0;
        if (!(now >= -exactIndexLimit && now < exactIndexLimit))
            throw new ArgumentOutOfRangeException(nameof(now));
    }

    public bool TryLand(double now, out Drop drop)
    {
        if (falling.TryPeek(out drop) && drop.ImpactTime <= now)
        {
            falling.Dequeue();
            return true;
        }
        return false;
    }

    private static float SampleDiameter(ulong key, Settings settings)
    {
        float mean = settings.Diameter;
        if (settings.Deviation > 0f)
        {
            double normal = Math.Sqrt(-2.0 * Math.Log(NodeEventSchedule.Unit(key, 0))) *
                Math.Cos(2.0 * Math.PI * NodeEventSchedule.Unit(key, 1));
            mean += (float)normal * settings.Deviation;
        }
        return Mathf.Clamp(mean, 0.006f, MaximumDiameter);
    }

    public static float ImpactImpulse(float diameter)
    {
        // Equal density means mass grows with diameter cubed. Tone down the
        // extremes so laboratory-sized hail can share a single height field.
        float relativeMass = Mathf.Pow(diameter / MaximumDiameter, 3f);
        return 0.68f + 4.82f * Mathf.Sqrt(relativeMass);
    }

    public static float GridStep(float intensity)
        => intensity <= 8f ? Mathf.Lerp(1.6f, 0.65f, intensity / 8f)
            : Mathf.Lerp(0.65f, 0.55f, Mathf.InverseLerp(8f, MaximumIntensity, intensity));

    public static Vector2[] CacheRainNodes(float radius, float step)
        => NodeEventSchedule.CacheNodes(CircularGrid(radius, step), node => WarpNode(node, radius, step));

    private static IEnumerable<Vector2> CircularGrid(float radius, float step)
    {
        int extent = Mathf.FloorToInt(radius / step);
        for (int z = -extent; z <= extent; z++)
        for (int x = -extent; x <= extent; x++)
        {
            Vector2 point = new Vector2(x, z) * step;
            if (point.sqrMagnitude <= radius * radius)
                yield return point;
        }
    }

    private static Vector2 WarpNode(Vector2 point, float radius, float step)
    {
        float edge = Mathf.Clamp01((radius - point.magnitude) / step);
        Vector2 warp = new(Mathf.PerlinNoise(point.x * 1.13f + 21.7f, point.y * 1.13f + 17.3f),
            Mathf.PerlinNoise(point.x * 0.97f + 37.1f, point.y * 0.97f + 45.9f));
        warp = 2f * new Vector2(Mathf.Clamp01(warp.x), Mathf.Clamp01(warp.y)) - Vector2.one;
        return point + warp * (step * 0.29f * edge);
    }
}
