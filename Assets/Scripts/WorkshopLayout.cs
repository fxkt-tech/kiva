using System;
using System.Collections.Generic;
using UnityEngine;

public sealed class WorkshopLayout : MonoBehaviour
{
    [SerializeField] WorkshopLevel definition;
    [SerializeField] WorkshopBlock fixedBlock, movingBlock;
    [SerializeField] GameObject checkpointPrefab, supplyPrefab, exitPrefab;
    [SerializeField] Transform spawn, checkpoint;
    [SerializeField] StageZone supply, exit;
    public WorkshopLevel Definition => definition;
    public Transform Spawn => spawn;
    public Transform Checkpoint => checkpoint;
    public StageZone Supply => supply;
    public StageZone Exit => exit;

    // Used by the editor and replay bench. The playable level is saved as nested prefab instances.
    public void Populate(WorkshopLevel source, Func<GameObject, Transform, GameObject> create)
    {
        definition = source;
        Place(spawn, source.spawnFeet, source.spawnYaw);
        if (source.supplyZone.size.sqrMagnitude > 0)
        {
            checkpoint = create(checkpointPrefab, transform).transform;
            Place(checkpoint, source.checkpointFeet, source.checkpointYaw);
            supply = create(supplyPrefab, transform).GetComponent<StageZone>();
            SetZone(supply, source.supplyZone);
        }
        if (source.exitZone.size.sqrMagnitude > 0)
        {
            exit = create(exitPrefab, transform).GetComponent<StageZone>();
            SetZone(exit, source.exitZone);
        }
        var regions = new List<BrickRegion>();
        foreach (var next in source.fixedBricks)
        {
            // Later regions override earlier cells. Split intersecting boxes once during authoring
            // so rendering and portal installation see the same material at every boundary.
            var pieces = new List<BrickRegion>();
            foreach (var old in regions) Subtract(old, next, pieces);
            pieces.Add(next);
            regions = pieces;
        }
        foreach (var region in regions)
        {
            var block = create(fixedBlock.gameObject, transform).GetComponent<WorkshopBlock>();
            block.name = region.label;
            block.transform.localPosition = ((Vector3)region.start + (Vector3)region.size * .5f) * PaperTerrain.Cell;
            block.transform.localScale = region.size;
            block.SetFaces(region.white);
        }
        foreach (var loose in source.physicalBricks)
        {
            var block = create(movingBlock.gameObject, transform).GetComponent<WorkshopBlock>();
            block.name = loose.label;
            block.transform.localPosition = ((Vector3)loose.cell + Vector3.one * .5f) * PaperTerrain.Cell;
            block.GetComponent<Rigidbody>().mass = Mathf.Max(.1f, loose.mass);
            block.SetFaces(loose.white);
        }
    }

    static void Subtract(BrickRegion old, BrickRegion next, List<BrickRegion> output)
    {
        var lo = old.start;
        var hi = lo + old.size;
        var first = Vector3Int.Max(lo, next.start);
        var last = Vector3Int.Min(hi, next.start + next.size);
        if (first.x >= last.x || first.y >= last.y || first.z >= last.z)
        {
            output.Add(old);
            return;
        }
        for (int axis = 0; axis < 3; axis++)
        {
            if (lo[axis] < first[axis])
            {
                var end = hi;
                end[axis] = first[axis];
                output.Add(new BrickRegion { label = old.label, start = lo, size = end - lo, white = old.white });
                lo[axis] = first[axis];
            }
            if (hi[axis] > last[axis])
            {
                var start = lo;
                start[axis] = last[axis];
                output.Add(new BrickRegion { label = old.label, start = start, size = hi - start, white = old.white });
                hi[axis] = last[axis];
            }
        }
    }

    static void Place(Transform point, Vector3 feet, float yaw) =>
        point.SetLocalPositionAndRotation(feet, Quaternion.Euler(0, yaw, 0));

    static void SetZone(StageZone zone, Bounds bounds)
    {
        zone.transform.localPosition = bounds.center;
        zone.transform.localScale = bounds.size;
    }
}
