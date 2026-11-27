using System.Collections.Generic;
using UnityEngine;

// Group exposed grid cells into continuous installation and collision surfaces.
public static class PaperTerrain
{
    public const float Cell=1.5f;
    static readonly Vector3Int[] Normals={Vector3Int.left,Vector3Int.right,Vector3Int.down,Vector3Int.up,Vector3Int.back,Vector3Int.forward};
    public static void BuildFaces(Transform host,Dictionary<Vector3Int,WhiteFaces> cells,Rigidbody body,Vector3 offset)
    {
        const float unit=PaperTerrain.Cell;
        var planes=new Dictionary<(int side,int plane,bool white),HashSet<Vector2Int>>();
        for(int side=0;side<Normals.Length;side++){
            var normal=Normals[side];
            var up=normal.y==0?Vector3Int.up:(normal.y>0?Vector3Int.back:Vector3Int.forward);
            var right=Vector3Int.RoundToInt(Vector3.Cross(up,normal));
            foreach(var cell in cells){
                if(cells.ContainsKey(cell.Key+normal))continue;
                bool white=((int)cell.Value & (1<<side))!=0;
                Vector3 center=(Vector3)cell.Key+Vector3.one*.5f+(Vector3)normal*.5f;
                int plane=Mathf.RoundToInt(Vector3.Dot(center,normal));
                var tile=new Vector2Int(Mathf.FloorToInt(Vector3.Dot(center,right)),Mathf.FloorToInt(Vector3.Dot(center,up)));
                var key=(side,plane,white);
                if(!planes.TryGetValue(key,out var tiles))planes.Add(key,tiles=new HashSet<Vector2Int>());
                tiles.Add(tile);
            }
        }
        foreach(var group in planes){
            var normal=Normals[group.Key.side];
            var up=normal.y==0?Vector3.up:(normal.y>0?Vector3.back:Vector3.forward);
            var face=new GameObject(group.Key.white?"白面":"竹面").AddComponent<WorkshopFace>();
            face.transform.SetParent(host,false);
            face.transform.SetLocalPositionAndRotation(offset+(Vector3)normal*(group.Key.plane*unit),Quaternion.LookRotation(normal,up));
            face.Build(group.Value,group.Key.white,body);
        }
    }
}
