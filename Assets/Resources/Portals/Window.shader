Shader "Kiva/PortalWindow"
{
    Properties
    {
        _MainTex ("Exit view", 2D) = "black" {}
        _ClampNearPlane ("Keep player aperture visible up close", Float) = 1
    }
    SubShader
    {
        Tags { "RenderType"="Opaque" "RenderPipeline"="UniversalPipeline" }
        Pass
        {
            Cull Off
            HLSLPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            TEXTURE2D(_MainTex); SAMPLER(sampler_MainTex);
            float _ClampNearPlane;
            struct Varyings { float4 pos : SV_POSITION; float4 screen : TEXCOORD0; };
            Varyings vert(float4 vertex : POSITION)
            {
                Varyings o;
                o.pos = TransformObjectToHClip(vertex.xyz);
                // Only the player's near plane needs this. Nested views must clip at the exit.
                if (_ClampNearPlane > 0.5)
                {
                    #if UNITY_REVERSED_Z
                        o.pos.z = min(o.pos.z, o.pos.w * UNITY_NEAR_CLIP_VALUE);
                    #else
                        o.pos.z = max(o.pos.z, o.pos.w * UNITY_NEAR_CLIP_VALUE);
                    #endif
                }
                o.screen = ComputeScreenPos(o.pos);
                return o;
            }
            half4 frag(Varyings i) : SV_Target
            {
                return SAMPLE_TEXTURE2D(_MainTex, sampler_MainTex, i.screen.xy / i.screen.w);
            }
            ENDHLSL
        }
    }
}
