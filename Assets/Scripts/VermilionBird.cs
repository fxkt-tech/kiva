using UnityEngine;

// A hollow-looking painted body with independent, two-sided cut-paper wings, crest and tail.
public sealed class VermilionBird : MonoBehaviour
{
    public const int ModelLayer=8; // VermilionBird in TagManager.
    Transform plumage, tail;
    readonly Transform[] wings=new Transform[2],feet=new Transform[2],eyes=new Transform[2];
    readonly Material[] inks=new Material[9];
    float stride;
    const int Scarlet=0, Oxide=1, Coral=2, Gold=3, Belly=4, Ink=5, Ivory=6, Cut=7, Reverse=8;

    Transform Pivot(Transform parent,string name,Vector3 position)
    {
        var t=new GameObject(name).transform;t.SetParent(parent,false);t.localPosition=position;return t;
    }

    public void Build()
    {
        Color[] colors={
            new Color(.85f,.265f,.135f),new Color(.66f,.19f,.11f),new Color(.96f,.395f,.18f),
            new Color(.93f,.60f,.19f),new Color(.96f,.79f,.53f),new Color(.14f,.063f,.034f),
            new Color(.99f,.94f,.78f),new Color(1f,.70f,.38f),FerryLighting.OchreBack
        };
        for(int i=0;i<inks.Length;i++)inks[i]=FerryLighting.Mineral("Quill",colors[i]);
        plumage=Pivot(transform,"剪纸躯干",Vector3.zero);
        var body=new PaperMesh.Builder(inks.Length);
        const int rings=14, sectors=48;
        // Each meridian meets the same continuous curved breast seam.
        for(int half=0;half<2;half++)for(int j=0;j<rings;j++)for(int i=0;i<sectors;i++){
            float a=2*Mathf.PI*i/sectors,b=2*Mathf.PI*(i+1)/sectors;
            float t=j/(float)rings,s=(j+1)/(float)rings;
            Vector3 Skin(float angle,float h){
                float edge=.435f+.205f*Mathf.Cos(angle);
                return Pear(angle,half==0?h*edge:Mathf.Lerp(edge,1f,h));
            }
            var p=Skin(a,t);var q=Skin(b,t);var r=Skin(b,s);var u=Skin(a,s);
            body.Quad(half==0?Belly:Scarlet,(p+q+r+u)*.25f-Vector3.up*.62f,p,q,r,u);
        }
        body.Mount(plumage,"朱砂腹与米金胸",inks);
        var bill=new PaperMesh.Builder(inks.Length);
        var tip=new Vector3(0,.84f,.74f);
        var l=new Vector3(-.12f,.88f,.48f);var rgt=new Vector3(.12f,.88f,.48f);
        var top=new Vector3(0,1.00f,.49f);var low=new Vector3(0,.78f,.51f);
        bill.Triangle(Gold,Vector3.up+Vector3.left,l,tip,top);bill.Triangle(Cut,Vector3.up+Vector3.right,rgt,top,tip);
        bill.Triangle(Oxide,Vector3.down+Vector3.left,l,low,tip);bill.Triangle(Gold,Vector3.down+Vector3.right,rgt,tip,low);
        bill.Mount(plumage,"折喙",inks);
        for(int i=0;i<2;i++){
            float side=i==0?-1:1;
            Vector3 eyePos=new Vector3(.405f*side,.90f,.34f);
            var eye=Pivot(plumage,"侧脸",eyePos);
            eyes[i]=eye;
            eye.localRotation=Quaternion.LookRotation(new Vector3(side*.80f,.12f,.64f));
            Disc(eye,"眼白",new Vector3(0,0,.012f),new Vector2(.091f,.113f),Ivory);
            Disc(eye,"墨眼",new Vector3(.012f*side,.002f,.025f),new Vector2(.051f,.074f),Ink);
            Disc(eye,"眼光",new Vector3(.023f*side,.040f,.038f),Vector2.one*.019f,Ivory);
            wings[i]=Pivot(plumage,"镂空纸翼",new Vector3(side*.51f,.73f,.23f));
            for(int f=1;f<3;f++)
            {
                float y=.10f-f*.092f;
                Blade(wings[i],"镂火羽",new Vector3(side*(-.015f-f*.035f),y,.09f),new Vector3(side*(.16f-.045f*f),y+.06f,-.31f),
                    new Vector3(-side*(.18f-f*.03f),y-.12f+f*.022f,-.83f+f*.12f),Vector3.up,
                    new Vector3(side,0,0),.224f-f*.021f,f==1?Coral:Scarlet,true);
            }
            WingCover(wings[i],side);
            feet[i]=Pivot(transform,"短足",new Vector3(side*.18f,.040f,.12f));
            var ankle=new PaperMesh.Builder(inks.Length);
            for(int fold=0;fold<6;fold++){
                float a=fold*Mathf.PI/3,b=(fold+1)*Mathf.PI/3;
                var p=new Vector3(Mathf.Cos(a),0,Mathf.Sin(a));
                var q=new Vector3(Mathf.Cos(b),0,Mathf.Sin(b));
                ankle.Quad(Gold,p+q,p*.072f,q*.072f,q*.047f+Vector3.up*.14f,p*.047f+Vector3.up*.14f);
            }
            ankle.Mount(feet[i],"折趾踝",inks);
            for(int toe=-1;toe<=1;toe++){
                var t=new Vector3(toe*.065f,-.033f,.18f-Mathf.Abs(toe)*.035f);
                Blade(feet[i],"趾",Vector3.zero,t*.62f+Vector3.up*.045f,t,Vector3.right,Vector3.up,.042f,Gold,false);
            }
        }
        tail=Pivot(plumage,"三叠尾扇",new Vector3(0,.37f,-.37f));
        for(int i=-1;i<=1;i++)
        {
            float s=i;
            var across=new Vector3(.70f,s*.72f+.18f,-.08f).normalized;
            var normal=new Vector3(s*.74f,-.78f,-.08f).normalized;
            Blade(tail,"尾翎",new Vector3(s*.09f,0,0),new Vector3(s*.40f,.22f+s*.12f,-.45f),
                new Vector3(s*.45f,.49f+s*.36f,-.93f+Mathf.Abs(s)*.18f),
                across,normal,.17f,i==0?Coral:Scarlet,false);
        }
        for(int i=-1;i<=1;i++){
            float s=i;
            Blade(plumage,"焰冠",new Vector3(s*.075f,1.11f,.17f),new Vector3(s*.17f,1.36f,-.015f),
                new Vector3(s*.18f,1.50f+s*.11f,-.32f+s*.095f),
                new Vector3(.48f,s*.26f+.65f,-.35f).normalized,new Vector3(.62f,.36f,.37f).normalized,.125f,
                i==0?Coral:Scarlet,false);
        }
        foreach(var part in GetComponentsInChildren<Renderer>(true))
            part.gameObject.layer=ModelLayer;
    }

    static Vector3 Pear(float angle,float t)
    {
        float lat=(t-.5f)*Mathf.PI;
        float radius=Mathf.Cos(lat)*(1f-.24f*Mathf.Sin(lat));
        return new Vector3(.52f*radius*Mathf.Sin(angle),.65f+.59f*Mathf.Sin(lat),
            .53f*radius*Mathf.Cos(angle)+.11f*t);
    }

    void WingCover(Transform wing,float side)
    {
        var mesh=new PaperMesh.Builder(inks.Length);
        Vector3 normal=Vector3.right*side;
        Vector3 Point(float t,float v){
            float w=Mathf.Max(0,Mathf.Sin(t*Mathf.PI));
            // Bow the leading edge around the shoulder, then turn the clipped tip up.
            return new Vector3(side*(-.13f+.24f*w-.055f*t),
                .15f-.14f*t+.085f*Mathf.Sin(2*Mathf.PI*t)+v*.217f*Mathf.Pow(w,.76f),
                .16f-.98f*t)+normal*(1-Mathf.Abs(v))*.029f*w;
        }
        float CutWidth(float t,float start,float end){
            if(t<=start || t>=end)return 0;
            float u=(t-start)/(end-start);
            return .28f*Mathf.Sin(u*Mathf.PI)*Mathf.Pow(1-u,.28f);
        }
        float Edge(float t,int k){
            float wave=.18f*Mathf.Sin(t*9);
            switch(k){
                case 0:return -1;
                case 1:return -.38f+wave-CutWidth(t,.19f,.66f);
                case 2:return -.38f+wave+CutWidth(t,.19f,.66f);
                case 3:return .32f+wave-CutWidth(t,.34f,.83f);
                case 4:return .32f+wave+CutWidth(t,.34f,.83f);
                default:return 1;
            }
        }
        for(int i=0;i<36;i++){
            float t=i/36f,u=(i+1)/36f;
            for(int k=0;k<6;k+=2){
                var a=Point(t,Edge(t,k));var b=Point(u,Edge(u,k));
                var c=Point(u,Edge(u,k+1));var d=Point(t,Edge(t,k+1));
                var lip=normal*.014f;
                mesh.Quad(Scarlet,normal,a,b,c,d);
                mesh.Quad(Reverse,-normal,a-lip,b-lip,c-lip,d-lip);
                mesh.Quad(Cut,Vector3.up,c,d,d-lip,c-lip);
                mesh.Quad(Gold,Vector3.down,a,b,b-lip,a-lip);
            }
        }
        mesh.Mount(wing,"双焰透雕覆翼",inks);
    }

    void Disc(Transform parent,string name,Vector3 center,Vector2 radii,int pigment)
    {
        var mesh=new PaperMesh.Builder(inks.Length);
        for(int i=0;i<26;i++){
            float a=i*2*Mathf.PI/26,b=(i+1)*2*Mathf.PI/26;
            mesh.Triangle(pigment,Vector3.forward,center+Vector3.forward*.008f,
                center+new Vector3(Mathf.Cos(a)*radii.x,Mathf.Sin(a)*radii.y,0),
                center+new Vector3(Mathf.Cos(b)*radii.x,Mathf.Sin(b)*radii.y,0));
        }
        mesh.Mount(parent,name,inks);
    }

    void Blade(Transform parent,string name,Vector3 root,Vector3 bend,Vector3 tip,
        Vector3 across,Vector3 normal,float width,int pigment,bool pierced)
    {
        var mesh=new PaperMesh.Builder(inks.Length);
        const int sections=24;
        Vector3 Point(float t,float v){
            float a=1-t;
            float w=Mathf.Pow(Mathf.Max(0,Mathf.Sin(Mathf.PI*t)),.86f);
            return a*a*root+2*a*t*bend+t*t*tip+across*(width*w*(v+.32f*Mathf.Sin(2*Mathf.PI*t)))+normal*(w*(1-Mathf.Abs(v))*.036f);
        }
        for(int i=0;i<sections;i++){
            float t0=i/(float)sections,t1=(i+1)/(float)sections;
            float Hole(float t) => pierced && t>.25f && t<.82f ? .21f*Mathf.Sin((t-.25f)/.57f*Mathf.PI) : 0;
            for(int side=-1;side<=1;side+=2){
                Vector3 a=Point(t0,side),b=Point(t1,side);
                Vector3 c=Point(t1,side*Hole(t1)),d=Point(t0,side*Hole(t0));
                var thickness=normal*.012f;
                mesh.Quad(side<0?pigment:(pigment==Scarlet?Oxide:pigment),normal,a,b,c,d);
                mesh.Quad(pigment==Gold?Gold:Reverse,-normal,a-thickness,b-thickness,c-thickness,d-thickness);
                mesh.Quad(Cut,side*across,a,b,b-thickness,a-thickness);
                if(Hole(t1)+Hole(t0)>0) mesh.Quad(Gold,-side*across,c,d,d-thickness,c-thickness);
            }
        }
        mesh.Mount(parent,name,inks);
    }

    public void Pose(Vector3 velocity,float yaw,bool grounded,float dt)
    {
        float step=grounded?Mathf.Clamp01(new Vector2(velocity.x,velocity.z).magnitude/4.8f):0;
        stride+=dt*step*14;
        float gait=Mathf.Sin(stride)*step;
        Face(yaw);
        float breath=grounded?(1-step)*Mathf.Sin(Time.time*2.4f):0;
        plumage.localPosition=Vector3.up*(Mathf.Abs(gait)*.052f+breath*.011f);
        plumage.localRotation=Quaternion.Euler(grounded?breath*.8f:Mathf.Clamp(-velocity.y,-12,16),0,gait*4);
        float blink=Mathf.Clamp01(Mathf.Abs(Mathf.Repeat(Time.time+.8f,4.1f)-.08f)/.08f);
        for(int i=0;i<2;i++){
            float sign=i==0?-1:1;
            eyes[i].localScale=new Vector3(1f,Mathf.Lerp(.07f,1f,blink),1f);
            feet[i].localRotation=Quaternion.Euler(gait*sign*23,0,0);
            wings[i].localRotation=Quaternion.Euler(0,0,sign*(grounded?gait*4:16+Mathf.Min(14,Mathf.Abs(velocity.y))));
        }
        tail.localRotation=Quaternion.Euler(grounded?gait*5:Mathf.Clamp(-velocity.y*2,-16,22),0,0);
    }

    public void Face(float degrees){transform.localRotation=Quaternion.Euler(0,degrees,0);}
    void OnDestroy(){foreach(var mat in inks)if(mat)Destroy(mat);}
}
