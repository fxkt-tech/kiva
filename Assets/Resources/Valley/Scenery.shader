Shader "Kiva/Valley Scenery"
{
    Properties {
        [MainTexture] _MainTex ("Painting", 2D) = "white" {}
        _Panorama ("Distant landscape", Float) = 0
    }
    SubShader
    {
        Tags { "RenderPipeline"="UniversalPipeline" "RenderType"="TransparentCutout" "Queue"="AlphaTest" }
        Pass {
            Cull Off
            HLSLPROGRAM
            #pragma vertex Vert
            #pragma fragment Frag
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            #include "Packages/com.unity.render-pipelines.core/ShaderLibrary/Color.hlsl"
            #include "PaintedHorizon.hlsl"
            TEXTURE2D(_MainTex); SAMPLER(sampler_MainTex);
            CBUFFER_START(UnityPerMaterial)
            float _Panorama;
            CBUFFER_END
            struct Varyings { float4 clip:SV_POSITION; float2 uv:TEXCOORD0; float3 ray:TEXCOORD1; half4 tint:COLOR; };
            Varyings Vert(float3 p:POSITION,float2 uv:TEXCOORD0,half4 tint:COLOR) {
                Varyings o;
                float3 world=TransformObjectToWorld(p);
                o.clip=TransformWorldToHClip(world);
                o.ray=world-GetCameraPositionWS(); o.uv=uv; o.tint=tint;
                return o;
            }
            half4 Frag(Varyings i):SV_Target {
                float2 uv=i.uv;
                float3 dir=normalize(i.ray);
                if(_Panorama>.5){
                    uv=ValleyHorizonUV(dir);
                }
                half4 c=SAMPLE_TEXTURE2D(_MainTex,sampler_MainTex,uv);
                clip(c.a-.38);
                half3 paint=c.rgb*SRGBToLinear(i.tint.rgb);
                if(_Panorama>.5)paint=ValleySkyTint(paint,dir.y);
                return half4(paint,1);
            }
            ENDHLSL
        }
    }
}
