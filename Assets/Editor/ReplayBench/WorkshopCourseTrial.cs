using System;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;

internal sealed class WorkshopCourseTrial
{
    readonly WorkshopMotor motor;
    readonly Rigidbody[] bricks;
    readonly IReadOnlyList<WorkshopCourse.Leg> legs;
    readonly WorkshopGenerationReport.Measurement result;
    readonly Vector3 bird;
    readonly int omit;
    readonly bool playerView;
    readonly WorkshopShellBypass shell;
    readonly int timingOffset;
    readonly float aimError;
    readonly bool perturbJump;
    int crossedTick,flightTicks;
    Vector3 flightStart;
    internal bool ShellUnavailable=>shell!=null && shell.Unavailable;
    internal string ShellName=>shell?.Name;
    bool atPortalEntry,passedJunction;
    int index,ready,installed,fired,brake;
    bool jumpRequested,jumped,bounced,glided,wind,ported,pushed,stoodOnBrick;
    Rigidbody brick;
    public bool Matched {get;private set;}=true;
    public bool Complete=>index==legs.Count;
    public int CompletionTick {get;private set;}
    WorkshopCourse.Leg Leg=>legs[index];
    internal int Index=>index;
    internal Vector3 MountPoint {get;private set;}
    internal Vector3? LookAtPoint{
        get{
            if(Complete || Leg.action!=WorkshopAction.Push || !pushed)return null;
            if(installed==0)return brick.position+Vector3.up*.75f;
            return Leg.brickSpring && Vector3.Dot(Leg.destination.center-motor.Body.position,Leg.Forward)>1?Leg.destination.center:(Vector3?)null;
        }
    }
    internal string Status=>Complete?"已走完各段":$"第 {index+1} 段 {WorkshopGenerationRequest.Names[(int)Leg.action]}，角色 {motor.Body.position:F1}，起步点 {Leg.approach:F1}，目标 {Leg.destination.center:F1}，"+
        $"就位步 {ready}，装菇步 {installed}，推砖 {pushed}，弹射 {bounced}，滑翔 {glided}，风力 {wind}，穿门 {ported}"+(brick?$"，砖心 {brick.position:F1}":"");

    internal WorkshopCourseTrial(FerryPuzzle puzzle,WorkshopLevel level,List<WorkshopCourse.Leg> legs,
        WorkshopGenerationReport.Measurement result,int omit,bool playerView=false,WorkshopShellBypass shell=null,int timingOffset=0,float aimError=0){
        this.shell=shell;
        this.timingOffset=timingOffset;this.aimError=aimError;
        perturbJump=(timingOffset!=0 || aimError!=0) && !legs.Any(l=>l.glide);
        this.legs=legs;this.result=result;this.omit=omit;this.playerView=playerView;bird=level.perches[0].feet;
        motor=puzzle.GetComponentInChildren<WorkshopMotor>();
        bricks=puzzle.GetComponentsInChildren<WorkshopBrick>().Select(b=>b.GetComponent<Rigidbody>()).ToArray();
        foreach(var d in puzzle.GetComponentsInChildren<PaperDevice>(true))
            d.Applied+=(body,host)=>{
                if(Complete)return;
                if(d.Kind==WorkshopTool.Fan && body==brick && brick)pushed=true;
                if(body!=motor.Body)return;
                if(d.Kind==WorkshopTool.Fan && Leg.action==WorkshopAction.Wind && Vector3.Distance(d.transform.position,Leg.mount)<.3f)wind=true;
                if(d.Kind==WorkshopTool.Mushroom && d.transform.forward.y>.95f){
                    if(Leg.portalSpring)bounced|=ported && !host && Vector3.Distance(d.transform.position,Leg.springMount)<.3f;
                    else if(Leg.brickSpring)bounced|=host && host==brick && pushed && Mathf.Abs(Vector3.Dot(host.position,Leg.Side)-Leg.stop)<.14f;
                    else bounced|=(Leg.action==WorkshopAction.LooseSpring?host && host==brick:!host) && Vector3.Distance(d.transform.position,Leg.mount)<.3f;
                }
            };
        SelectBrick();
    }
    void SelectBrick(){
        brick=null;
        if(Complete || (Leg.action!=WorkshopAction.Push && Leg.action!=WorkshopAction.LooseSpring))return;
        var start=Leg.action==WorkshopAction.Push?Leg.origin+Leg.Forward*.75f+Vector3.up*.75f+Leg.Side*2.25f:Leg.mount-Vector3.up*.75f;
        brick=bricks.OrderBy(b=>Vector3.Distance(b.position,start)).First();
    }
    internal void Observe(int tick){
        if(Complete)return;
        var p=motor.Body.position;
        jumped|=jumpRequested && !motor.Grounded && motor.Body.linearVelocity.y>1;
        glided|=motor.Gliding && (Leg.action==WorkshopAction.Jump?jumped:bounced);
        ported|=motor.CrossedThisStep;
        if(motor.CrossedThisStep)crossedTick=tick;
        if((bounced || fired>0) && !motor.Grounded && Leg.glide){
            if(flightTicks++==0){
                flightStart=p;
                if(Leg.portalSpring)result.portalSpringSeconds=(tick-crossedTick)*Time.fixedDeltaTime;
            }
            var v=motor.Body.linearVelocity;v.y=0;
            if(motor.Gliding && v.sqrMagnitude>1)
                result.airTurn=Mathf.Max(result.airTurn,Vector3.Angle(Leg.portalSpring?Leg.exitDirection:Leg.Forward,v));
        }
        if(brick && pushed && motor.Grounded && Mathf.Abs(Vector3.Dot(brick.position,Leg.Side)-Leg.stop)<.14f &&
            Mathf.Abs(p.x-brick.position.x)<.73f && Mathf.Abs(p.z-brick.position.z)<.73f &&
            Mathf.Abs(p.y-WorkshopMotor.HalfHeight-brick.position.y-.75f)<.15f)stoodOnBrick=true;
        var dest=Leg.destination;
        float margin=Mathf.Min(p.x-dest.min.x,dest.max.x-p.x,p.z-dest.min.z,dest.max.z-p.z)-motor.GetComponent<CapsuleCollider>().radius;
        if(!motor.Grounded || Mathf.Abs(p.y-WorkshopMotor.HalfHeight-dest.center.y)>.17f || margin<.2f)return;
        bool matched=Leg.action switch {WorkshopAction.Jump=>jumped,WorkshopAction.Spring or WorkshopAction.LooseSpring=>bounced,WorkshopAction.Portal=>ported,
            WorkshopAction.Wind=>wind,WorkshopAction.Push=>pushed && (Leg.brickSpring?bounced:stoodOnBrick),_=>true};
        matched&=(!Leg.glide || glided) && (!Leg.portalSpring || (bounced && result.portalSpringSeconds>0 && result.portalSpringSeconds<1.2f));
        Matched&=matched;
        result.trace.Add($"第 {tick} 步 · {WorkshopGenerationRequest.Names[(int)Leg.action]}{(Leg.brickSpring?" → 砖顶弹射":Leg.portalSpring?" → 出门弹射":"")}{(Leg.glide?" → 滑翔":"")} · {(matched?"动作已验证":"绕过动作")} · 抵达平台净余量 {margin:F2} m");
        result.edgeMargin=index==0?margin:Mathf.Min(result.edgeMargin,margin);
        result.landed=true;result.landing=p;
        index++;ready=installed=fired=brake=crossedTick=flightTicks=0;
        jumpRequested=jumped=bounced=glided=wind=ported=pushed=stoodOnBrick=atPortalEntry=passedJunction=false;
        if(Complete)CompletionTick=tick;
        SelectBrick();
    }
    internal void ObserveInput(WorkshopInput input){
        if(input.spacePressed && motor.Grounded)jumpRequested=true;
    }
    WorkshopInput Face(Vector3 direction,bool move){
        if(direction.sqrMagnitude<.00001f)return default;
        var angles=Quaternion.LookRotation(direction).eulerAngles;
        return new WorkshopInput{movement=move?Vector2.up:Vector2.zero,turn=new Vector2(Mathf.DeltaAngle(motor.Yaw,angles.y),
            Mathf.DeltaAngle(motor.Eyes.transform.eulerAngles.x,angles.x))};
    }
    WorkshopInput Walk(Vector3 target){
        var delta=target-motor.Body.position;delta.y=0;
        return delta.sqrMagnitude>.08f*.08f?Face(delta,true):default;
    }
    WorkshopInput Place(Vector3 point,WorkshopTool tool,bool secondary=false){
        MountPoint=point;
        var eye=motor.Body.position+Vector3.up*(.90f*WorkshopMotor.BirdScale-WorkshopMotor.HalfHeight);
        var input=Face(point-eye,false);input.selectMask=1<<(int)tool;input.primary=!secondary;input.secondary=secondary;return input;
    }
    internal WorkshopInput Command(int tick){
        if(Complete)return Walk(bird);
        var l=Leg;var p=motor.Body.position;
        if(l.hasJunction && !passedJunction){
            var to=l.junction-p;to.y=0;
            if(to.sqrMagnitude>.5f*.5f)return Walk(l.junction);
            passedJunction=true;
        }
        if(ready==0){
            var delta=l.approach-p;delta.y=0;
            if(delta.magnitude>.18f){
                var vel=motor.Body.linearVelocity;vel.y=0;
                if(Vector3.Dot(vel,delta)>0 && delta.magnitude<vel.sqrMagnitude/52f+.10f)return default;
                return Walk(l.approach);
            }
            if(motor.Body.linearVelocity.sqrMagnitude>1f)return default;
            ready=tick;
        }
        int elapsed=tick-ready;
        if(elapsed<4)return default;
        bool bypass=index==omit;
        if(bypass && shell!=null)return shell.Command(elapsed,l,motor,ported,Place,Walk);
        if(l.action==WorkshopAction.Push && !bypass){
            if(elapsed==4)return Place(l.mount,WorkshopTool.Fan);
            if(!pushed || elapsed<18)return default;
            if(brake==0){
                if(Vector3.Dot(p,l.Side)<l.stop-.38f)return Face(l.Side,true);
                brake=tick;
            }
            if(installed==0){
                if(tick-brake<14 || Mathf.Abs(Vector3.Dot(brick.position,l.Side)-l.stop)>.1f)return default;
                installed=tick;
                return l.brickSpring?Place(brick.position+Vector3.up*.75f,WorkshopTool.Mushroom):Place(l.exitMount,WorkshopTool.Fan);
            }
            if(tick-installed<6)return default;
            if(!l.brickSpring)return Walk(stoodOnBrick?l.destination.center:brick.position);
            var run=Face(Quaternion.Euler(0,-4f,0)*l.Forward,true);
            if(l.glide && bounced && !motor.Grounded)return Fly(l);
            return run;
        }
        if(!bypass){
            if(elapsed==4 && (l.action==WorkshopAction.Spring || l.action==WorkshopAction.LooseSpring || l.action==WorkshopAction.Wind))
                return Place(l.mount,l.action==WorkshopAction.Wind?WorkshopTool.Fan:WorkshopTool.Mushroom);
            if(l.action==WorkshopAction.Portal){
                if(l.portalSpring && elapsed==4)return Place(l.springMount,WorkshopTool.Mushroom);
                int doorTick=l.portalSpring?5:4;
                if(elapsed==doorTick)return Place(l.exitMount,WorkshopTool.Gun,true);
                if(elapsed==doorTick+1)return Place(l.mount,WorkshopTool.Gun);
                if(ported && l.portalSpring)return bounced && !motor.Grounded?Fly(l):Face(l.exitDirection,true);
                if(ported)return Walk(l.destination.center-l.Forward*2);
                if((playerView || l.portalSpring) && !atPortalEntry){
                    var entry=l.mount-l.Forward*2.1f;
                    var delta=entry-p;delta.y=0;
                    var speed=motor.Body.linearVelocity;speed.y=0;
                    atPortalEntry=delta.magnitude<.13f && speed.sqrMagnitude<.8f;
                    if(!atPortalEntry){
                        if(delta.magnitude<.13f || (Vector3.Dot(delta,speed)>0 && delta.magnitude<speed.sqrMagnitude/52f+.06f))return default;
                        return Walk(entry);
                    }
                }
                return Walk(l.mount);
            }
        }
        if(l.glide && !bypass && !motor.Grounded && (bounced || fired>0))return Fly(l);
        var forward=Face(perturbJump && l.action==WorkshopAction.Jump?Quaternion.Euler(0,aimError,0)*l.Forward:l.Forward,true);
        bool tryJump=bypass?(l.action!=WorkshopAction.Jump || l.glide):l.action==WorkshopAction.Jump || l.action==WorkshopAction.Wind;
        if(tryJump &&
            l.action!=WorkshopAction.Walk && fired==0 && Vector3.Dot(p,l.Forward)>=l.edge-.22f+(perturbJump?timingOffset*4.8f*Time.fixedDeltaTime:0) && motor.Grounded){
            fired=tick;forward.spacePressed=true;
        }
        return forward;
    }
    WorkshopInput Fly(WorkshopCourse.Leg l){
        var target=l.destination.center;
        if(aimError!=0)target=flightStart+Quaternion.Euler(0,aimError,0)*(target-flightStart);
        var input=Walk(target);
        if(timingOffset>0 && flightTicks<=timingOffset)return input;
        input.spaceHeld=true;input.spacePressed=!motor.Gliding || (l.portalSpring && flightTicks<=10);
        return input;
    }
}
