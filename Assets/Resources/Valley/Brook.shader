Shader "Kiva/Valley Brook"
{
    Properties { _Brush ("Brush marks",2D)="gray"{} }
    SubShader
    {
        Tags { "RenderPipeline"="UniversalPipeline" "RenderType"="Opaque" }
        Pass
        {
            HLSLPROGRAM
            #pragma vertex Vert
            #pragma fragment Frag
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            TEXTURE2D(_Brush); SAMPLER(sampler_Brush);
            struct Varyings { float4 clip:SV_POSITION; float2 water:TEXCOORD0; };
            Varyings Vert(float3 p:POSITION){
                Varyings o;o.clip=TransformObjectToHClip(p);o.water=p.xz;return o;
            }
            half4 Frag(Varyings i):SV_Target{
                float2 uv=i.water*float2(.13,.65);
                uv.x+=.045*_Time.y;
                half wash=SAMPLE_TEXTURE2D(_Brush,sampler_Brush,uv).g;
                half3 water=lerp(half3(.025,.29,.36),half3(.16,.80,.78),smoothstep(.16,.63,wash));
                float ripple=sin(i.water.x*2.8+i.water.y*5+wash*7+_Time.y*1.3);
                water=lerp(water,half3(.78,.95,.84),smoothstep(.89,.98,ripple)*smoothstep(.43,.62,wash)*.8);
                return half4(water,1);
            }
            ENDHLSL
        }
    }
}
