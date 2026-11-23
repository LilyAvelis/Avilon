Shader "Mira/Water Lab Rain Streaks"
{
    Properties
    {
        _Color("Tint", Color) = (0.65, 0.80, 0.86, 1)
        [HideInInspector] _GpuWaveMap("Wave field", 2D) = "black" {}
    }
    SubShader
    {
        Tags { "Queue"="Transparent+20" "RenderType"="Transparent" }
        Cull Off ZWrite Off Blend SrcAlpha OneMinusSrcAlpha
        Pass
        {
            CGPROGRAM
            #pragma target 4.5
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"
            #include "../../WaterLabWaveTexture.cginc"

            StructuredBuffer<float4> _RainDrops;
            float4 _Color;
            float _RainFallSpeed;
            static const float2 Corners[6] = {
                float2(-1, -1), float2(-1, 1), float2(1, -1),
                float2(1, -1), float2(-1, 1), float2(1, 1)
            };

            struct Streak
            {
                float4 position : SV_POSITION;
                float2 corner : TEXCOORD0;
                float2 style : TEXCOORD1;
            };

            Streak vert(uint vertex : SV_VertexID, uint instance : SV_InstanceID)
            {
                Streak o;
                float4 drop = _RainDrops[instance];
                float2 corner = Corners[vertex];
                float hail = smoothstep(0.035, 0.14, drop.w);
                float3 worldPosition = float3(drop.x, drop.z * _RainFallSpeed + WaveHeight(drop.xy), drop.y);
                float3 cameraRight = UNITY_MATRIX_V[0].xyz;
                float3 cameraUp = UNITY_MATRIX_V[1].xyz;
                float3 up = lerp(float3(0, 1, 0), cameraUp, hail);
                float width = lerp(0.013, drop.w * 0.5, hail);
                float halfLength = lerp(0.165, drop.w * 0.5, hail);
                worldPosition += cameraRight * corner.x * width + up * (corner.y * halfLength + halfLength);
                float distanceToEye = length(_WorldSpaceCameraPos - worldPosition);
                float fade = saturate(drop.z * 28.0) * smoothstep(0.45, 1.1, distanceToEye)
                    * (1.0 - smoothstep(23.0, 58.0, distanceToEye));
                o.position = mul(UNITY_MATRIX_VP, float4(worldPosition, 1));
                o.corner = corner;
                o.style = float2(hail, fade);
                return o;
            }

            fixed4 frag(Streak i) : SV_Target
            {
                float streak = (1.0 - abs(i.corner.x)) * (1.0 - smoothstep(-0.7, 1.0, i.corner.y));
                float squared = dot(i.corner, i.corner);
                float pellet = 1.0 - smoothstep(0.55, 1.0, squared);
                float3 ice = lerp(float3(0.38, 0.61, 0.75), float3(0.88, 0.96, 1.0),
                    saturate(1.0 - squared + i.corner.y * 0.6));
                return float4(lerp(_Color.rgb, ice, i.style.x),
                    lerp(streak * 0.94, pellet, i.style.x) * i.style.y);
            }
            ENDCG
        }
    }
}


