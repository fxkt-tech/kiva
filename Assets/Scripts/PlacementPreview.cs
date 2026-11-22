using UnityEngine;
using UnityEngine.Rendering;

public sealed class PlacementPreview : MonoBehaviour
{
    readonly LineRenderer[] lines=new LineRenderer[5];
    Material ink;
    WorkshopTool lastTool=(WorkshopTool)(-1);

    public void Build()
    {
        ink=new Material(Shader.Find("HDRP/Unlit"));
        ink.SetColor("_UnlitColor",new Color(.94f,.96f,.96f));
        for(int i=0;i<lines.Length;i++){
            var line=new GameObject("白墨预览").AddComponent<LineRenderer>();
            line.transform.SetParent(transform,false);line.gameObject.layer=2;
            line.useWorldSpace=false;line.widthMultiplier=.018f;line.sharedMaterial=ink;
            line.shadowCastingMode=ShadowCastingMode.Off;line.receiveShadows=false;
            lines[i]=line;
        }
        gameObject.SetActive(false);
    }

    public void Show(Camera eye,WorkshopTool tool,PlacementAim aim)
    {
        gameObject.SetActive(aim.Valid);
        if(!aim.Valid)return;
        transform.SetPositionAndRotation(aim.mount.Position+aim.mount.face.transform.forward*.026f,aim.mount.face.transform.rotation);
        float depth=eye.WorldToViewportPoint(aim.mount.Position).z;
        float pixelWidth=2f*depth*Mathf.Tan(eye.fieldOfView*Mathf.Deg2Rad*.5f)/Mathf.Max(1,eye.pixelHeight);
        foreach(var line in lines)line.widthMultiplier=Mathf.Max(.016f,pixelWidth*1.7f);
        if(tool==lastTool)return;
        lastTool=tool;
        foreach(var line in lines){line.enabled=false;line.loop=false;}
        float r=aim.mount.radius;
        var ring=lines[0];ring.enabled=true;ring.loop=true;ring.positionCount=56;
        for(int i=0;i<56;i++){
            float a=i*2*Mathf.PI/56;
            ring.SetPosition(i,new Vector3(Mathf.Cos(a)*r,Mathf.Sin(a)*r,0));
        }
        if(tool==WorkshopTool.Gun)return;
        void Stroke(int i,params Vector3[] points){
            lines[i].enabled=true;lines[i].positionCount=points.Length;lines[i].SetPositions(points);
        }
        var tip=Vector3.forward*(tool==WorkshopTool.Fan?.83f:.55f);
        Stroke(1,Vector3.zero,tip,tip+new Vector3(.085f,0,-.13f));
        Stroke(2,tip,tip+new Vector3(-.085f,0,-.13f));
        if(tool==WorkshopTool.Fan)for(int i=0;i<2;i++){
            var p=Vector3.right*(i==0?-.21f:.21f);
            Stroke(3+i,p,p+Vector3.forward*.3f,p*1.2f+Vector3.forward*.59f);
        }else{
            Stroke(3,new Vector3(-.25f,0,.22f),new Vector3(0,0,.34f),new Vector3(.25f,0,.22f));
        }
    }

    void OnDestroy(){if(ink)Destroy(ink);}
}
