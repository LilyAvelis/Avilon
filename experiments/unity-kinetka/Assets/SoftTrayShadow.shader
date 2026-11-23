Shader "Mira/Soft Tray Shadow"
{
    SubShader
    {
        Tags { "Queue" = "Transparent" "RenderType" = "Transparent" }
        Cull Off
        ZWrite Off
        Blend SrcAlpha OneMinusSrcAlpha

        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            struct ShadowVertex
            {
                float4 pixel : SV_POSITION;
                float2 uv : TEXCOORD0;
            };

            ShadowVertex vert(appdata_base input)
            {
                ShadowVertex result;
                result.pixel = UnityObjectToClipPos(input.vertex);
                result.uv = input.texcoord.xy;
                return result;
            }

            fixed4 frag(ShadowVertex input) : SV_Target
            {
                float2 inset = max(abs(input.uv * 2.0 - 1.0) - float2(0.59, 0.55), 0.0);
                float distance = length(inset / float2(0.39, 0.43));
                float shadow = (1.0 - smoothstep(0.0, 1.0, distance)) * 0.51;
                return fixed4(0.025, 0.065, 0.065, shadow);
            }
            ENDCG
        }
    }
}
