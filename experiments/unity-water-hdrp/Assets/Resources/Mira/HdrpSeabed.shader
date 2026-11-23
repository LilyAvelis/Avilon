Shader "Mira/HDRP Seabed"
{
    Properties
    {
        _Sun("Sun direction", Vector) = (-0.3, 0.55, 0.38, 0)
        _SunColor("Sun color", Color) = (1, 0.94, 0.83, 1)
        _SunPower("Sun power", Float) = 1
        _Horizon("Horizon", Color) = (0.55, 0.69, 0.71, 1)
        _Zenith("Zenith", Color) = (0.10, 0.29, 0.44, 1)
        _SeabedProfile("Profile", Float) = 1
        _SeabedNoise("Surface noise", Range(0, 1)) = 0.65
    }
    SubShader
    {
        Tags { "RenderPipeline"="HDRenderPipeline" "Queue"="Geometry" "RenderType"="Opaque" }
        HLSLINCLUDE
        #include "Packages/com.unity.render-pipelines.core/ShaderLibrary/Common.hlsl"
        #include "Packages/com.unity.render-pipelines.core/ShaderLibrary/Color.hlsl"
        #include "Packages/com.unity.render-pipelines.high-definition/Runtime/ShaderLibrary/ShaderVariables.hlsl"
        #define MIRA_SEABED_HDRP
        #include "Assets/Shared/Resources/Mira/WaterLabSeabedSurface.cginc"
        ENDHLSL

        Pass
        {
            Tags { "LightMode"="DepthForwardOnly" }
            ZWrite On
            HLSLPROGRAM
            #pragma target 4.5
            #pragma vertex Vert
            #pragma fragment Depth
            #include "Packages/com.unity.render-pipelines.high-definition/Runtime/Material/NormalBuffer.hlsl"
            float4 Depth(Varyings input) : SV_Target
            {
                NormalData data;
                data.normalWS = normalize(input.normal);
                data.perceptualRoughness = 0.9;
                float4 packed;
                EncodeIntoNormalBuffer(data, packed);
                return packed;
            }
            ENDHLSL
        }

        Pass
        {
            Tags { "LightMode"="ForwardOnly" }
            ZWrite On
            HLSLPROGRAM
            #pragma target 4.5
            #pragma vertex Vert
            #pragma fragment Frag
            ENDHLSL
        }
    }
}
