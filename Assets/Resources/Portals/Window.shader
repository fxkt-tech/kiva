Shader "Kiva/PortalWindow"
{
    Properties
    {
        _MainTex ("Exit view", 2D) = "black" {}
        _ClampNearPlane ("Keep player aperture visible up close", Float) = 1
    }
    SubShader
    {
        Tags { "RenderType"="Opaque" }
        Pass
        {
            Cull Off
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"
            sampler2D _MainTex;
            float _ClampNearPlane;
            struct Varyings { float4 pos : SV_POSITION; float4 screen : TEXCOORD0; };
            Varyings vert(float4 vertex : POSITION)
            {
                Varyings o;
                o.pos = UnityObjectToClipPos(vertex);
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
                return tex2Dproj(_MainTex, UNITY_PROJ_COORD(i.screen));
            }
            ENDCG
        }
    }
}
