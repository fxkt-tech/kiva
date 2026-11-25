using System;
using UnityEngine;

// Produces ordinary look, move and click inputs. Turning and waiting consume real physics time.
internal sealed class WorkshopPlayerView
{
    internal const int TapeVersion=2;
    readonly WorkshopMotor motor;
    readonly PlacementPreview preview;
    readonly float speed;
    readonly WorkshopReplayBench.Tape program;
    readonly bool recording;
    int cursor;
    bool followingWorld;
    WorkshopInput held;
    Vector2 velocity;
    WorkshopInput click;
    WorkshopFace face;
    Vector3 target,localPoint;
    float steady,waiting;
    public bool Aiming=>click.primary || click.secondary;
    public int Clicks {get;private set;}
    public float PeakTurnSpeed {get;private set;}

    internal WorkshopPlayerView(WorkshopMotor motor,PlacementPreview preview,WorkshopReplayBench.Tape program,bool recording){
        this.motor=motor;this.preview=preview;this.program=program;this.recording=recording;
        speed=program.viewProfile==1?225f:175f;
    }
    internal bool AllClicksPlayed=>recording || Clicks==program.aimedClicks.Count;
    internal int Profile=>program.viewProfile;

    internal WorkshopInput ReplayInput(int tick,WorkshopInput observed){
        if(followingWorld)return observed;
        // Between recorded changes only movement and held space persist.
        var saved=new WorkshopInput{movement=held.movement,spaceHeld=held.spaceHeld};
        if(cursor<program.commands.Count && program.commands[cursor].tick==tick)
            saved=program.commands[cursor++].input;
        held=saved;
        if(saved.selectMask==observed.selectMask && saved.primary==observed.primary && saved.secondary==observed.secondary &&
            saved.spacePressed==observed.spacePressed && saved.spaceHeld==observed.spaceHeld &&
            (saved.turn-observed.turn).sqrMagnitude<.0004f && (saved.movement-observed.movement).sqrMagnitude<.000001f)return saved;
        // Contact with a brick or a portal rim can shift the following takeoff or aim.
        // Finish the recorded route from the actual physics state, checking every installation.
        followingWorld=true;
        return observed;
    }

    Vector3 Eye=>motor.Body.position+Vector3.up*(.90f*WorkshopMotor.BirdScale-WorkshopMotor.HalfHeight);
    Vector2 ErrorTo(Vector3 point){
        Vector3 d=point-Eye;
        float yaw=Mathf.Atan2(d.x,d.z)*Mathf.Rad2Deg;
        float pitch=-Mathf.Atan2(d.y,new Vector2(d.x,d.z).magnitude)*Mathf.Rad2Deg;
        return new Vector2(Mathf.DeltaAngle(motor.Yaw,yaw),Mathf.DeltaAngle(motor.Eyes.transform.eulerAngles.x,pitch));
    }
    Vector2 Turn(Vector2 error){
        float dt=Time.fixedDeltaTime;
        if(motor.CrossedThisStep)velocity=Vector2.zero;
        Vector2 delta=Vector2.SmoothDamp(Vector2.zero,error,ref velocity,.105f,speed,dt);
        if(error.sqrMagnitude<.0004f){delta=error;velocity=Vector2.zero;}
        delta=Vector2.ClampMagnitude(delta,speed*dt);
        PeakTurnSpeed=Mathf.Max(PeakTurnSpeed,delta.magnitude/dt);
        return delta;
    }

    internal WorkshopInput Move(WorkshopInput intent,Vector3? lookAt){
        var desired=intent.turn;
        if(lookAt.HasValue)desired=ErrorTo(lookAt.Value);
        var turn=Turn(desired);
        // A player can strafe while watching the brick. Keep the requested direction of travel
        // in world space as the real head turns; jumps and gliding still use the real motor.
        var local=Quaternion.Euler(0,intent.turn.x-turn.x,0)*new Vector3(intent.movement.x,0,intent.movement.y);
        intent.movement=new Vector2(local.x,local.z);
        intent.turn=turn;
        return intent;
    }

    internal void StartAim(WorkshopInput intent,Vector3 point){
        var step=new WorkshopReplayBench.AimedClick{point=point,tool=intent.selectMask,secondary=intent.secondary};
        if(recording)program.aimedClicks.Add(step);
        else{
            if(Clicks>=program.aimedClicks.Count)throw new InvalidOperationException("回放出现了多余的安装动作。");
            var expected=program.aimedClicks[Clicks];
            if(expected.tool!=step.tool || expected.secondary!=step.secondary || Vector3.Distance(expected.point,point)>.2f)
                throw new InvalidOperationException("安装目标或道具与录制路线不一致。");
        }
        click=intent;target=point;face=null;steady=waiting=0;
        var ray=point-Eye;
        if(Physics.Raycast(Eye,ray,out var hit,35f,Physics.DefaultRaycastLayers,QueryTriggerInteraction.Collide)
            && Vector3.Distance(hit.point,point)<.12f){
            face=hit.collider.GetComponentInParent<WorkshopFace>();
            if(face)localPoint=face.transform.InverseTransformPoint(point);
        }
    }

    internal WorkshopInput Aim(){
        if(face)target=face.transform.TransformPoint(localPoint);
        var error=ErrorTo(target);
        var input=new WorkshopInput{selectMask=click.selectMask,turn=Turn(error)};
        // The placement preview is populated by the game's own aim and footprint checks.
        // Require several consecutive ticks on that same patch before allowing either button.
        bool onTarget=preview.gameObject.activeSelf && error.magnitude<.6f &&
            Vector3.Distance(preview.transform.position-preview.transform.forward*.026f,target)<.09f;
        steady=onTarget?steady+Time.fixedDeltaTime:0;
        waiting+=Time.fixedDeltaTime;
        if(steady>=.10f){
            input.primary=click.primary;input.secondary=click.secondary;click=default;Clicks++;
        }else if(waiting>3f)throw new InvalidOperationException($"准星未能在预定安装面上停稳（{target:F2}）。");
        return input;
    }
}
