Shader "百鸟卷/卷中记忆"
{
    Properties
    {
        [PerRendererData] _MainTex ("画作", 2D) = "white" {}
        _Crop ("画幅 UV", Vector) = (0,0,1,1)
        _State ("整卷复色", Float) = 0
        _Thumbnail ("缩略画片", Float) = 0
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
            #pragma multi_compile_local _ UNITY_UI_CLIP_RECT
            #include "UnityCG.cginc"
            #include "UnityUI.cginc"

            struct Input { float4 vertex:POSITION; float2 uv:TEXCOORD0; fixed4 tint:COLOR; };
            struct Output { float4 vertex:SV_POSITION; float2 uv:TEXCOORD0; fixed4 tint:COLOR; float2 position:TEXCOORD1; };
            sampler2D _MainTex;
            float4 _Crop, _ClipRect;
            float4 _RestoredSpots[10];
            int _RestoredCount;
            float _State, _Thumbnail;

            Output vert(Input input)
            {
                Output o;
                o.vertex=UnityObjectToClipPos(input.vertex);
                o.uv=input.uv;
                o.tint=input.tint;
                o.position=input.vertex.xy;
                return o;
            }

            fixed4 frag(Output input):SV_Target
            {
                float2 uv=_Crop.xy+input.uv*_Crop.zw;
                fixed4 paint=tex2D(_MainTex,uv);
                if(_Thumbnail>.5)
                {
                    float2 spread=fwidth(uv)*.29;
                    paint=(tex2D(_MainTex,uv+spread)+tex2D(_MainTex,uv-spread)+
                        tex2D(_MainTex,uv+float2(spread.x,-spread.y))+
                        tex2D(_MainTex,uv+float2(-spread.x,spread.y)))*.25;
                }
                float grey=dot(paint.rgb,float3(.30,.59,.11));
                float3 ink=lerp(float3(.23,.29,.31),float3(.94,.91,.81),grey);
                float colorAmount=_State;
                for(int i=0;i<_RestoredCount;i++)
                {
                    float4 spot=_RestoredSpots[i];
                    float2 offset=(input.uv-spot.xy)/spot.zw;
                    colorAmount=max(colorAmount,1-smoothstep(.33,1,length(offset)));
                }
                #if defined(UNITY_UI_CLIP_RECT)
                paint.a *= UnityGet2DClipping(input.position,_ClipRect);
                #endif
                return fixed4(lerp(ink,paint.rgb,colorAmount),paint.a)*input.tint;
            }
            ENDCG
        }
    }
}
