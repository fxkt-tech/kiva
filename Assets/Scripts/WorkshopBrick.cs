using System.Collections.Generic;
using UnityEngine;

// A closed brick uses one box, so contacts along a narrow guide cannot catch on six shell edges.
// Opening a portal switches it to the same perforated face colliders used by fixed surfaces.
public sealed class WorkshopBrick : MonoBehaviour
{
    readonly HashSet<WorkshopFace> holes=new HashSet<WorkshopFace>();
    WorkshopFace[] faces;
    BoxCollider solid;
    PhysicsMaterial finish;

    public void Seal()
    {
        faces=GetComponentsInChildren<WorkshopFace>(true);
        var core=new GameObject("整砖碰撞");
        core.transform.SetParent(transform,false);
        core.layer=2;
        solid=core.AddComponent<BoxCollider>();
        solid.size=Vector3.one*PaperTerrain.Cell;
        finish=new PhysicsMaterial("磨光竹轨"){staticFriction=.16f,dynamicFriction=.12f,frictionCombine=PhysicsMaterialCombine.Minimum};
        solid.sharedMaterial=finish;
        foreach(var face in faces)face.SetShell(false,finish);
    }

    public void SetOpening(WorkshopFace face,bool open)
    {
        bool wasClosed=holes.Count==0;
        if(open)holes.Add(face);else holes.Remove(face);
        bool closed=holes.Count==0;
        if(closed==wasClosed)return;
        solid.enabled=closed;
        foreach(var side in faces)side.SetShell(!closed,finish);
    }

    void OnDestroy(){if(finish)Destroy(finish);}
}
