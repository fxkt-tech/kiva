using System;
using UnityEngine;
using UnityEngine.Serialization;

[Flags]
public enum WorkshopGear { None=0, Gun=1, Mushroom=2, Fan=4, Glider=8, All=15 }

[Serializable]
public struct BrickRegion
{
    public string label;
    public Vector3Int start, size;
    [FormerlySerializedAs("white")] public BlockFaces faces;
}

[Serializable]
public struct LooseBrick
{
    public string label;
    public Vector3Int cell;
    [FormerlySerializedAs("white")] public BlockFaces faces;
    [Min(.1f)] public float mass;
}

[CreateAssetMenu(menuName="纸坊/关卡")]
public sealed class WorkshopLevel : ScriptableObject
{
    public string stageId;

    public string title, description;
    public string locationName;
    public BrickRegion[] fixedBricks=Array.Empty<BrickRegion>();
    public LooseBrick[] physicalBricks=Array.Empty<LooseBrick>();
    public Vector3 spawnFeet, checkpointFeet, exitFeet;
    public float spawnYaw, checkpointYaw;
    public Bounds supplyZone, exitZone;
    public WorkshopGear startingGear=WorkshopGear.Gun, supplyGear;
    public string entryHint, supplyHint, retryHint;
}
