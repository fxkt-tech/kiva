Shader "Kiva/Water and Glass"
{
    Properties
    {
        _Tint ("Transmission tint and opacity", Color) = (0.84,0.96,1,0.075)
        [Toggle(_FLOWING_WATER)] _FlowingWater ("Flowing water", Float) = 0
        [HideInInspector] _LocalMapping ("Moving block UVs", Float) = 0
    }
    SubShader
    {
        Tags { "RenderPipeline"="UniversalPipeline" "RenderType"="Transparent" "Queue"="Transparent" "DisableBatching"="True" }
        Pass
        {
            Name "Clear surface"
            Tags { "LightMode"="UniversalForwardOnly" }
            Blend One OneMinusSrcAlpha
            ZWrite Off
            Cull Back
            HLSLPROGRAM
            #pragma target 4.5
            #pragma vertex Vert
            #pragma fragment Frag
            #pragma shader_feature_local _FLOWING_WATER
            #pragma multi_compile_fragment _ _REFLECTION_PROBE_ATLAS
            #pragma multi_compile_fragment _ _REFLECTION_PROBE_BOX_PROJECTION
            // Transparent fragments must sample the shadow map at their own depth.
            #define _SURFACE_TYPE_TRANSPARENT 1
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/DeclareOpaqueTexture.hlsl"

            CBUFFER_START(UnityPerMaterial)
                half4 _Tint;
                float _LocalMapping;
            CBUFFER_END

            struct Attributes
            {
                float4 vertex : POSITION;
                float3 normal : NORMAL;
                float4 tangent : TANGENT;
                float2 uv : TEXCOORD0;
            };

            struct Varyings
            {
                float4 clip : SV_POSITION;
                float3 world : TEXCOORD0;
                half3 normal : TEXCOORD1;
                #if defined(_FLOWING_WATER)
                    float2 metres : TEXCOORD2;
                    half3 axisU : TEXCOORD3;
                    half3 axisV : TEXCOORD4;
                #endif
            };

            Varyings Vert(Attributes v)
            {
                Varyings o;
                o.world = TransformObjectToWorld(v.vertex.xyz);
                o.clip = TransformWorldToHClip(o.world);
                o.normal = TransformObjectToWorldNormal(v.normal);
                #if defined(_FLOWING_WATER)
                    if (_LocalMapping > 0.5)
                    {
                        VertexNormalInputs basis = GetVertexNormalInputs(v.normal, v.tangent);
                        o.axisU = basis.tangentWS;
                        o.axisV = basis.bitangentWS;
                        o.metres = v.uv * float2(length(TransformObjectToWorldDir(v.tangent.xyz, false)),
                            length(TransformObjectToWorldDir(cross(v.normal, v.tangent.xyz), false)));
                    }
                    else
                    {
                        o.axisU = abs(o.normal.x) > 0.5 ? float3(0,0,1) : float3(1,0,0);
                        o.axisV = abs(o.normal.y) > 0.5 ? float3(0,0,1) : float3(0,1,0);
                        o.metres = float2(dot(o.world,o.axisU), dot(o.world,o.axisV));
                    }
                #endif
                return o;
            }

            #if defined(_FLOWING_WATER)
            // Gradient of a wave whose crests bend as they pass through a second swell.
            float2 WaveSlope(float2 position, float2 wave, float2 swell, out float phase)
            {
                float bend = dot(position, swell);
                phase = dot(position, wave) + 1.35 * sin(bend);
                return cos(phase) * (wave + 1.35 * cos(bend) * swell);
            }
            #endif

            struct SurfaceShading
            {
                half3 normal;
                half3 view;
            };

            half3 SurfaceSpecular(SurfaceShading s, half3 direction)
            {
                #if defined(_FLOWING_WATER)
                    const half sharpness = 84;
                #else
                    const half sharpness = 180;
                #endif
                return pow(saturate(dot(s.normal, SafeNormalize(direction + s.view))), sharpness);
            }

            #include_with_pragmas "SurfaceLighting.hlsl"

            half4 Frag(Varyings i) : SV_Target
            {
                half3 normal = normalize(i.normal);
                half3 view = GetWorldSpaceNormalizeViewDir(i.world);
                float2 uv = GetNormalizedScreenSpaceUV(i.clip);
                half3 distortion = 0;
                #if defined(_FLOWING_WATER)
                    float2 broad = i.metres + _Time.y * float2(-0.18, 0.29);
                    float2 fine = i.metres + _Time.y * float2(0.12, 0.37);
                    float longPhase, shortPhase;
                    float2 slope = 0.023 * WaveSlope(broad, float2(2.8,7.1), float2(2.4,-1.7), longPhase)
                                 + 0.009 * WaveSlope(fine, float2(-5.8,12.6), float2(3.4,2.6), shortPhase);
                    float3 tilt = slope.x * i.axisU + slope.y * i.axisV;
                    normal = normalize(normal + tilt);
                    float2 offset = TransformWorldToViewDir(tilt, false).xy * float2(1,_ProjectionParams.x);
                    offset *= 0.045 / max(1.0, distance(i.world, GetCameraPositionWS()) * 0.16);
                    // Apply the change in background colour so transparent layers already
                    // drawn behind this face still contribute through the water.
                    distortion = SampleSceneColor(saturate(uv + offset)) - SampleSceneColor(uv);
                    float ribbon = abs(sin(longPhase + 0.33 * sin(shortPhase)));
                    half crest = 1.0 - smoothstep(0.02, 0.20 + fwidth(longPhase), ribbon);
                    half glimmer = crest * smoothstep(0.06,0.90,sin(shortPhase * 0.50));
                    half3 ripple = half3(0.13,0.32,0.34) * (glimmer + 0.24 * (0.3 + slope.y));
                #endif
                half fresnel = 0.028h + 0.72h * Pow4(1.0h - saturate(dot(normal,view)));
                half3 reflection = GlossyEnvironmentReflection(reflect(-view,normal), i.world, 0.09h, 1.0h, uv);
                SurfaceShading surface;
                surface.normal = normal;
                surface.view = view;
                half3 diffuse, highlight;
                SurfaceDirectLighting(i.world, uv, surface, diffuse, highlight);
                #if defined(_FLOWING_WATER)
                    highlight += ripple * (0.43 * diffuse + 0.30 * SampleSH(normal));
                #endif
                half tintWeight = (1.0h - fresnel) * _Tint.a;
                half alpha = fresnel + tintWeight;
                return half4(_Tint.rgb * tintWeight + reflection * fresnel + highlight + distortion * (1.0h - alpha), alpha);
            }
            ENDHLSL
        }
    }
}
