Shader "Mira/Water Lab Sky"
{
    Properties
    {
        _Horizon("Horizon", Color) = (0.55, 0.69, 0.71, 1)
        _Zenith("Zenith", Color) = (0.10, 0.29, 0.44, 1)
        _Sun("Sun direction", Vector) = (-0.3, 0.55, 0.38, 0)
        _SunColor("Sun color", Color) = (1, 0.91, 0.78, 1)
        _SunPower("Sun power", Float) = 1
        [HideInInspector] _PoincareOverview("Disk overview", Float) = 0
    }
    SubShader
    {
        Tags { "Queue"="Background" "RenderType"="Background" "PreviewType"="Skybox" }
        Cull Off ZWrite Off
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #pragma target 3.0
            #include "UnityCG.cginc"
            float4 _Horizon, _Zenith, _Sun, _SunColor;
            float _SunPower, _PoincareOverview;
            #include "WaterLabAtmosphere.cginc"
            struct Output { float4 position : SV_POSITION; float3 direction : TEXCOORD0; };
            Output vert(appdata_base v)
            {
                Output o;
                o.position = UnityWorldToClipPos(_WorldSpaceCameraPos + v.vertex.xyz * 16.0);
                o.direction = v.vertex.xyz;
                return o;
            }
            float4 frag(Output i) : SV_Target
            {
                float3 ray = normalize(i.direction);
                if (_PoincareOverview > 0.5 && ray.y < 0.0)
                    return float4(_Horizon.rgb, 1);
                float3 sun = normalize(_Sun.xyz);
                float distanceToSun = length(ray - sun);
                float visible = smoothstep(-0.012, 0.012, sun.y) * smoothstep(-0.003, 0.003, ray.y);
                float edge = max(fwidth(distanceToSun), 0.0001);
                float disk = (1.0 - smoothstep(0.009 - edge, 0.009 + edge, distanceToSun)) * visible;
                float halo = exp2(-24.0 * distanceToSun) * visible;
                float3 sky = LabSkyGradient(ray, sun) + _SunColor.rgb * (0.12 * halo + 1.35 * disk);
                return float4(LabSkyClouds(sky, ray, sun, false), 1);
            }
            ENDCG
        }
    }
}
