using System.Collections.Generic;
using UnityEngine;

// Each face has its own vertices so a crease stays sharp under a moving light.
public sealed class FoldedPaper : MonoBehaviour
{
    Mesh mesh;
    void OnDestroy(){ if(mesh) Destroy(mesh); }

    sealed class Sheet
    {
        readonly List<Vector3> points = new List<Vector3>();
        readonly List<Vector2> uv = new List<Vector2>();
        readonly List<int>[] faces;
        readonly Material[] inks;

        public Sheet(params Material[] inks)
        {
            this.inks=inks;
            faces=new List<int>[inks.Length];
            for(int i=0;i<faces.Length;i++)faces[i]=new List<int>();
        }

        public void Face(int ink,Vector3 outward,Vector3 a,Vector3 b,Vector3 c)
        {
            if(Vector3.Dot(Vector3.Cross(b-a,c-a),outward)<0){var tmp=b;b=c;c=tmp;}
            int first=points.Count;
            points.Add(a);points.Add(b);points.Add(c);
            foreach(var v in new[]{a,b,c}) uv.Add(new Vector2(v.x+.37f*v.y,v.z+.61f*v.y));
            faces[ink].Add(first);faces[ink].Add(first+1);faces[ink].Add(first+2);
        }

        public void Quad(int ink,Vector3 normal,Vector3 a,Vector3 b,Vector3 c,Vector3 d)
        {
            Face(ink,normal,a,b,c);Face(ink,normal,a,c,d);
        }

        public void Mount(Transform parent,string name,Vector3 pos)
        {
            var go=new GameObject(name,typeof(MeshFilter),typeof(MeshRenderer));
            go.transform.SetParent(parent,false);
            go.transform.localPosition=pos;
            var m=new Mesh {name=name};
            go.AddComponent<FoldedPaper>().mesh=m;
            m.SetVertices(points);m.SetUVs(0,uv);m.subMeshCount=inks.Length;
            for(int i=0;i<faces.Length;i++)m.SetTriangles(faces[i],i);
            m.RecalculateNormals();m.RecalculateTangents();m.RecalculateBounds();
            go.GetComponent<MeshFilter>().sharedMesh=m;
            go.GetComponent<MeshRenderer>().sharedMaterials=inks;
        }
    }

    public static void Bridge(Transform parent,string name,Vector3 center,Vector3 size,
        Material face,Material back,Material deep,Material ochre)
    {
        var sheet=new Sheet(face,back,deep,ochre);
        float w=size.x*.5f,d=size.z*.5f,h=size.y;
        // Follow the sheet from one inward-facing return, over the deck, to the opposite return.
        var section=new[]{new Vector2(-w+.43f,-h+.13f),new Vector2(-w+.21f,-h),
            new Vector2(-w,-.12f),new Vector2(-w+.095f,0),new Vector2(-w*.46f,-.018f),
            new Vector2(w*.22f,.012f),new Vector2(w-.095f,0),
            new Vector2(w,-.12f),new Vector2(w-.21f,-h),new Vector2(w-.43f,-h+.13f)};
        const float gauge=.035f;
        for(int i=0;i<section.Length-1;i++)
        {
            Vector2 a=section[i], b=section[i+1];
            Vector3 normal=new Vector3(a.y-b.y,b.x-a.x,0);
            var p=new Vector3(a.x,a.y,-d);var q=new Vector3(b.x,b.y,-d);
            var run=Vector3.forward*size.z;var thickness=Vector3.down*gauge;
            sheet.Quad(0,normal,p,q,q+run,p+run);
            sheet.Quad(1,-normal,p+thickness,q+thickness,q+run+thickness,p+run+thickness);
            sheet.Quad(1,Vector3.back,p,p+thickness,q+thickness,q);
            sheet.Quad(1,Vector3.forward,p+run,q+run,q+run+thickness,p+run+thickness);
        }
        foreach(int side in new[]{-1,1}){
            var p=new Vector3(side*(w-.18f),.006f,-d+.54f);
            var stripe=Vector3.right*side*.018f; var span=Vector3.forward*(size.z-1.08f);
            sheet.Quad(1,Vector3.up,p,p+stripe,p+stripe+span,p+span);
        }
        // Folded corner reinforcements expose a triangular patch of the paper reverse.
        foreach(int side in new[]{-1,1})
        foreach(int end in new[]{-1,1}){
            var corner=new Vector3(side*(w-.10f),.005f,end*(d-.025f));
            var across=corner-Vector3.right*side*.46f;
            var along=corner-Vector3.forward*end*.58f;
            var ridge=(across+along)*.5f+Vector3.up*.043f;
            sheet.Face(1,Vector3.up,corner,across,ridge);
            sheet.Face(3,Vector3.up,corner,ridge,along);
        }
        float length=center.y+3;
        if(length>0)
        foreach(int side in new[]{-1,1})
        foreach(int end in new[]{-1,1}){
            var top=new Vector3(side*(w-.36f),-h*.55f,end*(d-.36f));
            for(int i=0;i<6;i++){
                Vector3 n=Radial(i),m=Radial(i+1);
                var neck=top+Vector3.down*.47f;
                var foot=top-Vector3.up*length;
                sheet.Quad(2,n+m,foot+.16f*n,foot+.16f*m,neck+.145f*m,neck+.145f*n);
                sheet.Quad(i%2==0?0:1,n+m,neck+.145f*n,neck+.145f*m,top+.36f*m,top+.36f*n);
            }
        }
        sheet.Mount(parent,name+" · 折纸",center+Vector3.up*h*.5f);
    }

    // A shallow concertina against the solid gable. Each summit is a mountain fold;
    // the recessed valley and the unpainted cut edge share its silhouette.
    public static void Ridges(Transform parent,float width,Material shadow,Material green,Material paper,Material gold)
    {
        var sheet=new Sheet(shadow,green,paper,gold);
        float[] peaks={.38f,.62f,.80f,.48f,.59f,.88f,.66f};
        for(int layer=0;layer<2;layer++){
            int count=peaks.Length;
            float span=width/count;
            float inset=.06f+layer*.24f;
            for(int i=0;i<count;i++){
                float x0=-width*.5f+span*i;
                float height=(layer==0?13:7)*peaks[(i+3*layer)%count];
                var a=new Vector3(x0,layer==0?3.1f:1.5f,inset);
                var b=a+Vector3.right*span;
                var summit=new Vector3(x0+span*(i%2==0?.40f:.63f),height,inset+.12f);
                var ridge=new Vector3(summit.x+.20f,.25f,inset+.34f);
                var left=new Vector3(a.x,.25f,inset);
                var right=new Vector3(b.x,.25f,inset);
                int leftInk=layer==0?1:0;
                sheet.Quad(leftInk,Vector3.forward,left,a,summit,ridge);
                sheet.Quad(layer==0?0:1,Vector3.forward,ridge,summit,b,right);
                var lip=Vector3.down*.045f;
                sheet.Quad(2,Vector3.forward,a,summit,summit+lip+Vector3.forward*.008f,a+lip+Vector3.forward*.008f);
            }
        }
        var center=new Vector3(-width*.28f,13.3f,.13f);
        for(int i=0;i<12;i++){
            float a=i*Mathf.PI/6,b=(i+1)*Mathf.PI/6;
            var p=center+new Vector3(Mathf.Cos(a),Mathf.Sin(a),0)*1.05f;
            var q=center+new Vector3(Mathf.Cos(b),Mathf.Sin(b),0)*1.05f;
            sheet.Face(3,Vector3.forward,center+Vector3.forward*.22f,p,q);
        }
        sheet.Mount(parent,"石青山折与赭日",Vector3.zero);
    }

    static Vector3 Radial(int i)
    {
        float a=i*Mathf.PI/3;
        return new Vector3(Mathf.Cos(a),0,Mathf.Sin(a));
    }

    public static void Beacon(Transform parent,Material silk,Material overlap,Material binding)
    {
        var box=new PaperMesh.Builder(3);
        for(int side=0;side<4;side++){
            Quaternion turn=Quaternion.Euler(0,side*90,0);
            Vector3 P(float x,float y,float z=.30f)=>turn*new Vector3(x,y,z);
            Vector3 n=turn*Vector3.forward;
            box.Quad(0,n,P(-.29f,-.35f),P(.29f,-.35f),P(.29f,.34f),P(-.29f,.34f));
            for(int edge=-1;edge<=1;edge+=2){
                float x=edge*.29f;
                box.Quad(2,n,P(x-.021f,-.38f,.32f),P(x+.021f,-.38f,.32f),P(x+.021f,.38f,.32f),P(x-.021f,.38f,.32f));
                float y=edge*.35f;
                box.Quad(2,n,P(-.32f,y-.022f,.32f),P(.32f,y-.022f,.32f),P(.32f,y+.022f,.32f),P(-.32f,y+.022f,.32f));
            }
            box.Quad(1,n,P(0,-.16f,.308f),P(.085f,-.015f,.308f),P(0,.16f,.308f),P(-.085f,-.015f,.308f));
            box.Triangle(2,Vector3.up,P(-.32f,.37f,.32f),P(.32f,.37f,.32f),Vector3.up*.43f);
        }
        box.Mount(parent,"方形渡灯",silk,overlap,binding);
    }

    public static void Lantern(Transform parent,Material single,Material overlap,Material binding)
    {
        var sheet=new Sheet(single,overlap,binding);
        for(int i=0;i<6;i++){
            var n=Radial(i);var m=Radial(i+1);
            var mid=(n+m).normalized;
            var a=n*.19f+Vector3.up*.28f;var b=m*.19f+Vector3.up*.28f;
            var c=n*.19f-Vector3.up*.26f;var d=m*.19f-Vector3.up*.26f;
            var ridge=mid*.31f;
            var e=n*.24f;var f=m*.24f;
            sheet.Face(0,mid,a,b,ridge);
            sheet.Face(1,mid,a,ridge,e);
            sheet.Face(0,mid,b,f,ridge);
            sheet.Face(0,mid,c,e,ridge);
            sheet.Face(1,mid,d,ridge,f);
            sheet.Face(0,mid,c,ridge,d);
            var crown=Vector3.up*.39f;
            sheet.Face(2,Vector3.up,a,b,crown);
            sheet.Face(2,Vector3.down,c,d,Vector3.down*.35f);
            sheet.Quad(1,mid,a,b,b+Vector3.down*.024f,a+Vector3.down*.024f);
        }
        var root=Vector3.down*.34f;
        for(int i=0;i<4;i++){
            var p=Quaternion.Euler(0,90*i,0)*Vector3.right*.047f;
            var q=Quaternion.Euler(0,90*(i+1),0)*Vector3.right*.047f;
            sheet.Face(2,p+q,root,root+Vector3.down*.12f+p,root+Vector3.down*.12f+q);
            sheet.Face(1,p+q,root+Vector3.down*.12f+p,root+Vector3.down*.12f+q,root+Vector3.down*.29f);
        }
        sheet.Mount(parent,"六折灯罩",Vector3.zero);
    }
}
