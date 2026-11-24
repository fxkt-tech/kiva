Shader "百鸟卷/剧情画卷"
{
    Properties
    {
        [PerRendererData] _MainTex ("画幅", 2D) = "white" {}
        _Crop ("卷首与卷尾 UV", Vector) = (0,0,1,1)
        _Bird ("栖鸟 UV 及范围", Vector) = (0,0,0,0)
        _Opening ("展卷", Range(0,1)) = 1
        _Recovery ("复色", Range(0,1)) = 0
        _PaperEdge ("纸边衔接", Range(0,1)) = 0
        _Water ("水纹", Range(0,1)) = 0
        _Dimming ("卷签退光", Range(0,1)) = 0
        _Clock ("动效时间", Float) = 0
    }

    SubShader
    {
        Tags { "Queue"="Transparent" "RenderType"="Transparent" "IgnoreProjector"="True" }
        Cull Off Lighting Off ZWrite Off ZTest [unity_GUIZTestMode]
        Blend SrcAlpha OneMinusSrcAlpha

        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            struct VertexData { float4 vertex : POSITION; float2 uv : TEXCOORD0; fixed4 tint : COLOR; };
            struct FragmentData { float4 vertex : SV_POSITION; float2 uv : TEXCOORD0; fixed4 tint : COLOR; };
            sampler2D _MainTex;
            float4 _Crop, _Bird;
            float _Opening, _Recovery, _PaperEdge, _Water, _Clock, _Dimming;

            FragmentData vert(VertexData vertex)
            {
                FragmentData output;
                output.vertex = UnityObjectToClipPos(vertex.vertex);
                output.uv = vertex.uv;
                output.tint = vertex.tint;
                return output;
            }

            fixed4 frag(FragmentData input) : SV_Target
            {
                float2 local = input.uv;
                float2 delta = (local - _Bird.xy) / max(_Bird.zw, .0001);
                float influence = 1 - smoothstep(.14, .96, length(delta));
                local.y += influence * sin(_Clock * 5.2) * .0024;
                float2 uv = _Crop.xy + saturate(local) * _Crop.zw;
                float3 color = tex2D(_MainTex, uv).rgb;

                float water = smoothstep(.32,.65,local.x) * (1-smoothstep(.28,.56,local.y));
                float ripples = max(0, sin(local.y * 350 + local.x * 25 - _Clock * 1.8));
                color += water * pow(ripples,8) * _Water * float3(.019,.031,.033);

                float restore = _Recovery * exp2(-1.65 * dot(delta,delta) * .10);
                float luminance = dot(color,float3(.29,.58,.13));
                color = lerp(color, luminance * float3(.99,.95,.91), restore * .42);
                color += restore * influence * float3(.065,.036,.012);

                float roll = smoothstep(.001,.021,_Opening-abs(input.uv.x-.5)*.994);
                float edge = min(min(input.uv.x,1-input.uv.x),min(input.uv.y,1-input.uv.y));
                color *= 1 - _Dimming * .44 * smoothstep(.13,.54,luminance) * smoothstep(0,.093,edge);
                float alpha = lerp(1,smoothstep(0,.014,edge),_PaperEdge);
                float3 closed = lerp(float3(.07,.044,.065),float3(.21,.145,.115),abs(local.y-.5));
                return fixed4(lerp(closed,color,roll),alpha) * input.tint;
            }
            ENDCG
        }
    }
}
