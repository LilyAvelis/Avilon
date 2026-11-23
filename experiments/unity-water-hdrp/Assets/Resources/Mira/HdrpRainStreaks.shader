Shader "Mira/HDRP Rain Streaks"
{
    Properties
    {
        _Color("Tint", Color) = (0.65, 0.80, 0.86, 1)
        [HideInInspector] _GpuWaveMap("Wave field", 2D) = "black" {}
    }
    SubShader
    {
        Tags { "RenderPipeline"="HDRenderPipeline" "Queue"="Transparent+20" "RenderType"="Transparent" }
        Pass
        {
            Name "ForwardOnly"
            Tags { "LightMode"="ForwardOnly" }
            Cull Off ZWrite Off Blend SrcAlpha OneMinusSrcAlpha

            HLSLPROGRAM
            #pragma target 4.5
            #pragma vertex Vert
            #pragma fragment Frag
            #include "Packages/com.unity.render-pipelines.core/ShaderLibrary/Common.hlsl"
            #include "Packages/com.unity.render-pipelines.high-definition/Runtime/ShaderLibrary/ShaderVariables.hlsl"

            StructuredBuffer<float4> _RainDrops;
            TEXTURE2D(_GpuWaveMap);
            SAMPLER(sampler_GpuWaveMap);
            float4 _Color, _GpuWaveGrid;
            float3 _RainEye;
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

            float ImpactHeight(float2 worldXZ)
            {
                if (_GpuWaveGrid.x <= 0.0 || dot(worldXZ, worldXZ) >= _GpuWaveGrid.w * _GpuWaveGrid.w)
                    return 0.0;
                return SAMPLE_TEXTURE2D_LOD(_GpuWaveMap, sampler_GpuWaveMap, 0.5 + worldXZ * _GpuWaveGrid.x, 0).r;
            }

            Streak Vert(uint vertex : SV_VertexID, uint instance : SV_InstanceID)
            {
                Streak o;
                float4 drop = _RainDrops[instance];
                float2 corner = Corners[vertex];
                float hail = smoothstep(0.035, 0.14, drop.w);
                float3 world = float3(drop.x, drop.z * _RainFallSpeed + ImpactHeight(drop.xy), drop.y);
                float3 cameraRight = UNITY_MATRIX_V[0].xyz;
                float3 cameraUp = UNITY_MATRIX_V[1].xyz;
                float3 up = lerp(float3(0, 1, 0), cameraUp, hail);
                float halfWidth = lerp(0.018, drop.w * 0.5, hail);
                float halfLength = lerp(0.20, drop.w * 0.5, hail);
                world += cameraRight * corner.x * halfWidth + up * ((corner.y + 1.0) * halfLength);

                float distanceToEye = distance(_RainEye, world);
                float fade = saturate(drop.z * 28.0) * smoothstep(0.45, 1.1, distanceToEye)
                    * (1.0 - smoothstep(23.0, 58.0, distanceToEye));
                o.position = TransformWorldToHClip(GetCameraRelativePositionWS(world));
                o.corner = corner;
                o.style = float2(hail, fade);
                return o;
            }

            half4 Frag(Streak i) : SV_Target
            {
                float streak = (1.0 - abs(i.corner.x)) * (1.0 - smoothstep(-0.7, 1.0, i.corner.y));
                float squared = dot(i.corner, i.corner);
                float pellet = 1.0 - smoothstep(0.55, 1.0, squared);
                float3 ice = lerp(float3(0.38, 0.61, 0.75), float3(0.88, 0.96, 1.0),
                    saturate(1.0 - squared + i.corner.y * 0.6));
                float3 color = lerp(_Color.rgb, ice, i.style.x) * GetCurrentExposureMultiplier();
                return half4(color, lerp(streak * 0.94, pellet, i.style.x) * i.style.y);
            }
            ENDHLSL
        }
    }
}
