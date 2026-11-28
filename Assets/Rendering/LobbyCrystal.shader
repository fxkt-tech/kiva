Shader "Kiva/UI/Crystal Plaque"
{
    Properties
    {
        [PerRendererData] _MainTex ("Crystal face", 2D) = "white" {}
    }
    SubShader
    {
        Tags { "Queue"="Transparent" "RenderType"="Transparent" }
        Cull Off ZWrite Off ZTest [unity_GUIZTestMode]
        Blend SrcAlpha OneMinusSrcAlpha
        Pass
        {
            CGPROGRAM
            #pragma vertex vert_img
            #pragma fragment Frag
            #include "UnityCG.cginc"

            sampler2D _MainTex;

            half4 Frag(v2f_img i) : SV_Target
            {
                float2 p = abs(i.uv - 0.5);
                // The diagonal cuts are also planes of the bevel.
                float2 edge = 0.48 - p;
                float diagonal = (0.82 - p.x - p.y) * 0.7071;
                float distance = min(min(edge.x, edge.y), diagonal);
                float aa = max(fwidth(distance), 0.0001);
                half alpha = saturate(distance / aa + 0.5);
                float bevel = 1.0 - smoothstep(0.033 - aa, 0.033 + aa, distance);
                half3 glass = tex2D(_MainTex, i.uv * 0.92 + 0.04).rgb;
                half3 reflection = i.uv.y > 0.5 ? half3(0.86,0.74,1.0) : half3(0.29,0.23,0.53);
                if (edge.x < edge.y && edge.x < diagonal)
                    reflection = i.uv.x < 0.5 ? half3(0.72,0.81,1.0) : half3(0.48,0.36,0.67);
                glass = lerp(glass, reflection, bevel * 0.66);
                float glint = 1.0 - smoothstep(aa, aa * 2.0, abs(distance - 0.033));
                glass = lerp(glass, half3(1.0,0.86,0.71), glint * 0.60);
                return half4(glass, alpha);
            }
            ENDCG
        }
    }
}
