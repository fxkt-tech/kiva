using System;
using System.Collections.Generic;
using System.IO;
using UnityEngine;

// One saved record shared by the lobby settings and the in-level settings.
[Serializable]
public sealed class GameSettings
{
    public float sensitivity=1;
    public bool invertY,glideToggle;
    public int fieldOfView=70,portalDepth=-1,frameCap;
    public bool vSync=true;
    public int shadowResolution=2048;
    // The launch resolution and window mode are kept until the player chooses otherwise.
    public bool customDisplay,windowed;
    public int width,height;
    public bool crosshair=true,exitGuide=true,hints=true;

    static GameSettings current,defaults;
    public static GameSettings Current=>current??=Load();
    public static GameSettings Defaults=>defaults??=new GameSettings().Normalised();
    static string SavePath=>Path.Combine(Application.persistentDataPath,"settings.json");

    GameSettings Normalised()
    {
        if(portalDepth<0)portalDepth=Resources.Load<PortalRenderSettings>("Portals/Rendering").MaxDepth;
        sensitivity=Mathf.Clamp(sensitivity,.2f,3);
        fieldOfView=Mathf.Clamp(fieldOfView,60,100);
        portalDepth=Mathf.Clamp(portalDepth,0,4);
        return this;
    }

    static GameSettings Load()
    {
        var loaded=new GameSettings();
        try{
            if(File.Exists(SavePath))JsonUtility.FromJsonOverwrite(File.ReadAllText(SavePath),loaded);
        }catch(Exception e){
            Debug.LogWarning("设置读取失败，使用默认值："+e.Message);
            loaded=new GameSettings();
        }
        return loaded.Normalised();
    }

    public void Save()
    {
        try{File.WriteAllText(SavePath,JsonUtility.ToJson(this,true));}
        catch(Exception e){Debug.LogWarning("设置保存失败："+e.Message);}
    }

    public void Apply()
    {
        QualitySettings.vSyncCount=vSync?1:0;
        Application.targetFrameRate=frameCap>0?frameCap:-1;
    }

    public void ApplyDisplay()
    {
        if(!customDisplay)return;
        var mode=windowed?FullScreenMode.Windowed:FullScreenMode.FullScreenWindow;
        if(width>0 && height>0)Screen.SetResolution(width,height,mode);
        else Screen.fullScreenMode=mode;
    }

    void StepResolution(int direction)
    {
        var sizes=new List<Vector2Int>();
        foreach(var r in Screen.resolutions){
            var size=new Vector2Int(r.width,r.height);
            if(!sizes.Contains(size))sizes.Add(size);
        }
        if(sizes.Count==0)return;
        var now=width>0?new Vector2Int(width,height):new Vector2Int(Screen.width,Screen.height);
        int index=sizes.IndexOf(now);
        index=index<0?sizes.Count-1:(index+direction+sizes.Count)%sizes.Count;
        width=sizes[index].x;height=sizes[index].y;customDisplay=true;
    }

    void ForgetDisplay(){customDisplay=false;width=height=0;}

    // The single definition table. Both screens are generated from it; lobby-only rows are
    // hidden inside a level.
    public static readonly SettingItem[] Items={
        SettingItem.Slider("操作","鼠标灵敏度",s=>s.sensitivity,(s,v)=>s.sensitivity=v,.2f,3,.1f,"0.0")
            .In("鼠标").Help("转动视角的快慢。1.0 是默认手感，数值越大转得越快。"),
        SettingItem.Toggle("操作","反转 Y 轴",s=>s.invertY,(s,v)=>s.invertY=v)
            .In("鼠标").Help("开启后，鼠标向前推视角向下，向后拉视角向上。"),
        SettingItem.Toggle("操作","滑翔方式",s=>s.glideToggle,(s,v)=>s.glideToggle=v,"按住空格","按一下切换")
            .In("键盘").Help("按住：松开空格就收起纸翼。切换：空中按一下展开，再按一下收起。"),
        SettingItem.Slider("画面","视野角",s=>s.fieldOfView,(s,v)=>s.fieldOfView=Mathf.RoundToInt(v),60,100,5,"0")
            .Help("视野的宽窄。数值大看得更广，但画面边缘会拉伸。"),
        SettingItem.Slider("画面","门中门层数",s=>s.portalDepth,(s,v)=>s.portalDepth=Mathf.RoundToInt(v),0,4,1,"0")
            .Help("从一扇门里最多能连续看穿几层门。0 时门内只显示纸面，层数越高越耗性能。"),
        SettingItem.Toggle("画面","垂直同步",s=>s.vSync,(s,v)=>s.vSync=v)
            .Help("让帧率跟随显示器刷新率，消除画面撕裂，可能略增操作延迟。"),
        SettingItem.Choice("画面","帧率上限",s=>s.frameCap,(s,v)=>s.frameCap=v,
            new[]{0,30,60,120,144,240},new[]{"不限","30","60","120","144","240"})
            .Help("限制最高帧率，降低发热和风扇噪音。"),
        SettingItem.Choice("画面","阴影精度",s=>s.shadowResolution,(s,v)=>s.shadowResolution=v,
            new[]{1024,2048,4096},new[]{"低","中","高"})
            .LobbyOnly().Help("西窗日光投影的清晰度。进入关卡时生效。"),
        new SettingItem(SettingItem.Kind.Options,"画面","显示模式",new[]{"窗口","全屏"},0,
            s=>(s.customDisplay?!s.windowed:Screen.fullScreen)?1:0,(s,i)=>{s.windowed=i==0;s.customDisplay=true;},null)
            .Display(s=>s.customDisplay,s=>s.ForgetDisplay()).Help("窗口或无边框全屏。改动后需要确认，10 秒不确认自动还原。"),
        new SettingItem(SettingItem.Kind.Stepper,"画面","分辨率",null,0,null,(s,d)=>s.StepResolution(d),
            s=>s.width>0?s.width+" × "+s.height:Screen.width+" × "+Screen.height)
            .Display(s=>s.width>0,s=>s.ForgetDisplay()).Help("游戏画面的像素尺寸。改动后需要确认，10 秒不确认自动还原。"),
        SettingItem.Toggle("界面","准星",s=>s.crosshair,(s,v)=>s.crosshair=v)
            .Help("屏幕中央的瞄准标记，同时显示两扇门是否已放置。"),
        SettingItem.Toggle("界面","出口指引",s=>s.exitGuide,(s,v)=>s.exitGuide=v)
            .Help("看不到出口时，在屏幕边缘提示出口方向。"),
        SettingItem.Toggle("界面","操作提示",s=>s.hints,(s,v)=>s.hints=v)
            .Help("左下角的引导文字，比如取得装置后的用法说明。"),
    };
}

// Slider: Index runs over 0..steps. Options: Index picks one of names. Stepper: Choose takes a direction.
public sealed class SettingItem
{
    public enum Kind { Slider, Options, Stepper }
    public readonly Kind kind;
    public readonly string category,label;
    public readonly string[] names;
    public readonly int steps;
    public string section,help;
    // Display rows change the window itself: lobby only, and confirmed before they are kept.
    public bool lobbyOnly,display;
    readonly Func<GameSettings,int> index;
    readonly Action<GameSettings,int> choose;
    readonly Func<GameSettings,string> show;
    Func<GameSettings,bool> changed;
    Action<GameSettings> reset;

    public SettingItem(Kind kind,string category,string label,string[] names,int steps,Func<GameSettings,int> index,
        Action<GameSettings,int> choose,Func<GameSettings,string> show)
    {
        this.kind=kind;this.category=category;this.label=label;this.names=names;this.steps=steps;
        this.index=index;this.choose=choose;this.show=show;
    }

    public SettingItem In(string heading){section=heading;return this;}
    public SettingItem Help(string text){help=text;return this;}
    public SettingItem LobbyOnly(){lobbyOnly=true;return this;}
    public SettingItem Display(Func<GameSettings,bool> isChanged,Action<GameSettings> forget)
    {
        lobbyOnly=display=true;changed=isChanged;reset=forget;return this;
    }

    public int Index=>index(GameSettings.Current);
    public string Value=>show(GameSettings.Current);
    public bool Changed=>changed!=null?changed(GameSettings.Current):index(GameSettings.Current)!=index(GameSettings.Defaults);

    // Change, apply and save in one place.
    public void Choose(int value)
    {
        var settings=GameSettings.Current;
        choose(settings,value);
        settings.Apply();
        if(display)settings.ApplyDisplay();
        settings.Save();
    }

    public void Reset()
    {
        var settings=GameSettings.Current;
        if(reset!=null)reset(settings);
        else choose(settings,index(GameSettings.Defaults));
        settings.Apply();
        settings.Save();
    }

    public static SettingItem Slider(string category,string label,Func<GameSettings,float> get,Action<GameSettings,float> set,
        float min,float max,float unit,string format)
        =>new SettingItem(Kind.Slider,category,label,null,Mathf.RoundToInt((max-min)/unit),
            s=>Mathf.RoundToInt((get(s)-min)/unit),(s,i)=>set(s,Mathf.Clamp(min+i*unit,min,max)),s=>get(s).ToString(format));

    public static SettingItem Toggle(string category,string label,Func<GameSettings,bool> get,Action<GameSettings,bool> set,
        string off="关",string on="开")
        =>new SettingItem(Kind.Options,category,label,new[]{off,on},0,s=>get(s)?1:0,(s,i)=>set(s,i==1),null);

    public static SettingItem Choice(string category,string label,Func<GameSettings,int> get,Action<GameSettings,int> set,
        int[] values,string[] names)
        =>new SettingItem(Kind.Options,category,label,names,0,s=>Mathf.Max(0,Array.IndexOf(values,get(s))),
            (s,i)=>set(s,values[i]),null);
}
