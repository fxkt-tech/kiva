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

    static GameSettings current;
    public static GameSettings Current=>current??=Load();
    static string SavePath=>Path.Combine(Application.persistentDataPath,"settings.json");

    static GameSettings Load()
    {
        var loaded=new GameSettings();
        try{
            if(File.Exists(SavePath))JsonUtility.FromJsonOverwrite(File.ReadAllText(SavePath),loaded);
        }catch(Exception e){
            Debug.LogWarning("设置读取失败，使用默认值："+e.Message);
            loaded=new GameSettings();
        }
        if(loaded.portalDepth<0)loaded.portalDepth=Resources.Load<PortalRenderSettings>("Portals/Rendering").MaxDepth;
        loaded.sensitivity=Mathf.Clamp(loaded.sensitivity,.2f,3);
        loaded.fieldOfView=Mathf.Clamp(loaded.fieldOfView,60,100);
        loaded.portalDepth=Mathf.Clamp(loaded.portalDepth,0,4);
        return loaded;
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

    // The single definition table. Both screens are generated from it; lobbyOnly rows are
    // hidden inside a level.
    public static readonly SettingItem[] Items={
        SettingItem.Slider("操作","鼠标灵敏度",s=>s.sensitivity,(s,v)=>s.sensitivity=v,.2f,3,.1f,"0.0").In("鼠标"),
        SettingItem.Toggle("操作","反转 Y 轴",s=>s.invertY,(s,v)=>s.invertY=v).In("鼠标"),
        SettingItem.Toggle("操作","滑翔方式",s=>s.glideToggle,(s,v)=>s.glideToggle=v,"按住空格","按一下切换").In("键盘"),
        SettingItem.Slider("画面","视野角",s=>s.fieldOfView,(s,v)=>s.fieldOfView=Mathf.RoundToInt(v),60,100,5,"0"),
        SettingItem.Slider("画面","门中门层数",s=>s.portalDepth,(s,v)=>s.portalDepth=Mathf.RoundToInt(v),0,4,1,"0"),
        SettingItem.Toggle("画面","垂直同步",s=>s.vSync,(s,v)=>s.vSync=v),
        SettingItem.Choice("画面","帧率上限",s=>s.frameCap,(s,v)=>s.frameCap=v,
            new[]{0,30,60,120,144,240},new[]{"不限","30","60","120","144","240"}),
        SettingItem.Choice("画面","阴影精度",s=>s.shadowResolution,(s,v)=>s.shadowResolution=v,
            new[]{1024,2048,4096},new[]{"低","中","高"},true),
        new SettingItem(SettingItem.Kind.Options,"画面","显示模式",new[]{"窗口","全屏"},0,
            s=>(s.customDisplay?!s.windowed:Screen.fullScreen)?1:0,(s,i)=>{s.windowed=i==0;s.customDisplay=true;},null,true,true),
        new SettingItem(SettingItem.Kind.Stepper,"画面","分辨率",null,0,null,(s,d)=>s.StepResolution(d),
            s=>s.width>0?s.width+" × "+s.height:"默认",true,true),
        SettingItem.Toggle("界面","准星",s=>s.crosshair,(s,v)=>s.crosshair=v),
        SettingItem.Toggle("界面","出口指引",s=>s.exitGuide,(s,v)=>s.exitGuide=v),
        SettingItem.Toggle("界面","操作提示",s=>s.hints,(s,v)=>s.hints=v),
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
    public readonly bool lobbyOnly,display;
    // Optional sub-heading inside the category.
    public string section;
    public SettingItem In(string heading){section=heading;return this;}
    readonly Func<GameSettings,int> index;
    readonly Action<GameSettings,int> choose;
    readonly Func<GameSettings,string> show;

    public SettingItem(Kind kind,string category,string label,string[] names,int steps,Func<GameSettings,int> index,
        Action<GameSettings,int> choose,Func<GameSettings,string> show,bool lobbyOnly=false,bool display=false)
    {
        this.kind=kind;this.category=category;this.label=label;this.names=names;this.steps=steps;
        this.index=index;this.choose=choose;this.show=show;this.lobbyOnly=lobbyOnly;this.display=display;
    }

    public int Index=>index(GameSettings.Current);
    public string Value=>show(GameSettings.Current);

    // Change, apply and save in one place.
    public void Choose(int value)
    {
        var settings=GameSettings.Current;
        choose(settings,value);
        settings.Apply();
        if(display)settings.ApplyDisplay();
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
        int[] values,string[] names,bool lobbyOnly=false)
        =>new SettingItem(Kind.Options,category,label,names,0,s=>Mathf.Max(0,Array.IndexOf(values,get(s))),
            (s,i)=>set(s,values[i]),null,lobbyOnly);
}
