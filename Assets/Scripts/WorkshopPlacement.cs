using UnityEngine;

public enum WorkshopTool { Gun, Mushroom, Fan }

public readonly struct WorkshopToolSpec
{
    public readonly string name;
    public readonly int slot;
    public readonly float radius;
    WorkshopToolSpec(string name,int slot,float radius){this.name=name;this.slot=slot;this.radius=radius;}
    static readonly WorkshopToolSpec[] definitions={
        new WorkshopToolSpec("时空枪",0,WorkshopPortals.OuterRadius),
        new WorkshopToolSpec("弹弹菇",2,.49f),new WorkshopToolSpec("风扇",3,.53f)
    };
    public static WorkshopToolSpec For(WorkshopTool tool)=>definitions[(int)tool];
    public static WorkshopToolSpec ForMount(int slot)=>definitions[slot<2?0:slot-1];
    public static bool Owned(WorkshopGear gear,int i)=>((int)gear & (1<<i))!=0;
}

public struct SurfaceMount
{
    public WorkshopFace face;
    public Vector2 local;
    public float radius;
    public Vector3 Position=>face.transform.TransformPoint(local);

    public Pose Pose(bool physics)
    {
        var tile=face.transform;
        var body=face.Host;
        if(!physics || !body)return new Pose(Position,tile.rotation);
        // FixedUpdate uses the current rigidbody pose. The render view uses its interpolated pose.
        var localPoint=tile.localPosition+tile.localRotation*(Vector3)local;
        return new Pose(body.position+body.rotation*localPoint,body.rotation*tile.localRotation);
    }
}

public struct PlacementAim
{
    public SurfaceMount mount;
    public string reason;
    public bool Valid=>mount.face && reason==null;
}

public sealed class WorkshopPlacement
{
    // Blue, vermilion, mushroom and fan each retain one installation.
    readonly SurfaceMount[] mounts=new SurfaceMount[4];
    readonly PaperDevice[] devices=new PaperDevice[2];
    public WorkshopPortals Portals { get; }
    public bool Installed(int slot)=>mounts[slot].face;

    public WorkshopPlacement(Transform root,Material[] fronts,Material[] backs,Material paper,Material bamboo)
    {
        Portals=new WorkshopPortals(root,mounts,fronts,backs);
        for(int i=0;i<2;i++){
            var d=new GameObject(i==0?"弹弹菇":"风扇").AddComponent<PaperDevice>();
            d.transform.SetParent(root,false);
            d.Build(i==0?WorkshopTool.Mushroom:WorkshopTool.Fan,paper,bamboo,backs[1]);
            devices[i]=d;
            d.gameObject.SetActive(false);
        }
    }

    public PlacementAim Aim(Camera eye,int slot)
    {
        if(!Physics.Raycast(eye.transform.position,eye.transform.forward,out var hit,35f,Physics.DefaultRaycastLayers,QueryTriggerInteraction.Collide))
            return new PlacementAim{reason="需要白面"};
        var face=hit.collider.GetComponentInParent<WorkshopFace>();
        if(!face || !face.White || Vector3.Dot(face.transform.forward,hit.normal)<.95f)
            return new PlacementAim{reason="需要白面"};
        return Check(face,hit.point,slot);
    }

    public PlacementAim Check(WorkshopFace face,Vector3 world,int slot)
    {
        var mount=new SurfaceMount{face=face,local=face.transform.InverseTransformPoint(world),radius=WorkshopToolSpec.ForMount(slot).radius};
        var aim=new PlacementAim{mount=mount};
        if(!face.White)aim.reason="需要白面";
        else if(!face.Fits(mount.local,mount.radius))aim.reason="边缘空间不足";
        else for(int i=0;i<mounts.Length;i++){
            var other=mounts[i];
            if(i==slot || !other.face)continue;
            Vector3 delta=mount.Position-other.Position;
            if(Vector3.Dot(face.transform.forward,other.face.transform.forward)>.99f &&
                Mathf.Abs(Vector3.Dot(delta,face.transform.forward))<.04f &&
                delta.magnitude<mount.radius+other.radius-.0002f){aim.reason="此处已有装置";break;}
        }
        return aim;
    }

    public void Install(int slot,PlacementAim aim)
    {
        if(!aim.Valid)return;
        if(slot<2){Portals.Place(slot,aim.mount);return;}
        mounts[slot]=aim.mount;
        devices[slot-2].Mount(aim.mount);
    }

    public void Clear(Transform root)
    {
        Portals.Clear();
        for(int i=0;i<2;i++){
            mounts[i+2]=default;
            devices[i].gameObject.SetActive(false);
            devices[i].transform.SetParent(root,false);
        }
    }

    public void ApplyDevices(Rigidbody player,System.Collections.Generic.IReadOnlyList<Rigidbody> bricks)
    {
        foreach(var device in devices){
            if(!device.gameObject.activeInHierarchy)continue;
            device.Apply(player);
            foreach(var brick in bricks)device.Apply(brick);
        }
    }
}
