Shader "Kiva/Valley Maple Bough"
{
    Properties {
        _BaseMap ("Foliage spray or bark strokes", 2D) = "white" {}
        _IsBark ("Bark", Float) = 0
        _Cutoff ("Leaf cutoff", Range(0,1)) = .42
    }
    SubShader {
        Tags { "RenderPipeline"="UniversalPipeline" "RenderType"="TransparentCutout" "Queue"="AlphaTest" }
        Cull Off
        HLSLINCLUDE
        #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"
        TEXTURE2D(_BaseMap); SAMPLER(sampler_BaseMap);
        CBUFFER_START(UnityPerMaterial)
            float _Cutoff, _IsBark;
        CBUFFER_END
        struct Attributes {float3 p:POSITION; half3 normal:NORMAL; float2 uv:TEXCOORD0; half4 color:COLOR;};
        struct Varyings {
            float4 clip:SV_POSITION;
            float3 world:TEXCOORD0;
            half3 normal:TEXCOORD1;
            float2 uv:TEXCOORD2;
            half3 paint:COLOR;
        };
        Varyings Vert(Attributes a){
            Varyings o;
            o.world=TransformObjectToWorld(a.p); o.clip=TransformWorldToHClip(o.world);
            o.normal=TransformObjectToWorldNormal(a.normal); o.uv=a.uv; o.paint=a.color.rgb;
            return o;
        }
        half4 Pigment(float2 uv){
            half4 dab=SAMPLE_TEXTURE2D(_BaseMap,sampler_BaseMap,uv);
            clip(dab.a-_Cutoff);
            return dab;
        }
        ENDHLSL
        Pass {
            Name "Paint" Tags { "LightMode"="UniversalForwardOnly" }
            HLSLPROGRAM
            #pragma vertex Vert
            #pragma fragment Frag
            #pragma multi_compile _ _MAIN_LIGHT_SHADOWS _MAIN_LIGHT_SHADOWS_CASCADE _MAIN_LIGHT_SHADOWS_SCREEN
            #pragma multi_compile_fragment _ _SHADOWS_SOFT _SHADOWS_SOFT_LOW _SHADOWS_SOFT_MEDIUM _SHADOWS_SOFT_HIGH
            half4 Frag(Varyings i):SV_Target{
                half4 dab=Pigment(i.uv);
                #if defined(_MAIN_LIGHT_SHADOWS_SCREEN)
                    float4 coord=ComputeScreenPos(TransformWorldToHClip(i.world));
                #else
                    float4 coord=TransformWorldToShadowCoord(i.world);
                #endif
                Light sun=GetMainLight(coord);
                half facing=dot(normalize(i.normal),sun.direction);
                half lit=saturate(.42+.58*facing)*lerp(.40,1,sun.shadowAttenuation);
                if(_IsBark>.5){
                    half amount=saturate(lit*.74+i.paint.r*.22+dab.g*.12);
                    half3 bark=lerp(half3(.083,.050,.088),half3(.54,.32,.27),smoothstep(.19,.78,amount));
                    return half4(bark*lerp(.78,1.18,dab.r),1);
                }
                half amount=saturate(lit*.54+i.paint.g*.23+i.paint.r*.14+(dab.r-.58)*.52);
                half3 berry=lerp(half3(.15,.022,.077),half3(.31,.031,.105),i.paint.r);
                half3 rose=lerp(half3(.57,.047,.15),half3(.82,.09,.145),i.paint.r);
                half3 coral=lerp(half3(.95,.15,.22),half3(1.17,.31,.17),i.paint.b);
                half3 color=lerp(berry,rose,smoothstep(.19,.51,amount));
                color=lerp(color,coral,smoothstep(.58,.88,amount));
                return half4(color,1);
            }
            ENDHLSL
        }
        Pass {
            Name "Leaf shadows" Tags { "LightMode"="ShadowCaster" }
            ColorMask 0 ZWrite On
            HLSLPROGRAM
            #pragma vertex ShadowVert
            #pragma fragment Depth
            #pragma multi_compile_vertex _ _CASTING_PUNCTUAL_LIGHT_SHADOW
            float3 _LightDirection, _LightPosition;
            Varyings ShadowVert(Attributes a){
                Varyings o=Vert(a);
                #if _CASTING_PUNCTUAL_LIGHT_SHADOW
                    float3 direction=normalize(_LightPosition-o.world);
                #else
                    float3 direction=_LightDirection;
                #endif
                o.clip=ApplyShadowClamping(TransformWorldToHClip(ApplyShadowBias(o.world,o.normal,direction)));
                return o;
            }
            half4 Depth(Varyings i):SV_Target{Pigment(i.uv);return 0;}
            ENDHLSL
        }
        Pass {
            Name "Leaf depth" Tags { "LightMode"="DepthOnly" }
            ColorMask R ZWrite On
            HLSLPROGRAM
            #pragma vertex Vert
            #pragma fragment Depth
            half Depth(Varyings i):SV_Target{Pigment(i.uv);return i.clip.z;}
            ENDHLSL
        }
        Pass {
            Name "Leaf normals" Tags { "LightMode"="DepthNormalsOnly" }
            ZWrite On
            HLSLPROGRAM
            #pragma vertex Vert
            #pragma fragment Normals
            #pragma multi_compile_fragment _ _GBUFFER_NORMALS_OCT
            half4 Normals(Varyings i):SV_Target{
                Pigment(i.uv);
                half3 normal=normalize(i.normal);
                #if defined(_GBUFFER_NORMALS_OCT)
                    return half4(PackFloat2To888(saturate(PackNormalOctQuadEncode(normal)*.5+.5)),0);
                #else
                    return half4(normal,0);
                #endif
            }
            ENDHLSL
        }
    }
}
