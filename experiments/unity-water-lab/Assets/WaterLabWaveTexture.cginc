#ifndef MIRA_WATER_LAB_WAVE_TEXTURE
#define MIRA_WATER_LAB_WAVE_TEXTURE

sampler2D _GpuWaveMap;
float4 _GpuWaveGrid; // World-to-UV scale, texel, inverse cell size, world radius.

float2 WaveUV(float2 worldPosition)
{
    return 0.5 + worldPosition * _GpuWaveGrid.x;
}

float WaveHeight(float2 worldPosition)
{
    if (_GpuWaveGrid.x <= 0.0 || dot(worldPosition, worldPosition) >= _GpuWaveGrid.w * _GpuWaveGrid.w)
        return 0.0;
    return tex2Dlod(_GpuWaveMap, float4(WaveUV(worldPosition), 0, 0)).r;
}

float3 WaveSurface(float2 worldPosition, float footprint)
{
    if (_GpuWaveGrid.x <= 0.0 || dot(worldPosition, worldPosition) >= _GpuWaveGrid.w * _GpuWaveGrid.w)
        return 0.0;
    float2 uv = WaveUV(worldPosition);
    float texelsPerPixel = clamp(footprint * _GpuWaveGrid.z, 1.0, 32.0);
    float lod = log2(texelsPerPixel);
    float2 offset = float2(_GpuWaveGrid.y * texelsPerPixel, 0.0);
    float left = tex2Dlod(_GpuWaveMap, float4(uv - offset.xy, 0, lod)).r;
    float right = tex2Dlod(_GpuWaveMap, float4(uv + offset.xy, 0, lod)).r;
    float near = tex2Dlod(_GpuWaveMap, float4(uv - offset.yx, 0, lod)).r;
    float far = tex2Dlod(_GpuWaveMap, float4(uv + offset.yx, 0, lod)).r;
    return float3(float2(left - right, near - far) * (0.5 * _GpuWaveGrid.z / texelsPerPixel),
        tex2Dlod(_GpuWaveMap, float4(uv, 0, lod)).r);
}

#endif

