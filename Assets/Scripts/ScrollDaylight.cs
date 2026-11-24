using UnityEngine;
using UnityEngine.UI;

[RequireComponent(typeof(CanvasRenderer))]
public sealed class ScrollDaylight : MaskableGraphic
{
    float next;
    void Update(){
        if(Time.unscaledTime<next)return;
        next=Time.unscaledTime+1f/24;SetVerticesDirty();
    }
    protected override void OnPopulateMesh(VertexHelper mesh){
        mesh.Clear();
        var r=rectTransform.rect;
        float time=Application.isPlaying?Time.unscaledTime:0;
        for(int i=0;i<22;i++){
            float phase=Mathf.Repeat(i*.618034f+time*.047f,1);
            float x=.202f+Mathf.Repeat(i*.171f,.43f)+.016f*Mathf.Sin(phase*4+i);
            float y=.28f+.40f*Mathf.Repeat(i*.719f,1)+.07f*phase;
            var pos=new Vector2(r.xMin+x*r.width,r.yMin+y*r.height);
            float radius=r.height*(.0013f+.00075f*Mathf.Sin(i));
            var tint=new Color(1,.83f,.57f,.72f*Mathf.Sin(phase*Mathf.PI));
            int k=mesh.currentVertCount;
            mesh.AddVert(pos+Vector2.up*radius,tint,Vector2.zero);mesh.AddVert(pos+Vector2.right*radius,tint,Vector2.zero);
            mesh.AddVert(pos+Vector2.down*radius,tint,Vector2.zero);mesh.AddVert(pos+Vector2.left*radius,tint,Vector2.zero);
            mesh.AddTriangle(k,k+1,k+2);mesh.AddTriangle(k,k+2,k+3);
        }
    }
}
