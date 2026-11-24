using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

// An open architectural model viewed from above the south-east corner.
[RequireComponent(typeof(CanvasRenderer))]
public sealed class StagePaperModel : MaskableGraphic
{
    public WorkshopLevel stage;
    Vector2 Project(float x,float y,float z){
        var r=rectTransform.rect;float unit=Mathf.Min(r.width/18f,r.height/12.5f);
        return r.center+new Vector2((x+z)*.74f,(z-x)*.29f+y*.77f-1.35f)*unit;
    }
    protected override void OnPopulateMesh(VertexHelper vh){
        vh.Clear();if(!stage)return;
        void Quad(Vector2 a,Vector2 b,Vector2 c,Vector2 d,Color tone){
            int k=vh.currentVertCount;
            vh.AddVert(a,tone,Vector2.zero);vh.AddVert(b,tone,Vector2.zero);vh.AddVert(c,tone,Vector2.zero);vh.AddVert(d,tone,Vector2.zero);
            vh.AddTriangle(k,k+1,k+2);vh.AddTriangle(k,k+2,k+3);
        }
        // Use the open floor plan at collectible heights; remove the enclosing shell in the model.
        var cells=new Dictionary<Vector2Int,int>();
        foreach(var region in stage.fixedBricks){
            if(region.start.y>=5 || region.start.x<=-5 || region.start.x>=6 || region.start.z>=5 || region.start.z<=-5)continue;
            int h=region.start.y+region.size.y;
            for(int x=region.start.x;x<region.start.x+region.size.x;x++)
                for(int z=region.start.z;z<region.start.z+region.size.z;z++){
                    var k=new Vector2Int(x,z);
                    if(!cells.TryGetValue(k,out int old) || h>old)cells[k]=h;
                }
        }
        var coords=new List<Vector2Int>(cells.Keys);
        coords.Sort((a,b)=>(b.y-b.x).CompareTo(a.y-a.x));
        foreach(var k in coords){
            float x=k.x,z=k.y,h=cells[k],floor=-4;
            var a=Project(x,h,z);var b=Project(x+1,h,z);var c=Project(x+1,h,z+1);var d=Project(x,h,z+1);
            Quad(a,b,Project(x+1,floor,z),Project(x,floor,z),new Color(.62f,.75f,.78f));
            Quad(b,c,Project(x+1,floor,z+1),Project(x+1,floor,z),new Color(.46f,.62f,.67f));
            Color upper=h<0?new Color(.71f,.83f,.86f):new Color(.90f,.92f,.86f);
            if(((k.x+k.y)&1)==0)upper=Color.Lerp(upper,Color.white,.15f);
            Quad(a,b,c,d,upper);
        }
        foreach(var p in stage.perches){
            var at=p.feet/PaperTerrain.Cell;var v=Project(at.x,at.y+.16f,at.z);
            var pin=new Vector2(0,6);var side=new Vector2(4,0);
            Quad(v,v-side+pin,v+pin*2,v+side+pin,p.species.pigments[0]);
        }
    }
}
