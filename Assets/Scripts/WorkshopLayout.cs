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
        if (source.architecture) create(source.architecture, transform);
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
        // Regions are an authoring shortcut. A later entry paints over the same cell.
        var cells = new Dictionary<Vector3Int, BrickRegion>();
        foreach (var region in source.fixedBricks)
            foreach (var cell in new BoundsInt(region.start, region.size).allPositionsWithin)
                cells[cell] = region;

        foreach (var entry in cells)
        {
            var prefab = entry.Value.blockPrefab ? entry.Value.blockPrefab : fixedBlock;
            var block = create(prefab.gameObject, transform).GetComponent<WorkshopBlock>();
            var cell = entry.Key;
            block.name = $"{entry.Value.label} ({cell.x},{cell.y},{cell.z})";
            block.transform.localPosition = ((Vector3)cell + Vector3.one * .5f) * PaperTerrain.Cell;
            block.SetFaces(entry.Value.faces);
        }
        foreach (var loose in source.physicalBricks)
        {
            var block = create(movingBlock.gameObject, transform).GetComponent<WorkshopBlock>();
            block.name = loose.label;
            block.transform.localPosition = ((Vector3)loose.cell + Vector3.one * .5f) * PaperTerrain.Cell;
            block.GetComponent<Rigidbody>().mass = Mathf.Max(.1f, loose.mass);
            block.SetFaces(loose.faces);
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
