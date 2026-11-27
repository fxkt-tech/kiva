Shader "Kiva/Valley Painted Surface"
{
    Properties
    {
        _Meadow ("Meadow", 2D) = "white" {}
        _Rock ("Limestone", 2D) = "white" {}
        _Landscape ("Mountain wash", 2D) = "white" {}
        _Maple ("Maple", 2D) = "black" {}
        _Brush ("Paint strokes", 2D) = "gray" {}
        _Arch ("Mossy arch", Float) = 0
        _Paper ("Portal paper", Float) = 0
        _Value ("Pigment variation", Float) = 1
        _Cell ("Grid spacing", Float) = 1.5
        _GlazedWest ("Glazed west wall", Float) = 0
        _BoundaryPaint ("Boundary scenery", Float) = 0
        _Portable ("Movable block", Float) = 0
        _Minimum ("Landscape origin", Vector) = (0,0,0,0)
        _Span ("Landscape size", Vector) = (1,1,1,0)
        [HideInInspector] _Cull ("Cull", Float) = 2
        [HideInInspector] _ZWrite ("ZWrite", Float) = 1
    }
    SubShader
    {
        Tags { "RenderPipeline"="UniversalPipeline" "RenderType"="Opaque" }
        Pass
        {
            Tags { "LightMode"="UniversalForwardOnly" }
            HLSLPROGRAM
            #pragma target 4.5
            #pragma multi_compile _ _ADDITIONAL_LIGHTS
            #pragma multi_compile _ _CLUSTER_LIGHT_LOOP
            #pragma multi_compile_fragment _ _ADDITIONAL_LIGHT_SHADOWS
            #pragma vertex Vert
            #pragma fragment Frag
            #pragma multi_compile _ _MAIN_LIGHT_SHADOWS _MAIN_LIGHT_SHADOWS_CASCADE _MAIN_LIGHT_SHADOWS_SCREEN
            #pragma multi_compile_fragment _ _SCREEN_SPACE_OCCLUSION
            #pragma multi_compile_fragment _ _SHADOWS_SOFT _SHADOWS_SOFT_LOW _SHADOWS_SOFT_MEDIUM _SHADOWS_SOFT_HIGH
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"
            #include "PaintedHorizon.hlsl"
            TEXTURE2D(_Meadow); SAMPLER(sampler_Meadow);
            TEXTURE2D(_Brush); SAMPLER(sampler_Brush);
            TEXTURE2D(_Rock); SAMPLER(sampler_Rock);
            TEXTURE2D(_Landscape); SAMPLER(sampler_Landscape);
            TEXTURE2D(_Maple); SAMPLER(sampler_Maple);
            CBUFFER_START(UnityPerMaterial)
            float4 _Minimum, _Span;
            float _Arch, _Paper, _Value, _Cell, _Portable, _BoundaryPaint, _GlazedWest;
            CBUFFER_END
            struct Varyings {
                float4 clip : SV_POSITION;
                float3 local : TEXCOORD0;
                float3 world : TEXCOORD1;
                half3 normal : TEXCOORD2;
                half3 localNormal : TEXCOORD3;
                float2 uv : TEXCOORD4;
                half3 archBands : COLOR;
            };
            Varyings Vert(float3 p:POSITION, float3 normal:NORMAL, float2 uv:TEXCOORD0, half4 bands:COLOR)
            {
                Varyings o;
                o.local=p; o.world=TransformObjectToWorld(p); o.clip=TransformWorldToHClip(o.world);
                o.normal=TransformObjectToWorldNormal(normal); o.localNormal=normal;
                o.uv=uv/.76; o.archBands=bands.rgb;
                return o;
            }
            half4 Frag(Varyings i):SV_Target
            {
                float3 n = normalize(i.localNormal);
                float3 t = saturate((i.local-_Minimum.xyz)/max(_Span.xyz,.01));
                float along=abs(n.x)>.5 ? i.local.z : i.local.x;
                float3 margin=min(t,1-t)*_Span.xyz;
                float perimeter=(1-smoothstep(1.51,2.25,abs(n.x)>.5 ? margin.x : margin.z))
                    *(1-abs(n.y))*(1-_Portable)*_BoundaryPaint;
                // Keep solid masonry below and beside the actual glazed aperture.
                if (_GlazedWest>.5 && abs(n.x)>.5 && i.local.x < _Minimum.x+2.0) perimeter=0;
                // On carved stone the wash crosses facet boundaries. On a brick the UVs travel with the mesh.
                float2 paintUV = _Arch > .5 ? float2(i.local.z + .36*i.local.x, i.local.y + .28*i.local.x) : i.uv;
                float2 strokeUV = float2(paintUV.x, paintUV.y - paintUV.x*lerp(.33,.08,abs(n.y))) * .28;
                half3 brush = SAMPLE_TEXTURE2D(_Brush,sampler_Brush,strokeUV).rgb;
                half3 stone=SAMPLE_TEXTURE2D(_Rock,sampler_Rock,paintUV*.17).rgb;
                half3 grass=SAMPLE_TEXTURE2D(_Meadow,sampler_Meadow,i.uv*.18).rgb*half3(.74,1.02,.88);
                float ridge=.39+.08*sin(along*.35+n.z)+.055*sin(along*.72)+.026*(stone.r-.45);
                float slope=t.y-ridge;
                float3 ray=normalize(i.world-GetCameraPositionWS());
                half3 distancePaint=SAMPLE_TEXTURE2D(_Landscape,sampler_Landscape,ValleyHorizonUV(ray)).rgb;
                distancePaint=ValleySkyTint(distancePaint,ray.y);
                float vista=perimeter*smoothstep(-.004,.004,slope)*(1-_Paper);
                // A second, finite-distance paint plane gives the ridge grove some parallax
                // against the sky; the crest silhouette still follows the solid wall.
                float3 rayOS=TransformWorldToObjectDir(ray);
                float facing=abs(n.x)>.5 ? abs(rayOS.x) : abs(rayOS.z);
                float3 beyond=rayOS*(1.8/max(.25,facing));
                float projectedAlong=along+(abs(n.x)>.5 ? beyond.z : beyond.x);
                float groveWidth=15;
                float2 treeUV=float2(frac(projectedAlong/groveWidth+n.z*.38),
                    (i.local.y+beyond.y-(_Minimum.y+.29*_Span.y))/(groveWidth*.54));
                vista*=lerp(.12,1,smoothstep(.45,3.8,distance(i.world,GetCameraPositionWS())));
                half4 tree=SAMPLE_TEXTURE2D(_Maple,sampler_Maple,saturate(treeUV));
                half3 ridgeTrees=tree.rgb*.90+half3(.014,.022,.032);
                half3 backdrop=lerp(distancePaint,ridgeTrees,tree.a);
                half3 pigment=stone*half3(.84,.87,1.08);
                float moss=perimeter*(1-smoothstep(.012,.073,abs(slope)));
                pigment=lerp(pigment,grass,moss*.85);
                pigment=lerp(pigment,grass,saturate(n.y));
                float2 cellUV=i.uv/_Cell+_Portable*.5;
                float2 edge=abs(frac(cellUV+.5)-.5);
                float seam=1-smoothstep(.002,.008+max(fwidth(cellUV.x),fwidth(cellUV.y))*.55,min(edge.x,edge.y));
                float3 normalWS=normalize(i.normal);
                #if defined(_MAIN_LIGHT_SHADOWS_SCREEN)
                    float4 shadowCoord=ComputeScreenPos(TransformWorldToHClip(i.world));
                #else
                    float4 shadowCoord=TransformWorldToShadowCoord(i.world);
                #endif
                Light sun=GetMainLight(shadowCoord);
                half ndl=dot(normalWS,sun.direction);
                half shift=(brush.g-.5)*.20+(brush.r-.5)*.10;
                half exposure=saturate(ndl*.80+.20+shift)*sun.shadowAttenuation;
                half mid=smoothstep(.07,.36,exposure);
                half warm=smoothstep(.40,.78,exposure);
                half3 key=lerp(1,sun.color/max(max(sun.color.r,sun.color.g),max(.001,sun.color.b)),.26);
                half3 wash=lerp(lerp(half3(.70,.75,.92),half3(.84,1.0,.99),mid),half3(1.30,1.17,.98)*key,warm);
                if(_Paper>.5){
                    // Large pools, directional strokes, and a few dry flecks survive even in the shade.
                    half tint=smoothstep(.07,.94,brush.r*.59+brush.g*.41);
                    half3 shadow=lerp(half3(.50,.54,.75),half3(.80,.89,.92),tint);
                    half3 pale=lerp(half3(.69,.83,.87),half3(.97,.98,.89),tint);
                    half3 cream=lerp(half3(.98,.91,.78),half3(1.28,1.20,1.01),tint);
                    wash=lerp(lerp(shadow,pale,mid),cream*key,warm);
                    pigment=(1-seam*.135)*(1+(brush.b-.5)*.055+(stone.g-.45)*.075);
                } else {
                    half grassPatch=saturate((brush.r-.32)*1.6);
                    pigment*=lerp(1,lerp(half3(.70,.95,1.11),half3(1.06,1.02,.81),grassPatch),saturate(n.y));
                    pigment*=1-seam*lerp(.045,.20,_Portable);
                    if(_Arch>.5){
                        half recess=i.archBands.r;
                        half crest=i.archBands.g;
                        half mossCoverage=smoothstep(.41,.56,crest*.66+brush.r*.22+brush.g*.27);
                        half3 chalk=stone*lerp(half3(.43,.59,.87),half3(1.22,1.13,1.01),1-recess);
                        half3 lichen=lerp(half3(.055,.13,.080),half3(.40,.52,.15),brush.r*.60+brush.g*.40);
                        pigment=lerp(chalk,lichen,mossCoverage*crest);
                        pigment+=half3(.20,.17,.10)*i.archBands.b*smoothstep(.49,.69,brush.g);
                    }
                }
                float2 screenUV=GetNormalizedScreenSpaceUV(i.clip);
                AmbientOcclusionFactor contact=GetScreenSpaceAmbientOcclusion(screenUV);
                half occlusion=lerp(1,contact.indirectAmbientOcclusion,lerp(.44,.13,warm));
                #if defined(_ADDITIONAL_LIGHTS)
                InputData inputData=(InputData)0;
                inputData.positionWS=i.world;
                inputData.normalizedScreenSpaceUV=screenUV;
                half3 fill=0;
                LIGHT_LOOP_BEGIN(GetAdditionalLightsCount())
                    Light lamp=GetAdditionalLight(lightIndex,i.world,half4(1,1,1,1));
                    fill+=lamp.color*lamp.distanceAttenuation*lamp.shadowAttenuation*saturate(dot(normalWS,lamp.direction));
                LIGHT_LOOP_END
                wash+=min(fill*.045,.16);
                #endif
                return half4(lerp(pigment*wash*occlusion*_Value,backdrop,vista),1);
            }
            ENDHLSL
        }
        UsePass "Universal Render Pipeline/Lit/ShadowCaster"
        UsePass "Universal Render Pipeline/Lit/DepthOnly"
        UsePass "Universal Render Pipeline/Lit/DepthNormals"
    }
}
