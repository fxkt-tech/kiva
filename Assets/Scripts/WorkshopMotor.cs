using UnityEngine;
using UnityEngine.InputSystem;

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
    bool jump,glideHeld,steerJump;
    Vector2 movement;
    Vector3 exitStart;
    double exitTick=double.NegativeInfinity;
    VermilionBird bird;
    PhysicsMaterial friction;
    WorkshopPortals portals;

    public void Build(WorkshopPortals pair)
    {
        portals=pair;
        gameObject.layer=2;
        Body=gameObject.AddComponent<Rigidbody>();
        Body.constraints=RigidbodyConstraints.FreezeRotation;
        Body.interpolation=RigidbodyInterpolation.Interpolate;
        Body.collisionDetectionMode=CollisionDetectionMode.ContinuousDynamic;
        var cap=gameObject.AddComponent<CapsuleCollider>();cap.height=HalfHeight*2;cap.radius=.52f*BirdScale;
        friction=new PhysicsMaterial("纸羽侧滑"){dynamicFriction=0,staticFriction=0,frictionCombine=PhysicsMaterialCombine.Minimum};
        cap.sharedMaterial=friction;
        bird=new GameObject("朱雀剪纸").AddComponent<VermilionBird>();
        bird.transform.SetParent(transform,false);
        bird.transform.localPosition=Vector3.down*HalfHeight;
        bird.transform.localScale=Vector3.one*BirdScale;
        bird.Build();
        Eyes=new GameObject("朱雀第一人称").AddComponent<Camera>();
        Eyes.transform.SetParent(transform,false);
        Eyes.cullingMask &= ~(1<<VermilionBird.ModelLayer);
        Eyes.fieldOfView=70;Eyes.nearClipPlane=.04f;Eyes.depth=5;
        Eyes.clearFlags=CameraClearFlags.SolidColor;Eyes.backgroundColor=new Color(.77f,.74f,.61f);
    }

    Transform glider;
    GameObject gun;
    public void BuildEquipment(Material paper,Material bamboo,Material ochre)
    {
        var gear=new PaperMesh.Builder(2);
        var back=new Vector3(.46f,.62f,.10f);
        var front=back+new Vector3(0,.06f,.55f);
        var side=Vector3.right*.07f;
        gear.Quad(0,Vector3.up,back-side,back+side,front+side,front-side);
        gear.Quad(1,Vector3.right,back+side,back+side-Vector3.up*.13f,front+side-Vector3.up*.07f,front+side);
        gun=gear.Mount(bird.transform,"随身时空枪",bamboo,ochre);
        gun.layer=VermilionBird.ModelLayer;
        var wing=new PaperMesh.Builder(2);
        for(int s=-1;s<=1;s+=2){
            var nose=new Vector3(0,.12f,.2f);var tail=new Vector3(0,0,-.62f);
            var fold=new Vector3(s*.54f,.07f,-.38f);var tip=new Vector3(s*1.52f,.17f,-.64f);
            wing.Triangle(0,Vector3.up,nose,fold,tip);wing.Triangle(1,Vector3.up,nose,tail,fold);
            wing.Triangle(1,Vector3.down,nose,fold,tip);wing.Triangle(0,Vector3.down,nose,tail,fold);
        }
        glider=wing.Mount(bird.transform,"折纸滑翔翼",paper,ochre).transform;
        glider.gameObject.layer=VermilionBird.ModelLayer;
        glider.localPosition=new Vector3(0,1.02f,-.18f);
        Equip(WorkshopGear.Gun);
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

    public void InputFrame(bool hasWing)
    {
        var keys=Keyboard.current;var mouse=Mouse.current;
        if(keys==null || mouse==null){StopInput();Look();return;}
        // .22 degrees per pixel matches the former Input Manager mouse axes.
        var options=GameSettings.Current;
        var turn=mouse.delta.ReadValue()*(.22f*options.sensitivity);
        Yaw+=turn.x;
        pitch=Mathf.Clamp(pitch+(options.invertY?turn.y:-turn.y),-89,89);
        Eyes.fieldOfView=options.fieldOfView;
        roll=Mathf.MoveTowards(roll,0,240*Time.deltaTime);
        static float Axis(bool negative,bool positive)=>(positive?1:0)-(negative?1:0);
        movement=Vector2.ClampMagnitude(new Vector2(
            Axis(keys.aKey.isPressed || keys.leftArrowKey.isPressed,keys.dKey.isPressed || keys.rightArrowKey.isPressed),
            Axis(keys.sKey.isPressed || keys.downArrowKey.isPressed,keys.wKey.isPressed || keys.upArrowKey.isPressed)),1);
        if(keys.spaceKey.wasPressedThisFrame){
            if(Grounded)jump=true;
            else if(hasWing)glideHeld=!options.glideToggle || !glideHeld;
        }
        if(!options.glideToggle && !keys.spaceKey.isPressed)glideHeld=false;
        Look();
    }

    public void Step()
    {
        var velocity=Body.linearVelocity;
        Grounded=velocity.y<=.15f && Physics.Raycast(Body.position,Vector3.down,HalfHeight+.085f,~0,QueryTriggerInteraction.Ignore);
        var walk=Quaternion.Euler(0,Yaw,0)*new Vector3(movement.x,0,movement.y);
        var horizontal=new Vector3(velocity.x,0,velocity.z);
        if(Grounded){
            glideHeld=steerJump=false;
            horizontal=Vector3.MoveTowards(horizontal,walk*4.8f,27*Time.fixedDeltaTime);
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
        if(!portals.Crossing(Body.position+Vector3.up*EyeOffset,velocity,out var point,out var turn,out var after,true))return;
        exitStart=point-Vector3.up*EyeOffset;
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

    void Look()
    {
        var center=transform.position;
        if(exitTick==Time.fixedTimeAsDouble)
            center=Vector3.Lerp(exitStart,Body.position,(float)((Time.timeAsDouble-exitTick)/Time.fixedDeltaTime));
        var eye=center+Vector3.up*EyeOffset;
        var look=Quaternion.Euler(pitch,Yaw,roll);
        if(portals.Crossing(eye,Body.linearVelocity,out var mapped,out var turn,out _)){eye=mapped;look=turn*look;}
        Eyes.transform.SetPositionAndRotation(eye,look);
    }

    public void Pose(){
        Look();bird.Pose(Body.linearVelocity,Yaw,Grounded,Time.deltaTime);
        if(glider.gameObject.activeSelf)glider.localScale=new Vector3(Gliding?1f:.24f,1,1);
    }
    void OnDestroy(){if(friction)Destroy(friction);}
}
