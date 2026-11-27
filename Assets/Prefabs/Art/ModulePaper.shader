Shader "Kiva/Module Paper"
{
    Properties
    {
        _Pigment ("Pigment", Color) = (0.95,0.94,0.86,1)
        _Fibres ("Paper fibres", 2D) = "gray" {}
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
            #pragma multi_compile _ _MAIN_LIGHT_SHADOWS _MAIN_LIGHT_SHADOWS_CASCADE _MAIN_LIGHT_SHADOWS_SCREEN
            #pragma multi_compile _ _ADDITIONAL_LIGHTS
            #pragma multi_compile _ _CLUSTER_LIGHT_LOOP
            #pragma multi_compile_fragment _ _SHADOWS_SOFT
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"

            TEXTURE2D(_Fibres);
            SAMPLER(sampler_Fibres);
            CBUFFER_START(UnityPerMaterial)
                half4 _Pigment;
            CBUFFER_END

            struct Attributes { float4 position : POSITION; float3 normal : NORMAL; float2 uv : TEXCOORD0; };
            struct Varyings
            {
                float4 clip : SV_POSITION;
                float3 world : TEXCOORD0;
                float3 normal : TEXCOORD1;
                float2 grid : TEXCOORD2;
            };

            Varyings Vert(Attributes input)
            {
                Varyings o;
                o.world = TransformObjectToWorld(input.position.xyz);
                o.clip = TransformWorldToHClip(o.world);
                o.normal = TransformObjectToWorldNormal(input.normal);
                float2 metres = float2(length(TransformObjectToWorldDir(float3(1,0,0), false)),
                                       length(TransformObjectToWorldDir(float3(0,1,0), false)));
                o.grid = input.uv * metres / 1.5;
                return o;
            }

            half4 Frag(Varyings i) : SV_Target
            {
                Light light = GetMainLight(TransformWorldToShadowCoord(i.world));
                half3 normal = normalize(i.normal);
                half facing = saturate(dot(normal, light.direction));
                half grain = SAMPLE_TEXTURE2D(_Fibres, sampler_Fibres, i.grid * .72).r;
                float2 cell = floor(i.grid);
                half checker = frac((cell.x + cell.y) * .5) * 2.0;
                // Keep the installation grid legible on the unlit side of an enclosed room.
                half3 lighting = half3(.62,.65,.68) + light.color * (.39 * facing * light.shadowAttenuation);
                #if defined(_ADDITIONAL_LIGHTS)
                    InputData inputData = (InputData)0;
                    inputData.positionWS = i.world;
                    inputData.normalizedScreenSpaceUV = GetNormalizedScreenSpaceUV(i.clip);
                    uint lightCount = GetAdditionalLightsCount();
                    LIGHT_LOOP_BEGIN(lightCount)
                        Light localLight = GetAdditionalLight(lightIndex, i.world);
                        lighting += localLight.color * localLight.distanceAttenuation * saturate(dot(normal, localLight.direction));
                    LIGHT_LOOP_END
                #endif
                return half4(_Pigment.rgb * lighting * (.96 + checker * .04 + (grain - .5) * .12), 1);
            }
            ENDHLSL
        }
        UsePass "Universal Render Pipeline/Lit/ShadowCaster"
        UsePass "Universal Render Pipeline/Lit/DepthOnly"
        UsePass "Universal Render Pipeline/Lit/DepthNormals"
    }
}
