using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Rendering;

public sealed class WaterLabRainStreaks
{
    private const int Capacity = 6144;
    private const float DrawDistance = 58f;
    private readonly GraphicsBuffer instances = new(GraphicsBuffer.Target.Structured, Capacity, 4 * sizeof(float));
    private readonly List<Vector4> upload = new(Capacity);
    private readonly Camera view;
    private readonly CommandBuffer draw = new() { name = "Water Lab precipitation" };
    private readonly Material material;
    private bool attached;

    public WaterLabRainStreaks(Camera view, Color tint, bool attach = true)
    {
        this.view = view;
        material = new Material(Resources.Load<Shader>("Mira/WaterLabRainStreaks")) { color = tint };
        material.SetBuffer("_RainDrops", instances);
        SetAttached(attach);
    }

    public void SetAttached(bool enabled)
    {
        if (view == null || attached == enabled)
            return;
        if (enabled)
            view.AddCommandBuffer(CameraEvent.AfterForwardAlpha, draw);
        else
        {
            view.RemoveCommandBuffer(CameraEvent.AfterForwardAlpha, draw);
            draw.Clear();
        }
        attached = enabled;
    }

    public void Draw(IEnumerable<WaterLabRain.Drop> falling, double now, Texture waveTexture, Vector4 waveGrid)
    {
        draw.Clear();
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
        material.SetTexture("_GpuWaveMap", waveTexture);
        material.SetVector("_GpuWaveGrid", waveGrid);
        material.SetFloat("_RainFallSpeed", WaterLabRain.DropSpeed);
        draw.DrawProcedural(Matrix4x4.identity, material, 0, MeshTopology.Triangles, 6, upload.Count);
    }

    public void Clear() => draw.Clear();

    public void Dispose()
    {
        SetAttached(false);
        draw.Dispose();
        instances.Dispose();
        Object.Destroy(material);
    }
}
