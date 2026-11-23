using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Experimental.Rendering;
using UnityEngine.Rendering;

public sealed class WaterLabGpuRain : IDisposable
{
    public const int Resolution = 2048;
    private const int FieldThreads = 8;
    private const int ScatterThreads = 64;
    private const float MaximumCourantNumber = 0.62f;
    private const float EdgeDamping = 7.5f;
    private const float SettleSeconds = 15f;

    private readonly List<Vector4> pending = new(512);
    private readonly float spacing;
    private readonly int clearKernel, scatterKernel, waveKernel, restoreKernel;
    private ComputeShader compute;
    private GraphicsBuffer impulses, drops;
    private RenderTexture current, next;
    private float settleRemaining;
    private Vector2 samplePosition, requestedPosition, sampleBlend;
    private float sampleHeight;
    private int generation, requestedGeneration;
    private bool samplePending, sampleAvailable;

    public float Radius { get; }
    public int GridResolution { get; }
    public Texture HeightTexture => current;
    public bool IsActive => pending.Count > 0 || settleRemaining > 0f;

    public WaterLabGpuRain(float radius, int resolution = Resolution)
    {
        if (!(radius > 0f) || float.IsInfinity(radius))
            throw new ArgumentOutOfRangeException(nameof(radius));
        if (resolution < 1024 || resolution > 6144 || resolution % FieldThreads != 0)
            throw new ArgumentOutOfRangeException(nameof(resolution), "Rain grid must be a multiple of eight from 1024 to 6144.");
        if (!SystemInfo.supportsComputeShaders)
            throw new NotSupportedException("The graphics device does not support compute shaders.");

        Radius = radius;
        GridResolution = resolution;
        spacing = 2f * radius / (GridResolution - 1);
        var resource = Resources.Load<ComputeShader>("Mira/WaterLabGpuRain");
        if (resource == null)
            throw new InvalidOperationException("Missing Mira/WaterLabGpuRain compute shader.");

        try
        {
            compute = UnityEngine.Object.Instantiate(resource);
            clearKernel = compute.FindKernel("ClearField");
            scatterKernel = compute.FindKernel("ScatterDrops");
            waveKernel = compute.FindKernel("AdvanceWaves");
            restoreKernel = compute.FindKernel("RestoreField");
            current = NewField("Water Lab rain state A");
            next = NewField("Water Lab rain state B");
            impulses = new GraphicsBuffer(GraphicsBuffer.Target.Structured, GridResolution * GridResolution, sizeof(int));
            drops = new GraphicsBuffer(GraphicsBuffer.Target.Structured, pending.Capacity, sizeof(float) * 4);

            compute.SetInt("_FieldSize", GridResolution);
            compute.SetFloat("_Radius", Radius);
            compute.SetFloat("_Spacing", spacing);
            compute.SetBuffer(clearKernel, "_Impulses", impulses);
            compute.SetBuffer(scatterKernel, "_Impulses", impulses);
            compute.SetBuffer(waveKernel, "_Impulses", impulses);
            compute.SetBuffer(restoreKernel, "_Impulses", impulses);
            Clear();
        }
        catch
        {
            Dispose();
            throw;
        }
    }

    /// <summary>Queues a world-XZ impact. Diameter is in metres and a positive impulse pushes down in m/s.</summary>
    public void Add(Vector2 point, float diameter, float impulse)
    {
        if (!(point.sqrMagnitude <= Radius * Radius) || !(diameter > 0f) ||
            float.IsInfinity(diameter) || impulse == 0f || float.IsNaN(impulse) || float.IsInfinity(impulse))
            return;

        pending.Add(new Vector4(point.x, point.y, Mathf.Min(diameter, 4f), Mathf.Clamp(impulse, -8f, 8f)));
    }

    public void Step(float dt, float speed, float damping)
    {
        if (!(dt > 0f) || float.IsInfinity(dt))
            return;

        bool hasDrops = pending.Count != 0;
        if (!IsActive)
            return;
        settleRemaining = hasDrops ? SettleSeconds : settleRemaining - dt;
        if (settleRemaining <= 0f)
        {
            Clear();
            return;
        }
        if (hasDrops)
        {
            if (pending.Count > drops.count)
            {
                var larger = new GraphicsBuffer(GraphicsBuffer.Target.Structured,
                    Mathf.NextPowerOfTwo(pending.Count), sizeof(float) * 4);
                drops.Dispose();
                drops = larger;
            }

            drops.SetData(pending);
            compute.SetBuffer(scatterKernel, "_Drops", drops);
            compute.SetInt("_DropCount", pending.Count);
            compute.Dispatch(scatterKernel, (pending.Count + ScatterThreads - 1) / ScatterThreads, 1, 1);
            pending.Clear();
        }

        speed = Mathf.Max(0f, speed);
        int substeps = Mathf.Max(1, Mathf.CeilToInt(speed * dt / (spacing * MaximumCourantNumber)));
        float step = dt / substeps;
        float gridSpeed = speed / spacing;
        compute.SetFloat("_DeltaTime", step);
        compute.SetFloat("_LaplacianFactor", gridSpeed * gridSpeed * step);
        compute.SetFloat("_VelocityRetention", Mathf.Exp(-Mathf.Max(0f, damping) * step));
        compute.SetFloat("_BoundaryRetention", Mathf.Exp(-EdgeDamping * step));

        for (int i = 0; i < substeps; i++)
        {
            compute.SetInt("_ConsumeImpulses", hasDrops && i == 0 ? 1 : 0);
            compute.SetTexture(waveKernel, "_CurrentState", current);
            compute.SetTexture(waveKernel, "_NextState", next);
            DispatchField(waveKernel);
            (current, next) = (next, current);
        }
        current.GenerateMips();
    }

    public void Clear()
    {
        pending.Clear();
        settleRemaining = 0f;
        generation++;
        sampleAvailable = false;
        compute.SetTexture(clearKernel, "_NextState", current);
        compute.SetTexture(clearKernel, "_OtherState", next);
        DispatchField(clearKernel);
        current.GenerateMips();
        next.GenerateMips();
    }

    public void RestoreFrom(WaterLabGpuRain source)
    {
        if (source == null)
            throw new ArgumentNullException(nameof(source));
        if (compute == null || source.compute == null)
            throw new ObjectDisposedException(nameof(WaterLabGpuRain));
        if (Radius != source.Radius)
            throw new ArgumentException("The source and destination must cover the same radius.", nameof(source));
        if (ReferenceEquals(source, this))
            return;
        if (!source.IsActive)
        {
            if (IsActive)
                Clear();
            return;
        }

        float sourceSize = source.GridResolution;
        compute.SetFloat("_RestoreUvScale", (sourceSize - 1f) / sourceSize / (Radius * 2f));
        compute.SetFloat("_RestoreMip", Mathf.Max(0f, Mathf.Log(sourceSize / GridResolution, 2f)));
        compute.SetTexture(restoreKernel, "_RestoreSource", source.current);
        compute.SetTexture(restoreKernel, "_NextState", current);
        DispatchField(restoreKernel);
        current.GenerateMips();

        pending.Clear();
        pending.AddRange(source.pending);
        settleRemaining = source.settleRemaining;
        generation++;
        sampleAvailable = source.sampleAvailable;
        sampleHeight = source.sampleHeight;
        samplePosition = source.samplePosition;
        if (source.samplePending)
            SampleHeight(source.requestedPosition);
    }

    /// <summary>Returns the latest local GPU sample, while requesting a fresh sample without waiting.</summary>
    public float SampleHeight(Vector2 point)
    {
        if (!IsActive || !(point.sqrMagnitude < Radius * Radius) || !SystemInfo.supportsAsyncGPUReadback)
            return 0f;

        if (!samplePending)
        {
            Vector2 grid = (point + Vector2.one * Radius) / spacing;
            int x = Mathf.Clamp(Mathf.FloorToInt(grid.x), 0, GridResolution - 2);
            int z = Mathf.Clamp(Mathf.FloorToInt(grid.y), 0, GridResolution - 2);
            sampleBlend = grid - new Vector2(x, z);
            requestedPosition = point;
            requestedGeneration = generation;
            samplePending = true;
            AsyncGPUReadback.Request(current, 0, x, 2, z, 2, 0, 1, StoreHeightSample);
        }

        return sampleAvailable && (point - samplePosition).sqrMagnitude < 1f ? sampleHeight : 0f;
    }

    private void StoreHeightSample(AsyncGPUReadbackRequest request)
    {
        samplePending = false;
        if (request.hasError || compute == null || requestedGeneration != generation)
            return;

        var heights = request.GetData<Vector2>();
        if (heights.Length < 4)
            return;
        float first = Mathf.Lerp(heights[0].x, heights[1].x, sampleBlend.x);
        float second = Mathf.Lerp(heights[2].x, heights[3].x, sampleBlend.x);
        sampleHeight = Mathf.Lerp(first, second, sampleBlend.y);
        samplePosition = requestedPosition;
        sampleAvailable = true;
    }

    private RenderTexture NewField(string name)
    {
        var descriptor = new RenderTextureDescriptor(GridResolution, GridResolution)
        {
            graphicsFormat = GraphicsFormat.R32G32_SFloat,
            depthBufferBits = 0,
            msaaSamples = 1,
            enableRandomWrite = true,
            useMipMap = true,
            autoGenerateMips = false,
            mipCount = -1,
        };
        var field = new RenderTexture(descriptor)
        {
            name = name,
            filterMode = FilterMode.Trilinear,
            wrapMode = TextureWrapMode.Clamp,
        };
        if (field.Create())
            return field;
        DestroyAsset(field);
        throw new InvalidOperationException("Could not allocate the rain simulation texture.");
    }

    private void DispatchField(int kernel)
    {
        int groups = GridResolution / FieldThreads;
        compute.Dispatch(kernel, groups, groups, 1);
    }

    public void Dispose()
    {
        pending.Clear();
        settleRemaining = 0f;
        impulses?.Dispose();
        drops?.Dispose();
        impulses = drops = null;
        ReleaseField(current);
        ReleaseField(next);
        current = next = null;
        DestroyAsset(compute);
        compute = null;
    }

    private static void ReleaseField(RenderTexture field)
    {
        if (field == null)
            return;
        field.Release();
        DestroyAsset(field);
    }

    private static void DestroyAsset(UnityEngine.Object asset)
    {
        if (asset == null)
            return;
        if (Application.isPlaying)
            UnityEngine.Object.Destroy(asset);
        else
            UnityEngine.Object.DestroyImmediate(asset);
    }
}
