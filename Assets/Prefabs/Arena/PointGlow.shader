Shader "Kiva/Point light glow"
{
    Properties
    {
        [HDR] _Color ("Light colour", Color) = (1, .78, .43, 1)
    }
    SubShader
    {
        Tags { "RenderPipeline"="UniversalPipeline" "RenderType"="Transparent" "Queue"="Transparent-10" "DisableBatching"="True" }
        Pass
        {
            Blend One OneMinusSrcAlpha
            ZWrite Off
            Cull Off
            HLSLPROGRAM
            #pragma vertex Vert
            #pragma fragment Frag
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"

            CBUFFER_START(UnityPerMaterial)
                half4 _Color;
            CBUFFER_END

            struct Varyings
            {
                float4 clip : SV_POSITION;
                float2 offset : TEXCOORD0;
            };

            Varyings Vert(float3 vertex : POSITION, float2 uv : TEXCOORD0)
            {
                Varyings o;
                float3 centre = TransformWorldToView(TransformObjectToWorld(float3(0,0,0)));
                float diameter = length(TransformObjectToWorldDir(float3(1,0,0), false));
                centre.xy += vertex.xy * diameter;
                o.clip = mul(UNITY_MATRIX_P, float4(centre, 1));
                o.offset = uv * 2 - 1;
                return o;
            }

            half4 Frag(Varyings i) : SV_Target
            {
                float radius = length(i.offset);
                half core = 1 - smoothstep(.28 - fwidth(radius), .28 + fwidth(radius), radius);
                half halo = .50 * pow(saturate(1 - radius), 3);
                half alpha = saturate(core + halo);
                return half4(lerp(_Color.rgb, half3(1, .96, .82), core) * alpha, alpha);
            }
            ENDHLSL
        }
    }
}
