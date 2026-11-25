using System;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;

// Each crossing contributes its own runway, mounting surfaces and destination.
// Coordinates are brick cells until the controller's landmarks are recorded in metres.
internal static class WorkshopCourse
{
    const float Cell=PaperTerrain.Cell;
    [Serializable] internal sealed class Leg
    {
        public WorkshopAction action;
        public bool brickSpring,glide;
        public Vector3 approach,mount,exitMount,origin;
        public Bounds destination;
        public float edge,stop;
        public int heading;
        public bool hasJunction;
        public Vector3 junction;
        public bool portalSpring;
        public Vector3 springMount,exitDirection;
        public Vector3 Forward=>WorkshopCourseSpace.Axis(heading);
        public Vector3 Side=>new Vector3(-Forward.z,0,Forward.x);
    }
    static BrickRegion Region(string name,int x,int y,int z,int w,int h,int d,WhiteFaces faces=WhiteFaces.Bamboo)=>
        new BrickRegion{label=name,start=new Vector3Int(x,y,z),size=new Vector3Int(w,h,d),white=faces};

    static void Recess(List<BrickRegion> blocks,int edge,int height,int lane){
        // Split the runway around the moving brick so the rigidbody has its own supporting tile.
        int index=blocks.FindLastIndex(b=>b.start.x+b.size.x==edge && b.start.y==height-1 &&
            b.start.z<=lane && lane<b.start.z+b.size.z);
        var pad=blocks[index];blocks.RemoveAt(index);
        blocks.Add(Region(pad.label,pad.start.x,height-1,pad.start.z,pad.size.x-1,1,pad.size.z));
        int left=lane-pad.start.z,right=pad.size.z-left-1;
        if(left>0)blocks.Add(Region(pad.label,edge-1,height-1,pad.start.z,1,1,left));
        if(right>0)blocks.Add(Region(pad.label,edge-1,height-1,lane+1,1,1,right));
        blocks.Add(Region("活动砖座",edge-1,height-2,lane,1,1,1));
    }

    internal static void Channel(List<BrickRegion> blocks,List<LooseBrick> bricks,Vector3Int at,int start,int stop){
        int x=at.x,y=at.y,z=at.z;
        blocks.Add(Region("直槽底板",x,y-1,z,1,1,stop));
        blocks.Add(Region("风扇安装面",x,y,z-1,1,1,1,WhiteFaces.Front));
        blocks.Add(Region("起步侧台",x-2,y,z,2,1,stop+1));
        blocks.Add(Region("槽边",x+1,y,z,1,1,stop));
        blocks.Add(Region("止挡",x,y,z+stop,1,1,1));
        bricks.Add(new LooseBrick{label="风槽纸砖",cell=at+new Vector3Int(0,0,start),mass=2,white=WhiteFaces.Top});
    }
    internal static WorkshopLevel Map(WorkshopGenerationRequest r,string id,string title,List<BrickRegion> blocks,List<LooseBrick> bricks,Vector3 spawn,Vector3 bird,float yaw){
        var map=ScriptableObject.CreateInstance<WorkshopLevel>();
        map.stageId=id;map.title=title;map.spawnFeet=spawn;map.spawnYaw=yaw;
        map.entryHint=r.Description;map.startingGear=r.gear;map.exitFeet=bird;map.exitSign=bird+Vector3.down*.5f;
        map.perches=new[]{new WorkshopLevel.Perch{id="landing",species=Resources.Load<BirdSpecies>("Birds/barn-swallow"),feet=bird,yaw=-90}};
        map.fixedBricks=blocks.ToArray();map.physicalBricks=bricks.ToArray();
        if(!r.Has(WorkshopModule.Bamboo)){
            for(int i=0;i<map.fixedBricks.Length;i++)map.fixedBricks[i].white=WhiteFaces.All;
            for(int i=0;i<map.physicalBricks.Length;i++)map.physicalBricks[i].white=WhiteFaces.All;
        }
        return map;
    }
    internal static WorkshopLevel PushMap(WorkshopGenerationRequest r,string id,WorkshopGenerationReport.Layout layout){
        var blocks=new List<BrickRegion>();var bricks=new List<LooseBrick>();
        Channel(blocks,bricks,Vector3Int.zero,layout.start,layout.stop);
        blocks.Add(Region("目标高台",layout.nearX,3,layout.nearZ,8-layout.nearX,1,10-layout.nearZ));
        if(r.Enclosed)WorkshopCourseSpace.Enclose(blocks,new[]{WorkshopCourseSpace.Room(blocks,8)});
        var map=Map(r,id,"推砖渡槽",blocks,bricks,new Vector3(-.55f,1.5f,1.4f),new Vector3(9.8f,6f,11.4f),137);
        map.exitSign=new Vector3(8,5.45f,layout.nearZ*Cell-.04f);return map;
    }

    internal static WorkshopLevel Build(WorkshopGenerationRequest r,string id,int seed,int attempt,int[] bypasses,out List<Leg> legs){
        var random=new System.Random(unchecked(seed+attempt*7919));
        var blocks=new List<BrickRegion>();var bricks=new List<LooseBrick>();legs=new List<Leg>();
        int edge=random.Next(1,4),height=0,lane=random.Next(-3,4);
        var spawn=new Vector3((edge-4.5f)*Cell,height*Cell,(lane+.5f)*Cell);
        var rooms=new List<BoundsInt>();
        var headings=r.version>=2?WorkshopCourseSpace.Headings(r.actions,random):null;
        if(r.version<2)blocks.Add(Region("起步台",edge-7,height-1,lane-2,7,1,5));
        Vector3 bird=default;
        for(int i=0;i<r.actions.Length;i++){
            var leg=new Leg{action=r.actions[i]};
            if(leg.action==WorkshopAction.Push && i+1<r.actions.Length && r.actions[i+1]==WorkshopAction.Spring){leg.brickSpring=true;i++;}
            if(i+1<r.actions.Length && r.actions[i+1]==WorkshopAction.Glide){leg.glide=true;i++;}
            int index=legs.Count;
            var tiles=r.version>=2?new List<BrickRegion>():blocks;
            var loose=r.version>=2?new List<LooseBrick>():bricks;
            var offset=Vector3Int.zero;
            if(r.version>=2){
                leg.heading=headings[index];
                if(index>0){
                    offset=WorkshopCourseSpace.Join(legs[index-1].destination,leg.heading);
                    edge=7;height=0;lane=0;
                    leg.hasJunction=true;leg.junction=legs[index-1].destination.center;
                }
                tiles.Add(Region(index==0?"起步台":"连接走台",edge-7,height-1,lane-2,7,1,5));
            }
            int near=edge,width=random.Next(7,10),depth=5;
            float line=(lane+.5f)*Cell;
            leg.edge=edge*Cell;
            leg.approach=new Vector3((edge-3.5f)*Cell,height*Cell,line);
            leg.mount=new Vector3((edge-.5f)*Cell,height*Cell,line);
            switch(leg.action){
            case WorkshopAction.Walk: break;
            case WorkshopAction.Jump:
                near+=leg.glide?random.Next(5,7):random.Next(1,3);
                if(leg.glide)height-=2;
                break;
            case WorkshopAction.Spring:
            case WorkshopAction.LooseSpring:
                if(leg.action==WorkshopAction.LooseSpring){
                    Recess(tiles,edge,height,lane);
                    loose.Add(new LooseBrick{label="弹射活动砖",cell=new Vector3Int(edge-1,height-1,lane),mass=2,white=WhiteFaces.Top});
                }else tiles.Add(Region("弹射白面",edge-1,height-1,lane,1,1,1,WhiteFaces.Top));
                near+=leg.glide?random.Next(9,11):random.Next(4,6);height+=leg.glide?0:2;
                break;
            case WorkshopAction.Wind:
                tiles.Add(Region("顺风安装面",edge-5,height,lane+1,1,1,1,WhiteFaces.Right));
                leg.approach=new Vector3((edge-2.5f)*Cell,height*Cell,line+Cell);
                leg.mount=new Vector3((edge-4)*Cell,(height+.5f)*Cell,line+Cell);
                int windLeg=legs.Count;
                near+=3+Math.Min(2,bypasses.Count(n=>n==windLeg));height--;
                break;
            case WorkshopAction.Portal:
                tiles.Add(Region("入口门面",edge-1,height,lane+1,1,1,1,WhiteFaces.Left));
                leg.mount=new Vector3((edge-1)*Cell,(height+.5f)*Cell,line+Cell);
                near+=random.Next(4,7);height+=2;
                leg.exitMount=new Vector3((near+1.5f)*Cell,(height+.5f)*Cell,(lane+1)*Cell);
                break;
            case WorkshopAction.Push:
                var at=new Vector3Int(edge+2,height-1,lane);
                int stop=4;
                Channel(tiles,loose,at,1,stop);
                leg.origin=(Vector3)at*Cell;
                leg.approach=leg.origin+new Vector3(-.42f,Cell,1.1f);
                leg.stop=leg.origin.z+(stop-.5f)*Cell;
                leg.mount=leg.origin+new Vector3(.75f,.92f,0);
                if(leg.brickSpring){near=at.x+(leg.glide?10:4);height+=3;lane=at.z+(leg.glide?7:8);depth=7;}
                else{
                    near=at.x+2;lane=at.z+3;
                    tiles.Add(Region("移开风扇的位置",at.x-2,at.y,at.z+stop-2,1,1,1,WhiteFaces.Top));
                    leg.exitMount=leg.origin+new Vector3(-1.5f*Cell,Cell,(stop-1.5f)*Cell);
                }
                break;
            }
            int low=lane-depth/2;
            tiles.Add(Region("落脚台 "+(legs.Count+1),near,height-1,low,width,1,depth));
            if(leg.action==WorkshopAction.Portal)
                tiles.Add(Region("出口门面",near+1,height,lane+1,1,1,1,WhiteFaces.Back));
            leg.destination=new Bounds(new Vector3((near+width*.5f)*Cell,height*Cell,(low+depth*.5f)*Cell),new Vector3(width*Cell,.2f,depth*Cell));
            bird=new Vector3(leg.destination.max.x-2*Cell,leg.destination.center.y,leg.destination.center.z);
            if(r.version>=2){
                int clearance=leg.brickSpring || leg.action==WorkshopAction.Spring || leg.action==WorkshopAction.LooseSpring?8:4;
                var room=WorkshopCourseSpace.Room(tiles,clearance);
                var frame=new WorkshopCourseSpace.Frame(leg.heading,offset);
                rooms.Add(frame.Box(room));
                if(r.Enclosed && index>0)WorkshopCourseSpace.Door(tiles,room,edge-6,0,0);
                foreach(var tile in tiles)blocks.Add(frame.Tile(tile));
                foreach(var body in loose)bricks.Add(frame.Brick(body));
                frame.Place(leg);bird=frame.Point(bird);
            }
            edge=near+width;legs.Add(leg);
        }
        if(r.Enclosed)WorkshopCourseSpace.Enclose(blocks,rooms);
        return Map(r,id,"寻鸟 · "+WorkshopGenerationRequest.Names[(int)legs[0].action],blocks,bricks,spawn,bird,90);
    }
}
