Shader "Mira/HDRP Wave Decal"
{
    Properties
    {
        [HideInInspector] _AffectDeformation("Affect Deformation", Float) = 1
        [HideInInspector] _GpuWaveMap("Wave height", 2D) = "black" {}
        [HideInInspector] _GpuWaveGrid("Wave grid", Vector) = (0, 0, 0, 0)
    }
    SubShader
    {
        Tags { "RenderPipeline"="HDRenderPipeline" "ShaderGraphTargetId"="WaterDecalSubTarget" }
        Pass
        {
            Name "Deformation"
            Cull Off
            ZWrite Off
            ZTest Always
            Blend Off
            HLSLPROGRAM
            #pragma target 4.5
            #pragma vertex Vert
            #pragma fragment Frag
            #include "Packages/com.unity.render-pipelines.core/ShaderLibrary/Common.hlsl"
            TEXTURE2D(_GpuWaveMap);
            SAMPLER(sampler_GpuWaveMap);
            float4 _GpuWaveGrid;

            struct Varyings
            {
                float4 position : SV_POSITION;
                float2 uv : TEXCOORD0;
            };

            Varyings Vert(uint vertex : SV_VertexID)
            {
                Varyings result;
                result.position = GetFullScreenTriangleVertexPosition(vertex);
                result.uv = result.position.xy * 0.5 + 0.5;
                #if UNITY_UV_STARTS_AT_TOP
                result.uv.y = 1.0 - result.uv.y;
                #endif
                return result;
            }

            float4 Frag(Varyings input) : SV_Target
            {
                float2 xz = (input.uv * 2.0 - 1.0) * _GpuWaveGrid.w;
                if (_GpuWaveGrid.x <= 0 || dot(xz, xz) >= _GpuWaveGrid.w * _GpuWaveGrid.w)
                    return 0;
                float height = SAMPLE_TEXTURE2D_LOD(_GpuWaveMap, sampler_GpuWaveMap, 0.5 + xz * _GpuWaveGrid.x, 0).r;
                // Preserve faint rain in HDRP's filtered optical surface.
                // The gameplay height field never reads this gain back.
                height *= lerp(9.0, 1.0, smoothstep(0.004, 0.025, abs(height)));
                return float4(clamp(height, -1.0, 1.0), 0, 0, 0);
            }
            ENDHLSL
        }
    }
}
