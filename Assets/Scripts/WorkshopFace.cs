using System.Collections.Generic;
using UnityEngine;

// One coordinate system for coplanar faces of one kind on the same rigid host.
public sealed class WorkshopFace : MonoBehaviour
{
    public bool White { get; private set; }
    public Rigidbody Host { get; private set; }
    HashSet<Vector2Int> tiles;
    readonly List<PortalSurfaceCollider> colliders=new List<PortalSurfaceCollider>();
    Rect rectangle;
    const float Cell=PaperTerrain.Cell;

    public void Build(HashSet<Vector2Int> cells,bool white,Rigidbody host)
    {
        tiles=cells;
        White=white;
        Host=host;
        var remaining=new HashSet<Vector2Int>(cells);
        var ordered=new List<Vector2Int>(cells);
        ordered.Sort((a,b)=>a.y==b.y?a.x.CompareTo(b.x):a.y.CompareTo(b.y));
        foreach(var at in ordered){
            if(!remaining.Contains(at))continue;
            int width=1,height=1;
            while(remaining.Contains(at+Vector2Int.right*width))width++;
            bool CompleteRow(int y){
                for(int x=0;x<width;x++)if(!remaining.Contains(at+new Vector2Int(x,y)))return false;
                return true;
            }
            while(CompleteRow(height))height++;
            for(int y=0;y<height;y++)for(int x=0;x<width;x++)remaining.Remove(at+new Vector2Int(x,y));
            var rect=new Rect(at.x*Cell,at.y*Cell,width*Cell,height*Cell);
            AddPatch(rect);
        }
    }

    public void Build(Rect bounds, bool white)
    {
        White = white;
        rectangle = bounds;
        AddPatch(bounds);
    }

    void AddPatch(Rect bounds)
    {
        var patch = new GameObject("面碰撞").AddComponent<PortalSurfaceCollider>();
        patch.transform.SetParent(transform, false);
        patch.Initialize(bounds, White || Host);
        colliders.Add(patch);
    }

    public bool Fits(Vector2 center,float radius)
    {
        if(!White)return false;
        // A tangent tile edge counts as fitting, even on a rotated physical brick.
        float r=radius-.0002f;
        if (tiles == null)
            return center.x-r >= rectangle.xMin && center.x+r <= rectangle.xMax &&
                   center.y-r >= rectangle.yMin && center.y+r <= rectangle.yMax;
        var lo=Vector2Int.FloorToInt((center-Vector2.one*r)/Cell);
        var hi=Vector2Int.FloorToInt((center+Vector2.one*r)/Cell);
        for(int y=lo.y;y<=hi.y;y++)for(int x=lo.x;x<=hi.x;x++){
            if(tiles.Contains(new Vector2Int(x,y)))continue;
            var nearest=new Vector2(Mathf.Clamp(center.x,x*Cell,(x+1)*Cell),Mathf.Clamp(center.y,y*Cell,(y+1)*Cell));
            if((center-nearest).sqrMagnitude<r*r)return false;
        }
        return true;
    }

    public void SetHoles(List<Vector2> centers)
    {
        foreach(var patch in colliders)patch.Rebuild(centers);
        if(Host)Host.GetComponent<WorkshopBrick>().SetOpening(this,centers.Count>0);
    }

    public void SetShell(bool enabled,PhysicsMaterial material)
    {
        foreach(var patch in colliders)patch.SetShell(enabled,material);
    }

    public Vector3 PointVelocity(Vector3 worldPoint)=>Host?Host.GetPointVelocity(worldPoint):Vector3.zero;
}
