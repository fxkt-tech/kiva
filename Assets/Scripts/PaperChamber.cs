using UnityEngine;
using UnityEngine.Rendering;

// Decorative joinery; the level asset owns the brick wall around this aperture.
public static class PaperChamber
{
    const float Cell=PaperTerrain.Cell;
    static GameObject Beam(Transform root,string name, Vector3 center, Vector3 size,Material mat)
    {
        // Eight-sided extrusions catch a thin honey-coloured highlight along sawn edges.
        bool vertical=size.y>size.z && size.y>size.x;
        Vector3 axis=vertical?Vector3.up:Vector3.forward;
        Vector3 u=Vector3.right,v=vertical?Vector3.forward:Vector3.up;
        float length=(vertical?size.y:size.z)*.5f;
        float w=size.x*.5f,h=(vertical?size.z:size.y)*.5f;
        float cut=Mathf.Min(.022f,Mathf.Min(w,h)*.33f);
        Vector2[] profile={new Vector2(-w+cut,-h),new Vector2(w-cut,-h),new Vector2(w,-h+cut),
            new Vector2(w,h-cut),new Vector2(w-cut,h),new Vector2(-w+cut,h),new Vector2(-w,h-cut),new Vector2(-w,-h+cut)};
        var builder=new PaperMesh.Builder(1);
        for(int i=0;i<profile.Length;i++){
            var a=profile[i];var b=profile[(i+1)%profile.Length];
            Vector3 p=u*a.x+v*a.y,q=u*b.x+v*b.y;
            builder.Quad(0,(p+q).normalized,p-axis*length,q-axis*length,q+axis*length,p+axis*length);
            builder.Triangle(0,axis,axis*length,p+axis*length,q+axis*length);
            builder.Triangle(0,-axis,-axis*length,p-axis*length,q-axis*length);
        }
        var go=builder.Mount(root,name,mat);
        go.transform.localPosition=center;
        return go;
    }

    public static void WestWindowFrame(Transform root,Material wood,Material glazing,Material ochre)
    {
        const float wall=-6f, sill=1.5f, head=6f, near=-4.5f, far=3f;
        float midZ=(near+far)*.5f;
        float width=far-near,height=head-sill;
        var sheet=new PaperMesh.Builder(1);
        var corner=new Vector3(0,-height*.5f,-width*.5f);
        sheet.Quad(0,Vector3.right,corner,corner+Vector3.up*height,
            -corner,corner+Vector3.forward*width);
        var paper=sheet.Mount(root,"透光窗纸",glazing);
        paper.transform.localPosition=new Vector3(wall-.25f,(head+sill)*.5f,midZ);
        // The thickness map covers all five by seven panes, aligned with the real bars.
        glazing.SetTextureScale("_BaseMap",new Vector2(-1f/(width*.76f),1f/(height*.76f)));
        glazing.SetTextureOffset("_BaseMap",Vector2.one*.5f);
        paper.GetComponent<Renderer>().shadowCastingMode=ShadowCastingMode.Off;
        var seal=new GameObject("窗内挡板").AddComponent<BoxCollider>();
        seal.transform.SetParent(root,false);seal.transform.localPosition=new Vector3(wall-.12f,(head+sill)*.5f,midZ);
        seal.size=new Vector3(.18f,head-sill,far-near);
        for(int i=0;i<=5;i++){
            float z=near+i*Cell;
            Beam(root,"窗梃",new Vector3(wall+.045f,(head+sill)*.5f,z),new Vector3(.28f,head-sill+.17f,.135f),wood);
            Beam(root,"窗梃亮边",new Vector3(wall+.19f,(head+sill)*.5f,z-.055f),new Vector3(.018f,head-sill,.020f),ochre);
        }
        for(int i=1;i<=6;i++){
            float y=sill+i*(head-sill)/7;
            Beam(root,"横向窗棂",new Vector3(wall+.07f,y,midZ),new Vector3(.25f,.083f,far-near),wood);
        }
        foreach(float y in new[]{sill,head}){
            Beam(root,"朱木窗框",new Vector3(wall+.09f,y,midZ),new Vector3(.41f,.19f,far-near+.38f),wood);
            Beam(root,"赭金压条",new Vector3(wall+.31f,y+.072f,midZ),new Vector3(.052f,.045f,far-near+.44f),ochre);
        }
        Beam(root,"金纸窗台",new Vector3(wall+.21f,sill-.13f,midZ),new Vector3(.64f,.12f,far-near+.54f),ochre);
    }

    public static void Cartouche(Transform root,Vector3 center,float yaw,Material cloud,Material ochre)
    {
        var plaque=new PaperMesh.Builder(2);
        const float w=.57f,h=.86f;
        Vector3 a=new Vector3(-w,-h,.035f),b=new Vector3(w,-h,.035f);
        Vector3 c=new Vector3(w,h,.035f),d=new Vector3(-w,h,.035f);
        plaque.Quad(0,Vector3.forward,a,b,c,d);
        Vector3 depth=Vector3.back*.034f;
        plaque.Quad(1,Vector3.right,b,c,c+depth,b+depth);
        plaque.Quad(1,Vector3.left,a,d,d+depth,a+depth);
        plaque.Quad(1,Vector3.up,d,c,c+depth,d+depth);
        plaque.Quad(1,Vector3.down,a,b,b+depth,a+depth);
        var item=plaque.Mount(root,"金云纹剪纸",cloud,ochre);
        item.transform.SetLocalPositionAndRotation(center,Quaternion.Euler(0,yaw,0));
    }
}
