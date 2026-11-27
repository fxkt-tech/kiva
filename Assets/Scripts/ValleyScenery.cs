using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Rendering;

// All scenery uses render meshes only. The level's cell map remains the source for
// placement, collision, moving bricks and portal apertures.
public sealed class ValleyScenery
{
    readonly Bounds bounds;
    readonly float eyeHeight;
    readonly bool glazedWest;
    readonly MaterialPropertyBlock properties = new MaterialPropertyBlock();
    public readonly Material[] surfaces;

    public ValleyScenery(Transform root, WorkshopLevel level, Dictionary<Vector3Int, WhiteFaces> cells)
    {
        float unit = PaperTerrain.Cell;
        bounds = new Bounds(level.spawnFeet, Vector3.zero);
        foreach (var region in level.fixedBricks) {
            bounds.Encapsulate((Vector3)region.start * unit);
            bounds.Encapsulate((Vector3)(region.start + region.size) * unit);
        }
        eyeHeight = level.spawnFeet.y + 1;
        glazedWest = level.westWindow;
        var paper = Resources.Load<Material>("Valley/Chalk");
        var hillside = Resources.Load<Material>("Valley/Hillside");
        surfaces = new[] { paper, hillside };
        BuildHorizon(root);
        BuildGroves(root);
        BuildStream(root);
        BuildWildflowers(root, cells);
    }

    public void Paint(MeshRenderer renderer, bool portable)
    {
        properties.SetVector("_Minimum", bounds.min);
        properties.SetVector("_Span", bounds.size);
        properties.SetFloat("_BoundaryPaint", 1);
        properties.SetFloat("_GlazedWest", glazedWest ? 1 : 0);
        properties.SetFloat("_Portable", portable ? 1 : 0);
        properties.SetFloat("_Cell", PaperTerrain.Cell);
        renderer.SetPropertyBlock(properties);
    }

    public static void WindowArch(Transform root,Vector3 center,float width,float height)
    {
        // Vary the cross-section along the span: broad footings, narrow crown and a mossy lip.
        const int segments=20, sides=8;
        var arch=new PaperMesh.Builder(1,true);
        var rings=new Vector3[segments+1,sides];
        for(int j=0;j<=segments;j++){
            float t=j/(float)segments;
            float angle=Mathf.PI*t;
            float lift=Mathf.Sin(angle);
            Vector3 axis=new Vector3(.12f*Mathf.Sin(3*angle),height*lift,width*.5f*Mathf.Cos(angle));
            Vector3 radial=new Vector3(0,lift,Mathf.Cos(angle)).normalized;
            float thick=Mathf.Lerp(.54f,.32f,lift)+.09f*Mathf.Sin(angle*7);
            for(int k=0;k<sides;k++){
                float a=2*Mathf.PI*k/sides;
                float chip=1+.11f*Mathf.Sin(j*2.3f+k*1.6f);
                rings[j,k]=axis+chip*(radial*(Mathf.Cos(a)*thick)+Vector3.right*(.37f*Mathf.Sin(a)));
            }
        }
        for(int j=0;j<segments;j++)for(int k=0;k<sides;k++){
            int next=(k+1)%sides;
            Vector3 a=rings[j,k],b=rings[j,next],c=rings[j+1,next],d=rings[j+1,k];
            var normal=Vector3.Cross(d-a,b-a).normalized;
            // The span runs clockwise in the YZ plane; derive the outward side explicitly.
            var radial=new Vector3(0,Mathf.Sin(Mathf.PI*(j+.5f)/segments),Mathf.Cos(Mathf.PI*(j+.5f)/segments));
            var facing=radial*Mathf.Cos((k+.5f)*2*Mathf.PI/sides)+Vector3.right*Mathf.Sin((k+.5f)*2*Mathf.PI/sides);
            if(Vector3.Dot(normal,facing)<0)normal=-normal;
            float radialFacing=Vector3.Dot(normal,radial);
            float underside=Mathf.Clamp01(-radialFacing);
            float crest=Mathf.Clamp01(radialFacing*1.3f)*Mathf.Clamp01(.28f+radial.y);
            float rim=Mathf.Clamp01(normal.x)*Mathf.Clamp01(radialFacing+.25f);
            arch.FoldMask=new Color(underside,crest,rim,1);
            arch.Quad(0,normal,a,b,c,d);
        }
        arch.Mount(root,"苔石拱",Resources.Load<Material>("Valley/Arch")).transform.localPosition=center;
        ArchGrowth(root,center,width,height);
        ValleyMaple.Plant(root,center+new Vector3(-3.05f,-2.38f,-width*.48f));
    }

    static void ArchGrowth(Transform root,Vector3 center,float width,float height)
    {
        var marks=new Marks();
        var random=new System.Random(1903);
        float Pick(float low,float high)=>Mathf.Lerp(low,high,(float)random.NextDouble());
        // Low cushions follow the exposed lip. Three uneven trails turn under the arch.
        foreach(float patch in new[]{.19f,.27f,.46f,.56f,.72f,.82f}){
            for(int i=0;i<19;i++){
                float angle=(patch+Pick(-.047f,.047f))*Mathf.PI;
                var outward=new Vector3(0,Mathf.Sin(angle),Mathf.Cos(angle));
                var along=new Vector3(0,Mathf.Cos(angle),-Mathf.Sin(angle));
                float girth=Mathf.Lerp(.54f,.32f,outward.y)+.09f*Mathf.Sin(angle*7);
                var point=center+new Vector3(.12f*Mathf.Sin(3*angle)+.28f,height*outward.y,width*.5f*outward.z);
                point+=outward*(girth*Pick(.49f,.87f));
                var blade=outward*Pick(.04f,.13f)+along*Pick(-.17f,.16f);
                var breadth=along*Pick(.027f,.061f);
                var tint=Color.Lerp(new Color(.26f,.43f,.23f),new Color(.63f,.73f,.29f),(float)random.NextDouble());
                marks.Quad(point,point+blade*.44f-breadth,point+blade,point+blade*.53f+breadth,tint);
            }
        }
        foreach(float t in new[]{.27f,.32f,.66f,.74f,.79f}){
            float a=t*Mathf.PI;
            var start=center+new Vector3(.30f+.12f*Mathf.Sin(3*a),height*Mathf.Sin(a),width*.5f*Mathf.Cos(a));
            var prev=start;
            float length=Pick(.46f,1.01f);
            for(int j=1;j<=8;j++){
                float progress=j/8f;
                var tip=start+new Vector3(-progress*.10f,-length*progress,.13f*(Mathf.Sin(a*6+progress*3)-Mathf.Sin(a*6)));
                var edge=Vector3.forward*.009f*(1-progress*.66f);
                marks.Quad(prev-edge,prev+edge,tip+edge,tip-edge,new Color(.28f,.37f,.24f));
                var end=tip+new Vector3(.012f,-Pick(.055f,.12f),Pick(.058f,.145f)*(j%2==0?1:-1))*(1-progress*.35f);
                var halfWidth=Vector3.up*Pick(.027f,.044f);
                marks.Quad(tip,Vector3.Lerp(tip,end,.42f)+halfWidth,end,Vector3.Lerp(tip,end,.51f)-halfWidth,
                    Color.Lerp(new Color(.27f,.45f,.31f),new Color(.55f,.63f,.28f),(float)random.NextDouble()));
                prev=tip;
            }
        }
        marks.Mount(root,"石拱垂藤",Resources.Load<Material>("Valley/Petals"));
    }

    void BuildHorizon(Transform root)
    {
        float radius = Mathf.Max(bounds.extents.magnitude + 28, 65);
        var sky = new PaperMesh.Builder(1);
        foreach (var normal in new[] { Vector3.right, Vector3.left, Vector3.forward, Vector3.back, Vector3.up, Vector3.down }) {
            var across = Mathf.Abs(normal.y) > .5f ? Vector3.right : Vector3.Cross(Vector3.up,normal);
            var rise = Vector3.Cross(normal, across);
            var p = normal * radius;
            sky.Quad(0,-normal,p-(across+rise)*radius,p+(across-rise)*radius,
                p+(across+rise)*radius,p+(-across+rise)*radius);
        }
        var dome=sky.Mount(root,"远山云海",Resources.Load<Material>("Valley/Horizon"));
        dome.transform.localPosition=bounds.center;
        dome.GetComponent<MeshRenderer>().shadowCastingMode=ShadowCastingMode.Off;
    }

    void BuildGroves(Transform root)
    {
        var trees=new Marks();
        var random=new System.Random(4561);
        for(int side=0;side<4;side++){
            Vector3 normal=Quaternion.Euler(0,side*90,0)*Vector3.forward;
            Vector3 along=Vector3.Cross(Vector3.up,normal);
            float depth=Mathf.Abs(Vector3.Dot(normal,bounds.extents));
            float width=Mathf.Abs(Vector3.Dot(along,bounds.extents));
            int count=Mathf.Clamp(Mathf.CeilToInt(width/4),3,12);
            for(int j=0;j<count;j++){
                float distance=depth+4+(float)random.NextDouble()*10;
                Vector3 foot=bounds.center+normal*distance+along*(Mathf.Lerp(-width-6,width+6,(j+.35f)/count));
                foot.y=eyeHeight-5-(float)random.NextDouble()*2;
                float height=6+(float)random.NextDouble()*6;
                Vector3 halfWidth=along*(height*.75f*(j%2==0?1:-1));
                trees.Quad(foot-halfWidth,foot+halfWidth,foot+halfWidth+Vector3.up*height,
                    foot-halfWidth+Vector3.up*height,Color.white);
            }
        }
        trees.Mount(root,"谷外红枫",Resources.Load<Material>("Valley/Grove"));
    }

    void BuildStream(Transform root)
    {
        // This strip starts beyond the westernmost playable cell. The banks and
        // water share vertices at the shore, so there is no coplanar overlay.
        var mesh=new PaperMesh.Builder(2);
        const int stations=36;
        float[] columns={-1f,-.48f,-.13f,0f,1f,1.13f,1.48f,2f};
        Vector3 Bank(int row,int column){
            float d=row*1.1f;
            float bend=Mathf.Sin(d*.18f)*2.1f+Mathf.Sin(d*.075f+.8f)*2.6f;
            float width=1.2f+.6f*Mathf.Sin(.11f*d+.4f);
            float c=columns[column];
            float lateral=c>=0 && c<=1?c*width:(c<0?c*17:width+(c-1)*17);
            float away=c<0?-c:Mathf.Max(0,c-1);
            float y=eyeHeight-2.2f-.063f*d+away*(1.8f+Mathf.Sin(d*.23f));
            return new Vector3(bounds.min.x-.4f-d,y,bounds.center.z+bend+lateral);
        }
        for(int row=0;row<stations;row++)for(int col=0;col<columns.Length-1;col++)
            mesh.Quad(col==3?1:0,Vector3.up,Bank(row,col),Bank(row+1,col),Bank(row+1,col+1),Bank(row,col+1));
        var hillside=Resources.Load<Material>("Valley/Hillside");
        mesh.Mount(root,"溪谷河岸",hillside,Resources.Load<Material>("Valley/Brook"));
        var boulders=new PaperMesh.Builder(1);
        for(int i=0;i<12;i++){
            Vector3 center=Bank(2+i*3,i%2==0?2:5);
            float size=.35f+.52f*(1+Mathf.Sin(i*7.1f));
            Vector3 crest=center+new Vector3(size*.22f,size*.81f,-size*.13f);
            for(int k=0;k<7;k++){
                Vector3 Foot(int corner){
                    float angle=corner*2*Mathf.PI/7+i*.6f;
                    return center+new Vector3(Mathf.Cos(angle)*size,-.06f,Mathf.Sin(angle)*size*.67f);
                }
                Vector3 a=Foot(k),b=Foot(k+1),upperA=Vector3.Lerp(a,crest,.49f),upperB=Vector3.Lerp(b,crest,.49f);
                upperA.x+=size*.14f;upperB.x+=size*.14f;
                boulders.Quad(0,(a+b)*.5f-center,a,b,upperB,upperA);
                boulders.Triangle(0,(crest+(upperA+upperB)*.5f)*.5f-center,upperA,upperB,crest);
            }
        }
        boulders.Mount(root,"溪岸山石",hillside);
    }

    static readonly Vector3Int[] Edges={Vector3Int.right,Vector3Int.left,Vector3Int.forward,Vector3Int.back};

    static void BuildWildflowers(Transform root,Dictionary<Vector3Int,WhiteFaces> cells)
    {
        var marks=new Marks();
        const float unit=PaperTerrain.Cell;
        Color[] petals={new Color(1,.76f,.17f),new Color(.98f,.34f,.49f),new Color(.61f,.62f,.98f)};
        foreach(var tile in cells){
            Vector3Int at=tile.Key;
            if(((int)tile.Value & 8)!=0 || cells.ContainsKey(at+Vector3Int.up))continue;
            int seed=unchecked(at.x*73856093 ^ at.z*19349663 ^ at.y*83492791);
            var rng=new System.Random(seed);
            if(rng.NextDouble()>.34)continue;
            // Accent the lip of grassy ledges. Keep the middle of every landing clear.
            var edge=Vector3Int.zero;
            foreach(var dir in Edges)
                if(!cells.ContainsKey(at+dir)){edge=dir;break;}
            if(edge==Vector3Int.zero)continue;
            Vector3 tangent=Vector3.Cross(Vector3.up,(Vector3)edge);
            var center=((Vector3)at+new Vector3(.5f,1,.5f)+(Vector3)edge*.38f)*unit;
            Color tint=petals[rng.Next(petals.Length)];
            for(int j=0;j<8;j++){
                var basePoint=center+tangent*((float)rng.NextDouble()-.5f)*.87f-(Vector3)edge*(float)rng.NextDouble()*.23f;
                float h=.09f+(float)rng.NextDouble()*.12f;
                var stem=basePoint+Vector3.up*h+tangent*.04f;
                marks.Triangle(basePoint-tangent*.021f,basePoint+tangent*.021f,stem,new Color(.35f,.61f,.23f));
                if(j%2!=0)continue;
                for(int k=0;k<5;k++){
                    float a=k*2*Mathf.PI/5;
                    var outwards=new Vector3(Mathf.Cos(a),.42f,Mathf.Sin(a))*.05f;
                    var lateral=Vector3.Cross(Vector3.up,outwards)*.55f;
                    marks.Quad(stem,stem+outwards*.55f-lateral,stem+outwards,stem+outwards*.55f+lateral,tint);
                }
            }
        }
        marks.Mount(root,"崖畔花草",Resources.Load<Material>("Valley/Petals"));
    }

    sealed class Marks
    {
        readonly List<Vector3> vertices=new List<Vector3>();
        readonly List<Vector2> uv=new List<Vector2>();
        readonly List<Color32> colors=new List<Color32>();
        readonly List<int> triangles=new List<int>();
        void Vertex(Vector3 p,Vector2 tex,Color color){vertices.Add(p);uv.Add(tex);colors.Add(color);}
        public void Triangle(Vector3 a,Vector3 b,Vector3 c,Color color){
            int i=vertices.Count;
            Vertex(a,Vector2.zero,color);Vertex(b,Vector2.right,color);Vertex(c,Vector2.up,color);
            triangles.Add(i);triangles.Add(i+1);triangles.Add(i+2);
        }
        public void Quad(Vector3 a,Vector3 b,Vector3 c,Vector3 d,Color color){
            int i=vertices.Count;
            Triangle(a,b,c,color);uv[i+2]=Vector2.one;Vertex(d,Vector2.up,color);
            triangles.Add(i);triangles.Add(i+2);triangles.Add(i+3);
        }
        public void Mount(Transform root,string label,Material material){
            if(vertices.Count==0)return;
            var go=new GameObject(label,typeof(MeshFilter),typeof(MeshRenderer));
            go.transform.SetParent(root,false);
            var mesh=new Mesh{name=label,indexFormat=vertices.Count>65535?IndexFormat.UInt32:IndexFormat.UInt16};
            mesh.SetVertices(vertices);mesh.SetUVs(0,uv);mesh.SetColors(colors);mesh.SetTriangles(triangles,0);
            go.GetComponent<MeshFilter>().sharedMesh=mesh;
            go.AddComponent<PaperMesh>().Own(mesh);
            var renderer=go.GetComponent<MeshRenderer>();
            renderer.sharedMaterial=material;renderer.shadowCastingMode=ShadowCastingMode.Off;
        }
    }
}
