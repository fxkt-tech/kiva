using UnityEngine;
using UnityEngine.UI;

// Flat shapes for the settings page, drawn as meshes so they stay crisp at any size.
// `color` is the top (or left) colour and `second` the bottom (or right) one.
[RequireComponent(typeof(CanvasRenderer))]
public sealed class SettingShape : MaskableGraphic
{
    public enum Kind { Diamond=0, Spark=3, Chevron=4, Hair=7, FoldTab=9, SoftShadow=11, SideFade=12 }
    public Kind kind;
    public Color second=Color.white,rim=Color.clear;
    public float rimWidth;
    // Hair: the share of the width over which each end fades out.
    public float fade=.12f;

    static void Quad(VertexHelper vh,Vector2 a,Vector2 b,Vector2 c,Vector2 d,Color ca,Color cb,Color cc,Color cd)
    {
        int n=vh.currentVertCount;
        vh.AddVert(a,ca,Vector2.zero);vh.AddVert(b,cb,Vector2.zero);vh.AddVert(c,cc,Vector2.zero);vh.AddVert(d,cd,Vector2.zero);
        vh.AddTriangle(n,n+1,n+2);vh.AddTriangle(n,n+2,n+3);
    }

    void Fan(VertexHelper vh,Vector2[] points,Rect r,bool flat,Color tint)
    {
        Color At(Vector2 p)=>flat?tint:Color.Lerp(second,color,Mathf.InverseLerp(r.yMin,r.yMax,p.y));
        int n=vh.currentVertCount;
        vh.AddVert(r.center,At(r.center),Vector2.zero);
        foreach(var p in points)vh.AddVert(p,At(p),Vector2.zero);
        for(int i=0;i<points.Length;i++)vh.AddTriangle(n,n+1+i,n+1+(i+1)%points.Length);
    }

    protected override void OnPopulateMesh(VertexHelper vh)
    {
        vh.Clear();
        var r=GetPixelAdjustedRect();
        float x0=r.xMin,x1=r.xMax,y0=r.yMin,y1=r.yMax,xm=r.center.x,ym=r.center.y;
        var clear=new Color(color.r,color.g,color.b,0);
        Vector2[] points=null;
        switch(kind){
            case Kind.SideFade:{
                float bottom=y0+r.height*.2f,top=y1-r.height*.2f;
                Color rightClear=new Color(second.r,second.g,second.b,0);
                Quad(vh,new Vector2(x0,y0),new Vector2(x0,bottom),new Vector2(x1,bottom),new Vector2(x1,y0),clear,color,second,rightClear);
                Quad(vh,new Vector2(x0,bottom),new Vector2(x0,top),new Vector2(x1,top),new Vector2(x1,bottom),color,color,second,second);
                Quad(vh,new Vector2(x0,top),new Vector2(x0,y1),new Vector2(x1,y1),new Vector2(x1,top),color,clear,rightClear,second);
                return;}
            case Kind.SoftShadow:{
                vh.AddVert(r.center,color,Vector2.zero);
                for(int i=0;i<32;i++){
                    float a=i*Mathf.PI/16;
                    vh.AddVert(r.center+new Vector2(r.width*.5f*Mathf.Cos(a),r.height*.5f*Mathf.Sin(a)),clear,Vector2.zero);
                }
                for(int i=0;i<32;i++)vh.AddTriangle(0,i+1,1+(i+1)%32);
                return;
            }
            case Kind.FoldTab:
                float cut=Mathf.Min(r.width*.12f,r.height*.45f);
                points=new[]{new Vector2(x0,y0),new Vector2(x0+cut,y1),new Vector2(x1,y1),new Vector2(x1-cut,y0)};
                break;
            case Kind.Diamond:
                points=new[]{new Vector2(xm,y1),new Vector2(x1,ym),new Vector2(xm,y0),new Vector2(x0,ym)};break;
            case Kind.Spark:{
                float k=Mathf.Min(r.width,r.height)*.08f;
                points=new[]{new Vector2(xm,y1),new Vector2(xm+k,ym+k),new Vector2(x1,ym),new Vector2(xm+k,ym-k),
                    new Vector2(xm,y0),new Vector2(xm-k,ym-k),new Vector2(x0,ym),new Vector2(xm-k,ym+k)};break;}
            case Kind.Chevron:{
                float t=r.height*.14f;
                Vector2 tip=new Vector2(x0,ym),up=new Vector2(x1-t,y1),down=new Vector2(x1-t,y0),shift=new Vector2(t*1.5f,0);
                Quad(vh,tip,up,up+shift,tip+shift,color,color,color,color);
                Quad(vh,tip,tip+shift,down+shift,down,color,color,color,color);
                return;}
            case Kind.Hair:{
                float a=x0+r.width*fade,b=x1-r.width*fade;
                Quad(vh,new Vector2(x0,y0),new Vector2(x0,y1),new Vector2(a,y1),new Vector2(a,y0),clear,clear,color,color);
                Quad(vh,new Vector2(a,y0),new Vector2(a,y1),new Vector2(b,y1),new Vector2(b,y0),color,color,color,color);
                Quad(vh,new Vector2(b,y0),new Vector2(b,y1),new Vector2(x1,y1),new Vector2(x1,y0),color,color,clear,clear);
                return;}
        }
        if(points==null)return;
        if(rimWidth>0){
            Fan(vh,points,r,true,rim);
            var inner=new Vector2[points.Length];
            for(int i=0;i<points.Length;i++)inner[i]=Vector2.MoveTowards(points[i],r.center,rimWidth*1.4f);
            points=inner;
        }
        Fan(vh,points,r,false,color);
    }
}
