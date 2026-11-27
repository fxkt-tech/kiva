using System;
using UnityEngine;
using UnityEngine.UI;
using static NightUi;
using static SilkUi;

public sealed class SettingsPanel : MonoBehaviour
{
    const float ContentX=351, Width=863, RowHeight=63, ViewTop=153, ViewHeight=448, FooterY=633;
    const float DefaultSeconds=10;
    static readonly string[] Categories={"界面","操作","画面"};
    static readonly Color DarkSilk=new Color(.12f,.086f,.145f);
    static readonly Color MenuIvory=new Color(.985f,.942f,.839f);
    static readonly Color BodyInk=new Color(.245f,.175f,.25f);
    static readonly Color Notes=new Color(.355f,.29f,.37f);
    static readonly Color Gilt=new Color(.66f,.48f,.28f);
    static readonly Rect RibbonUv=new Rect(705/1536f,1-670/1024f,781/1536f,187/1024f);
    static readonly Rect PaperSignUv=new Rect(2/1536f,1-1007/1024f,760/1536f,212/1024f);

    [SerializeField] RectTransform page;
    RectTransform rows,footer;
    CanvasGroup controls;
    ScrollRect scroll;
    Text timer;
    [SerializeField] Texture2D paperSigns,workshopView;
    Action resume,leave;
    bool pending;
    float deadline;
    string category=Categories[0],progress;
    DisplaySnapshot before;

    struct DisplaySnapshot
    {
        public bool custom,windowed;
        public int width,height,visibleWidth,visibleHeight;
        public FullScreenMode mode;

        public static DisplaySnapshot Capture()
        {
            var saved=GameSettings.Current;
            return new DisplaySnapshot {
                custom=saved.customDisplay,windowed=saved.windowed,width=saved.width,height=saved.height,
                visibleWidth=Screen.width,visibleHeight=Screen.height,mode=Screen.fullScreenMode
            };
        }

        public void Restore()
        {
            var saved=GameSettings.Current;
            saved.customDisplay=custom;saved.windowed=windowed;saved.width=width;saved.height=height;
            saved.RequestDisplay(new Vector2Int(visibleWidth,visibleHeight),mode);
        }
    }

    public void Open(Action onResume,Action onLeave,string status)
    {
        resume=onResume;leave=onLeave;progress=status;
        transform.SetAsLastSibling();
        gameObject.SetActive(true);
        Draw();
    }

    void Draw(float offset=0,string focusName=null)
    {
        Clear(page);
        var content=Place(page,"设置内容",0,0,1280,720);
        controls=content.gameObject.AddComponent<CanvasGroup>();
        var currentTab=Navigation(content);
        Label(content,category,ContentX-3,50,342,75,47,BodyInk,NightUi.Display);
        Label(content,progress,697,52,516,55,20,Notes,Hand,TextAnchor.MiddleRight);
        Rule(content,ContentX,132,Width);

        var view=Place(content,"选项列表",ContentX,ViewTop,Width,ViewHeight);
        var scrollHit=view.gameObject.AddComponent<Image>();
        scrollHit.color=new Color(1,1,1,.012f);
        view.gameObject.AddComponent<RectMask2D>();
        scroll=view.gameObject.AddComponent<ScrollRect>();
        scroll.viewport=view;
        scroll.horizontal=false;
        scroll.scrollSensitivity=39;
        scroll.movementType=ScrollRect.MovementType.Clamped;
        rows=Place(view,"行",0,0,Width,0);
        scroll.content=rows;

        float top=0;
        if(category=="界面"){CrosshairPreview();top=174;}
        foreach(var item in GameSettings.Items)
        {
            if(item.category!=category || item.label=="准星")continue;
            Row(item,top);
            top+=RowHeight;
        }
        if(category=="操作")top=KeyGuide(top);
        rows.sizeDelta=new Vector2(Width,top);
        rows.anchoredPosition=Vector2.up*Mathf.Clamp(offset,0,Mathf.Max(0,top-ViewHeight));
        if(top>ViewHeight)ScrollRail(content);

        Rule(content,ContentX,619,Width);
        footer=Place(content,"设置操作",ContentX,FooterY,Width,57);
        ActionStamp(footer,"恢复本页默认",0,1,183,48,ResetCategory);
        ActionStamp(footer,"返回大厅",487,1,163,48,leave);
        ActionStamp(footer,"继续",673,0,190,51,Back,true);
        if(focusName!=null)
        {
            foreach(var choice in rows.GetComponentsInChildren<Selectable>())
                if(choice.name==focusName){Focus(choice);return;}
        }
        Focus(currentTab);
    }

    Button Navigation(Transform content)
    {
        var back=ActionStamp(content,"返回",46,39,132,48,Back);
        Shape(back.transform,"左箭头",SettingShape.Kind.Chevron,22,17,13,14,BodyInk,BodyInk);
        back.GetComponentInChildren<Text>().rectTransform.anchoredPosition+=Vector2.right*9;

        var title=Label(content,"设置",49,114,220,97,71,MenuIvory,NightUi.Display);
        ShadowText(title,new Color(.09f,.045f,.12f),2);
        Shape(content,"书脊金缕",SettingShape.Kind.Hair,54,221,190,1.5f,new Color(.82f,.65f,.42f,.68f),Color.clear);
        Button current=null;
        for(int i=0;i<Categories.Length;i++)
        {
            string name=Categories[i];
            bool chosen=name==category;
            var hit=Panel(content,name,47,250+76*i,228,61,new Color(0,0,0,.006f));
            var tab=SelectableButton(hit,()=>{if(category!=name){category=name;Draw();}});
            tab.transition=Selectable.Transition.None;
            var ribbon=Art(hit.transform,"绢签",paperSigns,0,0,228,61,false);
            ribbon.uvRect=RibbonUv;
            ribbon.color=chosen?new Color(1,.93f,1):new Color(.77f,.67f,.88f,.44f);
            var tint=chosen?MenuIvory:new Color(.91f,.85f,.91f);
            Mark(hit.transform,43,23,chosen,true);
            var label=Label(hit.transform,name,73,1,121,58,29,tint,Hand);
            ShadowText(label,DarkSilk,1);
            FocusGlint(tab,228,61);
            if(chosen)current=tab;
        }
        return current;
    }

    Button ActionStamp(Transform parent,string name,float x,float y,float w,float h,Action action,bool primary=false)
    {
        var hit=Panel(parent,name,x,y,w,h,new Color(0,0,0,.006f));
        var button=SelectableButton(hit,action);
        button.transition=Selectable.Transition.None;
        var skin=Art(hit.transform,"纸签",paperSigns,0,0,w,h,false);
        skin.uvRect=PaperSignUv;
        skin.color=primary?new Color(1,.96f,.84f):new Color(.89f,.84f,.86f);
        Label(hit.transform,name,10,0,w-20,h,primary?25:20,BodyInk,Hand,TextAnchor.MiddleCenter);
        FocusGlint(button,w,h);
        return button;
    }

    static void FocusGlint(Button button,float width,float height)
    {
        var aura=Place(button.transform,"焦点微光",0,0,width,height).gameObject.AddComponent<CanvasGroup>();
        aura.alpha=0;aura.blocksRaycasts=false;aura.interactable=false;
        Shape(aura.transform,"签面反光",SettingShape.Kind.SoftShadow,8,2,width-16,height-4,new Color(1,.91f,.75f,.3f),Color.clear);
        Shape(aura.transform,"暖光",SettingShape.Kind.Spark,width-22,8,7,11,new Color(1,.95f,.83f,.91f),Color.clear);
        button.gameObject.AddComponent<LobbyPlaqueFocus>().Bind(aura);
    }

    static void Rule(Transform parent,float x,float y,float w)
    {
        Shape(parent,"细金分隔",SettingShape.Kind.Hair,x,y,w,1,new Color(.63f,.47f,.33f,.60f),Color.clear).fade=.05f;
    }

    void ScrollRail(Transform content)
    {
        var rail=Panel(content,"滚动轨道",1231,ViewTop,8,ViewHeight,new Color(.48f,.37f,.51f,.18f));
        rail.raycastTarget=true;
        var travel=Place(rail.transform,"滑块区域",0,0,8,ViewHeight);
        var amber=Panel(travel,"滑块",0,0,8,1,Violet);
        var bar=rail.gameObject.AddComponent<Scrollbar>();
        bar.targetGraphic=amber;
        bar.handleRect=amber.rectTransform;
        bar.direction=Scrollbar.Direction.BottomToTop;
        scroll.verticalScrollbar=bar;
    }

    void CrosshairPreview()
    {
        var item=Array.Find(GameSettings.Items,value=>value.label=="准星");
        Label(rows,"准星显示",4,22,245,43,27,BodyInk,Hand);
        Label(rows,"调整第一人称准星是否可见。",5,71,220,60,16,Notes,Body);
        for(int i=0;i<2;i++)
        {
            int option=1-i;
            string name=option==1?"显示准星":"隐藏准星";
            bool selected=item.Index==option;
            var hit=Link(rows,name,294+i*284,0,266,154,()=>Change(item,option,name));
            hit.transition=Selectable.Transition.None;
            var bed=Panel(hit.transform,"织边",0,0,266,154,selected?new Color(.868f,.82f,.884f):new Color(.91f,.87f,.866f,.72f));
            Frame(bed.transform,266,154,selected?Gilt:new Color(.57f,.46f,.55f,.44f),1);
            var preview=Art(hit.transform,"纸坊预览",workshopView,7,7,252,106);
            if(option==1)
            {
                var center=Place(preview.transform,"时空枪准星",110,37,32,32);
                center.gameObject.AddComponent<CanvasRenderer>();
                var cross=center.gameObject.AddComponent<WorkshopGlyph>();
                cross.raycastTarget=false;
                cross.Show(0,true,false,true,true);
            }
            Mark(hit.transform,16,124,selected,false);
            Label(hit.transform,name,46,113,206,40,20,BodyInk,Hand);
            FocusGlint(hit,266,154);
            hit.gameObject.AddComponent<ScrollFocus>().scroll=scroll;
        }
        Rule(rows,0,164,Width);
    }

    static void Mark(Transform parent,float x,float y,bool selected,bool silk)
    {
        Color rim=silk?new Color(.9f,.71f,.49f,.88f):new Color(.52f,.37f,.50f,.64f);
        var diamond=Shape(parent,"菱纹",SettingShape.Kind.Diamond,x,y,16,16,
            selected?new Color(.63f,.43f,.27f):Color.clear,silk?DarkSilk:Color.clear);
        diamond.rimWidth=1.1f;
        diamond.rim=rim;
        if(selected)Shape(diamond.transform,"菱心",SettingShape.Kind.Diamond,5,5,6,6,MenuIvory,MenuIvory);
    }

    void Row(SettingItem item,float top)
    {
        var row=Place(rows,item.label,0,top,Width,RowHeight);
        Label(row,item.label,4,1,331,35,23,BodyInk,Hand);
        Label(row,item.help,5,35,Width-12,26,15,Notes,Body);
        Rule(row,0,RowHeight-1,Width);

        if(item.kind==SettingItem.Kind.Slider && item.label!="门中门层数") SliderRow(row,item);
        else if(item.kind==SettingItem.Kind.Options && item.names.Length==2) ToggleRow(row,item);
        else StepRow(row,item);
        foreach(var button in row.GetComponentsInChildren<Selectable>())
            button.gameObject.AddComponent<ScrollFocus>().scroll=scroll;
    }

    void SliderRow(Transform row,SettingItem item)
    {
        var position=Place(row,item.label+"滑轨",440,0,321,42);
        var hit=position.gameObject.AddComponent<Image>();
        hit.color=new Color(1,1,1,.014f);
        Panel(position,"紫绢轨",0,19,321,3,new Color(.55f,.43f,.55f,.55f));
        var handleTrack=Place(position,"游标行",0,9,321,23);
        var handle=Shape(handleTrack,"琉璃游标",SettingShape.Kind.Diamond,0,0,18,23,Gilt,new Color(.46f,.30f,.24f));
        handle.rimWidth=1;handle.rim=MenuIvory;
        handle.rectTransform.sizeDelta=new Vector2(18,0);
        handle.rectTransform.pivot=Vector2.one*.5f;
        handle.rectTransform.anchoredPosition=Vector2.zero;
        var slider=position.gameObject.AddComponent<Slider>();
        slider.targetGraphic=handle;
        slider.handleRect=handle.rectTransform;
        slider.minValue=0;slider.maxValue=item.steps;slider.wholeNumbers=true;
        slider.SetValueWithoutNotify(item.Index);
        var value=Label(row,item.Value,770,0,85,40,19,BodyInk,Body,TextAnchor.MiddleRight);
        slider.onValueChanged.AddListener(v=>{item.Choose(Mathf.RoundToInt(v));value.text=item.Value;});
    }

    void ToggleRow(Transform row,SettingItem item)
    {
        for(int i=0;i<2;i++)
        {
            int option=i;
            bool selected=item.Index==i;
            string key=item.label+"/"+item.names[i];
            var hit=Link(row,key,480+187*i,0,180,39,()=>Change(item,option,key));
            hit.transition=Selectable.Transition.None;
            if(selected)Shape(hit.transform,"选中绢底",SettingShape.Kind.FoldTab,0,2,180,33,new Color(.79f,.69f,.81f,.47f),new Color(.83f,.75f,.83f,.26f));
            Mark(hit.transform,9,11,selected,false);
            Label(hit.transform,item.names[i],34,0,144,39,20,BodyInk,Hand);
            FocusGlint(hit,180,39);
        }
    }

    void StepRow(Transform row,SettingItem item)
    {
        if(item.kind==SettingItem.Kind.Resolution){
            var settings=GameSettings.Current;
            var currentSize=settings.Resolution;
            for(int i=0;i<GameSettings.ResolutionPresets.Length;i++){
                var preset=GameSettings.ResolutionPresets[i];
                string key=item.label+"/"+preset.name;
                var button=ActionStamp(row,preset.name,175+i*105,1,97,38,()=>{
                    if(settings.width==preset.size.x && settings.height==preset.size.y)return;
                    PreviewDisplay(()=>{settings.SetResolution(preset.size);settings.ApplyDisplay();},key);
                },currentSize==preset.size);
                button.name=key;
            }
        }
        Label(row,item.Value,568,0,231,40,19,BodyInk,Body,TextAnchor.MiddleCenter);
        foreach(int increment in new[]{-1,1})
        {
            int offset=increment;
            string sign=offset<0?"−":"＋";
            string key=item.label+sign;
            var button=ActionStamp(row,sign,offset<0?500:801,0,53,39,()=>{
                int value=item.kind==SettingItem.Kind.Resolution?offset:Mathf.Clamp(item.Index+offset,0,item.steps);
                Change(item,value,key);
            });
            button.name=key;
        }
    }

    float KeyGuide(float top)
    {
        Label(rows,"按键",4,top+7,240,39,26,BodyInk,Hand);
        top+=48;
        string[,] keys={
            {"移动","W A S D / 方向键"},{"转动视角","移动鼠标"},
            {"跳跃 / 滑翔","空格"},{"青门 / 放置装置","鼠标左键"},
            {"朱门","鼠标右键"},{"切换道具","1 / 2 / 3 / 滚轮"},
            {"重试 / 重开","R / T"},{"暂停 / 继续","Esc"}
        };
        for(int i=0;i<keys.GetLength(0);i++)
        {
            float x=i%2==0?4:455,y=top+i/2*38;
            Label(rows,keys[i,0],x,y,193,34,17,BodyInk,Body);
            Label(rows,keys[i,1],x+182,y,216,34,16,Notes,Body,TextAnchor.MiddleRight);
        }
        return top+152;
    }

    void Change(SettingItem item,int choice,string focusName)
    {
        if(item.display){
            PreviewDisplay(()=>item.Choose(choice,false),focusName);
            return;
        }
        item.Choose(choice);
        Draw(rows.anchoredPosition.y,focusName);
    }

    void PreviewDisplay(Action apply,string focusName=null)
    {
        before=DisplaySnapshot.Capture();
        apply();
        Draw(rows.anchoredPosition.y,focusName);
        pending=true;
        deadline=Time.unscaledTime+DefaultSeconds;
        controls.interactable=false;
        controls.blocksRaycasts=false;
        footer.gameObject.SetActive(false);
        var banner=Place(page,"显示设置确认栏",ContentX,FooterY-5,Width,57);
        var fill=Shape(banner,"紫绢底",SettingShape.Kind.FoldTab,0,0,Width,57,
            new Color(.23f,.145f,.29f),new Color(.16f,.10f,.21f));
        fill.raycastTarget=true;
        timer=Label(banner,"",25,0,480,55,19,MenuIvory,Body);
        UpdateTimer(DefaultSeconds);
        var keep=ActionStamp(banner,"保留",522,8,141,41,()=>Resolve(true),true);
        ActionStamp(banner,"还原",686,8,141,41,()=>Resolve(false));
        Focus(keep);
    }

    void UpdateTimer(float seconds)
        =>timer.text=GameSettings.Current.DisplayChanging?"正在切换画面尺寸…":
            $"保留 {Screen.width} × {Screen.height}？　{Mathf.CeilToInt(seconds)} 秒后还原";

    void Resolve(bool keep)
    {
        var settings=GameSettings.Current;
        if(keep && settings.DisplayChanging)return;
        if(!keep)before.Restore();
        else if(settings.width>0 && settings.height>0)
            settings.SetResolution(new Vector2Int(Screen.width,Screen.height));
        settings.Save();
        pending=false;
        Draw(rows.anchoredPosition.y);
    }

    void ResetCategory()
    {
        var items=Array.FindAll(GameSettings.Items,item=>item.category==category);
        void ResetPage(){
            foreach(var item in items)item.ResetValue();
            GameSettings.Current.Apply();
        }
        if(Array.Exists(items,item=>item.display && item.Changed)){
            PreviewDisplay(()=>{ResetPage();GameSettings.Current.ApplyDisplay();});
        }else{
            ResetPage();
            GameSettings.Current.Save();
            Draw();
        }
    }

    public void Back()
    {
        if(pending){Resolve(false);return;}
        resume();
    }

    public void Close()
    {
        if(pending)
        {
            before.Restore();
            GameSettings.Current.Save();
            pending=false;
        }
        gameObject.SetActive(false);
    }

    void Update()
    {
        if(!pending)return;
        float remain=deadline-Time.unscaledTime;
        if(remain<=0)Resolve(false);else UpdateTimer(remain);
    }
}
