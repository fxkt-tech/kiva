#ifndef KIVA_FOLD_PIGMENT_INCLUDED
#define KIVA_FOLD_PIGMENT_INCLUDED

// The mesh marks exposed pulp, pigment in a valley, and turned-over paper in RGB.
// White is an untreated face. Fibre relief falls away once its texels are subpixel.
void FoldPigment_float(float2 UV, float4 PaperMask, float3 Pigment, UnityTexture2D Fibres, UnityTexture2D MineralWash,
                       out float3 Albedo, out float3 NormalTS, out float Smoothness)
{
    float2 sheet = UV * 0.73;
    float4 grain = SAMPLE_TEXTURE2D(Fibres.tex, Fibres.samplerstate, sheet);
    float wash = SAMPLE_TEXTURE2D(MineralWash.tex, MineralWash.samplerstate, sheet * 0.43).r;
    float footprint = max(fwidth(sheet.x), fwidth(sheet.y)) * Fibres.texelSize.z;
    float detail = 1.0 - smoothstep(1.5, 11.0, footprint);
    // Tagged folds have half alpha. Meshes without vertex colours retain their own pigment.
    float tagged = saturate(1.0 - abs(2.0 * PaperMask.a - 1.0));
    float3 marks = saturate(1.0 - PaperMask.rgb) * tagged;
    marks.rg *= lerp(1.0, 0.44, smoothstep(7.0, 22.0, footprint));
    float3 pulp = float3(0.92, 0.82, 0.62);
    float3 paint = Pigment * (0.95 + (wash - 0.2) * 0.31);
    paint = lerp(paint, pulp, marks.b * 0.70);
    paint = lerp(paint, pulp, marks.r * (0.57 + 0.18 * grain.r));
    paint *= lerp(float3(1, 1, 1), float3(0.62, 0.55, 0.49), marks.g);
    Albedo = saturate(paint * (1.0 + (grain.r - 0.5) * 0.34 * detail));
    NormalTS = normalize(float3((grain.ag * 2.0 - 1.0) * 0.16 * detail, 1.0));
    Smoothness = 0.13 + 0.04 * marks.r + (grain.r - 0.5) * 0.05 * detail;
}

#endif
