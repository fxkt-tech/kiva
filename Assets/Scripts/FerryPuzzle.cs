using System;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.UI;

// Session and progression for one campaign stage.
public sealed class FerryPuzzle : MonoBehaviour
{
    WorkshopLevel current;
    Transform hall;
    WorkshopGeometry geometry;
    WorkshopPlacement placement;
    WorkshopMotor motor;
    WorkshopHud hud;
    PlacementPreview preview;
    WorkshopTool selected;
    PlacementAim aim;
    WorkshopGear gear;
    bool paused,checkpoint;
    float previousTimeScale;
    Action finish;
    Action<bool> pauseChanged;
    FerryLighting lighting;
    readonly System.Collections.Generic.List<Material> materials=new System.Collections.Generic.List<Material>();
    Texture2D fibers;

    Transform Group(Transform parent,string label)
    {
        var t=new GameObject(label).transform;
        t.SetParent(parent,false);return t;
    }

    Material Keep(Material m){materials.Add(m);return m;}
    Material Pigment(Color c)=>Keep(FerryLighting.FoldedMineral(c));

    Material[] FaceInks()
    {
        var white=Pigment(new Color(.94f,.87f,.72f));
        var shade=Pigment(new Color(.91f,.84f,.69f));
        var bamboo=Pigment(new Color(.44f,.54f,.39f));
        var alternate=Pigment(new Color(.42f,.52f,.38f));
        // A seamless long-fibre wash distinguishes bamboo even in diffuse window light.
        const int size=96;
        fibers=new Texture2D(size,size,TextureFormat.RGBA32,true,true){name="竹纤维",wrapMode=TextureWrapMode.Repeat,filterMode=FilterMode.Trilinear,anisoLevel=4};
        var pixels=new Color32[size*size];
        for(int y=0;y<size;y++)for(int x=0;x<size;x++){
            float u=x*2*Mathf.PI/size,v=y*2*Mathf.PI/size;
            float grain=.48f+.12f*Mathf.Sin(17*u+.4f*Mathf.Sin(v))+.08f*Mathf.Sin(29*u+v)+.045f*Mathf.Cos(3*u-v);
            pixels[y*size+x]=new Color(grain,grain,grain,1);
        }
        fibers.SetPixels32(pixels);fibers.Apply(true,true);
        bamboo.SetTexture("_MineralWash",fibers);alternate.SetTexture("_MineralWash",fibers);
        return new[]{white,shade,bamboo,alternate};
    }

    public void Build(Text status,WorkshopLevel level,Action onFinish,Action<bool> onPause)
    {
        pauseChanged=onPause;
        current=level;
        finish=onFinish;hud=new WorkshopHud(status);
        var inks=FaceInks();
        hall=Group(transform,current.title);
        geometry=new WorkshopGeometry(current,hall,inks);
        var ochre=Pigment(FerryLighting.OchreBack);
        if(current.westWindow){
            var wood=Keep(FerryLighting.Mineral("Timber",new Color(.79f,.46f,.23f)));
            var glass=Keep(new Material(Resources.Load<Material>("Mineral/WestWindowPaper")));
            PaperChamber.WestWindowFrame(hall,wood,glass,ochre);
        }
        var exitLamp=Group(hall,"出口灯");
        exitLamp.localPosition=current.exitFeet+new Vector3(-.5f,.38f,.9f);
        var plaque=Keep(FerryLighting.Mineral("Paper",new Color(.41f,.43f,.31f)));
        var sign=current.exitSign;
        PaperChamber.Cartouche(hall,sign,180,plaque,ochre);
        var canvas=Group(hall,"出口纸牌").gameObject.AddComponent<Canvas>();
        canvas.renderMode=RenderMode.WorldSpace;
        canvas.transform.localPosition=sign-Vector3.forward*.046f;
        canvas.transform.localScale=Vector3.one*.008f;
        var signRect=(RectTransform)canvas.transform;signRect.sizeDelta=new Vector2(125,100);
        var lettering=new GameObject("出口",typeof(RectTransform),typeof(Text)).GetComponent<Text>();
        lettering.rectTransform.SetParent(signRect,false);
        lettering.rectTransform.sizeDelta=signRect.sizeDelta;
        lettering.font=status.font;lettering.fontSize=47;lettering.color=Color.white;
        lettering.alignment=TextAnchor.MiddleCenter;lettering.raycastTarget=false;lettering.text="出口";
        var fronts=new[]{Pigment(new Color(.13f,.48f,.66f)),Pigment(new Color(.78f,.27f,.16f))};
        var backs=new[]{Pigment(new Color(.60f,.79f,.77f)),ochre};
        placement=new WorkshopPlacement(transform,fronts,backs,inks[0],inks[2]);
        motor=Group(transform,"朱雀").gameObject.AddComponent<WorkshopMotor>();
        motor.Build(placement.Portals);
        motor.BuildEquipment(inks[0],inks[2],ochre);
        preview=Group(transform,"安装预览").gameObject.AddComponent<PlacementPreview>();
        preview.Build();
        lighting=gameObject.AddComponent<FerryLighting>();
        lighting.Initialize(motor.Eyes,exitLamp);
    }

    public void Begin()
    {
        checkpoint=false;
        lighting.ApplyQuality();
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
        var spawn=checkpoint?current.checkpointFeet:current.spawnFeet;
        motor.Spawn(hall.TransformPoint(spawn),checkpoint?current.checkpointYaw:current.spawnYaw,checkpoint?8:-4);
        Physics.SyncTransforms();
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
        paused=value;motor.StopInput();
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
        var keys=Keyboard.current;var mouse=Mouse.current;
        if(keys==null || mouse==null)return;
        if(keys.escapeKey.wasPressedThisFrame)Pause(!paused);
        if(paused)return;
        if(keys.tKey.wasPressedThisFrame){Begin();return;}
        if(keys.rKey.wasPressedThisFrame){Restart();return;}
        motor.InputFrame((gear & WorkshopGear.Glider)!=0);
        for(int i=0;i<3;i++)if(keys[Key.Digit1+i].wasPressedThisFrame)Select(i);
        float scroll=mouse.scroll.ReadValue().y;
        if(scroll!=0)Cycle(scroll>0?-1:1);
        bool primary=mouse.leftButton.wasPressedThisFrame,secondary=mouse.rightButton.wasPressedThisFrame;
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
            if(aim.Valid){
                placement.Install(slot,aim);
                if(slot>=2){
                    if(placement.Installed(2) && placement.Installed(3))hud.Hint("踩上弹弹菇，腾空后再次按住空格展开纸翼",5);
                    else hud.Hint(slot==3?"按 2 选择弹弹菇，装到移动纸砖的白色顶面":"按 3 选择风扇，装到槽端的白面",5);
                }
            }
            else hud.Hint(aim.reason,1.3f);
        }
        preview.Show(motor.Eyes,selected,aim);
        hud.Paint(selected,gear,aim.Valid,placement.Portals.Open(0),placement.Portals.Open(1),motor.Gliding);
    }

    void FixedUpdate()
    {
        motor.Step();
        placement.ApplyDevices(motor.Body,geometry.Bodies);
        var at=hall.InverseTransformPoint(motor.Body.position);
        if(at.y<current.spawnFeet.y-14){Restart();return;}
        if(!checkpoint && motor.Grounded && current.supplyZone.Contains(at)){
            checkpoint=true;gear|=current.supplyGear;
            motor.Equip(gear);
            if(!WorkshopToolSpec.Owned(gear,(int)selected))Cycle(1);
            hud.Hint(current.supplyHint,7);
        }
        if(motor.Grounded && current.exitZone.Contains(at))finish();
    }

    void LateUpdate()
    {
        motor.Pose();
        placement.Portals.Draw(motor.Eyes);
        hud.Exit(motor.Eyes,hall.TransformPoint(current.exitSign-Vector3.forward*.06f));
    }

    void OnDestroy(){foreach(var m in materials)if(m)Destroy(m);if(fibers)Destroy(fibers);}
}
