#ifndef MIRA_WATER_LAB_REFRACTION
#define MIRA_WATER_LAB_REFRACTION

#include "UnityCG.cginc"

float4 WaterLabRefractionOffsets(float3 worldNormal, float strength)
{
    float3 viewNormal = mul((float3x3)UNITY_MATRIX_V, worldNormal);
    // Convert a screen displacement using the same UV conventions as each texture.
    float4 clipOffset = float4(viewNormal.xy * (2.0 * strength), 0.0, 1.0);
    return float4(ComputeScreenPos(clipOffset).xy, ComputeGrabScreenPos(clipOffset).xy) - 0.5;
}

#endif
