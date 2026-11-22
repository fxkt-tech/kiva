using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.InputSystem;
using UnityEngine.UI;
using static NightUi;

// The one full-screen settings page, in the shared night-mountain style (see NightUi).
// The lobby shows every row; inside a level the lobby-only rows are left out.
public sealed class SettingsPanel : MonoBehaviour
{
    const float RowHeight=52,ListWidth=680,ViewHeight=422,ListLeft=340,ListTop=126;
    const string Controls="操作";
    static readonly string[] Sections={"鼠标","键盘"};
    // Fixed bindings: section, action, what it does, then the key caps.
    static readonly string[][] Keys={
        new[]{"鼠标","视角","移动鼠标转动视角。","移动鼠标"},
        new[]{"鼠标","青鸾门 / 装置","持时空枪时放青鸾门；持弹弹菇或风扇时把装置装到白面上。","左键"},
        new[]{"鼠标","朱雀门","持时空枪时放朱雀门。","右键"},
        new[]{"鼠标","切换道具","滚动滚轮在已拥有的道具间切换。","滚轮"},
        new[]{"键盘","移动","前后左右移动。方向键同样可用。","W","A","S","D"},
        new[]{"键盘","跳跃","站在地面时起跳。","空格"},
        new[]{"键盘","滑翔","取得纸翼后，在空中按空格展开。","空中","空格"},
        new[]{"键盘","切换道具","1 时空枪，2 弹弹菇，3 风扇。","1","2","3"},
        new[]{"键盘","重试","回到最近的检查点。","R"},
        new[]{"键盘","重开本关","从关卡起点重新开始。","T"},
        new[]{"键盘","设置","暂停并打开设置，再按一次返回游戏。","Esc"},
    };

    CanvasGroup group;
    SettingShape thumb;
    RectTransform page,view,rows,confirm;
    Text title,note,helpName,helpText,confirmText;
    Action resume;
    bool inLevel;
    string category,shown;
    readonly List<string> categories=new List<string>();
    // A display change waits for the player to keep it.
    bool pending;
    float deadline;
    bool oldCustom,oldWindowed;
    int oldWidth,oldHeight,screenWidth,screenHeight;
    FullScreenMode screenMode;

    public static SettingsPanel Create(Transform canvas)
    {
        var panel=new GameObject("设置",typeof(RectTransform),typeof(CanvasGroup)).AddComponent<SettingsPanel>();
        Fill((RectTransform)panel.transform,canvas);
        panel.group=panel.GetComponent<CanvasGroup>();
        Backdrop(panel.transform);
        panel.page=Page(panel.transform,"版心");
        panel.gameObject.SetActive(false);
        return panel;
    }

    // ---------- the page ----------

    // onResume closes the page from the caller's side (a level unpauses); without it the page closes itself.
    public void Open(bool level,Action onResume=null,Action onLeave=null)
    {
        inLevel=level;resume=onResume;pending=false;
        for(int i=page.childCount-1;i>=0;i--)Destroy(page.GetChild(i).gameObject);
        view=rows=null;thumb=null;shown=null;
        categories.Clear();
        foreach(var item in GameSettings.Items)
            if(!(level && item.lobbyOnly) && !categories.Contains(item.category))categories.Add(item.category);
        if(!categories.Contains(category))category=categories[0];
        Back(page,"设置",Resume);
        var restore=Box(page,"恢复默认",826,22,236,46,Color.clear);
        Shape(page,"恢复标",SettingShape.Kind.Diamond,836,37,16,16,White,White);
        Write(page,"恢复本页默认",Hand,22,White,862,26,200,38,TextAnchor.MiddleLeft,true);
        Press(restore,ResetCategory);

        Shape(page,"列表底",SettingShape.Kind.Hair,304,112,742,450,new Color(.024f,.12f,.2f,.6f),Color.clear).fade=.06f;
        Shape(page,"上线",SettingShape.Kind.Hair,284,112,782,1,new Color(White.r,White.g,White.b,.85f),Color.clear);
        Shape(page,"下线",SettingShape.Kind.Hair,284,561,782,1,new Color(White.r,White.g,White.b,.85f),Color.clear);
        Shape(page,"星",SettingShape.Kind.Spark,290,104,16,16,White,White);
        Shape(page,"星",SettingShape.Kind.Spark,1042,553,16,16,White,White);
        note=Write(page,"",Body,13,new Color(White.r,White.g,White.b,.7f),ListLeft,84,700,24);

        Shape(page,"说明标",SettingShape.Kind.Diamond,330,588,16,16,VermilionLight,Vermilion);
        helpName=Write(page,"",Hand,22,Blush,360,578,400,32);
        helpText=Write(page,"",Body,15,new Color(White.r,White.g,White.b,.9f),360,612,560,48,TextAnchor.UpperLeft);
        helpText.horizontalOverflow=HorizontalWrapMode.Wrap;

        title=Banner(page,category);

        if(level){
            Diamond(page,"选关",1036,620,96,25,false,()=>{if(onLeave!=null)onLeave();else Close();});
            Diamond(page,"继续",1170,620,124,31,true,Resume);
        }else Diamond(page,"完成",1170,620,124,31,true,Resume);

        // Shown while a display change waits to be kept.
        confirm=Place(page,"确认",330,500,690,50);
        Shape(confirm,"底",SettingShape.Kind.Hex,0,0,690,50,new Color(.07f,.42f,.58f),new Color(.04f,.23f,.34f)).raycastTarget=true;
        confirmText=Write(confirm,"",Body,15,White,36,0,400,50);
        var keep=Box(confirm,"保留",480,10,80,30,new Color(1,.89f,.84f));
        Write(keep.transform,"保留",Hand,19,Vermilion,0,0,80,30,TextAnchor.MiddleCenter).GetComponent<Shadow>().enabled=false;
        Press(keep,()=>{pending=false;ShowRows();});
        var revert=Box(confirm,"还原",572,10,80,30,new Color(1,1,1,.16f));
        Write(revert.transform,"还原",Hand,19,White,0,0,80,30,TextAnchor.MiddleCenter);
        Press(revert,Revert);

        ShowRows();
        transform.SetAsLastSibling();
        group.alpha=0;
        gameObject.SetActive(true);
    }

    void ShowRows()
    {
        // Choosing an option rebuilds the rows; stay where the list was scrolled to.
        float scrolled=rows && shown==category?rows.anchoredPosition.y:0;
        if(view){view.gameObject.SetActive(false);Destroy(view.gameObject);}
        var old=page.Find("分类");
        if(old){old.gameObject.SetActive(false);Destroy(old.gameObject);}
        shown=category;
        title.text=Vertical(category);

        var tabs=Place(page,"分类",0,0,300,720);
        tabs.SetSiblingIndex(0);
        for(int i=0;i<categories.Count;i++){
            string name=categories[i];
            bool chosen=name==category;
            float y=128+i*78;
            var hit=Box(tabs,name,44,y,250,56,Color.clear);
            if(chosen){
                var band=Shape(tabs,"带",SettingShape.Kind.Bar,72,y+6,232,44,new Color(.16f,.59f,.8f,.75f),new Color(.16f,.59f,.8f,0));
                Shape(band.transform,"线",SettingShape.Kind.Bar,0,0,232,1,new Color(White.r,White.g,White.b,.5f),new Color(1,1,1,0));
                Shape(band.transform,"线",SettingShape.Kind.Bar,0,43,232,1,new Color(White.r,White.g,White.b,.5f),new Color(1,1,1,0));
            }
            Gem(tabs,71,y+28,chosen?62:48,chosen);
            Write(tabs,name,Hand,chosen?32:28,new Color(White.r,White.g,White.b,chosen?1:.62f),116,y,170,56,TextAnchor.MiddleLeft,chosen)
                .text=Spaced(name);
            Press(hit,()=>{category=name;ShowRows();});
        }

        view=Place(page,"设置项",ListLeft,ListTop,ListWidth,ViewHeight);
        view.gameObject.AddComponent<RawImage>().color=Color.clear;
        view.gameObject.AddComponent<RectMask2D>();
        rows=Place(view,"行",0,0,ListWidth,0);
        bool hidden=false;
        float top=0;
        void Rows(string section)
        {
            foreach(var item in GameSettings.Items){
                if(item.category!=category || item.section!=section)continue;
                if(inLevel && item.lobbyOnly){hidden=true;continue;}
                var row=Row(item.label,item.help,top,item.Changed);
                if(item.kind==SettingItem.Kind.Slider)Slide(row,item);
                else if(item.kind==SettingItem.Kind.Options)Options(row,item);
                else Stepper(row,item);
                top+=RowHeight;
            }
        }
        if(category==Controls)foreach(string section in Sections){
            if(top>0)top+=8;
            Shape(rows,section+"标",SettingShape.Kind.Diamond,2,top+13,13,13,Blush,Blush);
            Write(rows,section,Hand,22,Blush,26,top,120,40);
            Shape(rows,"线",SettingShape.Kind.Bar,96,top+20,ListWidth-110,1,new Color(Blush.r,Blush.g,Blush.b,.7f),new Color(Blush.r,Blush.g,Blush.b,0));
            top+=40;
            Rows(section);
            foreach(var key in Keys)if(key[0]==section){
                var row=Row(key[1],key[2],top,false);
                float right=ListWidth-12;
                for(int k=key.Length-1;k>=3;k--){
                    float width=22+TextWidth(key[k],10,17);
                    var cap=Shape(row,key[k],SettingShape.Kind.Octagon,right-width,11,width,30,new Color(.1f,.27f,.37f),new Color(.07f,.2f,.29f));
                    cap.rim=new Color(White.r,White.g,White.b,.6f);cap.rimWidth=1;
                    Write(cap.transform,key[k],Body,15,White,0,0,width,30,TextAnchor.MiddleCenter);
                    right-=width+6;
                }
                top+=RowHeight;
            }
        }
        else Rows(null);
        note.text=hidden?"分辨率、显示模式和阴影精度在大厅的设置中调整":"";
        Describe(null,null);
        confirm.gameObject.SetActive(pending);
        confirm.SetAsLastSibling();

        rows.sizeDelta=new Vector2(ListWidth,top);
        var scroll=view.gameObject.AddComponent<ScrollRect>();
        scroll.viewport=view;scroll.content=rows;
        scroll.horizontal=false;scroll.movementType=ScrollRect.MovementType.Clamped;
        scroll.scrollSensitivity=36;
        rows.anchoredPosition=new Vector2(0,Mathf.Clamp(scrolled,0,Mathf.Max(0,top-ViewHeight)));
        if(!thumb)thumb=Shape(page,"滚动标",SettingShape.Kind.Fall,1030,ListTop,3,60,AzureLight,Azure);
        MoveThumb();
    }

    // A row lights up under the pointer and explains itself below the list.
    RectTransform Row(string label,string help,float top,bool changed)
    {
        var row=Place(rows,label,0,top,ListWidth,RowHeight);
        row.gameObject.AddComponent<RawImage>().color=Color.clear;
        var lit=Place(row,"高亮",0,0,ListWidth,RowHeight);
        Shape(lit,"带",SettingShape.Kind.Bar,0,0,ListWidth*.8f,RowHeight,new Color(.24f,.67f,.88f,.4f),new Color(.24f,.67f,.88f,0));
        Shape(lit,"标",SettingShape.Kind.Diamond,0,RowHeight*.5f-6,12,12,new Color(1,.89f,.84f),VermilionLight);
        lit.gameObject.SetActive(false);
        Shape(row,"界线",SettingShape.Kind.Hair,0,RowHeight-1,ListWidth,1,new Color(White.r,White.g,White.b,.24f),Color.clear).fade=.08f;
        var name=Write(row,label,Hand,25,White,18,0,300,RowHeight);
        if(changed)Shape(row,"已改",SettingShape.Kind.Diamond,28+name.preferredWidth,RowHeight*.5f-4,9,9,new Color(.89f,.4f,.29f),new Color(.89f,.4f,.29f));
        var hover=row.gameObject.AddComponent<HoverWatch>();
        hover.entered=()=>{lit.gameObject.SetActive(true);Describe(label,help);};
        hover.left=()=>lit.gameObject.SetActive(false);
        return row;
    }

    void Describe(string name,string text)
    {
        helpName.text=name??category;
        helpText.text=text??"把鼠标移到某一项上，这里显示它的作用。";
    }

    void Slide(RectTransform row,SettingItem item)
    {
        const float length=300;
        float right=ListWidth-12;
        var value=Write(row,item.Value,Body,19,White,right-54,0,54,RowHeight,TextAnchor.MiddleRight);
        var root=Place(row,"滑条",right-54-18-length,0,length,RowHeight);
        root.gameObject.AddComponent<RawImage>().color=Color.clear;
        Box(root,"轨",0,RowHeight*.5f-1,length,2,new Color(White.r,White.g,White.b,.3f));
        if(item.steps<=40)for(int i=0;i<=item.steps;i++)
            Box(root,"刻度",i*length/item.steps,RowHeight*.5f+5,1,5,new Color(White.r,White.g,White.b,.4f));
        var fillArea=new GameObject("填充区",typeof(RectTransform)).GetComponent<RectTransform>();
        Fill(fillArea,root);
        // The slider stretches the fill and the handle to their areas, so the areas carry the height.
        fillArea.anchorMin=new Vector2(0,.5f);fillArea.anchorMax=new Vector2(1,.5f);fillArea.sizeDelta=new Vector2(0,4);
        var fill=new GameObject("填充",typeof(RectTransform),typeof(CanvasRenderer)).AddComponent<SettingShape>();
        fill.rectTransform.SetParent(fillArea,false);
        fill.rectTransform.sizeDelta=Vector2.zero;
        fill.kind=SettingShape.Kind.Bar;fill.color=new Color(.24f,.67f,.88f,.6f);fill.second=new Color(1,.54f,.4f);fill.raycastTarget=false;
        var handleArea=new GameObject("游标区",typeof(RectTransform)).GetComponent<RectTransform>();
        Fill(handleArea,root);
        handleArea.anchorMin=new Vector2(0,.5f);handleArea.anchorMax=new Vector2(1,.5f);handleArea.sizeDelta=Vector2.zero;
        var handle=new GameObject("游标",typeof(RectTransform)).GetComponent<RectTransform>();
        handle.SetParent(handleArea,false);
        handle.sizeDelta=new Vector2(24,24);
        var glow=Box(handle,"光",-12,-12,48,48,new Color(1,.47f,.31f,.8f));
        glow.texture=Glow;
        var bead=Shape(handle,"菱",SettingShape.Kind.Diamond,0,0,24,24,Color.white,new Color(1,.93f,.9f));
        bead.rim=new Color(.85f,.28f,.17f);bead.rimWidth=2.4f;

        var slider=root.gameObject.AddComponent<Slider>();
        slider.transition=Selectable.Transition.None;
        slider.navigation=new Navigation{mode=Navigation.Mode.None};
        slider.fillRect=fill.rectTransform;slider.handleRect=handle;
        slider.direction=Slider.Direction.LeftToRight;
        slider.wholeNumbers=true;slider.minValue=0;slider.maxValue=item.steps;
        slider.SetValueWithoutNotify(item.Index);
        slider.onValueChanged.AddListener(v=>{item.Choose(Mathf.RoundToInt(v));value.text=item.Value;});
    }

    void Options(RectTransform row,SettingItem item)
    {
        float right=ListWidth-12;
        int chosen=item.Index;
        for(int i=item.names.Length-1;i>=0;i--){
            string name=item.names[i];
            float width=38+TextWidth(name,11,21);
            bool on=i==chosen;
            var chip=Shape(row,name,SettingShape.Kind.Hex,right-width,9,width,34,
                on?AzureLight:Color.clear,on?new Color(.07f,.42f,.58f):Color.clear);
            Write(chip.transform,name,Hand,21,new Color(White.r,White.g,White.b,on?1:.55f),0,0,width,34,TextAnchor.MiddleCenter);
            int pick=i;
            Press(chip,()=>Change(item,pick));
            right-=width+4;
        }
    }

    void Stepper(RectTransform row,SettingItem item)
    {
        float right=ListWidth-12;
        for(int side=1;side>=-1;side-=2){
            int direction=side;
            float x=side>0?right-36:right-36-190-36;
            var arrow=Shape(row,side<0?"上一项":"下一项",SettingShape.Kind.Diamond,x,8,36,36,new Color(.36f,.75f,.88f),AzureDeep);
            arrow.rim=new Color(1,.89f,.84f,.8f);arrow.rimWidth=1.5f;
            Write(arrow.transform,side<0?"‹":"›",Body,20,White,0,-2,36,36,TextAnchor.MiddleCenter);
            Press(arrow,()=>Change(item,direction));
        }
        Write(row,item.Value,Body,19,White,right-36-190,0,190,RowHeight,TextAnchor.MiddleCenter);
    }

    void Change(SettingItem item,int value)
    {
        if(item.display){
            if(!pending){
                var s=GameSettings.Current;
                oldCustom=s.customDisplay;oldWindowed=s.windowed;oldWidth=s.width;oldHeight=s.height;
                screenWidth=Screen.width;screenHeight=Screen.height;screenMode=Screen.fullScreenMode;
            }
            pending=true;deadline=Time.unscaledTime+10;
        }
        item.Choose(value);
        ShowRows();
    }

    void Revert()
    {
        var s=GameSettings.Current;
        s.customDisplay=oldCustom;s.windowed=oldWindowed;s.width=oldWidth;s.height=oldHeight;
        Screen.SetResolution(screenWidth,screenHeight,screenMode);
        s.Save();
        pending=false;
        ShowRows();
    }

    void ResetCategory()
    {
        if(pending)Revert();
        foreach(var item in GameSettings.Items)
            if(item.category==category && !(inLevel && item.lobbyOnly))item.Reset();
        ShowRows();
    }

    void MoveThumb()
    {
        if(!thumb || !rows)return;
        float extra=rows.sizeDelta.y-ViewHeight;
        thumb.enabled=extra>1;
        if(!thumb.enabled)return;
        float length=ViewHeight*ViewHeight/rows.sizeDelta.y;
        thumb.rectTransform.sizeDelta=new Vector2(3,length);
        thumb.rectTransform.anchoredPosition=new Vector2(1030,-ListTop-(ViewHeight-length)*Mathf.Clamp01(rows.anchoredPosition.y/extra));
    }

    void Resume()
    {
        if(resume!=null)resume();
        else Close();
    }

    // Leaving the page keeps a display change that was still waiting.
    public void Close()
    {
        pending=false;
        if(EventSystem.current)EventSystem.current.SetSelectedGameObject(null);
        gameObject.SetActive(false);
    }

    void Update()
    {
        group.alpha=Mathf.MoveTowards(group.alpha,1,Time.unscaledDeltaTime*7);
        MoveThumb();
        if(pending){
            float left=deadline-Time.unscaledTime;
            if(left<=0)Revert();
            else confirmText.text="显示设置已更改，"+Mathf.CeilToInt(left)+" 秒后自动还原。";
        }
        // Inside a level the level itself owns Esc.
        if(!inLevel && Keyboard.current!=null && Keyboard.current.escapeKey.wasPressedThisFrame)Close();
    }

}
