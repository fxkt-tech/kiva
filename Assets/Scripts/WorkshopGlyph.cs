using UnityEngine;
using UnityEngine.UI;

// Draw crisp uGUI strokes at canvas resolution, including the two independent gate indicators.
public sealed class WorkshopGlyph : MaskableGraphic
{
    int shape;
    bool reticle,invalid,blue,red,openWing;
    static readonly Color Azure=new Color(.34f,.85f,.95f),Vermilion=new Color(1,.46f,.32f);

    public void Show(int icon,bool sight=false,bool blocked=false,bool first=false,bool second=false,bool flying=false)
    {
        if(shape==icon && reticle==sight && invalid==blocked && blue==first && red==second && openWing==flying)return;
        shape=icon;reticle=sight;invalid=blocked;blue=first;red=second;openWing=flying;SetVerticesDirty();
    }

    protected override void OnPopulateMesh(VertexHelper vh)
    {
        vh.Clear();
        Color ink=color;
        void Line(Vector2 a,Vector2 b,Color tint,float weight=1.8f){
            var v=(b-a).normalized;
            var off=new Vector2(-v.y,v.x)*weight*.5f;
            int n=vh.currentVertCount;
            vh.AddVert(a-off,tint,Vector2.zero);vh.AddVert(a+off,tint,Vector2.zero);
            vh.AddVert(b+off,tint,Vector2.zero);vh.AddVert(b-off,tint,Vector2.zero);
            vh.AddTriangle(n,n+1,n+2);vh.AddTriangle(n,n+2,n+3);
        }
        Vector2 At(float a,float r)=>new Vector2(Mathf.Cos(a*Mathf.Deg2Rad),Mathf.Sin(a*Mathf.Deg2Rad))*r;
        void Arc(float a,float b,float r,Color c){
            var prev=At(a,r);
            for(int i=1;i<=12;i++){var next=At(Mathf.Lerp(a,b,i/12f),r);Line(prev,next,c);prev=next;}
        }
        void Stroke(float x,float y,float a,float b)=>Line(new Vector2(x,y),new Vector2(a,b),ink);
        if(shape<0){Stroke(-4,0,4,0);return;}
        if(shape==0 && reticle){
            Line(Vector2.left,Vector2.right,ink,2);
            Arc(142,218,9,blue?Azure:ink);Arc(-38,38,9,red?Vermilion:ink);
        }else if(shape==0){
            Stroke(-10,5,10,5);Stroke(10,5,10,1);Stroke(10,1,-1,1);
            Stroke(-1,1,-5,-7);Stroke(-5,-7,-9,-7);Stroke(-9,-7,-7,1);Stroke(-7,1,-10,5);
        }else if(shape==1){
            Arc(8,172,8,ink);Stroke(-8,1,8,1);
            Stroke(1,0,-3,-3);Stroke(-3,-3,2,-5);Stroke(2,-5,-2,-8);
        }else if(shape==2){
            for(int i=0;i<3;i++){
                float a=120*i+82;var l=At(a+25,8);var r=At(a-8,9);
                Line(At(a+60,2),l,ink);Line(l,r,ink);Line(r,At(a-40,1.8f),ink);
            }
        }else{
            float span=openWing?13:8;
            for(int i=-1;i<=1;i+=2){
                Stroke(0,-3,i*span,6);Stroke(i*span,6,i*(span-3),-3);
                Stroke(i*(span-3),-3,i*3,-6);Stroke(i*3,-6,0,-3);
            }
        }
        if(invalid)Line(new Vector2(-10,-10),new Vector2(10,10),ink,1.3f);
    }
}
