using System.Collections.Generic;
using UnityEngine;

public sealed class PaperDevice : MonoBehaviour
{
    WorkshopTool kind;
    WorkshopFace host;
    Transform rotor;
    readonly HashSet<Rigidbody> bounced=new HashSet<Rigidbody>();
    float pulse;

    static Vector3 Ring(int i,int sides,float radius,float z)
    {
        float angle=i*2*Mathf.PI/sides;
        return new Vector3(Mathf.Cos(angle)*radius,Mathf.Sin(angle)*radius,z);
    }

    public void Build(WorkshopTool tool,Material white,Material green,Material ochre)
    {
        kind=tool;
        var baseMesh=new PaperMesh.Builder(3);
        int n=tool==WorkshopTool.Fan?16:12;
        for(int i=0;i<n;i++){
            var a=Ring(i,n,.49f,.04f);var b=Ring(i+1,n,.49f,.04f);
            if(tool==WorkshopTool.Fan){
                var c=Ring(i,n,.46f,.16f);var d=Ring(i+1,n,.46f,.16f);
                baseMesh.Quad(1,a+b,a,b,d,c);
                baseMesh.Quad(0,Vector3.forward,c,d,Ring(i+1,n,.39f,.15f),Ring(i,n,.39f,.15f));
                baseMesh.Triangle(1,Vector3.forward,Vector3.forward*.05f,a,b);
            }else{
                var stemA=Ring(i,n,.15f,.03f);var stemB=Ring(i+1,n,.15f,.03f);
                baseMesh.Quad(1,stemA+stemB,stemA,stemB,stemB+Vector3.forward*.19f,stemA+Vector3.forward*.19f);
            }
        }
        baseMesh.Mount(transform,"装置底座",white,green,ochre);
        var top=new PaperMesh.Builder(3);
        if(kind==WorkshopTool.Fan){
            for(int i=0;i<3;i++){
                var turn=Quaternion.Euler(0,0,120*i);
                var hub=Vector3.forward*.21f;
                var l=turn*new Vector3(-.08f,.34f,.19f);var r=turn*new Vector3(.17f,.30f,.16f);
                var fold=turn*new Vector3(.025f,.36f,.245f);
                top.Triangle(0,Vector3.forward,hub,l,fold);
                top.Triangle(2,Vector3.forward,hub,fold,r);
            }
        }else for(int i=0;i<n;i++){
            var a=Ring(i,n,.48f,.22f);var b=Ring(i+1,n,.48f,.22f);
            var ridge=Ring(i,n,.26f,.34f);var next=Ring(i+1,n,.26f,.34f);
            top.Quad(i%2==0?0:2,Vector3.forward,a,b,next,ridge);
            top.Triangle(0,Vector3.forward,ridge,next,Vector3.forward*.38f);
            top.Triangle(1,Vector3.back,a,b,Vector3.forward*.12f);
        }
        rotor=top.Mount(transform,kind==WorkshopTool.Fan?"三叶折扇":"纸褶菌盖",white,green,ochre).transform;
    }

    public void Mount(SurfaceMount mount)
    {
        host=mount.face;
        transform.SetParent(host.transform,false);
        transform.SetLocalPositionAndRotation(mount.local,Quaternion.identity);
        bounced.Clear();
        gameObject.SetActive(true);
    }

    void Update()
    {
        if(kind==WorkshopTool.Fan)rotor.Rotate(0,0,220*Time.deltaTime);
        else{
            pulse=Mathf.MoveTowards(pulse,0,Time.deltaTime*3.8f);
            rotor.localScale=new Vector3(1+pulse*.13f,1+pulse*.13f,1-pulse*.6f);
        }
    }

    public void Apply(Rigidbody body)
    {
        if(body==host.Host)return;
        Vector3 normal=transform.forward,center=body.worldCenterOfMass;
        var p=transform.InverseTransformPoint(center);
        float radial=new Vector2(p.x,p.y).magnitude;
        if(kind==WorkshopTool.Fan){
            if(p.z<=0 || p.z>9 || radial>1.05f+.075f*p.z)return;
            var origin=transform.position+normal*.16f;
            var line=center-origin;
            if(Physics.Raycast(origin,line,out var hit,line.magnitude,~0,QueryTriggerInteraction.Ignore) && hit.rigidbody!=body)return;
            float speed=Vector3.Dot(body.linearVelocity-host.PointVelocity(center),normal);
            body.AddForce(normal*(48f*Mathf.Clamp01(1-speed/12f)),ForceMode.Force);
            return;
        }
        float extent;
        if(body.TryGetComponent<CapsuleCollider>(out var cap))
            extent=cap.radius+(cap.height*.5f-cap.radius)*Mathf.Abs(Vector3.Dot(normal,body.transform.up));
        else{
            var n=body.transform.InverseTransformDirection(normal);
            extent=PaperTerrain.Cell*.5f*(Mathf.Abs(n.x)+Mathf.Abs(n.y)+Mathf.Abs(n.z));
        }
        if(p.z<=0 || p.z>extent+.38f || radial>.49f+Mathf.Min(.34f,extent)){bounced.Remove(body);return;}
        float approach=Vector3.Dot(body.linearVelocity-host.PointVelocity(center),normal);
        if(approach>.3f || !bounced.Add(body))return;
        body.AddForce(normal*(12.4f-approach),ForceMode.VelocityChange);
        pulse=1;
    }
}
