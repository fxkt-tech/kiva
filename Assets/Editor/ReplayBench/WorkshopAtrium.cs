using System.Collections.Generic;
using UnityEngine;
using Leg=WorkshopCourse.Leg;

// One light well separates a low working quay from the raised return gallery. Every static
// platform grows from the same foundation; only the working brick is free-standing.
internal static class WorkshopAtrium
{
    const float Cell=PaperTerrain.Cell;
    const int Bed=-11;
    static Vector3 At(float x,float y,float z)=>new Vector3(x,y,z)*Cell;
    static Bounds Deck(int x,int y,int z,int w,int d)=>new Bounds(At(x+w*.5f,y,z+d*.5f),new Vector3(w*Cell,.2f,d*Cell));

    internal static void Inspect(WorkshopLevel map,Leg first,WorkshopMotor motor,WorkshopGenerationReport.Attempt attempt){
        var tiles=map.fixedBricks;
        var seen=new HashSet<int>{0};var queue=new Queue<int>();queue.Enqueue(0);
        while(queue.Count>0){
            var a=tiles[queue.Dequeue()];
            for(int i=0;i<tiles.Length;i++){
                if(seen.Contains(i))continue;
                var b=tiles[i];
                // A shared face can bear a platform. Edges and corners alone cannot.
                var overlap=Vector3Int.Min(a.start+a.size,b.start+b.size)-Vector3Int.Max(a.start,b.start);
                if(overlap.x<0 || overlap.y<0 || overlap.z<0)continue;
                if((overlap.x==0?1:0)+(overlap.y==0?1:0)+(overlap.z==0?1:0)>1)continue;
                seen.Add(i);queue.Enqueue(i);
            }
        }
        attempt.connected=seen.Count==tiles.Length;
        var eye=motor.Eyes.transform.position;
        bool Clear(Vector3 point){
            var ray=point-eye;
            return !Physics.Raycast(eye,ray.normalized,ray.magnitude-.05f,Physics.DefaultRaycastLayers,QueryTriggerInteraction.Ignore);
        }
        attempt.exitVisible=Clear(map.exitFeet+Vector3.up*.25f);
        attempt.stationVisible=Clear(first.approach+Vector3.up*.10f);
    }

    internal static WorkshopLevel Build(WorkshopGenerationRequest r,string id,int seed,int attempt,out List<Leg> legs){
        var random=new System.Random(unchecked(seed+7919*attempt));
        bool push=r.grade==WorkshopGrade.Advanced,chain=r.grade==WorkshopGrade.Challenge;
        int east=random.Next(9,12),north=random.Next(8,11),south=-random.Next(8,11);
        int width=chain?3:push?4:5, upper=push?3:2, cut=chain?-3:-1, gap=chain?2:1;
        int end=north+2,west=-5,outside=east+width,roof=10;
        var blocks=new List<BrickRegion>();var loose=new List<LooseBrick>();legs=new List<Leg>();
        void Block(string label,int x,int y,int z,int w,int h,int d,BlockFaces faces=BlockFaces.Rock){
            blocks.Add(new BrickRegion{label=label,start=new Vector3Int(x,y,z),size=new Vector3Int(w,h,d),faces=faces});
        }
        void Bank(string label,int x,int z,int w,int d,int top)=>Block(label,x,Bed+1,z,w,top-Bed-1,d);
        void Gallery(string label,int z,int length){
            Block(label,east,upper-1,z,width,1,length);
            Bank("廊后挡墙",outside-1,z,1,length,upper-1);
            for(int bay=z;bay<z+length;bay+=4)Bank("廊下支墩",east,bay,width-1,1,upper-1);
            Bank("廊端承台",east,z+length-1,width-1,1,upper-1);
        }

        Block("纸槽基底",west-1,Bed,south-1,outside-west+3,1,end-south+3);
        Bank("起步台",west,south,-west,end-south,0);
        if(push)Gallery("落脚台 对岸回廊",south,end-south);
        else{
            Gallery("落脚台 上廊",cut,end-cut);
            Gallery("落脚台 出口廊",south,cut-gap-south);
        }
        // Piers and a continuous lintel mark the outer gallery edge in both space modes.
        for(int z=south;z<end;z+=4)
            Block("回廊外柱",outside,Bed+1,z,1,upper+4-Bed-1,1);
        Block("回廊檐梁",outside,upper+4,south,1,1,end-south);
        if(r.Enclosed){
            int left=west-1,right=outside+1,back=south-1,front=end+1;
            Block("房间围墙",left,Bed+1,back,1,roof-Bed-1,front-back+1);
            Block("房间围墙",right,Bed+1,back,1,roof-Bed-1,front-back+1);
            Block("房间围墙",left+1,Bed+1,back,right-left-1,roof-Bed-1,1);
            Block("房间围墙",left+1,Bed+1,front,right-left-1,roof-Bed-1,1);
            Block("房间顶棚",left,roof,back,right-left+1,1,front-back+1);
        }

        var spawn=At(-2.5f,0,south+1.5f);
        var goal=At(east+.45f,upper,south+1.5f);
        float lane=east+width*.5f;
        if(push){
            // The wooden quay supports the side of the channel, leaving the brick's lane clear.
            Bank("工位地基",0,0,2,6,-2);
            Block("槽边承台",1,-2,0,1,1,4);
            WorkshopCourse.Channel(blocks,loose,new Vector3Int(0,-1,0),1,4);
            Bank("风扇墙基",0,-1,1,1,-1);
            var station=new Leg{action=WorkshopAction.Push,brickSpring=true,glide=true,origin=At(0,-1,0),
                stop=3.5f*Cell,approach=new Vector3(-.42f,0,1.1f),mount=new Vector3(.75f,-.58f,0),
                destination=Deck(east,upper,north-3,width,5)};
            legs.Add(station);
            legs.Add(new Leg{action=WorkshopAction.Walk,heading=-1,approach=station.destination.center,
                destination=Deck(east,upper,south,width,4)});
        }else{
            int gateZ=chain?0:north-3;
            Bank("入门工位",0,gateZ,1,3,0);
            Block("入口门面",0,0,gateZ+1,1,1,1,BlockFaces.WhiteLeft);
            var portal=new Leg{action=WorkshopAction.Portal,approach=At(-2.5f,0,gateZ+.5f),mount=At(0,.5f,gateZ+1.5f),
                destination=chain?Deck(east,upper,cut+1,width,5):Deck(east,upper,gateZ-2,width,5)};
            if(chain){
                // The exit discharges straight onto a spring visible from the entrance quay.
                Bank("北侧弹射工位",0,north-3,4,5,0);
                Block("出口门面",2,0,north,1,1,1,BlockFaces.WhiteBack);
                Block("出门弹射白面",2,-1,north-2,1,1,1,BlockFaces.WhiteTop);
                portal.exitMount=At(2.5f,.5f,north);
                portal.springMount=At(2.5f,0,north-1.5f);portal.exitDirection=Vector3.back;
                portal.portalSpring=portal.glide=true;
            }else{
                int wallX=east+width/2;
                Block("出口门面",wallX,upper,gateZ+1,1,1,1,BlockFaces.WhiteBack);
                portal.exitMount=At(wallX+.5f,upper+.5f,gateZ+1);
            }
            legs.Add(portal);
            legs.Add(new Leg{action=WorkshopAction.Jump,heading=-1,approach=At(lane,upper,cut+3.5f),
                edge=-cut*Cell,destination=Deck(east,upper,south,width,cut-gap-south)});
        }
        // Rotation applies to brick faces, normal directions and all validation landmarks together.
        int rotation=random.Next(4);if(rotation==3)rotation=-1;
        var frame=new WorkshopCourseSpace.Frame(rotation,Vector3Int.zero);
        for(int i=0;i<blocks.Count;i++)blocks[i]=frame.Tile(blocks[i]);
        for(int i=0;i<loose.Count;i++)loose[i]=frame.Brick(loose[i]);
        foreach(var leg in legs){
            var along=frame.Point(leg.Forward);frame.Place(leg);
            leg.heading=along.x>.5f?0:along.x<-.5f?2:along.z>0?1:-1;
            leg.springMount=frame.Point(leg.springMount);leg.exitDirection=frame.Point(leg.exitDirection);
        }
        spawn=frame.Point(spawn);goal=frame.Point(goal);
        var glance=(goal-spawn).normalized+(legs[0].approach-spawn).normalized;
        float yaw=Mathf.Atan2(glance.x,glance.z)*Mathf.Rad2Deg;
        var map=WorkshopCourse.Map(r,id,"纸坊中庭 · "+r.GradeText,blocks,loose,spawn,goal,yaw);
        map.entryHint=push?"沿下层工位推砖，在砖顶装菇，展开纸翼飞向对面回廊。":
            chain?"先在远端门面前的白砖上装菇，再布置双门。出门弹起后，展开纸翼转向对岸回廊。":"在工位布置双门，到上层回廊后跳过排水缺口。";
        return map;
    }
}
