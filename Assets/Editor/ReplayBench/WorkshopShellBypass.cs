using System.Collections.Generic;
using UnityEngine;

// Probe the actual inside faces of the new enclosure from the normal approach.
internal sealed class WorkshopShellBypass
{
    readonly WorkshopLevel map;
    readonly WorkshopTool tool;
    Vector3 near,far;
    bool attempted,hasTargets,leftWall;
    public string Name=>WorkshopToolSpec.For(tool).name+"借用围墙";
    public bool Unavailable=>attempted && !hasTargets;
    internal WorkshopShellBypass(WorkshopLevel map,WorkshopTool tool){this.map=map;this.tool=tool;}
    internal static bool OnWall(WorkshopLevel map,Vector3 point){
        foreach(var tile in map.fixedBricks){
            if(tile.label!="门洞侧墙" && tile.label!="房间围墙")continue;
            var size=(Vector3)tile.size*PaperTerrain.Cell;
            if(new Bounds((Vector3)tile.start*PaperTerrain.Cell+size*.5f,size).SqrDistance(point)<.001f)return true;
        }
        return false;
    }

    internal static List<WorkshopTool> Tools(WorkshopGenerationRequest r){
        var tools=new List<WorkshopTool>();
        // With a non-installable finish enabled the enclosure has no installable face.
        if(r==null || !r.Enclosed || r.HasSolidFinish)return tools;
        foreach(var pair in new[]{(WorkshopGear.Gun,WorkshopTool.Gun),(WorkshopGear.Mushroom,WorkshopTool.Mushroom),(WorkshopGear.Fan,WorkshopTool.Fan)})
            if((r.gear&pair.Item1)!=0)tools.Add(pair.Item2);
        return tools;
    }
    bool Find(Vector3 eye,Vector3 goal,Vector3 exclude,bool excludeNear,out Vector3 best){
        float closest=float.PositiveInfinity;best=default;
        foreach(var tile in map.fixedBricks){
            if(tile.label!="门洞侧墙" && tile.label!="房间围墙")continue;
            var lo=(Vector3)tile.start*PaperTerrain.Cell;var size=(Vector3)tile.size*PaperTerrain.Cell;
            var box=new Bounds(lo+size*.5f,size);
            var p=box.ClosestPoint(goal);
            // Use the visible side of the wall, inset from brick corners and at capsule height.
            for(int axis=0;axis<3;axis++){
                if(axis!=1 && (Mathf.Abs(eye[axis]-box.center[axis])>size[axis]*.5f) && size[axis]<=PaperTerrain.Cell+.01f)
                    p[axis]=eye[axis]<box.center[axis]?box.min[axis]:box.max[axis];
                else p[axis]=Mathf.Clamp(p[axis],box.min[axis]+.7f,box.max[axis]-.7f);
            }
            var ray=p-eye;
            if(ray.sqrMagnitude>35*35 || !Physics.Raycast(eye,ray.normalized,out var hit,ray.magnitude+.1f,Physics.DefaultRaycastLayers,QueryTriggerInteraction.Collide))continue;
            var face=hit.collider.GetComponentInParent<WorkshopFace>();
            if(!face || !face.AllowsInstallation || Mathf.Abs(hit.normal.y)>.1f || (hit.point-p).sqrMagnitude>.025f ||
               !face.Fits(face.transform.InverseTransformPoint(hit.point),WorkshopToolSpec.For(tool).radius))continue;
            if(excludeNear && Vector3.Distance(exclude,hit.point)<2f)continue;
            float score=(goal-hit.point).sqrMagnitude;
            if(score<closest){closest=score;best=hit.point;}
        }
        return closest<float.PositiveInfinity;
    }
    internal WorkshopInput Command(int elapsed,WorkshopCourse.Leg leg,WorkshopMotor motor,bool ported,
        System.Func<Vector3,WorkshopTool,bool,WorkshopInput> place,System.Func<Vector3,WorkshopInput> walk){
        if(!attempted){
            attempted=true;var eye=motor.Eyes.transform.position;
            hasTargets=Find(eye,eye,default,false,out near);
            if(hasTargets && tool==WorkshopTool.Gun)hasTargets=Find(eye,leg.destination.center+Vector3.up*.75f,near,true,out far);
        }
        if(!hasTargets)return default;
        if(elapsed==4)return tool==WorkshopTool.Gun?place(far,tool,true):place(near,tool,false);
        if(elapsed==5 && tool==WorkshopTool.Gun)return place(near,tool,false);
        leftWall|=ported || (tool==WorkshopTool.Mushroom && motor.Body.linearVelocity.y>1);
        return walk(tool==WorkshopTool.Fan || leftWall?leg.destination.center:near);
    }
}
