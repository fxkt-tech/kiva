using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

// Orthographic drawing of the actual four bird meshes in a regular uGUI rect.
// Shared geometry keeps small silhouettes, animated inspections and in-world birds consistent.
[RequireComponent(typeof(CanvasRenderer))]
public sealed class BirdPortrait : MaskableGraphic
{
    public BirdSpecies species;
    public bool silhouette, inspect;
    struct Face { public Vector3 a,b,c; public Color pigment; public int part; }
    struct Painted { public Vector3 a,b,c; public Color color; public float depth; }
    readonly List<Face> faces=new List<Face>();
    readonly Matrix4x4[] matrices=new Matrix4x4[4];
    Painted[] paint=Array.Empty<Painted>();
    BirdSpecies cached;
    float time,next;
    static readonly IComparer<Painted> Depth=Comparer<Painted>.Create((a,b)=>b.depth.CompareTo(a.depth));
    static readonly Vector3 Light=new Vector3(-.38f,.65f,-.74f).normalized;

    public void Display(BirdSpecies value,bool hidden=false)
    {species=value;silhouette=hidden;SetVerticesDirty();}
    void Update()
    {
        if(!species || silhouette || Time.unscaledTime<next)return;
        time=Time.unscaledTime;next=time+1f/24;SetVerticesDirty();
    }
    void Read()
    {
        faces.Clear();cached=species;
        for(int part=0;part<4;part++){
            var mesh=species.Part(part);
            var vertices=mesh.vertices;
            for(int i=0;i<mesh.subMeshCount;i++){
                int[] tris=mesh.GetTriangles(i);
                for(int t=0;t<tris.Length;t+=3) faces.Add(new Face{a=vertices[tris[t]],b=vertices[tris[t+1]],c=vertices[tris[t+2]],pigment=species.pigments[i],part=part});
            }
        }
        paint=new Painted[faces.Count];
    }
    protected override void OnPopulateMesh(VertexHelper vh)
    {
        vh.Clear();if(!species || !species.body)return;
        if(cached!=species)Read();
        var r=rectTransform.rect;
        float yaw=inspect ? 109f+Mathf.Sin(time*.45f)*55 : 102f+Mathf.Sin(time*.62f)*3;
        bool beak=!species.bobsTail && !species.forkedTail;
        var view=silhouette ? Quaternion.Euler(species.forkedTail?58:22,beak?90:128,0) : Quaternion.Euler(9,yaw,0);
        var center=silhouette ? (beak?new Vector3(0,.315f,.26f):new Vector3(0,.15f,-.22f)) : new Vector3(0,.205f,-.005f);
        for(int i=0;i<4;i++)
            matrices[i]=Matrix4x4.Rotate(view)*Matrix4x4.TRS(species.Pivot(i)-center,species.Pose(i,time*2,0),Vector3.one);
        int used=0;
        foreach(var face in faces){
            if(silhouette && (beak ? face.part!=0 || Mathf.Max(face.a.z,face.b.z,face.c.z)<.28f : face.part!=3))continue;
            var m=matrices[face.part];var a=m.MultiplyPoint3x4(face.a);var b=m.MultiplyPoint3x4(face.b);var c=m.MultiplyPoint3x4(face.c);
            var n=Vector3.Cross(b-a,c-a).normalized;
            if(n.z>=0)continue;
            var tint=silhouette ? new Color(.61f,.68f,.69f,.5f) : face.pigment*(.73f+.27f*Mathf.Max(0,Vector3.Dot(n,Light)));
            tint.a=silhouette?.54f:1;
            paint[used++]=new Painted{a=a,b=b,c=c,depth=a.z+b.z+c.z,color=tint*color};
        }
        Array.Sort(paint,0,used,Depth);
        float scale=Mathf.Min(r.width/.72f,r.height/.50f);
        for(int j=0;j<used;j++){
            var f=paint[j];int k=vh.currentVertCount;
            void Vertex(Vector3 p)=>vh.AddVert(r.center+new Vector2(p.x,p.y)*scale,f.color,Vector2.zero);
            Vertex(f.a);Vertex(f.b);Vertex(f.c);vh.AddTriangle(k,k+1,k+2);
        }
    }
}
