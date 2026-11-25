using UnityEngine;

// Sample the verified player's actions. Rendering frames, aim jitter and portal rotations
// are not independent gaze decisions.
internal sealed class WorkshopViewAssessment
{
    internal readonly WorkshopGenerationReport.ViewAssessment result=new WorkshopGenerationReport.ViewAssessment();
    readonly WorkshopMotor motor;
    Vector3 previousPosition,motion,straightHeading,previousRoute,previousLook;
    float straight;
    string lastAction;
    int lastSight=-1;

    internal WorkshopViewAssessment(WorkshopMotor motor){
        this.motor=motor;previousPosition=motor.Body.position;
    }
    internal void Travel(){
        var p=motor.Body.position;
        if(motor.CrossedThisStep){
            previousLook=straightHeading=motion=Vector3.zero;straight=0;
        }else{
            var delta=p-previousPosition;delta.y=0;motion+=delta;
            // Sample a body-width of travel so idle contact jitter cannot split a straight.
            if(motion.sqrMagnitude>=.35f*.35f)MeasureMotion();
        }
        previousPosition=p;
    }
    void MeasureMotion(){
        if(motion.sqrMagnitude<.0001f)return;
        if(straightHeading==Vector3.zero || Vector3.Angle(straightHeading,motion)>=45){
            straightHeading=motion.normalized;straight=0;
        }
        straight+=motion.magnitude;motion=Vector3.zero;
        result.longestStraight=Mathf.Max(result.longestStraight,straight);
    }
    internal void Arrive(int i,WorkshopCourse.Leg leg){
        var direction=motor.Body.position-leg.approach;direction.y=0;
        if(previousRoute!=Vector3.zero){
            float angle=Vector3.Angle(direction,previousRoute);
            if(angle>=45){
                result.courseTurns++;
                result.details.Add($"第 {i} → {i+1} 段行进方向改变 {angle:F0}°");
            }
        }
        previousRoute=direction;
        if(leg.action==WorkshopAction.Walk)Look($"第 {i+1} 段抵达走台");
    }
    internal void Look(string action){
        var direction=motor.Eyes.transform.forward;
        float angle=Vector3.Angle(direction,previousLook);
        if(previousLook!=Vector3.zero && angle>=45){
            result.lookTurns++;
            result.details.Add($"{lastAction} → {action}，视线转动 {angle:F0}°");
        }
        previousLook=direction;lastAction=action;
    }
    internal void CheckLanding(int index,Bounds landing){
        if(lastSight==index)return;
        lastSight=index;result.checkedTargets++;
        var eye=motor.Eyes.transform.position;
        // The near lip is visible from below even when the middle of a raised platform is hidden.
        var rim=landing.ClosestPoint(eye);rim.y=landing.center.y+.25f;
        var ray=rim-eye;
        if(!Physics.Raycast(eye,ray.normalized,Mathf.Max(0,ray.magnitude-.08f),Physics.DefaultRaycastLayers,QueryTriggerInteraction.Ignore))
            result.visible++;
        else result.details.Add($"第 {index+1} 段起跳前，近侧落台边缘被遮挡");
    }
    internal void Finish(){
        MeasureMotion();
        result.verified=true;
    }
}
