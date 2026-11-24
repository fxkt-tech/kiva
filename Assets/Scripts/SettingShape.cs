using UnityEngine;
using UnityEngine.UI;

// Flat shapes for the settings page, drawn as meshes so they stay crisp at any size.
// `color` is the top (or left) colour and `second` the bottom (or right) one.
[RequireComponent(typeof(CanvasRenderer))]
public sealed class SettingShape : MaskableGraphic
{
    public enum Kind { Diamond, Hex, Octagon, Spark, Chevron, Bar, Fall, Hair, Feather, FoldTab, Gear, SoftShadow, Scroll, Disc }
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
            case Kind.Disc:
                points=new Vector2[40];
                for(int i=0;i<points.Length;i++){
                    float a=i*2*Mathf.PI/points.Length;
                    points[i]=r.center+new Vector2(Mathf.Cos(a)*r.width*.5f,Mathf.Sin(a)*r.height*.5f);
                }
                break;
            case Kind.SoftShadow:{
                vh.AddVert(r.center,color,Vector2.zero);
                for(int i=0;i<32;i++){
                    float a=i*Mathf.PI/16;
                    vh.AddVert(r.center+new Vector2(r.width*.5f*Mathf.Cos(a),r.height*.5f*Mathf.Sin(a)),clear,Vector2.zero);
                }
                for(int i=0;i<32;i++)vh.AddTriangle(0,i+1,1+(i+1)%32);
                return;
            }
            case Kind.Scroll:{
                Vector2 P(float x,float y)=>new Vector2(x0+x*r.width,y0+y*r.height);
                void Strip(float x,float y,float w,float h,Color ink)=>Quad(vh,P(x,y),P(x,y+h),P(x+w,y+h),P(x+w,y),ink,ink,ink,ink);
                Strip(.17f,.24f,.68f,.53f,color);Strip(.10f,.14f,.10f,.73f,second);Strip(.81f,.14f,.10f,.73f,second);
                Strip(.29f,.56f,.39f,.055f,second);Strip(.29f,.40f,.26f,.055f,second);
                return;
            }
            case Kind.Gear:{
                float radius=Mathf.Min(r.width,r.height)*.46f;
                for(int i=0;i<32;i++){
                    float a=i*Mathf.PI/16,b=(i+1)*Mathf.PI/16;
                    var d0=new Vector2(Mathf.Cos(a),Mathf.Sin(a));
                    var d1=new Vector2(Mathf.Cos(b),Mathf.Sin(b));
                    float ra=i%4<2?radius:radius*.76f,rb=(i+1)%4<2?radius:radius*.76f;
                    Quad(vh,r.center+d0*ra,r.center+d1*rb,r.center+d1*(radius*.34f),r.center+d0*(radius*.34f),color,color,color,color);
                }
                return;
            }
            case Kind.Feather:
                points=new[]{new Vector2(x0+r.width*.18f,y0),new Vector2(x0,ym),new Vector2(xm,y1),new Vector2(x1,y1),new Vector2(x1,ym)};
                break;
            case Kind.FoldTab:
                float cut=Mathf.Min(r.width*.12f,r.height*.45f);
                points=new[]{new Vector2(x0,y0),new Vector2(x0+cut,y1),new Vector2(x1,y1),new Vector2(x1-cut,y0)};
                break;
            case Kind.Diamond:
                points=new[]{new Vector2(xm,y1),new Vector2(x1,ym),new Vector2(xm,y0),new Vector2(x0,ym)};break;
            case Kind.Hex:{
                float k=Mathf.Min(r.height*.36f,r.width*.5f);
                points=new[]{new Vector2(x0+k,y1),new Vector2(x1-k,y1),new Vector2(x1,ym),new Vector2(x1-k,y0),new Vector2(x0+k,y0),new Vector2(x0,ym)};break;}
            case Kind.Octagon:{
                float k=Mathf.Min(r.width,r.height)*.2f;
                points=new[]{new Vector2(x0+k,y1),new Vector2(x1-k,y1),new Vector2(x1,y1-k),new Vector2(x1,y0+k),
                    new Vector2(x1-k,y0),new Vector2(x0+k,y0),new Vector2(x0,y0+k),new Vector2(x0,y1-k)};break;}
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
            case Kind.Bar:
                Quad(vh,new Vector2(x0,y0),new Vector2(x0,y1),new Vector2(x1,y1),new Vector2(x1,y0),color,color,second,second);return;
            case Kind.Fall:
                Quad(vh,new Vector2(x0,y0),new Vector2(x0,y1),new Vector2(x1,y1),new Vector2(x1,y0),second,color,color,second);return;
            case Kind.Hair:{
                float a=x0+r.width*fade,b=x1-r.width*fade;
                Quad(vh,new Vector2(x0,y0),new Vector2(x0,y1),new Vector2(a,y1),new Vector2(a,y0),clear,clear,color,color);
                Quad(vh,new Vector2(a,y0),new Vector2(a,y1),new Vector2(b,y1),new Vector2(b,y0),color,color,color,color);
                Quad(vh,new Vector2(b,y0),new Vector2(b,y1),new Vector2(x1,y1),new Vector2(x1,y0),color,color,clear,clear);
                return;}
        }
        if(rimWidth>0){
            Fan(vh,points,r,true,rim);
            var inner=new Vector2[points.Length];
            for(int i=0;i<points.Length;i++)inner[i]=Vector2.MoveTowards(points[i],r.center,rimWidth*1.4f);
            points=inner;
        }
        Fan(vh,points,r,false,color);
    }
}
