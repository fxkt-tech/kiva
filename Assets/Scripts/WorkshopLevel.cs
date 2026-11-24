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
    [Serializable]
    public struct Perch
    {
        public string id;
        public BirdSpecies species;
        public Vector3 feet, flightOffset;
        public float yaw;
    }

    public string stageId;
    public Perch[] perches = Array.Empty<Perch>();

    void OnValidate()
    {
        if(string.IsNullOrWhiteSpace(stageId) || perches==null || perches.Length==0){
            Debug.LogWarning("请填写关卡编号并添加至少一个栖点。",this);
            return;
        }
        var ids = new System.Collections.Generic.HashSet<string>();
        foreach (var perch in perches)
            if (!perch.species || string.IsNullOrWhiteSpace(perch.species.id) || string.IsNullOrWhiteSpace(perch.id) || !ids.Add(perch.id))
                Debug.LogWarning("栖点需要鸟种和唯一编号。", this);
    }

    public string title, description;
    public string locationName;
    [TextArea] public string arrivalStory;
    public BrickRegion[] fixedBricks=Array.Empty<BrickRegion>();
    public LooseBrick[] physicalBricks=Array.Empty<LooseBrick>();
    public Vector3 spawnFeet, checkpointFeet, exitFeet, exitSign;
    public float spawnYaw, checkpointYaw;
    public Bounds supplyZone, exitZone;
    public WorkshopGear startingGear=WorkshopGear.Gun, supplyGear;
    public string entryHint, supplyHint, retryHint;
    public bool westWindow;
}
