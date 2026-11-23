Shader "Mira/Water Lab Seabed"
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
        Tags { "Queue"="Geometry" "RenderType"="Opaque" }

        Pass
        {
            Tags { "LightMode"="ForwardBase" }
            CGPROGRAM
            #pragma target 3.0
            #pragma vertex Vert
            #pragma fragment Frag
            #include "UnityCG.cginc"

            #include "WaterLabSeabedSurface.cginc"
            ENDCG
        }

        Pass
        {
            Name "ShadowCaster"
            Tags { "LightMode"="ShadowCaster" }
            ZWrite On ZTest LEqual
            CGPROGRAM
            #pragma vertex VertDepth
            #pragma fragment FragDepth
            #pragma multi_compile_shadowcaster
            #include "UnityCG.cginc"

            struct DepthVaryings { V2F_SHADOW_CASTER; };

            DepthVaryings VertDepth(appdata_base v)
            {
                DepthVaryings o;
                TRANSFER_SHADOW_CASTER_NORMALOFFSET(o)
                return o;
            }

            float4 FragDepth(DepthVaryings i) : SV_Target
            {
                SHADOW_CASTER_FRAGMENT(i)
            }
            ENDCG
        }
    }
}
