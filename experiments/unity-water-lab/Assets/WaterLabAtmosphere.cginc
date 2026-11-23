#ifndef MIRA_WATER_LAB_ATMOSPHERE
#define MIRA_WATER_LAB_ATMOSPHERE

float3 LabSkyGradient(float3 ray, float3 sun)
{
    float altitude = max(ray.y, 0.0);
    float blue = 1.0 - exp2(-4.4 * altitude);
    float sunset = smoothstep(-0.17, -0.015, sun.y) * (1.0 - smoothstep(0.10, 0.50, sun.y));
    float towardsSun = saturate(dot(ray, sun) * 0.5 + 0.5);
    float3 color = lerp(_Horizon.rgb, _Zenith.rgb, blue);
    return color + _SunColor.rgb * (0.18 * sunset * towardsSun * towardsSun * exp2(-10.0 * altitude));
}

float LabCloudHash(float2 cell)
{
    float3 p = frac(cell.xyx * 0.1031);
    p += dot(p, p.yzx + 33.33);
    return frac((p.x + p.y) * p.z);
}

float LabCloudNoise(float2 p)
{
    float2 cell = floor(p);
    float2 f = frac(p);
    f = f * f * (3.0 - 2.0 * f);
    return lerp(
        lerp(LabCloudHash(cell), LabCloudHash(cell + float2(1, 0)), f.x),
        lerp(LabCloudHash(cell + float2(0, 1)), LabCloudHash(cell + 1.0), f.x),
        f.y);
}

float3 LabSkyClouds(float3 sky, float3 ray, float3 sun, bool reflectClouds)
{
    if (ray.y <= 0.025)
        return sky;

    float2 p = ray.xz / (0.20 + ray.y) * float2(2.25, 3.00) + float2(8.2, 14.4);
    float noise = LabCloudNoise(p) * 0.64 + LabCloudNoise(p.yx * 2.13 + 16.8) * 0.25;
    if (!reflectClouds)
        noise += LabCloudNoise(p * 4.47 - 9.4) * 0.075 + LabCloudNoise(p.yx * 8.6 + 6.7) * 0.035;
    else
        noise += 0.055;

    float edge = smoothstep(0.025, 0.13, ray.y);
    float cover = smoothstep(0.46, 0.67, noise) * edge;
    float daylight = smoothstep(-0.19, 0.52, sun.y);
    float facingSun = smoothstep(0.46, 0.96, dot(ray, sun));
    float3 shade = lerp(_Zenith.rgb, _Horizon.rgb, 0.75) * (0.72 + 0.24 * daylight);
    float3 lit = lerp(_Horizon.rgb * 0.85, float3(0.96, 0.94, 0.89), daylight);
    lit += _SunColor.rgb * facingSun * (0.22 + 0.08 * daylight);
    return lerp(sky, lerp(shade, lit, smoothstep(0.48, 0.72, noise)), cover * 0.94);
}

#endif
