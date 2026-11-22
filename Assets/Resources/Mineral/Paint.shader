Shader "Kiva/MineralPaint"
{
 Properties {
  _Color("Ground pigment",Color)=(1,1,1,1)
  _Pigment("Mineral, fibre, grain",2D)="gray"{}
  _Kind("Material family",Float)=0
 }
 SubShader {
  Tags { "RenderType"="Opaque" }
  CGPROGRAM
  #pragma surface surf Lambert fullforwardshadows addshadow
  #pragma target 3.0
  sampler2D _Pigment;
  fixed4 _Color;
  float _Kind;
  struct Input {float3 worldPos;float3 worldNormal;};
  void surf(Input i,inout SurfaceOutput o) {
   float3 n=abs(i.worldNormal);
   float2 uv=n.y>.7?i.worldPos.xz:(n.x>.7?i.worldPos.zy:i.worldPos.xy);
   float3 pigment=tex2D(_Pigment,uv*.51).rgb;
   float wash=tex2D(_Pigment,uv*.103).r;
   float3 base=_Color.rgb*(.82+.28*wash+.15*pigment.r);
   if (_Kind<.5) {
    // Strata of azurite and malachite with a narrow ochre edge.
    float ridge=2.4*sin(uv.x*.37)+1.8*sin(uv.x*.73+.9)+.5*sin(uv.x*2.4);
    float elevation=uv.y-ridge;
    float shade=smoothstep(3,3.15,elevation)*.25+smoothstep(7,7.2,elevation)*.22;
    base=lerp(base*float3(.68,.86,.90),base+float3(.09,.12,.09),shade+wash*.34);
    if(n.y<.7){
     float2 moon=uv-float2(-6.5,12.3);
     float disc=1-smoothstep(1.49,1.52,length(moon));
     base=lerp(base,float3(.76,.43,.23)*(.90+.15*pigment.r),disc*.85);
     float distant=8+1.3*sin(uv.x*.33+1)+.85*sin(uv.x*.69+.21)+.34*sin(uv.x*1.23);
     float nearer=4.1+1.5*sin(uv.x*.44-.8)+1.1*sin(uv.x*.87)+.23*sin(uv.x*1.77);
     base=lerp(base,base*float3(.79,.88,.84)+float3(.05,.065,.044),1-smoothstep(-.02,.06,uv.y-distant));
     base*=.89+.11*smoothstep(-.03,.06,uv.y-nearer);
    }
    base+=float3(.27,.18,.065)*(1-smoothstep(.025,.08,abs(elevation-7)));
   } else if (_Kind<1.5) {
    float grain=tex2D(_Pigment,uv*float2(.27,1)).b;
    base*=.88+.12*grain+.12*pigment.r;
   } else if (_Kind<2.5) {
    float2 sheet=frac(uv*float2(.36,.49));
    float fold=abs(sheet.x-.5)*2;
    base*=.89+.10*fold+.06*smoothstep(.48,.52,sheet.x);
    base+=float3(.028,.022,.012)*(pigment.g-.4);
   } else {
    base*=.88+.23*pigment.r;
    o.Emission=base*.22;
   }
   o.Albedo=base*(.96+.09*pigment.g);
   o.Alpha=1;
  }
  ENDCG
 }
 Fallback "Diffuse"
}
