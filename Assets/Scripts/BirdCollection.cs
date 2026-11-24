using System;
using System.Collections.Generic;
using System.IO;
using UnityEngine;

// One instance per campaign. Preserve unknown IDs when updating progress.
public sealed class BirdCollection
{
    [Serializable] public struct Entry { public string stage, perch, species; }
    [Serializable] sealed class Ledger { public int version; public List<Entry> entries; }

    readonly string path;
    readonly List<Entry> entries;
    readonly List<Entry> pending = new List<Entry>();
    bool backedUpCopyIsGood;
    public bool NeedsSave => pending.Count > 0;
    public IReadOnlyList<Entry> Entries => entries;

    public BirdCollection(string path)
    {
        this.path = path;
        var ledger = Read(path);
        backedUpCopyIsGood = ledger != null;
        entries = (ledger ?? Read(path+".bak"))?.entries ?? new List<Entry>();
    }

    static Ledger Read(string file)
    {
        if (!File.Exists(file)) return null;
        try {
            var data = JsonUtility.FromJson<Ledger>(File.ReadAllText(file));
            if (data == null || data.version != 1 || data.entries == null ||
                data.entries.Exists(e => string.IsNullOrEmpty(e.stage) || string.IsNullOrEmpty(e.perch) || string.IsNullOrEmpty(e.species)))
                throw new InvalidDataException("无效的鸟卷记录");
            return data;
        } catch (Exception e) { Debug.LogWarning("读取鸟卷失败："+e.Message); return null; }
    }

    public bool Has(string stage,string perch) => entries.Exists(e => e.stage == stage && e.perch == perch);
    public bool HasSpecies(string species) => entries.Exists(e => e.species == species);
    public bool IsPending(string stage,string perch) => pending.Exists(e => e.stage == stage && e.perch == perch);
    public int Count(WorkshopLevel stage)
    {
        int count=0;
        foreach(var perch in stage.perches) if(Has(stage.stageId,perch.id)) count++;
        return count;
    }

    public void Queue(string stage, WorkshopLevel.Perch perch)
    {
        if (!Has(stage,perch.id) && !IsPending(stage,perch.id))
            pending.Add(new Entry { stage=stage, perch=perch.id, species=perch.species.id });
    }

    public bool Flush()
    {
        if (!NeedsSave) return true;
        try {
            Directory.CreateDirectory(Path.GetDirectoryName(path));
            var next = new List<Entry>(entries);
            next.AddRange(pending);
            string tmp = path+".tmp";
            File.WriteAllText(tmp,JsonUtility.ToJson(new Ledger {version=1,entries=next},true));
            if (File.Exists(path)) File.Replace(tmp,path,backedUpCopyIsGood ? path+".bak" : null);
            else File.Move(tmp,path);
            backedUpCopyIsGood=true;
            entries.AddRange(pending);
            pending.Clear();
            return true;
        } catch (Exception e) { Debug.LogWarning("收藏待保存："+e.Message); return false; }
    }
}
