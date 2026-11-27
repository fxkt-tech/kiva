using System.Collections.Generic;
using UnityEngine;

public sealed class PaperDevice : MonoBehaviour
{
    [SerializeField] WorkshopTool kind;
    WorkshopFace host;
    [SerializeField] Transform rotor;
    readonly HashSet<Rigidbody> bounced=new HashSet<Rigidbody>();
    float pulse;
    public WorkshopTool Kind=>kind;
    public event System.Action<Rigidbody,Rigidbody> Applied;

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
            float force=48f*Mathf.Clamp01(1-speed/12f);
            body.AddForce(normal*force,ForceMode.Force);
            if(force>0)Applied?.Invoke(body,host.Host);
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
        Applied?.Invoke(body,host.Host);
        pulse=1;
    }
}
