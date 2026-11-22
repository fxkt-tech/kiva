using System.Collections.Generic;
using UnityEngine;

public sealed class WorkshopGeometry
{
    readonly List<Rigidbody> bodies=new List<Rigidbody>();
    readonly List<Vector3> origins=new List<Vector3>();
    public IReadOnlyList<Rigidbody> Bodies=>bodies;

    public WorkshopGeometry(WorkshopLevel level,Transform root,Material[] faceMaterials)
    {
        var fixedCells=new Dictionary<Vector3Int,WhiteFaces>();
        foreach(var region in level.fixedBricks){
            for(int z=0;z<region.size.z;z++)for(int y=0;y<region.size.y;y++)for(int x=0;x<region.size.x;x++)
                fixedCells[region.start+new Vector3Int(x,y,z)]=region.white;
        }
        PaperTerrain.BuildFaces(root,fixedCells,null,Vector3.zero,faceMaterials);
        foreach(var brick in level.physicalBricks){
            var body=new GameObject(brick.label).AddComponent<Rigidbody>();
            body.transform.SetParent(root,false);
            body.transform.localPosition=((Vector3)brick.cell+Vector3.one*.5f)*PaperTerrain.Cell;
            body.mass=Mathf.Max(.1f,brick.mass);
            body.interpolation=RigidbodyInterpolation.Interpolate;
            body.collisionDetectionMode=CollisionDetectionMode.ContinuousDynamic;
            PaperTerrain.BuildFaces(body.transform,new Dictionary<Vector3Int,WhiteFaces>{{Vector3Int.zero,brick.white}},body,-Vector3.one*(PaperTerrain.Cell*.5f),faceMaterials);
            body.gameObject.AddComponent<WorkshopBrick>().Seal();
            body.centerOfMass=Vector3.zero;
            body.inertiaTensor=Vector3.one*(body.mass*PaperTerrain.Cell*PaperTerrain.Cell/6f);
            bodies.Add(body);
            origins.Add(body.transform.localPosition);
        }
    }

    public void ResetBricks()
    {
        for(int i=0;i<bodies.Count;i++){
            var body=bodies[i];
            body.position=body.transform.parent.TransformPoint(origins[i]);
            body.rotation=body.transform.parent.rotation;
            body.linearVelocity=body.angularVelocity=Vector3.zero;
            body.WakeUp();
        }
    }
}
