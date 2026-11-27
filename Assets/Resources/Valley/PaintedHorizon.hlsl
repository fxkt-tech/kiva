#ifndef KIVA_PAINTED_HORIZON
#define KIVA_PAINTED_HORIZON
float2 ValleyHorizonUV(float3 direction)
{
    float angle=atan2(direction.x,direction.z)/TWO_PI;
    return float2(1-abs(frac(angle*2+.24)*2-1),saturate(.54+asin(clamp(direction.y,-1,1))*.68));
}
half3 ValleySkyTint(half3 painting,float elevation)
{
    return lerp(painting,half3(.17,.64,.91),smoothstep(.56,.92,elevation));
}
#endif
