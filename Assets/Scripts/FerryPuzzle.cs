using System;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.Universal;

public sealed class FerryPuzzle : MonoBehaviour
{
    [SerializeField] WorkshopMotor motor;
    [SerializeField] PortalWindow[] windows;
    [SerializeField] PaperDevice[] devices;
    [SerializeField] PlacementPreview preview;
    WorkshopLayout layout;
    WorkshopLevel current;
    WorkshopGeometry geometry;
    WorkshopPlacement placement;
    WorkshopHud hud;
    public Action<WorkshopInput> CollectInput { private get; set; }
    public Action BeforePhysics { private get; set; }
    public event Action Restarted;
    WorkshopTool selected;
    PlacementAim aim;
    WorkshopGear gear;
    bool paused,checkpoint,finished;
    float previousTimeScale;
    Action finish;
    Action<bool> pauseChanged;

    public void Build(WorkshopHud display, WorkshopLayout stage, Action onFinish, Action<bool> onPause)
    {
        hud = display;
        layout = stage;
        current = stage.Definition;
        finish = onFinish;
        pauseChanged = onPause;
        geometry = new WorkshopGeometry(stage);
        placement = new WorkshopPlacement(transform, windows, devices);
        motor.Bind(placement.Portals);
    }

    public void Begin()
    {
        checkpoint=false;
        finished=false;
        if (GraphicsSettings.currentRenderPipeline is UniversalRenderPipelineAsset pipeline)
            pipeline.mainLightShadowmapResolution = GameSettings.Current.shadowResolution;
        Pause(false);
        placement.Portals.Prepare(motor.Eyes);
        Restart();
    }

    void Restart()
    {
        placement.Clear(transform);
        preview.gameObject.SetActive(false);
        geometry.ResetBricks();
        gear=current.startingGear | (checkpoint?current.supplyGear:WorkshopGear.None);
        selected=WorkshopTool.Gun;
        if(!WorkshopToolSpec.Owned(gear,0))Cycle(1);
        motor.Equip(gear);
        Transform spawn=checkpoint?layout.Checkpoint:layout.Spawn;
        motor.Spawn(spawn.position,spawn.eulerAngles.y,checkpoint?8:-4);
        Physics.SyncTransforms();
        Restarted?.Invoke();
        if(WorkshopToolSpec.Owned(gear,(int)selected))hud.Selected(selected);
        hud.Hint(checkpoint?current.retryHint:current.entryHint,6);
    }

    public void Resume()=>Pause(false);
    public void TogglePause()=>Pause(!paused);

    // Pausing opens the settings sheet; the owner shows and hides it through pauseChanged.
    void Pause(bool value)
    {
        bool changed=value!=paused;
        if(value && !paused){previousTimeScale=Time.timeScale;Time.timeScale=0;}
        if(!value && paused)Time.timeScale=previousTimeScale;
        paused=value;AudioListener.pause=value;motor.StopInput();
        Cursor.lockState=value?CursorLockMode.None:CursorLockMode.Locked;Cursor.visible=value;
        if(value)preview.gameObject.SetActive(false);
        hud.Hint("",0);
        if(!changed)return;
        // The portal depth may have changed while the sheet was open.
        if(!value)placement.Portals.Prepare(motor.Eyes);
        pauseChanged?.Invoke(value);
    }

    void OnDisable()
    {
        AudioListener.pause=false;
        if(paused){Time.timeScale=previousTimeScale;paused=false;}
        Cursor.lockState=CursorLockMode.None;Cursor.visible=true;
    }

    void Select(int i)
    {
        if(i<0 || i>2 || !WorkshopToolSpec.Owned(gear,i) || i==(int)selected)return;
        selected=(WorkshopTool)i;hud.Selected(selected);
    }

    void Cycle(int direction)
    {
        for(int n=1;n<=3;n++){
            int next=((int)selected+direction*n+3)%3;
            if(WorkshopToolSpec.Owned(gear,next)){Select(next);return;}
        }
    }

    void Update()
    {
        if(paused || !WorkshopInput.Read(out var input))return;
        if(CollectInput!=null)CollectInput(input);
        else ApplyInput(input);
    }

    public void ApplyInput(WorkshopInput input)
    {
        if(input.restart)checkpoint=false;
        if(input.restart || input.retry){Restart();return;}
        motor.InputFrame(input,(gear & WorkshopGear.Glider)!=0);
        for(int i=0;i<3;i++)if((input.selectMask & (1<<i))!=0)Select(i);
        if(input.cycle!=0)Cycle(input.cycle);
        bool primary=input.primary,secondary=input.secondary;
        int slot=WorkshopToolSpec.For(selected).slot;
        if(selected==WorkshopTool.Gun && secondary)slot=1;
        bool owned=WorkshopToolSpec.Owned(gear,(int)selected);
        aim=owned?placement.Aim(motor.Eyes,slot):default;
        bool fire=primary || (selected==WorkshopTool.Gun && secondary);
        if(!fire && selected==WorkshopTool.Gun && !aim.Valid && aim.mount.face){
            var other=placement.Check(aim.mount.face,aim.mount.Position,1);
            if(other.Valid)aim=other;
        }
        if(fire && owned){
            if(aim.Valid)placement.Install(slot,aim);
            else hud.Hint(aim.reason,1.3f);
        }
        preview.Show(motor.Eyes,selected,aim);
        hud.Paint(selected,gear,aim.Valid,placement.Portals.Open(0),placement.Portals.Open(1),motor.Gliding);
    }

    void FixedUpdate()
    {
        BeforePhysics?.Invoke();
        if(!isActiveAndEnabled || finished)return;
        bool wasGrounded = motor.Grounded;
        motor.Step();
        placement.ApplyDevices(motor.Body,geometry.Bodies);
        var at=motor.Body.position;
        if(at.y<layout.Spawn.position.y-14){Restart();return;}
        if(!checkpoint && layout.Supply && motor.Grounded && layout.Supply.Contains(at)){
            checkpoint=true;gear|=current.supplyGear;
            motor.Equip(gear);
            if(!WorkshopToolSpec.Owned(gear,(int)selected))Cycle(1);
            hud.Hint(current.supplyHint,7);
        }
        if(layout.Exit && wasGrounded && motor.Grounded && layout.Exit.Contains(at))
        {
            finished=true;
            finish?.Invoke();
        }
    }

    void LateUpdate()
    {
        motor.Pose();
        placement.Portals.Draw(motor.Eyes);
    }

}
