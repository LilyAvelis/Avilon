Shader "Hidden/Mira/Water Lab Rain Probe"
{
    Properties { _RainImpacts("Rain impact grid", 2D) = "black" {} }
    SubShader
    {
        Pass
        {
            ZTest Always Cull Off ZWrite Off
            CGPROGRAM
            #pragma target 3.0
            #pragma vertex vert_img
            #pragma fragment frag
            #include "UnityCG.cginc"
            #include "../WaterLabRainSurface.cginc"

            float4 _Probe;

            float4 frag(v2f_img i) : SV_Target
            {
                float rain = RainSurface(_Probe.xy, _Probe.w, _Probe.z).z;
                return float4(rain, 0, 0, 1);
            }
            ENDCG
        }
    }
}
