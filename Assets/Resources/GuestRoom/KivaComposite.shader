Shader "Kiva/Lobby Wordmark Composite"
{
    Properties
    {
        [PerRendererData] _MainTex ("Lobby HDR", 2D) = "black" {}
        _EmissionMask ("Glyph Coverage", 2D) = "black" {}
    }

    SubShader
    {
        Tags { "Queue" = "Transparent" "RenderType" = "Transparent" }
        Cull Off ZWrite Off ZTest [unity_GUIZTestMode]
        Blend One OneMinusSrcAlpha
        ColorMask RGB

        Pass
        {
            CGPROGRAM
            #pragma vertex Vert
            #pragma fragment Frag
            #include "UnityCG.cginc"

            sampler2D _MainTex;
            sampler2D _EmissionMask;

            struct Attributes
            {
                float4 vertex : POSITION;
                float2 uv : TEXCOORD0;
                fixed4 color : COLOR;
            };

            struct Varyings
            {
                float4 position : SV_POSITION;
                float2 uv : TEXCOORD0;
                fixed alpha : TEXCOORD1;
            };

            Varyings Vert(Attributes input)
            {
                Varyings output;
                output.position = UnityObjectToClipPos(input.vertex);
                output.uv = input.uv;
                output.alpha = input.color.a;
                return output;
            }

            float4 Frag(Varyings input) : SV_Target
            {
                float3 hdr = tex2D(_MainTex, input.uv).rgb;
                float luminance = dot(hdr, float3(0.2126, 0.7152, 0.0722));
                float opacity = tex2D(_EmissionMask, input.uv).r * input.alpha;
                float light = saturate(luminance) * opacity;
                return float4(light.xxx, opacity);
            }
            ENDCG
        }
    }
}
