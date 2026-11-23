#ifndef MIRA_WATER_LAB_RAIN_SURFACE
#define MIRA_WATER_LAB_RAIN_SURFACE

sampler2D _RainImpacts;
float4 _RainGrid; // Bucket size, grid extent, reciprocal texture width and height.
float4 _WaterReach; // Horizontal reach through the fog in y.
float _RainIntensity;

float3 RainRing(float2 p, float footprint, float4 impact, float age, float storm, float lifetime)
{
    float2 delta = p - impact.xy;
    float radius = length(delta);
    float size = clamp(impact.w, 0.32, 1.65);
    float speed = lerp(lerp(0.59, 1.04, saturate(size)), 0.57, storm);
    float width = 0.020 * size + min(footprint * 0.55, 0.18);
    float distant = smoothstep(0.22, 0.65, footprint);
    float glint = (1.0 - smoothstep(0.12, lerp(0.95, 0.58, storm), radius))
        * (1.0 - smoothstep(0.04, lerp(0.37, 0.27, storm), age)) * lerp(0.20, 0.13, storm);
    if (distant >= 1.0)
        return float3(0, 0, glint);
    if (radius > age * speed + width * 1.45 + 0.08)
        return float3(0, 0, glint * distant);

    float dotRadius = 0.053 * size;
    float light = (1.0 - smoothstep(dotRadius, dotRadius + width, radius))
        * (1.0 - smoothstep(0.02, 0.16, age)) * lerp(0.72, 0.42, storm);
    float slope = 0.0;
    [unroll] for (int ring = 0; ring < 3; ring++)
    {
        float time = age - ring * 0.115;
        float offset = radius - time * speed;
        float band = 1.0 - smoothstep(width * 0.42, width * 1.45, abs(offset));
        float fade = smoothstep(0.0, 0.07, time) * (1.0 - smoothstep(0.26, lifetime, age));
        float crest = band * fade * (0.37 - ring * 0.08) * saturate(size + 0.3) * lerp(1.0, 0.54, storm);
        light += crest;
        slope += crest * clamp(offset / width, -1.0, 1.0);
    }
    return lerp(float3(delta / max(radius, 0.02) * slope, light), float3(0, 0, glint), distant);
}

float3 RainSurface(float2 p, float footprint, float now)
{
    float clarity = 1.0 - smoothstep(0.95, 3.0, footprint);
    if (clarity <= 0.0)
        return 0;

    float storm = smoothstep(8.0, 65.0, _RainIntensity);
    float lifetime = lerp(1.05, 0.51, storm);
    float2 baseBucket = floor(p / _RainGrid.x) + _RainGrid.y;
    float3 rain = 0;
    // The maximum ring reach is below one bucket. Only these nine can overlap p.
    [loop] for (int z = -1; z <= 1; z++)
    [loop] for (int x = -1; x <= 1; x++)
    {
        float2 bucket = baseBucket + float2(x, z);
        if (any(bucket < 0) || any(bucket > 2.0 * _RainGrid.y))
            continue;

        float2 uv = (bucket * float2(2, 1) + 0.5) * _RainGrid.zw;
        float4 impact = tex2Dlod(_RainImpacts, float4(uv, 0, 0));
        float age = now - impact.z;
        if (impact.w <= 0 || age >= lifetime)
            continue;
        if (age >= 0.0)
            rain += RainRing(p, footprint, impact, age, storm, lifetime);

        uv.x += _RainGrid.z;
        impact = tex2Dlod(_RainImpacts, float4(uv, 0, 0));
        age = now - impact.z;
        if (impact.w > 0 && age >= 0.0 && age < lifetime)
            rain += RainRing(p, footprint, impact, age, storm, lifetime);
    }
    return rain * clarity;
}

#endif
