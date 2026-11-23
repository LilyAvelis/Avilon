using System;
using UnityEngine;

public sealed class WaterLabRainSurface : IDisposable
{
    private const float BucketSize = 1.5f;
    private static readonly int Impacts = Shader.PropertyToID("_RainImpacts");
    private static readonly int Grid = Shader.PropertyToID("_RainGrid");

    private readonly Texture2D texture;
    private readonly int extent;
    private bool dirty;
    private bool hasImpacts;

    public WaterLabRainSurface(Material material, float radius)
    {
        extent = Mathf.CeilToInt(radius / BucketSize);
        int side = 2 * extent + 1;
        texture = new Texture2D(2 * side, side, TextureFormat.RGBAFloat, false, true)
        {
            name = "Water Lab rain impacts",
            filterMode = FilterMode.Point,
            wrapMode = TextureWrapMode.Clamp
        };
        var pixels = texture.GetRawTextureData<Color>();
        pixels.AsSpan().Clear();
        texture.Apply(false);
        material.SetTexture(Impacts, texture);
        material.SetVector(Grid, new Vector4(BucketSize, extent, 1f / texture.width, 1f / texture.height));
    }

    public void Add(WaterLabRain.Drop impact)
    {
        int x = Mathf.FloorToInt(impact.Point.x / BucketSize) + extent;
        int z = Mathf.FloorToInt(impact.Point.y / BucketSize) + extent;
        int index = z * texture.width + 2 * x;
        var pixels = texture.GetRawTextureData<Color>();
        pixels[index + 1] = pixels[index];
        pixels[index] = new Color(impact.Point.x, impact.Point.y, (float)impact.ImpactTime, 1f);
        dirty = true;
        hasImpacts = true;
    }

    public void Publish()
    {
        if (!dirty)
            return;
        texture.Apply(false);
        dirty = false;
    }

    public void Clear()
    {
        if (!hasImpacts)
            return;
        texture.GetRawTextureData<Color>().AsSpan().Clear();
        hasImpacts = false;
        dirty = true;
        Publish();
    }

    public void Dispose()
    {
        if (Application.isPlaying)
            UnityEngine.Object.Destroy(texture);
        else
            UnityEngine.Object.DestroyImmediate(texture);
    }
}
