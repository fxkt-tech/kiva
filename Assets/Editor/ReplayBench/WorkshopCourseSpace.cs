using System.Collections.Generic;
using System.Linq;
using UnityEngine;

internal static class WorkshopCourseSpace
{
    const float Cell=PaperTerrain.Cell;
    internal static Vector3Int Axis(int heading)=>heading switch {
        1=>Vector3Int.forward, -1=>Vector3Int.back, 2=>Vector3Int.left, _=>Vector3Int.right
    };

    internal static int[] Headings(WorkshopAction[] actions,System.Random random){
        int count=0;
        for(int i=0;i<actions.Length;i++){
            if(actions[i]==WorkshopAction.Push && i+1<actions.Length && actions[i+1]==WorkshopAction.Spring)i++;
            if(i+1<actions.Length && actions[i+1]==WorkshopAction.Glide)i++;
            count++;
        }
        var headings=new int[count];
        // Two outward axes give either handedness without folding later rooms onto earlier ones.
        int side=random.Next(2)==0?1:-1;
        for(int i=1;i<count;i++){
            bool bend=i==1 || (i>=2 && headings[i-1]==headings[i-2]) || random.Next(3)==0;
            headings[i]=bend?(headings[i-1]==0?side:0):headings[i-1];
        }
        return headings;
    }

    internal static Vector3Int Join(Bounds platform,int heading){
        var axis=(Vector3)Axis(heading);
        var side=new Vector3(-axis.z,0,axis.x);
        var edge=platform.center+axis*Vector3.Dot(Vector3.Scale(axis,axis),platform.extents);
        // The local runway centre is half a brick from the grid origin.
        return Vector3Int.RoundToInt(edge/Cell-side*.5f);
    }

    internal readonly struct Frame
    {
        readonly Vector3Int along,side,offset;
        internal Frame(int heading,Vector3Int offset){
            along=Axis(heading);side=new Vector3Int(-along.z,0,along.x);this.offset=offset;
        }
        Vector3Int Grid(Vector3Int p)=>offset+along*p.x+Vector3Int.up*p.y+side*p.z;
        internal Vector3 Point(Vector3 p)=>(Vector3)offset*Cell+(Vector3)along*p.x+Vector3.up*p.y+(Vector3)side*p.z;
        internal BoundsInt Box(BoundsInt box){
            var a=Grid(box.min);var b=Grid(box.max);
            return new BoundsInt(Vector3Int.Min(a,b),Vector3Int.Max(a,b)-Vector3Int.Min(a,b));
        }
        static WhiteFaces Face(Vector3Int v)=>v.x>0?WhiteFaces.Right:v.x<0?WhiteFaces.Left:v.z>0?WhiteFaces.Front:WhiteFaces.Back;
        WhiteFaces Rotate(WhiteFaces faces){
            var rotated=faces & (WhiteFaces.Top|WhiteFaces.Bottom);
            if((faces&WhiteFaces.Right)!=0)rotated|=Face(along);
            if((faces&WhiteFaces.Left)!=0)rotated|=Face(-along);
            if((faces&WhiteFaces.Front)!=0)rotated|=Face(side);
            if((faces&WhiteFaces.Back)!=0)rotated|=Face(-side);
            return rotated;
        }
        internal BrickRegion Tile(BrickRegion tile){
            var bounds=Box(new BoundsInt(tile.start,tile.size));
            tile.start=bounds.min;tile.size=bounds.size;tile.white=Rotate(tile.white);return tile;
        }
        internal LooseBrick Brick(LooseBrick brick){
            brick.cell=Box(new BoundsInt(brick.cell,Vector3Int.one)).min;brick.white=Rotate(brick.white);return brick;
        }
        internal void Place(WorkshopCourse.Leg leg){
            var bounds=leg.destination;
            var a=Point(bounds.min);var b=Point(bounds.max);
            bounds.SetMinMax(Vector3.Min(a,b),Vector3.Max(a,b));leg.destination=bounds;
            leg.approach=Point(leg.approach);leg.mount=Point(leg.mount);leg.exitMount=Point(leg.exitMount);
            leg.origin=Point(leg.origin);
            leg.edge+=Vector3.Dot(offset,along)*Cell;leg.stop+=Vector3.Dot(offset,side)*Cell;
        }
    }

    internal static BoundsInt Room(List<BrickRegion> tiles,int clearance){
        var lo=tiles[0].start;var hi=lo+tiles[0].size;
        foreach(var t in tiles){lo=Vector3Int.Min(lo,t.start);hi=Vector3Int.Max(hi,t.start+t.size);}
        lo-=new Vector3Int(2,0,2);hi+=new Vector3Int(2,clearance,2);
        return new BoundsInt(lo,hi-lo);
    }
    static void Add(List<BrickRegion> blocks,string label,int x,int y,int z,int w,int h,int d){
        if(w>0 && h>0 && d>0)blocks.Add(new BrickRegion{label=label,start=new Vector3Int(x,y,z),size=new Vector3Int(w,h,d)});
    }
    internal static void Door(List<BrickRegion> blocks,BoundsInt room,int x,int lane,int floor){
        Add(blocks,"门洞侧墙",x,floor,room.zMin,1,room.yMax-floor,lane-1-room.zMin);
        Add(blocks,"门洞侧墙",x,floor,lane+2,1,room.yMax-floor,room.zMax-lane-2);
        Add(blocks,"门楣",x,floor+3,lane-1,1,room.yMax-floor-3,3);
    }

    internal static void Enclose(List<BrickRegion> blocks,IReadOnlyList<BoundsInt> rooms){
        var roof=new Dictionary<Vector2Int,int>();
        int bottom=Mathf.Min(-12,rooms.Min(r=>r.yMin)-12);
        foreach(var r in rooms)
            for(int x=r.xMin;x<r.xMax;x++)for(int z=r.zMin;z<r.zMax;z++){
                var p=new Vector2Int(x,z);
                if(!roof.TryGetValue(p,out int old) || old<r.yMax)roof[p]=r.yMax;
            }
        var offsets=new[]{Vector2Int.left,Vector2Int.right,Vector2Int.up,Vector2Int.down};
        // Merge equal adjacent columns. Overlapping room footprints share their interior.
        foreach(var row in roof.Keys.GroupBy(p=>p.x).OrderBy(group=>group.Key)){
            int start=0,end=0,top=0,wall=0;bool pending=false;
            void Flush(){
                if(!pending)return;
                Add(blocks,"房间顶棚",row.Key,top,start,1,1,end-start);
                Add(blocks,"深坑底板",row.Key,bottom,start,1,1,end-start);
                Add(blocks,"房间围墙",row.Key,wall,start,1,top-wall,end-start);
            }
            foreach(var p in row.OrderBy(p=>p.y)){
                int ceiling=roof[p],baseY=ceiling;
                foreach(var d in offsets){
                    if(!roof.TryGetValue(p+d,out int neighbor)){baseY=bottom;break;}
                    baseY=Mathf.Min(baseY,neighbor);
                }
                if(!pending || p.y!=end || ceiling!=top || baseY!=wall){
                    Flush();start=p.y;top=ceiling;wall=baseY;pending=true;
                }
                end=p.y+1;
            }
            Flush();
        }
    }
}
