using System.Collections.Generic;
using UnityEngine;

// A flat solid when closed. When a gate opens, split only intersected rectangles into
// convex strips. The same compound shape works on static walls and dynamic rigidbodies.
public sealed class PortalSurfaceCollider : MonoBehaviour
{
    const float Thickness=.045f;
    Rect rect;
    readonly List<BoxCollider> solid=new List<BoxCollider>();
    int used;
    bool cut;
    bool shell=true;
    PhysicsMaterial finish;

    public void Initialize(Rect bounds,bool white)
    {
        rect=bounds;
        if(white){
            var aim=gameObject.AddComponent<BoxCollider>();
            aim.isTrigger=true;
            Size(aim,bounds.xMin,bounds.yMin,bounds.xMax,bounds.yMax);
        }
        Rebuild(null);
    }

    static void Size(BoxCollider box,float x0,float y0,float x1,float y1)
    {
        box.center=new Vector3((x0+x1)*.5f,(y0+y1)*.5f,-Thickness*.5f);
        box.size=new Vector3(x1-x0,y1-y0,Thickness);
    }

    void Fill(float x0,float y0,float x1,float y1)
    {
        if(x1-x0<.0001f || y1-y0<.0001f)return;
        if(used==solid.Count)solid.Add(gameObject.AddComponent<BoxCollider>());
        var box=solid[used++];
        Size(box,x0,y0,x1,y1);
        box.sharedMaterial=finish;
        box.enabled=shell;
    }

    public void SetShell(bool value,PhysicsMaterial material)
    {
        shell=value;finish=material;
        for(int i=0;i<solid.Count;i++){solid[i].sharedMaterial=material;solid[i].enabled=value && i<used;}
    }

    public void Rebuild(List<Vector2> holes)
    {
        float radius=WorkshopPortals.Radius;
        var inside=new List<Vector2>(2);
        if(holes!=null)foreach(var c in holes){
            if(Rect.MinMaxRect(c.x-radius,c.y-radius,c.x+radius,c.y+radius).Overlaps(rect))inside.Add(c);
        }
        if(!cut && inside.Count==0 && solid.Count>0)return;
        cut=inside.Count>0;
        used=0;
        if(!cut)Fill(rect.xMin,rect.yMin,rect.xMax,rect.yMax);
        else{
            inside.Sort((a,b)=>a.y.CompareTo(b.y));
            var cuts=new SortedSet<float>{rect.xMin,rect.xMax};
            foreach(var c in inside)for(int i=0;i<=48;i++){
                float x=c.x+radius*(2*i/48f-1);
                if(x>rect.xMin && x<rect.xMax)cuts.Add(x);
            }
            float left=rect.xMin;
            foreach(float right in cuts){
                if(right<=left)continue;
                float bottom=rect.yMin;
                foreach(var c in inside){
                    float dx=(left+right)*.5f-c.x;
                    if(Mathf.Abs(dx)>=radius)continue;
                    float h=Mathf.Sqrt(radius*radius-dx*dx);
                    Fill(left,bottom,right,Mathf.Clamp(c.y-h,rect.yMin,rect.yMax));
                    bottom=Mathf.Max(bottom,c.y+h);
                }
                Fill(left,bottom,right,rect.yMax);
                left=right;
            }
        }
        for(int i=used;i<solid.Count;i++)solid[i].enabled=false;
    }
}
