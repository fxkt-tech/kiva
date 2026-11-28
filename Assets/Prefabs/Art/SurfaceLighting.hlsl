#ifndef KIVA_SURFACE_LIGHTING_INCLUDED
#define KIVA_SURFACE_LIGHTING_INCLUDED

#pragma multi_compile _ _MAIN_LIGHT_SHADOWS _MAIN_LIGHT_SHADOWS_CASCADE _MAIN_LIGHT_SHADOWS_SCREEN
#pragma multi_compile _ _ADDITIONAL_LIGHTS
#pragma multi_compile _ _CLUSTER_LIGHT_LOOP
#pragma multi_compile_fragment _ _ADDITIONAL_LIGHT_SHADOWS
#pragma multi_compile_fragment _ _SHADOWS_SOFT _SHADOWS_SOFT_LOW _SHADOWS_SOFT_MEDIUM _SHADOWS_SOFT_HIGH

// Each shader supplies its finish's specular response. Visibility belongs to
// the individual light, so an occluder only removes that light's contribution.
void AddSurfaceLight(SurfaceShading surface, Light light, inout half3 diffuse, inout half3 specular)
{
    half3 incident = light.color * light.distanceAttenuation * light.shadowAttenuation
                   * saturate(dot(surface.normal, light.direction));
    diffuse += incident;
    specular += incident * SurfaceSpecular(surface, light.direction);
}

void SurfaceDirectLighting(float3 positionWS, float2 screenUV, SurfaceShading surface,
                           out half3 diffuse, out half3 specular)
{
    diffuse = 0;
    specular = 0;
    const half4 realtimeMask = half4(1, 1, 1, 1);
    Light sun = GetMainLight(TransformWorldToShadowCoord(positionWS), positionWS, realtimeMask);
    AddSurfaceLight(surface, sun, diffuse, specular);
    #if defined(_ADDITIONAL_LIGHTS)
        #if USE_CLUSTER_LIGHT_LOOP
            [loop] for (uint lightIndex = 0; lightIndex < min(URP_FP_DIRECTIONAL_LIGHTS_COUNT, MAX_VISIBLE_LIGHTS); ++lightIndex)
            {
                CLUSTER_LIGHT_LOOP_SUBTRACTIVE_LIGHT_CHECK
                AddSurfaceLight(surface, GetAdditionalLight(lightIndex, positionWS, realtimeMask), diffuse, specular);
            }
        #endif
        // URP's clustered loop reads these two fields from inputData.
        InputData inputData = (InputData)0;
        inputData.positionWS = positionWS;
        inputData.normalizedScreenSpaceUV = screenUV;
        uint lightCount = (uint)GetAdditionalLightsCount();
        LIGHT_LOOP_BEGIN(lightCount)
            AddSurfaceLight(surface, GetAdditionalLight(lightIndex, positionWS, realtimeMask), diffuse, specular);
        LIGHT_LOOP_END
    #endif
}

#endif
