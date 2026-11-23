float4 _Sun, _SunColor, _Horizon, _Zenith;
float _SunPower, _SeabedProfile, _SeabedNoise;

struct Vertex
{
    float4 vertex : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR;
};

struct Varyings
{
    float4 clip : SV_POSITION;
    float3 world : TEXCOORD0;
    float3 normal : TEXCOORD1;
    float2 uv : TEXCOORD2;
    float4 color : TEXCOORD3;
};

Varyings Vert(Vertex v)
{
    Varyings o;
#if defined(MIRA_SEABED_HDRP)
    o.clip = TransformObjectToHClip(v.vertex.xyz);
    o.world = GetAbsolutePositionWS(TransformObjectToWorld(v.vertex.xyz));
    o.normal = TransformObjectToWorldNormal(v.normal);
#else
    o.clip = UnityObjectToClipPos(v.vertex);
    o.world = mul(unity_ObjectToWorld, v.vertex).xyz;
    o.normal = UnityObjectToWorldNormal(v.normal);
#endif
    o.uv = v.uv;
    o.color = v.color;
    return o;
}

float Hash(float2 p)
{
    float3 h = frac(float3(p.x, p.y, p.x) * float3(0.1031, 0.1099, 0.1137));
    h += dot(h, h.yzx + 19.19);
    return frac((h.x + h.y) * h.z);
}

float Noise(float2 p)
{
    float2 i = floor(p);
    float2 f = frac(p);
    float2 b = f * f * (3.0 - 2.0 * f);
    return lerp(lerp(Hash(i), Hash(i + float2(1, 0)), b.x),
        lerp(Hash(i + float2(0, 1)), Hash(i + float2(1, 1)), b.x), b.y);
}

float3 PoolTile(float2 p, float pixelWidth)
{
    // Two ceramic strips per square, with the strip direction alternating like parquet.
    float2 square = floor(p / 1.75);
    float2 piece = frac(p / 1.75);
    if (frac((square.x + square.y) * 0.5) > 0.0)
        piece = piece.yx;
    float2 tile = piece * float2(1.0, 2.0);
    float2 edge = min(frac(tile), 1.0 - frac(tile));
    float distanceToGrout = min(edge.x * 1.75, edge.y * 0.875);
    float grout = 1.0 - smoothstep(0.027, 0.042 + pixelWidth * 0.55, distanceToGrout);
    float variation = Hash(square + floor(tile.y) * float2(78.3, 41.7));
    float3 glaze = lerp(float3(0.23, 0.60, 0.67), float3(0.73, 0.89, 0.79), 0.17 + 0.72 * variation);
    float flecks = Noise(p * 8.3) - 0.5;
    glaze *= 1.0 + flecks * _SeabedNoise * 0.20 * (1.0 - smoothstep(0.02, 0.12, pixelWidth));
    float3 tileColor = lerp(glaze, float3(0.15, 0.31, 0.36), grout);
    return lerp(tileColor, float3(0.46, 0.71, 0.72), smoothstep(0.17, 0.8, pixelWidth));
}

float3 Sand(float2 p, float3 n, float pixelWidth)
{
    float broad = Noise(p * 0.058 + float2(13, -8));
    float streak = Noise(float2(p.x * 0.29 + 2.8 * broad, p.y * 0.12 - 2.1 * broad));
    float slope = 1.0 - saturate(n.y);
    float clay = smoothstep(0.36, 0.78, 0.68 * broad + 0.32 * streak + 0.60 * slope);
    float3 sand = float3(0.94, 0.83, 0.61);
    float3 silt = float3(0.48, 0.33, 0.23);
    float3 bed = lerp(sand, silt, clay);

    float2 dune = float2(dot(p, float2(0.91, 0.41)), dot(p, float2(-0.41, 0.91)));
    float fine = 1.0 - smoothstep(0.22, 0.68, pixelWidth * 4.6);
    float meander = Noise(dune * float2(0.40, 0.17)) * 3.4 + broad * 2.5;
    float ripple = sin(dune.x * 5.8 + dune.y * 0.21 + meander * 3.4);
    float grain = Noise(p * 8.5) - 0.5;
    float rippleCover = smoothstep(0.28, 0.74, streak) * n.y * n.y;
    bed *= 0.96 + _SeabedNoise * (0.29 * (streak - 0.5)
        + fine * (0.105 * ripple * rippleCover + 0.09 * grain));
    return bed;
}

float3 Stone(float2 p, float3 n, float pixelWidth)
{
    float warp = Noise(p * 0.16 + 7.7);
    float2 layered = float2(p.x * 0.28 + warp * 1.28, p.y * 0.66 - warp * 0.96);
    float strata = Noise(layered);
    float chips = Noise(layered * 3.27 + 11.3);
    float rough = abs(Noise(layered * 1.17 - 3.14) * 2.0 - 1.0);
    float seam = smoothstep(0.32, 0.68, rough + chips * 0.27);
    float fissure = seam * (1.0 - smoothstep(0.22, 0.9, pixelWidth));
    float slope = 1.0 - saturate(n.y);
    float3 slate = lerp(float3(0.23, 0.29, 0.33), float3(0.64, 0.64, 0.58),
        saturate(0.66 * strata + 0.22 * slope + 0.14));
    return slate * (1.0 + _SeabedNoise * (0.36 * chips - 0.20 - 0.31 * fissure));
}

float3 Mud(float2 p, float3 n, float pixelWidth)
{
    float2 folded = p + 19.0 * float2(Noise(p * 0.033 + 31.9), Noise(p * 0.039 - 8.7));
    float basin = Noise(folded * 0.093);
    float silt = Noise(float2(folded.x + folded.y * 0.46, folded.y - folded.x * 0.72) * 0.25 + 45.8);
    float blotch = Noise(folded * 0.91 + float2(-9.6, 17.2));
    float patch = smoothstep(0.34, 0.73, basin * 0.57 + silt * 0.28 + blotch * 0.15);
    float fine = 1.0 - smoothstep(0.10, 0.38, pixelWidth);
    float flecks = (Noise(folded * 4.4 + 9.0) - 0.5) * fine;
    float slope = 1.0 - saturate(n.y);
    float3 sediment = lerp(float3(0.29, 0.20, 0.13), float3(0.57, 0.43, 0.23), patch);
    float3 bed = lerp(float3(0.43, 0.31, 0.19), sediment, 0.35 + 0.65 * _SeabedNoise);
    return bed * (0.89 + 0.46 * flecks * _SeabedNoise + 0.27 * slope);
}

float3 Poincare(float2 p, float2 uv, float4 ceramic, float pixelWidth)
{
    float3 surround = float3(0.26, 0.46, 0.49);
    float2 sides = min(uv, 1.0 - uv);
    float edge = min(sides.x, sides.y);
    float filterWidth = max(0.001, fwidth(edge));
    float seamWidth = min(0.014, 0.003 + 0.12 * filterWidth / max(pixelWidth, 0.025));
    float grout = 1.0 - smoothstep(seamWidth - filterWidth, seamWidth + filterWidth, edge);
    float grain = (Noise(p * 6.0) - 0.5) * (1.0 - smoothstep(0.065, 0.32, pixelWidth));
    float3 glaze = ceramic.rgb * (1.0 + grain * _SeabedNoise * 0.23);
    float3 tile = lerp(glaze, float3(0.095, 0.15, 0.16), grout);
    return lerp(surround, tile, ceramic.a);
}

float3 SpectreSediment(float2 p, float3 n, float tone, float pixelWidth)
{
    float3 loam = lerp(float3(0.31, 0.24, 0.15), float3(0.78, 0.63, 0.40), tone);
    float grain = (Noise(p * 5.2) - 0.5) * (1.0 - smoothstep(0.07, 0.36, pixelWidth));
    float silt = Noise(p * 1.9 + float2(40.1, -12.8));
    return loam * (0.94 + 0.19 * (1.0 - n.y) + _SeabedNoise * (0.22 * (silt - 0.5) + 0.33 * grain));
}

float4 Frag(Varyings i) : SV_Target
{
    float3 n = normalize(i.normal);
    float2 p = i.world.xz;
    float pixelWidth = max(length(ddx(p)), length(ddy(p)));
    float3 bed;
    if (_SeabedProfile < 0.5)
        bed = PoolTile(p, pixelWidth);
    else if (_SeabedProfile < 1.5)
        bed = Sand(p, n, pixelWidth);
    else if (_SeabedProfile < 2.5)
        bed = Stone(p, n, pixelWidth);
    else if (_SeabedProfile < 3.5)
        bed = Mud(p, n, pixelWidth);
    else if (_SeabedProfile < 4.5)
        bed = Poincare(p, i.uv, i.color, pixelWidth);
    else
        bed = SpectreSediment(p, n, i.color.r, pixelWidth);

    float light = saturate(dot(n, normalize(_Sun.xyz)));
    float bright = saturate(_SunPower);
    float3 fill = lerp(_Zenith.rgb, _Horizon.rgb, 0.84) * lerp(0.28, 0.69, bright);
    float3 direct = _SunColor.rgb * light * bright * 0.73;
    float3 lit = bed * (fill + direct);
#if defined(MIRA_SEABED_HDRP)
    lit = SRGBToLinear(saturate(lit)) * GetCurrentExposureMultiplier();
#endif
    return float4(lit, 1.0);
}
