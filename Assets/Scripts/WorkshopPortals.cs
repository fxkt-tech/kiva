using System.Collections.Generic;
using UnityEngine;

public sealed class WorkshopPortals
{
    public const float OuterRadius=PaperTerrain.Cell*.45f;
    public const float Radius=OuterRadius*.93f;
    static readonly Quaternion HalfTurn=Quaternion.Euler(0,180,0);
    readonly PortalWindow[] windows=new PortalWindow[2];
    readonly Transform root;
    readonly SurfaceMount[] slots;
    readonly List<Vector2> holes=new List<Vector2>(2);
    public bool Paired=>slots[0].face && slots[1].face;
    public bool Open(int i)=>slots[i].face;

    public WorkshopPortals(Transform owner,SurfaceMount[] mounts,Material[] fronts,Material[] backs)
    {
        slots=mounts;root=owner;
        for(int i=0;i<2;i++){
            var w=new GameObject(i==0?"青鸾门":"朱雀门").AddComponent<PortalWindow>();
            w.transform.SetParent(root,false);
            w.gameObject.layer=LayerMask.NameToLayer(i==0?"AzurePortal":"VermilionPortal");
            w.Build(96,fronts[i],backs[i],i==1);
            windows[i]=w;
            w.Clear();
        }
    }

    public void Prepare(Camera eyes){foreach(var w in windows)w.PrepareTarget(eyes);}

    void Cut(WorkshopFace face)
    {
        if(!face)return;
        holes.Clear();
        if(Paired)for(int i=0;i<2;i++)if(slots[i].face==face)holes.Add(slots[i].local);
        face.SetHoles(holes);
        if(face.Host)face.Host.WakeUp();
    }

    public void Place(int index,SurfaceMount mount)
    {
        var old=slots[index].face;
        slots[index]=mount;
        for(int i=0;i<2;i++){
            var at=slots[i];
            if(at.face)windows[i].Place(at.face.transform,at.Position,Paired);
        }
        Cut(old);
        if(mount.face!=old)Cut(mount.face);
        var other=slots[1-index].face;
        if(other!=old && other!=mount.face)Cut(other);
    }

    public void Clear()
    {
        var first=slots[0].face;var second=slots[1].face;
        slots[0]=slots[1]=default;
        Cut(first);if(second!=first)Cut(second);
        foreach(var w in windows){w.Clear();w.transform.SetParent(root,false);}
    }

    public void Draw(Camera eye)
    {
        for(int i=0;i<2;i++)windows[i].Draw(eye,windows[1-i].transform,Paired);
    }

    public bool Crossing(Vector3 eye,Vector3 velocity,out Vector3 mapped,out Quaternion turn,out Vector3 mappedVelocity,bool physics=false)
    {
        mapped=eye;turn=Quaternion.identity;mappedVelocity=velocity;
        if(!Paired)return false;
        for(int i=0;i<2;i++){
            var entry=slots[i].Pose(physics);
            var inverse=Quaternion.Inverse(entry.rotation);
            var local=inverse*(eye-entry.position);
            var relative=velocity-slots[i].face.PointVelocity(eye);
            float inward=-Vector3.Dot(relative,entry.forward);
            if(inward<=.001f || local.z>0 || local.z < -inward*Time.fixedDeltaTime-.006f)continue;
            Vector2 at=local+(inverse*relative)*(local.z/inward);
            if(at.sqrMagnitude>=Radius*Radius)continue;
            var exit=slots[1-i].Pose(physics);
            turn=exit.rotation*HalfTurn*inverse;
            mapped=exit.position+exit.rotation*(HalfTurn*local);
            mappedVelocity=slots[1-i].face.PointVelocity(mapped)+turn*relative;
            return true;
        }
        return false;
    }
}
