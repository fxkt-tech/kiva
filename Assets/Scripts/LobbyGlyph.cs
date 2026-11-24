using UnityEngine;
using UnityEngine.UI;

// Small line icons for the guest room chrome. They follow the Canvas scale and tint.
[RequireComponent(typeof(CanvasRenderer))]
public sealed class LobbyGlyph : MaskableGraphic
{
    public enum Symbol { Bulletin, Letter, Tick, Feather, Journal, Lantern, Forge, Theatre, Market, Lock, More, Window }
    public Symbol symbol;

    protected override void OnPopulateMesh(VertexHelper mesh)
    {
        mesh.Clear();
        Rect bounds=GetPixelAdjustedRect();
        float radius=Mathf.Min(bounds.width,bounds.height)*.5f;
        var center=bounds.center;

        void Stroke(Vector2 a,Vector2 b){
            float weight=Mathf.Max(1.15f,radius*.14f);
            var direction=(b-a).normalized;
            var side=new Vector2(-direction.y,direction.x)*weight*.5f;
            a=center+a*radius;b=center+b*radius;
            int index=mesh.currentVertCount;
            mesh.AddVert(a-side,color,Vector2.zero);
            mesh.AddVert(a+side,color,Vector2.zero);
            mesh.AddVert(b+side,color,Vector2.zero);
            mesh.AddVert(b-side,color,Vector2.zero);
            mesh.AddTriangle(index,index+1,index+2);mesh.AddTriangle(index,index+2,index+3);
        }
        void Path(params Vector2[] points){
            for(int i=1;i<points.Length;i++)Stroke(points[i-1],points[i]);
        }
        void Box(float left,float bottom,float right,float top){
            Path(new Vector2(left,bottom),new Vector2(left,top),new Vector2(right,top),new Vector2(right,bottom),new Vector2(left,bottom));
        }
        void Line(float x,float y,float a,float b)=>Stroke(new Vector2(x,y),new Vector2(a,b));
        void Arc(Vector2 pivot,float rx,float ry,int start,int end){
            Vector2 Point(float a)=>pivot+new Vector2(Mathf.Cos(a*Mathf.Deg2Rad)*rx,Mathf.Sin(a*Mathf.Deg2Rad)*ry);
            var from=Point(start);
            for(int i=1;i<=10;i++){
                var to=Point(Mathf.Lerp(start,end,i/10f));Stroke(from,to);from=to;
            }
        }

        switch(symbol){
            case Symbol.Bulletin:
                Box(-.58f,-.73f,.59f,.57f);
                Line(-.35f,.77f,.35f,.77f);Line(-.30f,.19f,.30f,.19f);
                Line(-.30f,-.09f,.34f,-.09f);Line(-.30f,-.37f,.08f,-.37f);
                break;
            case Symbol.Letter:
                Box(-.78f,-.55f,.78f,.55f);
                Path(new Vector2(-.78f,.55f),new Vector2(0,-.04f),new Vector2(.78f,.55f));
                Line(-.77f,-.55f,-.31f,-.14f);Line(.77f,-.55f,.31f,-.14f);
                break;
            case Symbol.Tick:
                Box(-.61f,-.64f,.61f,.63f);Line(-.28f,.84f,-.28f,.44f);Line(.28f,.84f,.28f,.44f);
                Path(new Vector2(-.33f,-.10f),new Vector2(-.09f,-.32f),new Vector2(.36f,.16f));
                break;
            case Symbol.Feather:
                Path(new Vector2(-.65f,-.70f),new Vector2(.33f,.33f),new Vector2(.69f,.75f),new Vector2(.05f,.63f),
                    new Vector2(-.48f,.05f),new Vector2(-.41f,-.42f),new Vector2(.24f,-.20f),new Vector2(.69f,.75f));
                Line(-.16f,-.13f,.36f,-.07f);Line(.04f,.10f,-.05f,.48f);
                break;
            case Symbol.Journal:
                Path(new Vector2(-.74f,.49f),new Vector2(-.40f,.55f),new Vector2(0,.33f),new Vector2(.40f,.55f),
                    new Vector2(.74f,.49f),new Vector2(.74f,-.50f),new Vector2(.40f,-.40f),new Vector2(0,-.61f),
                    new Vector2(-.40f,-.40f),new Vector2(-.74f,-.50f),new Vector2(-.74f,.49f));
                Line(0,-.61f,0,.33f);Line(.25f,.07f,.56f,.16f);Line(-.56f,.12f,-.24f,.05f);
                break;
            case Symbol.Lantern:
                Arc(Vector2.zero,.58f,.59f,0,360);
                Arc(Vector2.zero,.27f,.59f,0,360);
                Line(-.28f,.66f,.28f,.66f);Line(-.28f,-.66f,.28f,-.66f);Line(0,-.69f,0,-.87f);
                break;
            case Symbol.Forge:
                Box(-.53f,-.49f,.56f,.24f);
                Line(-.77f,-.57f,.76f,-.57f);Line(0,-.49f,0,.22f);
                Path(new Vector2(-.75f,.24f),new Vector2(-.32f,.53f),new Vector2(0,.72f),new Vector2(.34f,.52f),new Vector2(.76f,.24f),new Vector2(-.75f,.24f));
                break;
            case Symbol.Theatre:
                Path(new Vector2(-.76f,-.66f),new Vector2(-.76f,.64f),new Vector2(.76f,.64f),new Vector2(.76f,-.66f));
                Path(new Vector2(-.72f,-.39f),new Vector2(-.41f,-.16f),new Vector2(-.32f,.24f),new Vector2(-.31f,.63f));
                Path(new Vector2(.72f,-.39f),new Vector2(.41f,-.16f),new Vector2(.32f,.24f),new Vector2(.31f,.63f));
                Line(-.56f,-.65f,.56f,-.65f);
                break;
            case Symbol.Market:
                Path(new Vector2(-.77f,.45f),new Vector2(.77f,.45f),new Vector2(.63f,-.02f),new Vector2(-.64f,-.02f),new Vector2(-.77f,.45f));
                Line(-.28f,.45f,-.22f,-.02f);Line(.28f,.45f,.22f,-.02f);
                Path(new Vector2(-.57f,-.02f),new Vector2(-.57f,-.63f),new Vector2(.57f,-.63f),new Vector2(.57f,-.02f));
                break;
            case Symbol.Lock:
                Arc(new Vector2(0,.18f),.29f,.36f,0,180);Box(-.49f,-.58f,.49f,.17f);Line(0,-.47f,0,-.19f);
                break;
            case Symbol.More:
                for(int i=-1;i<=1;i++)Arc(new Vector2(i*.53f,0),.07f,.07f,0,360);
                break;
            case Symbol.Window:
                Box(-.58f,-.65f,.58f,.65f);
                Line(0,-.65f,0,.65f);Line(-.58f,.04f,.58f,.04f);
                break;
        }
    }
}
