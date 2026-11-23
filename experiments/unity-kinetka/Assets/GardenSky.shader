Shader "Mira/Garden Sky"
{
    Properties
    {
        _Top ("Top", Color) = (0.33, 0.46, 0.43, 1)
        _Bottom ("Bottom", Color) = (0.14, 0.24, 0.22, 1)
    }
    SubShader
    {
        Tags { "Queue" = "Background" "RenderType" = "Background" }
        Cull Off
        ZWrite Off

        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            fixed4 _Top;
            fixed4 _Bottom;

            struct SkyVertex
            {
                float4 pixel : SV_POSITION;
                float4 screen : TEXCOORD0;
            };

            SkyVertex vert(float4 vertex : POSITION)
            {
                SkyVertex result;
                result.pixel = UnityObjectToClipPos(vertex);
                result.screen = ComputeScreenPos(result.pixel);
                return result;
            }

            fixed4 frag(SkyVertex input) : SV_Target
            {
                float height = saturate(input.screen.y / input.screen.w);
                return lerp(_Bottom, _Top, smoothstep(0.0, 1.0, height));
            }
            ENDCG
        }
    }
}
