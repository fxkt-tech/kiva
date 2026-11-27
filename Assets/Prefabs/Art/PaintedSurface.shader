Shader "Kiva/Painted Surface"
{
    Properties
    {
        [MainTexture] _BaseMap ("Gouache colour", 2D) = "white" {}
        _TilesPerMetre ("Texture repeats per metre", Float) = 0.3
        [KeywordEnum(Plaster, Rock, Metal, Crystal)] _Finish ("Surface finish", Float) = 0
        _FinishMap ("Metal roughness / crystal inclusions", 2D) = "gray" {}
        _MetalHighlight ("Metal highlight strength", Range(0, 3)) = 1
        _CrystalEdgeColor ("Crystal edge tint", Color) = (0.90, 0.73, 1.0, 1)
        [Normal] _BumpMap ("Surface normal", 2D) = "bump" {}
        _HeightMap ("Rock height", 2D) = "gray" {}
        _Depth ("Parallax depth", Range(0,0.08)) = 0.035
        [HideInInspector] _LocalMapping ("Moving block UVs", Float) = 0
    }
    SubShader
    {
        Tags { "RenderPipeline"="UniversalPipeline" "RenderType"="Opaque" "Queue"="Geometry" "DisableBatching"="True" }
        Pass
        {
            Name "Forward"
            Tags { "LightMode"="UniversalForwardOnly" }
            HLSLPROGRAM
            #pragma target 4.5
            #pragma vertex Vert
            #pragma fragment Frag
            #pragma shader_feature_local_fragment _ _FINISH_PLASTER _FINISH_ROCK _FINISH_METAL _FINISH_CRYSTAL _ROCK_RELIEF
            #pragma multi_compile_fragment _ _REFLECTION_PROBE_ATLAS
            #pragma multi_compile_fragment _ _REFLECTION_PROBE_BOX_PROJECTION
            #pragma multi_compile _ _MAIN_LIGHT_SHADOWS _MAIN_LIGHT_SHADOWS_CASCADE _MAIN_LIGHT_SHADOWS_SCREEN
            #pragma multi_compile _ _ADDITIONAL_LIGHTS
            #pragma multi_compile _ _CLUSTER_LIGHT_LOOP
            #pragma multi_compile_fragment _ _SHADOWS_SOFT _SHADOWS_SOFT_LOW _SHADOWS_SOFT_MEDIUM _SHADOWS_SOFT_HIGH
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"

            #if defined(_FINISH_METAL) || defined(_FINISH_CRYSTAL)
                #define POLISHED_SURFACE
                #include "Packages/com.unity.render-pipelines.core/ShaderLibrary/BSDF.hlsl"
            #endif

            TEXTURE2D(_BaseMap);
            SAMPLER(sampler_BaseMap);
            TEXTURE2D(_BumpMap);
            TEXTURE2D(_HeightMap);
            TEXTURE2D(_FinishMap);
            CBUFFER_START(UnityPerMaterial)
                float _TilesPerMetre;
                half _MetalHighlight;
                half4 _CrystalEdgeColor;
                float _Depth;
                float _LocalMapping;
            CBUFFER_END

            struct Attributes
            {
                float4 position : POSITION;
                float3 normal : NORMAL;
                float4 tangent : TANGENT;
                float2 uv : TEXCOORD0;
            };
            struct Varyings
            {
                float4 clip : SV_POSITION;
                float3 world : TEXCOORD0;
                half3 normal : TEXCOORD1;
                half3 tangent : TEXCOORD2;
                half3 bitangent : TEXCOORD3;
                float2 uv : TEXCOORD4;
            };

            Varyings Vert(Attributes input)
            {
                Varyings o;
                o.world = TransformObjectToWorld(input.position.xyz);
                o.clip = TransformWorldToHClip(o.world);
                VertexNormalInputs basis = GetVertexNormalInputs(input.normal, input.tangent);
                o.normal = basis.normalWS;
                o.tangent = basis.tangentWS;
                o.bitangent = basis.bitangentWS;
                // All coplanar fixed faces share an uninterrupted painting. Loose bricks
                // use their own UV frame so the colour, normal and height travel together.
                float2 metres = float2(dot(o.world, basis.tangentWS), dot(o.world, basis.bitangentWS));
                if (_LocalMapping > 0.5)
                    metres = input.uv * float2(length(TransformObjectToWorldDir(float3(1,0,0), false)),
                                               length(TransformObjectToWorldDir(float3(0,1,0), false)));
                o.uv = metres * _TilesPerMetre;
                return o;
            }

            #if defined(POLISHED_SURFACE)
            half3 Highlight(Light light, BRDFData brdf, half3 normal, half3 tangent, half3 view)
            {
                half nDotL = saturate(dot(normal, light.direction));
                #if defined(_FINISH_METAL)
                    // The brush direction stretches the specular lobe along the grain.
                    half3 t = normalize(tangent - normal * dot(normal, tangent));
                    half3 b = cross(normal, t);
                    half3 h = SafeNormalize(view + light.direction);
                    float response = DV_SmithJointGGXAniso(dot(t,h), dot(b,h), saturate(dot(normal,h)),
                        dot(t,view), dot(b,view), max(0.02h,dot(normal,view)),
                        dot(t,light.direction), dot(b,light.direction), nDotL,
                        max(0.19h,brdf.roughness * 2.4h), max(0.065h,brdf.roughness * 0.65h));
                    response = min(response, 10.0) * _MetalHighlight;
                #else
                    float response = min(DirectBRDFSpecular(brdf, normal, light.direction, view), 10.0);
                #endif
                return brdf.specular * response * nDotL * light.color
                    * light.distanceAttenuation * light.shadowAttenuation;
            }
            #endif

            half4 Frag(Varyings i) : SV_Target
            {
                float2 uv = i.uv;
                #if !defined(POLISHED_SURFACE)
                    uv += 0.125 * sin(uv.yx * float2(1.47, 1.83) + sin(uv * 2.31));
                #endif
                half3 normal = normalize(i.normal);
                half3 view = GetWorldSpaceNormalizeViewDir(i.world);
                float2 viewAlongFace = float2(dot(view, i.tangent), dot(view, i.bitangent));
                #if defined(_FINISH_ROCK) || defined(_ROCK_RELIEF)
                    float2 slope = _Depth * viewAlongFace / (saturate(dot(view, normal)) + 0.42);
                    float height = SAMPLE_TEXTURE2D(_HeightMap, sampler_BaseMap, uv).r - 0.5;
                    height = SAMPLE_TEXTURE2D(_HeightMap, sampler_BaseMap, uv + slope * height).r - 0.5;
                    uv += slope * height;
                #endif
                #if defined(_FINISH_ROCK) || defined(_ROCK_RELIEF) || defined(POLISHED_SURFACE)
                    half3 bump = UnpackNormal(SAMPLE_TEXTURE2D(_BumpMap, sampler_BaseMap, uv));
                    normal = normalize(i.tangent * bump.x + i.bitangent * bump.y + normal * bump.z);
                #endif
                half3 pigment = SAMPLE_TEXTURE2D(_BaseMap, sampler_BaseMap, uv).rgb;
                Light sun = GetMainLight(TransformWorldToShadowCoord(i.world));
                half facing = saturate(dot(normal, sun.direction));
                half3 lighting = half3(0.52, 0.56, 0.67) + sun.color * (0.54 * facing * sun.shadowAttenuation);
                #if defined(POLISHED_SURFACE)
                    half mask = SAMPLE_TEXTURE2D(_FinishMap, sampler_BaseMap, uv).r;
                    half fresnel = Pow4(1.0h - saturate(dot(normal, view)));
                    #if defined(_FINISH_METAL)
                        half metallic = 0.88h;
                        half smoothness = 1.0h - mask;
                    #else
                        // Cleavage lines behind a polished facet shift with view and facet tilt.
                        float2 innerUV = uv + (viewAlongFace + 0.18h * bump.xy) * _Depth * (0.3h + mask);
                        half3 interior = SAMPLE_TEXTURE2D(_BaseMap, sampler_BaseMap, innerUV).rgb;
                        half2 strata = SAMPLE_TEXTURE2D(_FinishMap, sampler_BaseMap, innerUV).rg;
                        pigment = lerp(pigment, interior * half3(0.94h, 0.79h, 1.04h), 0.30h + 0.34h * strata.r);
                        half metallic = 0.0h;
                        half smoothness = 0.64h + 0.17h * mask;
                    #endif
                    BRDFData brdf;
                    half alpha = 1;
                    half3 reflectance = pigment;
                    #if defined(_FINISH_METAL)
                        // Dark brush marks in the colour map still reflect light as bare steel.
                        reflectance = lerp(pigment, half3(0.55h, 0.57h, 0.59h), 0.68h);
                    #endif
                    InitializeBRDFData(reflectance, metallic, half3(0,0,0), smoothness, alpha, brdf);
                    half3 reflection = GlossyEnvironmentReflection(reflect(-view, normal), i.world,
                        brdf.perceptualRoughness, 1.0h, GetNormalizedScreenSpaceUV(i.clip));
                    half3 shine = reflection * EnvironmentBRDFSpecular(brdf, fresnel)
                                + Highlight(sun, brdf, normal, i.tangent, view);
                #endif
                #if defined(_ADDITIONAL_LIGHTS)
                    InputData inputData = (InputData)0;
                    inputData.positionWS = i.world;
                    inputData.normalizedScreenSpaceUV = GetNormalizedScreenSpaceUV(i.clip);
                    uint count = GetAdditionalLightsCount();
                    LIGHT_LOOP_BEGIN(count)
                        Light lamp = GetAdditionalLight(lightIndex, i.world);
                        lighting += lamp.color * lamp.distanceAttenuation * saturate(dot(normal, lamp.direction));
                        #if defined(POLISHED_SURFACE)
                            shine += Highlight(lamp, brdf, normal, i.tangent, view);
                        #endif
                    LIGHT_LOOP_END
                #endif
                #if defined(_FINISH_METAL)
                    return half4(pigment * lighting * 0.51h + shine, 1);
                #elif defined(_FINISH_CRYSTAL)
                    half3 glints = _CrystalEdgeColor.rgb * (0.23h * fresnel + strata.g * (0.10h + 0.18h * fresnel));
                    return half4(pigment * lighting + shine + glints, 1);
                #elif !defined(_FINISH_ROCK) && !defined(_ROCK_RELIEF)
                    // Preserve white-on-white brushwork even directly under a point light.
                    half3 highlights = max(lighting - 0.68h, 0.0h);
                    lighting = min(lighting, 0.68h) + highlights / (1.0h + highlights / 0.30h);
                #endif
                return half4(pigment * lighting, 1);
            }
            ENDHLSL
        }
        UsePass "Universal Render Pipeline/Lit/ShadowCaster"
        UsePass "Universal Render Pipeline/Lit/DepthOnly"
        UsePass "Universal Render Pipeline/Lit/DepthNormals"
    }
}
