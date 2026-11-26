Shader "Kiva/Lobby Wordmark Emission"
{
    Properties { _MainTex ("Glyph Mask", 2D) = "black" {} }

    SubShader
    {
        Tags { "RenderPipeline" = "HDRenderPipeline" }
        Pass
        {
            Name "Emission"
            Cull Off ZWrite Off ZTest Always

            HLSLPROGRAM
            #pragma target 4.5
            #pragma vertex Vert
            #pragma fragment Frag
            #include "Packages/com.unity.render-pipelines.core/ShaderLibrary/Common.hlsl"

            TEXTURE2D(_MainTex);
            SAMPLER(sampler_MainTex);

            struct Varyings
            {
                float4 position : SV_POSITION;
                float2 uv : TEXCOORD0;
            };

            Varyings Vert(uint vertexID : SV_VertexID)
            {
                Varyings output;
                output.position = GetFullScreenTriangleVertexPosition(vertexID);
                output.uv = GetFullScreenTriangleTexCoord(vertexID);
                return output;
            }

            float4 Frag(Varyings input) : SV_Target
            {
                float luminance = 4.0 * SAMPLE_TEXTURE2D(_MainTex, sampler_MainTex, input.uv).r;
                return float4(luminance.xxx, 1.0);
            }
            ENDHLSL
        }
    }
}
