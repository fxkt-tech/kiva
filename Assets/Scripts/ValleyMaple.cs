using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Rendering;

// The west-window maple is built once per room. All branches and sprays share two render meshes.
internal sealed class ValleyMaple
{
    readonly MeshData wood = new MeshData();
    readonly MeshData leaves = new MeshData();
    readonly System.Random random = new System.Random(2761);

    public static void Plant(Transform parent, Vector3 foot)
    {
        var maple = new ValleyMaple();
        maple.Grow();
        maple.wood.Mount(parent, foot, "近枫树干", "Valley/MapleBark");
        maple.leaves.Mount(parent, foot, "近枫树冠", "Valley/MapleLeaves");
    }

    float Range(float min, float max) => Mathf.Lerp(min,max,(float)random.NextDouble());

    void Grow()
    {
        var fork = new Vector3(.14f, 2.65f, .44f);
        var elbow = new Vector3(-.27f, 3.58f, 1.46f);
        var crown = new Vector3(-.35f, 4.68f, 1.96f);
        Limb(new[]{Vector3.down*.28f, new Vector3(.10f,.6f,.08f), new Vector3(.27f,1.52f,-.16f), fork, elbow, crown},
             new[]{.58f,.43f,.34f,.28f,.20f,.11f});
        for(int i=0;i<6;i++){
            float angle=i*Mathf.PI/3+.3f;
            var offset=new Vector3(Mathf.Cos(angle),0,Mathf.Sin(angle))*Range(.80f,1.46f);
            Limb(new[]{Vector3.up*.62f,offset*.39f+Vector3.up*.14f,offset-Vector3.up*.12f},new[]{.26f,.15f,.025f});
        }

        Bough(elbow,new Vector3(.31f,3.94f,2.66f),new Vector3(.60f,4.63f,3.92f),1.32f,.17f);
        Bough(crown,new Vector3(-.90f,5.25f,2.76f),new Vector3(-1.02f,5.76f,3.47f),1.26f,.11f);
        Bough(fork,new Vector3(.45f,3.58f,.24f),new Vector3(.85f,4.44f,-.64f),1.34f,.19f);
        Bough(elbow,new Vector3(-.79f,4.51f,.85f),new Vector3(-1.33f,5.71f,.28f),1.39f,.16f);
        Bough(crown,new Vector3(-.04f,5.48f,1.42f),new Vector3(.18f,6.23f,1.27f),1.23f,.12f);
        Bough(fork,new Vector3(-.64f,3.32f,-.46f),new Vector3(-1.43f,4.24f,-1.39f),1.17f,.17f);
    }

    void Bough(Vector3 start,Vector3 knuckle,Vector3 tip,float spread,float radius)
    {
        Limb(new[]{start,Vector3.Lerp(start,knuckle,.62f)-Vector3.up*.15f,knuckle,tip},
             new[]{radius,radius*.73f,radius*.54f,radius*.21f});
        float turn=Range(0,2*Mathf.PI);
        for(int i=0;i<5;i++){
            float angle=i*2.4f+turn;
            Vector3 reach=new Vector3(Mathf.Cos(angle)*.81f,Range(.24f,.67f),Mathf.Sin(angle))*spread;
            Vector3 basePoint=Vector3.Lerp(knuckle,tip,.22f+i*.16f);
            Vector3 end=basePoint+reach;
            Limb(new[]{basePoint,basePoint+reach*.54f-Vector3.up*.13f,end},new[]{radius*.36f,radius*.19f,.015f});
            var split=Vector3.Cross(Vector3.up,reach).normalized*spread*.36f;
            var twig=end-reach*.27f;
            Limb(new[]{twig,end+split*.31f,end+split+Vector3.up*.18f},new[]{.026f,.018f,.007f});
            LeafPad(end,spread*Range(.65f,.95f),angle);
        }
    }

    void Limb(Vector3[] line,float[] radii)
    {
        const int sides=8;
        int first=wood.Count;
        float length=0;
        Vector3 across=Vector3.forward;
        for(int j=0;j<line.Length;j++){
            if(j>0)length+=Vector3.Distance(line[j-1],line[j]);
            var axis=(line[Mathf.Min(j+1,line.Length-1)]-line[Mathf.Max(j-1,0)]).normalized;
            if(Mathf.Abs(Vector3.Dot(across,axis))>.96f)across=Vector3.right;
            across=Vector3.ProjectOnPlane(across,axis).normalized;
            var other=Vector3.Cross(axis,across);
            for(int k=0;k<=sides;k++){
                float angle=k*2*Mathf.PI/sides+.06f*j;
                var normal=across*Mathf.Cos(angle)+other*Mathf.Sin(angle);
                float ridge=1+.09f*Mathf.Sin(3*angle+.31f*j);
                wood.Vertex(line[j]+normal*(radii[j]*ridge),normal,new Vector2(length*.44f,k/(float)sides),
                            new Color(.50f+.27f*Mathf.Sin(angle*3),0,0));
                if(k<sides && j>0){
                    int lower=first+(j-1)*(sides+1)+k;
                    wood.Quad(lower,lower+1,lower+sides+2,lower+sides+1);
                }
            }
        }
    }

    void LeafPad(Vector3 center,float radius,float turn)
    {
        // A flattened, asymmetric crown with gaps between its boughs. Normals follow
        // the crown envelope so a spray keeps its light direction when the eye moves.
        float warmth=Range(.12f,.92f);
        for(int i=0;i<13;i++){
            float ring=Mathf.Sqrt((i+.4f)/13);
            float angle=turn+i*2.4f;
            float height=.45f*(1-ring*ring)+Range(-.16f,.12f);
            var outward=new Vector3(Mathf.Cos(angle),0,Mathf.Sin(angle));
            var tip=center+radius*(outward*ring+Vector3.up*height);
            var across=Vector3.Cross(Vector3.up,outward);
            float tilt=Range(.27f,.94f);
            var down=(outward*.64f-Vector3.up*tilt).normalized;
            var normal=(outward*ring*.90f+Vector3.up*(1.26f-ring*.55f)).normalized;
            float size=radius*Range(.48f,.79f);
            var u=across*size*.91f;
            var v=down*size*.68f;
            var color=new Color(Range(.20f,.91f),Mathf.Clamp01(.63f+height-ring*.28f),warmth);
            int first=leaves.Count;
            leaves.Vertex(tip-u+v,normal,Vector2.zero,color);
            leaves.Vertex(tip+u+v,normal,Vector2.right,color);
            leaves.Vertex(tip+u-v,(normal+Vector3.up*.3f).normalized,Vector2.one,color);
            leaves.Vertex(tip-u-v,(normal+Vector3.up*.3f).normalized,Vector2.up,color);
            leaves.Quad(first,first+1,first+2,first+3);
        }
    }

    sealed class MeshData
    {
        readonly List<Vector3> positions=new List<Vector3>(), normals=new List<Vector3>();
        readonly List<Vector2> uvs=new List<Vector2>();
        readonly List<Color32> colors=new List<Color32>();
        readonly List<int> indices=new List<int>();
        public int Count=>positions.Count;
        public void Vertex(Vector3 position,Vector3 normal,Vector2 uv,Color tint){
            positions.Add(position); normals.Add(normal); uvs.Add(uv); colors.Add(tint);
        }
        public void Quad(int a,int b,int c,int d){
            indices.Add(a);indices.Add(b);indices.Add(c);indices.Add(a);indices.Add(c);indices.Add(d);
        }
        public void Mount(Transform parent,Vector3 foot,string label,string material){
            var obj=new GameObject(label,typeof(MeshFilter),typeof(MeshRenderer));
            obj.transform.SetParent(parent,false);obj.transform.localPosition=foot;
            var mesh=new Mesh {name=label};
            mesh.SetVertices(positions);mesh.SetNormals(normals);mesh.SetUVs(0,uvs);
            mesh.SetColors(colors);mesh.SetTriangles(indices,0);
            obj.GetComponent<MeshFilter>().sharedMesh=mesh;
            obj.AddComponent<PaperMesh>().Own(mesh);
            var renderer=obj.GetComponent<MeshRenderer>();
            renderer.sharedMaterial=Resources.Load<Material>(material);
            renderer.shadowCastingMode=ShadowCastingMode.TwoSided;
        }
    }
}
