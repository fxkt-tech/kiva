using System.Collections.Generic;
using UnityEngine;

public sealed class WorkshopGeometry
{
    readonly List<Rigidbody> bodies=new List<Rigidbody>();
    readonly List<Pose> origins=new List<Pose>();
    public IReadOnlyList<Rigidbody> Bodies=>bodies;

    public WorkshopGeometry(WorkshopLayout layout)
    {
        var fixedCells = new Dictionary<Vector3Int, BlockFaces>();
        var unitBlocks = new Dictionary<Vector3Int, WorkshopBlock>();
        foreach (var block in layout.GetComponentsInChildren<WorkshopBlock>())
        {
            var t = block.transform;
            if (block.TryGetComponent<Rigidbody>(out var body))
            {
                var cells = new Dictionary<Vector3Int, BlockFaces> { { Vector3Int.zero, block.Faces } };
                PaperTerrain.BuildFaces(t, cells, body, -Vector3.one * (PaperTerrain.Cell * .5f));
                block.AnchorMovingTexture();
                block.GetComponent<WorkshopBrick>().Seal();
                body.centerOfMass = Vector3.zero;
                body.inertiaTensor = Vector3.one * (body.mass * PaperTerrain.Cell * PaperTerrain.Cell / 6f);
                bodies.Add(body);
                origins.Add(new Pose(t.localPosition, t.localRotation));
                continue;
            }
            var size = Vector3Int.RoundToInt(t.localScale);
            var start = Vector3Int.RoundToInt(t.localPosition / PaperTerrain.Cell - (Vector3)size * .5f);
            var center = ((Vector3)start + (Vector3)size * .5f) * PaperTerrain.Cell;
            if (t.parent != layout.transform || Quaternion.Angle(t.localRotation, Quaternion.identity) > .001f ||
                (t.localScale - (Vector3)size).sqrMagnitude > .000001f || (t.localPosition - center).sqrMagnitude > .000001f)
            {
                block.BuildIndividualFaces(layout.transform);
                continue;
            }
            foreach (var cell in new BoundsInt(start, size).allPositionsWithin)
                fixedCells[cell] = block.Faces;
            if (size == Vector3Int.one) unitBlocks[start] = block;
        }
        foreach (var entry in unitBlocks) entry.Value.HideInternalFaces(entry.Key, fixedCells);
        PaperTerrain.BuildFaces(layout.transform, fixedCells, null, Vector3.zero);
    }

    public void ResetBricks()
    {
        for(int i=0;i<bodies.Count;i++){
            var body=bodies[i];
            body.position=body.transform.parent.TransformPoint(origins[i].position);
            body.rotation=body.transform.parent.rotation * origins[i].rotation;
            body.linearVelocity=body.angularVelocity=Vector3.zero;
            body.WakeUp();
        }
    }
}
