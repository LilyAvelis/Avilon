using System;
using UnityEngine;

public sealed class WaveField
{
    public readonly float HalfSize;
    public readonly int Size;
    public readonly float Spacing;
    private float[] height;
    private float[] velocity;

    public WaveField(int size, float halfSize)
    {
        Size = size;
        HalfSize = halfSize;
        Spacing = 2f * halfSize / (size - 1);
        height = new float[size * size];
        velocity = new float[size * size];
    }

    public void Clear()
    {
        Array.Clear(height, 0, height.Length);
        Array.Clear(velocity, 0, velocity.Length);
    }

    public void Strike(Vector2 point, float strength, float radius)
    {
        int x = Mathf.RoundToInt((point.x + HalfSize) / Spacing);
        int z = Mathf.RoundToInt((point.y + HalfSize) / Spacing);
        int reach = Mathf.CeilToInt(radius * 2f / Spacing);
        for (int row = Mathf.Max(z - reach, 1); row <= Mathf.Min(z + reach, Size - 2); row++)
        for (int col = Mathf.Max(x - reach, 1); col <= Mathf.Min(x + reach, Size - 2); col++)
        {
            Vector2 delta = new Vector2(col * Spacing - HalfSize, row * Spacing - HalfSize) - point;
            float d = delta.sqrMagnitude / (radius * radius);
            velocity[row * Size + col] += strength * (d - 1f) * Mathf.Exp(-d * 1.7f);
        }
    }

    public void StrikeDrop(Vector2 point, float strength)
    {
        float x = (point.x + HalfSize) / Spacing;
        float z = (point.y + HalfSize) / Spacing;
        int col = (int)x;
        int row = (int)z;
        if (col < 2 || row < 2 || col >= Size - 3 || row >= Size - 3)
            return;

        float u = x - col;
        float v = z - row;
        int i = row * Size + col;
        AddDropImpulse(i, strength * (1f - u) * (1f - v));
        AddDropImpulse(i + 1, strength * u * (1f - v));
        AddDropImpulse(i + Size, strength * (1f - u) * v);
        AddDropImpulse(i + Size + 1, strength * u * v);
    }

    private void AddDropImpulse(int center, float strength)
    {
        // The dent displaces an equal amount into its four neighbors.
        velocity[center] -= strength;
        float rim = strength * 0.25f;
        velocity[center - 1] += rim;
        velocity[center + 1] += rim;
        velocity[center - Size] += rim;
        velocity[center + Size] += rim;
    }

    public void Step(float dt, float waveSpeed, float damping)
    {
        // Update all velocities from the same height field before any height changes.
        float factor = waveSpeed * waveSpeed * dt / (Spacing * Spacing);
        float persistence = Mathf.Exp(-damping * dt);
        for (int row = 1; row < Size - 1; row++)
        for (int col = 1; col < Size - 1; col++)
        {
            int i = row * Size + col;
            float laplacian = height[i - 1] + height[i + 1] + height[i - Size] + height[i + Size] - 4f * height[i];
            int edge = Mathf.Min(Mathf.Min(col, Size - 1 - col), Mathf.Min(row, Size - 1 - row));
            float absorber = Mathf.Clamp01(edge / 12f);
            float retention = Mathf.Lerp(0.80f, 1f, absorber);
            velocity[i] = (velocity[i] + factor * laplacian) * persistence * retention;
        }

        for (int i = 0; i < height.Length; i++)
            height[i] += velocity[i] * dt;
    }

    public float AtVertex(int index) => height[index];

    public float InterpolateSurface(Vector2 position, Func<int, Vector2, float> vertexHeight)
    {
        float x = (position.x + HalfSize) / Spacing;
        float z = (position.y + HalfSize) / Spacing;
        if (x < 0 || x >= Size - 1 || z < 0 || z >= Size - 1)
            return 0f;

        int col = (int)x;
        int row = (int)z;
        int i = row * Size + col;
        float u = x - col;
        float v = z - row;
        Vector2 a = new(col * Spacing - HalfSize, row * Spacing - HalfSize);
        Vector2 b = a + Vector2.up * Spacing;
        Vector2 c = a + Vector2.right * Spacing;
        // Match AddQuadIndices: the two triangles share the b–c diagonal.
        if (u + v <= 1f)
            return (1f - u - v) * vertexHeight(i, a) + v * vertexHeight(i + Size, b)
                + u * vertexHeight(i + 1, c);
        return (1f - u) * vertexHeight(i + Size, b) + (1f - v) * vertexHeight(i + 1, c)
            + (u + v - 1f) * vertexHeight(i + Size + 1, b + Vector2.right * Spacing);
    }
}
