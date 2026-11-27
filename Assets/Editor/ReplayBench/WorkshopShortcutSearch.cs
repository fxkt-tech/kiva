using System.Collections.Generic;
using UnityEngine;

// Sample approaches to the visible mounting surfaces. These trials can produce
// a witness of a second route; exhausting the samples cannot certify uniqueness.
sealed class WorkshopShortcutSearch
{
    internal static readonly int[] RunUpAngles={0,-12,12,-24,24};
    enum Phase { CrossBrick, ReachFarBank, AimFromBank, PlaceOnWall, LeaveFan, ReturnAcrossBrick,
        ReachRunUp, LineUp, PlaceOnBrick, Run, AfterWallKick, FindExit }
    struct Candidate
    {
        public Vector3 spring, wall, normal, direction;
        public bool fan,wallSpring,updraft;
        public float lane;
    }

    readonly WorkshopMotor motor;
    readonly Vector3 goal;
    readonly List<Candidate> candidates=new List<Candidate>();
    int index=-1,phaseStart;
    Phase phase;
    float pitch;
    public int Count=>candidates.Count;
    public int Number=>index+1;
    public bool WallSpring=>candidates[index].wallSpring;
    public bool Updraft=>candidates[index].updraft;
    public Vector3 Wall=>candidates[index].wall;
    public Vector3 Spring=>candidates[index].spring;

    public WorkshopShortcutSearch(FerryPuzzle puzzle,WorkshopLevel level,bool wallAssist,Vector3? landingEdge=null,bool roomWallsOnly=false)
    {
        motor=puzzle.GetComponentInChildren<WorkshopMotor>();
        goal=level.exitFeet;
        if((level.startingGear & WorkshopGear.Mushroom)==0)return;
        var walls=new List<(Vector3 point,Vector3 normal)>();
        var tops=new List<Vector3>();
        foreach(var face in puzzle.GetComponentsInChildren<WorkshopFace>()){
            if(!face.White)continue;
            var normal=face.transform.forward;
            foreach(var box in face.GetComponentsInChildren<BoxCollider>()){
                if(!box.isTrigger)continue;
                var center=box.transform.TransformPoint(box.center+Vector3.forward*(box.size.z*.5f));
                if(face.Host && normal.y>.99f)tops.Add(center);
                else if(!face.Host && Mathf.Abs(normal.y)<.01f && (!roomWallsOnly || WorkshopShellBypass.OnWall(level,center)))walls.Add((center,normal));
            }
        }
        foreach(var top in tops){
            Vector3 toward=Horizontal(goal-top).normalized;
            if(!wallAssist){
                void Sample(Vector3 target){
                    var heading=Horizontal(target-top).normalized;
                    foreach(float angle in RunUpAngles)
                        candidates.Add(new Candidate{spring=top,direction=Quaternion.Euler(0,angle,0)*heading});
                }
                Sample(goal);
                if(landingEdge.HasValue)Sample(landingEdge.Value);
                continue;
            }
            if((level.startingGear & WorkshopGear.Fan)==0)continue;
            foreach(var wall in walls){
                var along=Horizontal(Vector3.ProjectOnPlane(top-wall.point,wall.normal)).normalized;
                // Sample the upper edge towards the launch tile, inside the fan's legal footprint.
                var fan=wall.point+Vector3.up*.2f+along*.20f;
                var spring=top-along*.22f;
                if(along.sqrMagnitude>.5f){
                    foreach(float alongOffset in new[]{0f,.19f,-.19f})
                        foreach(float lane in new[]{.48f,.36f,.62f})
                            candidates.Add(new Candidate{spring=top+wall.normal*.20f+along*alongOffset,
                                normal=wall.normal,wall=wall.point+Vector3.up*.24f+along*.22f,
                                direction=along,wallSpring=true,updraft=true,lane=lane});
                    candidates.Add(new Candidate{spring=spring+wall.normal*.20f,normal=wall.normal,wall=wall.point+Vector3.up*.24f+along*.22f,
                        direction=along,wallSpring=true,lane=.48f});
                }
                foreach(float offset in new[]{0f,-.16f,.16f})
                    foreach(float angle in new[]{0f,-6f,6f,-12f,12f,-24f,24f}){
                        var dir=Quaternion.Euler(0,angle,0)*toward;
                        candidates.Add(new Candidate{spring=top+along*offset,normal=wall.normal,wall=fan,direction=dir,fan=true});
                    }
            }
        }
        // Prefer unobstructed wind directed across the proposed approach to the goal.
        if(wallAssist)candidates.Sort((a,b)=>Score(b).CompareTo(Score(a)));
    }

    static float Score(Candidate c)
    {
        var offset=Horizontal(c.spring-c.wall);
        if(c.updraft)return 20;
        if(c.wallSpring)return 10;
        return Mathf.Round(Vector3.ProjectOnPlane(offset,c.normal).magnitude)-Mathf.Abs(Vector3.Dot(offset,c.normal))*.1f;
    }
    static Vector3 Horizontal(Vector3 v)=>new Vector3(v.x,0,v.z);

    public bool Next(){
        phase=Phase.CrossBrick;
        phaseStart=0;
        pitch=-4f; // FerryPuzzle's normal spawn camera pitch
        return ++index<candidates.Count;
    }

    WorkshopInput Face(Vector3 direction,Vector2 movement)
    {
        float yaw=Mathf.Atan2(direction.x,direction.z)*Mathf.Rad2Deg;
        float tilt=-Mathf.Atan2(direction.y,Horizontal(direction).magnitude)*Mathf.Rad2Deg;
        var input=new WorkshopInput{turn=new Vector2(Mathf.DeltaAngle(motor.Yaw,yaw),tilt-pitch),movement=movement};
        pitch=tilt;
        return input;
    }

    WorkshopInput Install(Vector3 point,WorkshopTool tool)
    {
        var eye=motor.Body.position+Vector3.up*(.90f*WorkshopMotor.BirdScale-WorkshopMotor.HalfHeight);
        var input=Face(point-eye,Vector2.zero);
        input.selectMask=1<<(int)tool;
        input.primary=true;
        return input;
    }

    WorkshopInput Approach(Vector3 point,int step)
    {
        var delta=Horizontal(point-motor.Body.position);
        float speed=Horizontal(motor.Body.linearVelocity).magnitude;
        if(delta.magnitude<.13f && speed<1f){
            phase++;
            phaseStart=step;
            return default;
        }
        if(delta.magnitude<speed*speed/54f+.09f && Vector3.Dot(motor.Body.linearVelocity,delta)>0)return default;
        if(delta.magnitude<.09f)return default;
        return Face(delta,Vector2.up);
    }

    public WorkshopInput Command(int step,int wallKick)
    {
        var c=candidates[index];
        var approach=c.spring;
        if(c.fan){
            var along=Horizontal(Vector3.ProjectOnPlane(c.spring-c.wall,c.normal)).normalized;
            approach=c.wall+c.normal*.18f+along*.55f;
        }
        var behind=approach-c.direction*1.45f;
        switch(phase){
        case Phase.CrossBrick:
            if(step<8)return default;
            if(!c.fan && !c.wallSpring){phase=Phase.LineUp;return default;}
            return Approach(c.spring,step);
        case Phase.ReachFarBank:
            return Approach(c.spring+c.normal*1.35f,step);
        case Phase.AimFromBank:
            return Approach(c.wall+c.normal*1.95f,step);
        case Phase.PlaceOnWall:
            if(step-phaseStart<10)return default;
            phase++;
            return Install(c.wall,c.wallSpring?WorkshopTool.Mushroom:WorkshopTool.Fan);
        case Phase.LeaveFan:
            return Approach(c.spring+c.normal*1.35f,step);
        case Phase.ReturnAcrossBrick:
            return Approach(c.spring-c.normal*1.45f,step);
        case Phase.ReachRunUp:
            if(c.wallSpring)return Approach(c.wall-c.normal*.36f-c.direction*.63f,step);
            return Approach(behind,step);
        case Phase.LineUp:
            if(c.wallSpring)return Approach(c.wall+c.normal*c.lane-c.direction*1.5f,step);
            if(!c.fan)return Approach(behind,step);
            phase++;return default;
        case Phase.PlaceOnBrick:
            if(step-phaseStart<10)return default;
            phase++;phaseStart=step;
            if(c.updraft)return Install(c.spring,WorkshopTool.Fan);
            if(c.wallSpring)return default;
            return Install(c.spring,WorkshopTool.Mushroom);
        case Phase.Run:
            if(step-phaseStart<3)return default;
            if(c.wallSpring && wallKick>0){
                phase++;
                return c.updraft?Face(c.direction,Vector2.up):Install(c.spring,WorkshopTool.Mushroom);
            }
            if(motor.Grounded && motor.Body.position.y>=goal.y){phase=Phase.FindExit;return default;}
            return Face(c.direction,Vector2.up);
        case Phase.AfterWallKick:
            if(motor.Grounded && motor.Body.position.y>=goal.y){phase++;return default;}
            return Face(Horizontal(goal-motor.Body.position),Vector2.up);
        default:
            var delta=Horizontal(goal-motor.Body.position);
            return delta.magnitude>.28f?Face(delta,Vector2.up):default;
        }
    }
}
