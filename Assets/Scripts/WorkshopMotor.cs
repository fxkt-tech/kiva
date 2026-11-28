using UnityEngine;
using UnityEngine.Rendering.Universal;

public sealed class WorkshopMotor : MonoBehaviour
{
    public const float BirdScale=.68f, HalfHeight=.60f*BirdScale;
    const float EyeOffset=.90f*BirdScale-HalfHeight;
    public Camera Eyes { get; private set; }
    public Rigidbody Body { get; private set; }
    public bool Grounded { get; private set; }
    public bool Gliding { get; private set; }
    public float Yaw { get; private set; }
    float pitch,roll;
    float footRadius;
    bool jump,glideHeld,steerJump;
    Vector2 movement;
    Vector3 exitStart;
    public bool CrossedThisStep { get; private set; }
    public Vector3 EntryCenter { get; private set; }
    public Vector3 ExitCenter { get; private set; }
    double exitTick=double.NegativeInfinity;
    [SerializeField] VermilionBird bird;
    [SerializeField] Transform glider;
    [SerializeField] GameObject gun;
    WorkshopPortals portals;

    public void Bind(WorkshopPortals pair)
    {
        portals = pair;
        Body = GetComponent<Rigidbody>();
        footRadius = GetComponent<CapsuleCollider>().radius;
        Eyes = GetComponentInChildren<Camera>(true);
        Eyes.GetUniversalAdditionalCameraData().requiresColorTexture = true;
    }

    public void Equip(WorkshopGear inventory){
        glider.gameObject.SetActive((inventory & WorkshopGear.Glider)!=0);
        gun.SetActive((inventory & WorkshopGear.Gun)!=0);
    }

    public void Spawn(Vector3 feet,float heading,float tilt)
    {
        Body.position=feet+Vector3.up*(HalfHeight+.035f);
        transform.position=Body.position;
        Body.linearVelocity=Body.angularVelocity=Vector3.zero;
        Yaw=heading;pitch=tilt;roll=0;
        exitTick=double.NegativeInfinity;
        Grounded=steerJump=false;
        StopInput();
        bird.Face(Yaw);
        Look();
    }

    public void StopInput(){movement=Vector2.zero;jump=glideHeld=Gliding=false;}

    public void InputFrame(WorkshopInput input,bool hasWing)
    {
        Yaw+=input.turn.x;
        pitch=Mathf.Clamp(pitch+input.turn.y,-89,89);
        Eyes.fieldOfView=GameSettings.Current.fieldOfView;
        roll=Mathf.MoveTowards(roll,0,240*Time.deltaTime);
        movement=input.movement;
        if(input.spacePressed){
            if(Grounded)jump=true;
            else if(hasWing)glideHeld=!input.toggleGlide || !glideHeld;
        }
        if(!input.toggleGlide && !input.spaceHeld)glideHeld=false;
        Look(Time.inFixedTimeStep);
    }

    public void Step()
    {
        var velocity=Body.linearVelocity;
        const float groundMargin=.085f, minGroundNormalY=.62f;
        float reach=HalfHeight+groundMargin+footRadius*(1f/minGroundNormalY-1f);
        Grounded=Physics.Raycast(Body.position,Vector3.down,out var ground,reach,~0,QueryTriggerInteraction.Ignore)
            && ground.normal.y>=minGroundNormalY && Vector3.Dot(velocity,ground.normal)<=.15f
            && ground.distance<=HalfHeight+groundMargin+footRadius*(1f/ground.normal.y-1f);
        var walk=Quaternion.Euler(0,Yaw,0)*new Vector3(movement.x,0,movement.y);
        var horizontal=new Vector3(velocity.x,0,velocity.z);
        if(Grounded){
            glideHeld=steerJump=false;
            // A capsule rides higher above a slope. Drive along its support plane
            // so climbing a ramp does not count as launching into the air.
            var along=Vector3.ProjectOnPlane(velocity,ground.normal);
            var target=Vector3.ProjectOnPlane(walk,ground.normal).normalized*(walk.magnitude*4.8f);
            velocity+=Vector3.MoveTowards(along,target,27*Time.fixedDeltaTime)-along;
            horizontal=new Vector3(velocity.x,0,velocity.z);
            if(jump){velocity.y=Mathf.Sqrt(2*.9f*Mathf.Abs(Physics.gravity.y));steerJump=true;}
        }else if((steerJump || glideHeld) && movement.sqrMagnitude>.01f){
            // Retain launch momentum; bank towards the chosen direction while gliding.
            float speed=glideHeld?Mathf.Max(6.5f,horizontal.magnitude):4.8f;
            horizontal=Vector3.MoveTowards(horizontal,walk*speed,(glideHeld?11:12)*Time.fixedDeltaTime);
        }
        Gliding=glideHeld && !Grounded;
        if(Gliding && velocity.y< -1.5f)velocity.y=Mathf.MoveTowards(velocity.y,-1.5f,23*Time.fixedDeltaTime);
        velocity.x=horizontal.x;velocity.z=horizontal.z;
        jump=false;
        Body.linearVelocity=velocity;
        CrossedThisStep=portals.Crossing(Body.position+Vector3.up*EyeOffset,velocity,out var point,out var turn,out var after,out float overshoot,true);
        if(!CrossedThisStep)return;
        EntryCenter=Body.position-velocity*overshoot;
        exitStart=point-Vector3.up*EyeOffset;
        ExitCenter=exitStart-after*overshoot;
        exitTick=Time.fixedTimeAsDouble;
        Body.position=exitStart;Body.linearVelocity=after;
        steerJump=false;
        var rotated=turn*Quaternion.Euler(pitch,Yaw,roll);
        var f=rotated*Vector3.forward;
        pitch=Mathf.Clamp(-Mathf.Asin(Mathf.Clamp(f.y,-1,1))*Mathf.Rad2Deg,-89,89);
        var flat=Vector3.ProjectOnPlane(f,Vector3.up);
        if(flat.sqrMagnitude<.000001f)flat=Vector3.Cross(rotated*Vector3.right,Vector3.up);
        Yaw=Mathf.Atan2(flat.x,flat.z)*Mathf.Rad2Deg;
        roll=Vector3.SignedAngle(Quaternion.Euler(pitch,Yaw,0)*Vector3.up,rotated*Vector3.up,f);
        Look();bird.Face(Yaw);
    }

    void Look(bool physics=false)
    {
        var center=physics?Body.position:transform.position;
        if(!physics && exitTick==Time.fixedTimeAsDouble)
            center=Vector3.Lerp(exitStart,Body.position,(float)((Time.timeAsDouble-exitTick)/Time.fixedDeltaTime));
        var eye=center+Vector3.up*EyeOffset;
        var look=Quaternion.Euler(pitch,Yaw,roll);
        if(portals.Crossing(eye,Body.linearVelocity,out var mapped,out var turn,out _,physics)){eye=mapped;look=turn*look;}
        Eyes.transform.SetPositionAndRotation(eye,look);
    }

    public void Pose(){
        Look();bird.Pose(Body.linearVelocity,Yaw,Grounded,Time.deltaTime);
        if(glider.gameObject.activeSelf)glider.localScale=new Vector3(Gliding?1f:.24f,1,1);
    }
}
