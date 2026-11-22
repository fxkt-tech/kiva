using System;
using UnityEngine;

[Flags]
public enum WhiteFaces { Bamboo=0, Left=1, Right=2, Bottom=4, Top=8, Back=16, Front=32, All=63 }

[Flags]
public enum WorkshopGear { None=0, Gun=1, Mushroom=2, Fan=4, Glider=8, All=15 }

[Serializable]
public struct BrickRegion
{
    public string label;
    public Vector3Int start, size;
    public WhiteFaces white;
}

[Serializable]
public struct LooseBrick
{
    public string label;
    public Vector3Int cell;
    public WhiteFaces white;
    [Min(.1f)] public float mass;
}

[CreateAssetMenu(menuName="纸坊/关卡")]
public sealed class WorkshopLevel : ScriptableObject
{
    public string title, description;
    public BrickRegion[] fixedBricks=Array.Empty<BrickRegion>();
    public LooseBrick[] physicalBricks=Array.Empty<LooseBrick>();
    public Vector3 spawnFeet, checkpointFeet, exitFeet, exitSign;
    public float spawnYaw, checkpointYaw;
    public Bounds supplyZone, exitZone;
    public WorkshopGear startingGear=WorkshopGear.Gun, supplyGear;
    public string entryHint, supplyHint, retryHint;
    public bool westWindow, practice;
}
