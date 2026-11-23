using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Rendering;

public sealed class WaterLabHdrpRainStreaks : IDisposable
{
    private const int Capacity = 6144;
    private const float DrawDistance = 58f;
    private static readonly int Drops = Shader.PropertyToID("_RainDrops");
    private static readonly int Eye = Shader.PropertyToID("_RainEye");
    private static readonly int FallSpeed = Shader.PropertyToID("_RainFallSpeed");
    private static readonly int WaveMap = Shader.PropertyToID("_GpuWaveMap");
    private static readonly int WaveGrid = Shader.PropertyToID("_GpuWaveGrid");

    private readonly Camera view;
    private readonly Material material;
    private readonly GraphicsBuffer instances;
    private readonly MaterialPropertyBlock properties = new();
    private readonly List<Vector4> upload = new(Capacity);

    public WaterLabHdrpRainStreaks(Camera view, Color tint)
    {
        this.view = view;
        var shader = Resources.Load<Shader>("Mira/HdrpRainStreaks");
        if (shader == null)
            throw new InvalidOperationException("Missing Mira/HdrpRainStreaks shader.");

        material = new Material(shader) { color = tint, hideFlags = HideFlags.HideAndDontSave };
        instances = new GraphicsBuffer(GraphicsBuffer.Target.Structured, Capacity, 4 * sizeof(float));
        properties.SetBuffer(Drops, instances);
        properties.SetFloat(FallSpeed, WaterLabRain.DropSpeed);
    }

    public void Draw(IEnumerable<WaterLabRain.Drop> falling, double now, Texture waveTexture, Vector4 waveGrid)
    {
        Vector3 eye = view.transform.position;
        Vector2 horizontalEye = new(eye.x, eye.z);
        upload.Clear();
        foreach (WaterLabRain.Drop drop in falling)
        {
            float timeLeft = (float)(drop.ImpactTime - now);
            if (timeLeft <= 0f || (drop.Point - horizontalEye).sqrMagnitude > DrawDistance * DrawDistance)
                continue;

            upload.Add(new Vector4(drop.Point.x, drop.Point.y, timeLeft, drop.Diameter));
            if (upload.Count == Capacity)
                break;
        }
        if (upload.Count == 0)
            return;

        instances.SetData(upload, 0, 0, upload.Count);
        properties.SetVector(Eye, eye);
        properties.SetTexture(WaveMap, waveTexture);
        properties.SetVector(WaveGrid, waveGrid);
        var pass = new RenderParams(material)
        {
            camera = view,
            matProps = properties,
            worldBounds = new Bounds(new Vector3(eye.x, 2.5f, eye.z), new Vector3(118f, 24f, 118f)),
            shadowCastingMode = ShadowCastingMode.Off,
            receiveShadows = false,
            lightProbeUsage = LightProbeUsage.Off,
            reflectionProbeUsage = ReflectionProbeUsage.Off,
            motionVectorMode = MotionVectorGenerationMode.ForceNoMotion,
        };
        Graphics.RenderPrimitives(pass, MeshTopology.Triangles, 6, upload.Count);
    }

    // RenderPrimitives is a per-frame submission, so no camera command buffer persists when rain stops.
    public void Clear() => upload.Clear();

    public void Dispose()
    {
        instances.Dispose();
        if (Application.isPlaying)
            UnityEngine.Object.Destroy(material);
        else
            UnityEngine.Object.DestroyImmediate(material);
    }
}
