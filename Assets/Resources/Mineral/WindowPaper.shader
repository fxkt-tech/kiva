Shader "Kiva/Transmitted Window Paper"
{
    Properties
    {
        [MainTexture] _BaseMap ("Paper fibres", 2D) = "white" {}
        _ThicknessMap ("Pane thickness", 2D) = "gray" {}
        [HDR] _Transmission ("Afternoon light", Color) = (2.6,1.95,1.35,1)
    }
    SubShader
    {
        Tags { "RenderPipeline"="UniversalPipeline" "RenderType"="Transparent" "Queue"="Transparent" }
        Pass
        {
            Cull Off ZWrite Off
            Blend SrcAlpha OneMinusSrcAlpha
            HLSLPROGRAM
            #pragma vertex Vert
            #pragma fragment Frag
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            TEXTURE2D(_BaseMap); SAMPLER(sampler_BaseMap);
            TEXTURE2D(_ThicknessMap); SAMPLER(sampler_ThicknessMap);
            CBUFFER_START(UnityPerMaterial)
                float4 _BaseMap_ST;
                half4 _Transmission;
            CBUFFER_END

            struct Varyings { float4 position : SV_POSITION; float2 uv : TEXCOORD0; float2 fibreUV : TEXCOORD1; };
            Varyings Vert(float3 position : POSITION, float2 uv : TEXCOORD0)
            {
                Varyings output;
                output.position = TransformObjectToHClip(position);
                output.uv = TRANSFORM_TEX(uv, _BaseMap);
                output.fibreUV = uv;
                return output;
            }
            half4 Frag(Varyings input) : SV_Target
            {
                half thickness = SAMPLE_TEXTURE2D(_ThicknessMap, sampler_ThicknessMap, input.uv).r;
                half3 paper = SAMPLE_TEXTURE2D(_BaseMap, sampler_BaseMap, input.fibreUV).rgb;
                return half4(_Transmission.rgb * lerp(1.35h, 0.58h, thickness) * pow(paper, 1.6h), .018h + thickness * .052h);
            }
            ENDHLSL
        }
    }
}
