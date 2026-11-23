Shader "Mira/Water Laboratory"
{
    Properties
    {
        _Tint("Tint", Color) = (0.08, 0.40, 0.48, 1)
        _Extinction("Extinction", Float) = 0.55
        _Optical("Optical rendering", Float) = 0
        _NoiseMode("Surface noise", Float) = 0
        _NoiseAmount("Noise amplitude", Range(0, 1)) = 0.65
        _Wind("Wind direction XZ, speed and travel", Vector) = (1, 0, 0, 0)
        _RainEnabled("Rain impacts", Float) = 0
        [HideInInspector] _RainImpacts("Rain impact grid", 2D) = "black" {}
        [HideInInspector] _GpuWaveMap("Wave field", 2D) = "black" {}
        [HideInInspector] _WaterReach("Water reach", Vector) = (9, 118, 0, 0)
        [HideInInspector] _PoincareOverview("Poincare overview", Float) = 0
        _Horizon("Horizon", Color) = (0.55, 0.69, 0.71, 1)
        _Zenith("Zenith", Color) = (0.10, 0.29, 0.44, 1)
        _Sun("Sun direction", Vector) = (-0.3, 0.55, 0.38, 0)
        _SunColor("Sun color", Color) = (1, 0.94, 0.83, 1)
        _SunPower("Sun power", Range(0, 1.6)) = 1
    }
    SubShader
    {
        Tags { "Queue"="Transparent" "RenderType"="Transparent" }
        Cull Off
        ZWrite Off
        GrabPass { "_MiraWaterBackground" }
        Pass
        {
            CGPROGRAM
            #pragma target 4.5
            #pragma vertex vert
            #pragma fragment frag
            #include "WaterLabRefraction.cginc"
            #include "WaterLabRainSurface.cginc"
            #include "WaterLabWaveTexture.cginc"

            UNITY_DECLARE_DEPTH_TEXTURE(_CameraDepthTexture);
            sampler2D _MiraWaterBackground;
            float4 _Tint, _Horizon, _Zenith, _Sun, _SunColor, _Wind;
            #include "WaterLabAtmosphere.cginc"
            float _Extinction, _Optical, _NoiseMode, _NoiseAmount, _SunPower, _RainEnabled, _WaveContrast, _ProceduralBed;
            float _PoincareOverview;

            struct Vertex {
                float4 vertex : POSITION;
                float3 normal : NORMAL;
            };
            struct Interpolators {
                float4 position : SV_POSITION;
                float3 world : TEXCOORD0;
                float3 normal : TEXCOORD1;
                float4 grab : TEXCOORD2;
                float4 screen : TEXCOORD3;
                float eye : TEXCOORD4;
            };

            Interpolators vert(Vertex v)
            {
                Interpolators o;
                o.world = mul(unity_ObjectToWorld, v.vertex).xyz;
                o.world.y += WaveHeight(o.world.xz);
                o.position = mul(UNITY_MATRIX_VP, float4(o.world, 1.0));
                o.normal = UnityObjectToWorldNormal(v.normal);
                o.grab = ComputeGrabScreenPos(o.position);
                o.screen = ComputeScreenPos(o.position);
                o.eye = -mul(UNITY_MATRIX_V, float4(o.world, 1.0)).z;
                return o;
            }

            float2 LatticeGradient(float2 cell)
            {
                float3 h = frac(float3(cell.x, cell.y, cell.x) * float3(0.1031, 0.1030, 0.0973));
                h += dot(h, h.yzx + 33.33);
                float2 direction = frac((h.xx + h.yz) * h.zy) * 2.0 - 1.0;
                return direction * rsqrt(max(dot(direction, direction), 0.001));
            }

            // Noise value and its two derivatives, using the same interpolated gradients.
            float3 GradientNoise(float2 p)
            {
                float2 cell = floor(p);
                float2 f = frac(p);
                float2 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
                float2 du = 30.0 * f * f * (f - 1.0) * (f - 1.0);

                float2 g00 = LatticeGradient(cell);
                float2 g10 = LatticeGradient(cell + float2(1, 0));
                float2 g01 = LatticeGradient(cell + float2(0, 1));
                float2 g11 = LatticeGradient(cell + float2(1, 1));
                float n00 = dot(g00, f);
                float n10 = dot(g10, f - float2(1, 0));
                float n01 = dot(g01, f - float2(0, 1));
                float n11 = dot(g11, f - float2(1, 1));
                float low = lerp(n00, n10, u.x);
                float high = lerp(n01, n11, u.x);
                float2 derivative = lerp(lerp(g00, g10, u.x), lerp(g01, g11, u.x), u.y);
                derivative += du * float2(lerp(n10 - n00, n11 - n01, u.y), high - low);
                return float3(lerp(low, high, u.y), derivative);
            }

            float3 WindNoise(float2 p, float footprint)
            {
                // Stretch the ridges across the wind. All layers travel with the same world-space flow.
                const float lateralScale = 0.44;
                float2 flow = p * float2(1.0, lateralScale);
                if (_NoiseMode > 1.5)
                {
                    float wx = GradientNoise(flow * 0.77 + float2(7.3, 31.7)).x;
                    float wz = GradientNoise(flow * 0.69 + float2(-19.1, 1.8)).x;
                    flow += float2(wx, wz) * 0.96;
                }
                float3 broad = GradientNoise(flow * 1.9);
                float3 small = GradientNoise(flow * 4.3 + float2(18.6, 10.3));
                float3 fine = GradientNoise(flow * 8.2 + float2(-8.3, 6.1));
                broad *= 1.0 - smoothstep(0.20, 0.52, footprint * 1.9);
                small *= 1.0 - smoothstep(0.18, 0.45, footprint * 4.3);
                fine *= 1.0 - smoothstep(0.18, 0.45, footprint * 8.2);
                return (broad * 0.62 + small * 0.38 + fine * 0.23) * float3(1.0, 1.0, lateralScale);
            }

            float3 WindSurface(float2 p, float footprint)
            {
                float speed = clamp(_Wind.z, 0.0, 12.0);
                if (speed <= 0.0)
                    return 0.0;

                float lengthSquared = dot(_Wind.xy, _Wind.xy);
                float2 forward = lengthSquared > 0.0001
                    ? _Wind.xy * rsqrt(max(lengthSquared, 0.0001)) : float2(1.0, 0.0);
                float2 across = float2(-forward.y, forward.x);
                float2 lane = float2(dot(p, forward), dot(p, across));
                lane.x -= _Wind.w;

                float3 warp = GradientNoise(lane * float2(0.48, 0.58) + float2(4.7, 9.1));
                float a = dot(lane, float2(6.1, 0.64)) + 1.9 * warp.x;
                float b = dot(lane, float2(11.4, -1.9)) + 1.3 * warp.x;
                float c = dot(lane, float2(18.3, 1.7)) - 0.9 * warp.x;
                float3 bands = 1.0 - smoothstep(0.55, 2.05, footprint * float3(6.1, 11.4, 18.3));
                float3 waves = cos(float3(a, b, c)) * bands;
                float2 slope = waves.x * float2(0.74, 0.08) + waves.y * float2(0.29, -0.05)
                    + waves.z * float2(0.14, 0.015);
                float detail = dot(sin(float3(a, b, c)) * bands, float3(0.72, 0.39, 0.21));
                if (_NoiseMode > 0.5)
                {
                    float3 noise = WindNoise(lane, footprint);
                    slope = slope * 0.57 + noise.yz * (_NoiseAmount * 1.7);
                    detail = detail * 0.50 + noise.x * _NoiseAmount * 2.7;
                }

                float strength = smoothstep(0.0, 12.0, speed);
                return float3(forward * slope.x + across * slope.y, detail) * strength;
            }

            float4 frag(Interpolators i) : SV_Target
            {
                float3 V = normalize(_WorldSpaceCameraPos - i.world);
                float distanceToView = distance(_WorldSpaceCameraPos, i.world);
                float nearDetail = 1.0 - smoothstep(11, 44, distanceToView);
                float2 p = i.world.xz;
                float footprint = max(length(ddx(p)), length(ddy(p)));
                float3 L = normalize(_Sun.xyz);
                float3 fogColor = _Horizon.rgb;
                if (_PoincareOverview <= 0.5)
                {
                    float2 horizonRay = -V.xz * rsqrt(max(dot(V.xz, V.xz), 0.000001));
                    fogColor = LabSkyGradient(float3(horizonRay.x, 0, horizonRay.y), L);
                }
                float fog = _PoincareOverview > 0.5
                    ? smoothstep(130.0, 147.0, length(p))
                    : smoothstep(40.0, _WaterReach.y, distanceToView);
                if (fog >= 1.0)
                    return float4(fogColor, 1.0);
                float3 wind = nearDetail > 0.0 ? WindSurface(p, footprint) : 0.0;
                float reachFade = 1.0 - smoothstep(_WaterReach.y * 0.72, _WaterReach.y, length(p));
                float3 rain = _RainEnabled > 0.5 && reachFade > 0.0
                    ? RainSurface(p, footprint, _Time.y) : 0.0;
                float3 wave = WaveSurface(p, footprint);
                float waveDetail = 1.0 - smoothstep(0.28, 1.30, footprint);
                float2 windSlope = wind.xy * lerp(0.040, 0.060, _Optical) * nearDetail;
                float2 impactSlope = (rain.xy * 0.17 + wave.xy * (1.0 + waveDetail * 0.7)) * reachFade;
                float2 surfaceSlope = windSlope + impactSlope;
                float3 baseNormal = normalize(i.normal);
                float3 N = normalize(baseNormal + float3(surfaceSlope.x, 0, surfaceSlope.y));
                float ndv = saturate(dot(V, N));
                float fresnel = 0.02 + 0.98 * pow(1.0 - ndv, 5);
                float3 normalX = ddx(N);
                float3 normalY = ddy(N);
                float variance = 0.35 * (dot(normalX, normalX) + dot(normalY, normalY));
                float sharpness = lerp(48.0, 160.0, _Optical);
                float filteredSharpness = sharpness / (1.0 + sharpness * variance);
                float gloss = pow(saturate(dot(N, normalize(V + L))), filteredSharpness)
                    * (filteredSharpness + 2.0) / (sharpness + 2.0);
                float3 sunGloss = gloss * _SunColor.rgb * _SunPower;
                float ambient = lerp(0.24, 0.64, saturate(_SunPower));
                float direct = 0.44 * saturate(dot(L, N)) * _SunPower;
                float3 water = _Tint.rgb * (ambient + direct);
                float3 color = lerp(water, fogColor, fresnel * 0.75) + sunGloss * 0.55;

                if (_Optical > 0.5 || _PoincareOverview > 0.5)
                {
                    float d = SAMPLE_DEPTH_TEXTURE_PROJ(_CameraDepthTexture, UNITY_PROJ_COORD(i.screen));
                    float depth = max(0.0, LinearEyeDepth(d) - i.eye);
                    // Very shallow wind ripples remain visible in reflection without
                    // pulling the image of the bed along with the ripple pattern.
                    float2 refractionSlope = windSlope * 0.18 + impactSlope;
                    float3 refractionNormal = normalize(baseNormal + float3(refractionSlope.x, 0, refractionSlope.y));
                    float4 distortion = WaterLabRefractionOffsets(refractionNormal - baseNormal, min(depth, 1.2) * 0.017 * nearDetail);
                    float2 originalDepthUV = i.screen.xy / i.screen.w;
                    float2 originalGrabUV = i.grab.xy / i.grab.w;
                    float2 depthEdge = min(originalDepthUV, 1.0 - originalDepthUV);
                    float2 grabEdge = min(originalGrabUV, 1.0 - originalGrabUV);
                    float2 edge = min(depthEdge, grabEdge);
                    distortion *= smoothstep(0.0, 0.036, min(edge.x, edge.y));
                    float2 depthUV = originalDepthUV + distortion.xy;
                    float2 grabUV = originalGrabUV + distortion.zw;
                    float distortedDepth = LinearEyeDepth(SAMPLE_DEPTH_TEXTURE(_CameraDepthTexture, depthUV));
                    if (distortedDepth < i.eye || any(depthUV < 0.0) || any(depthUV > 1.0) ||
                        any(grabUV < 0.0) || any(grabUV > 1.0))
                        grabUV = originalGrabUV;
                    else
                        depth = distortedDepth - i.eye;
                    float3 background = tex2D(_MiraWaterBackground, grabUV).rgb;
                    float3 transmission = exp(-min(depth, 28) * _Extinction * float3(1.9, 0.76, 0.52));
                    if (_ProceduralBed > 0.5)
                    {
                        float rayLength = depth * distanceToView / max(i.eye, 0.1);
                        float3 clearWaterAbsorption = float3(0.125, 0.047, 0.029) * (_Extinction / 0.34);
                        transmission = exp(-min(rayLength, 150.0) * clearWaterAbsorption);
                    }
                    float3 refracted = background * transmission + water * (1.0 - transmission);
                    float3 reflected = reflect(-V, N);
                    float3 sky = LabSkyGradient(reflected, L);
                    float reflectedClouds = smoothstep(0.09, 0.16, fresnel);
                    if (reflectedClouds > 0.0)
                        sky = lerp(sky, LabSkyClouds(sky, reflected, L, true), reflectedClouds);
                    color = lerp(refracted, sky, fresnel) + sunGloss * 1.65;
                    float shallow = (1.0 - smoothstep(0.08, 0.65, depth)) * nearDetail;
                    color += shallow * lerp(0.012, 0.09, saturate(wind.z - 0.55));
                }

                float crest = saturate(max(0.0, wave.z) * 6.0 + length(wave.xy) * 0.75);
                color += float3(0.46, 0.67, 0.72) * saturate(rain.z + crest * _WaveContrast) * reachFade;

                return float4(lerp(color, fogColor, fog), 1);
            }
            ENDCG
        }
    }
}
