Shader "Hidden/Mira/Water Lab Refraction Probe"
{
    SubShader
    {
        Cull Off
        Pass
        {
            CGPROGRAM
            #pragma vertex vert_img
            #pragma fragment frag
            #include "../WaterLabRefraction.cginc"

            float4 frag(v2f_img i) : SV_Target
            {
                float3 normal = normalize(float3(1.0, 1.0, 0.0));
                return 0.5 + WaterLabRefractionOffsets(normal, 0.25);
            }
            ENDCG
        }
    }
}
