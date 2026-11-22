using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.InputSystem;
using UnityEngine.UI;

// The one full-screen settings page: mulberry paper over the mountain painting, a mineral-green
// spine with the categories, and the rows of the chosen category. The lobby shows every row;
// inside a level the lobby-only rows are left out.
public sealed class SettingsPanel : MonoBehaviour
{
    static readonly Color Ink=new Color(.13f,.17f,.17f);
    static readonly Color Faint=new Color(.13f,.17f,.17f,.16f);
    static readonly Color Paper=new Color(.95f,.89f,.75f);
    static readonly Color Teal=new Color(.13f,.36f,.38f);
    static readonly Color Cinnabar=new Color(.72f,.33f,.23f);
    static readonly Color Gold=new Color(.80f,.62f,.33f);
    const float RowHeight=58,Control=300;
    const float ViewHeight=470;
    const string Controls="操作";
    static readonly string[] Sections={"鼠标","键盘"};
    // Fixed bindings: section, action, then the key caps.
    static readonly string[][] Keys={
        new[]{"鼠标","视角","移动鼠标"},
        new[]{"鼠标","青鸾门 / 装置","左键"},
        new[]{"鼠标","朱雀门","右键"},
        new[]{"鼠标","切换道具","滚轮"},
        new[]{"键盘","移动","W","A","S","D"},
        new[]{"键盘","跳跃","空格"},
        new[]{"键盘","滑翔","空中","空格"},
        new[]{"键盘","切换道具","1","2","3"},
        new[]{"键盘","重试","R"},
        new[]{"键盘","重开本关","T"},
        new[]{"键盘","设置","Esc"},
    };
    RectTransform view;
    RawImage thumb;
    Font body,brush;
    Texture2D grain;
    CanvasGroup group;
    RectTransform page,rows;
    Text heading,note;
    Action resume;
    bool inLevel;
    string category;
    readonly List<string> categories=new List<string>();
    readonly List<Text> tabLabels=new List<Text>();
    readonly List<RawImage> tabMarks=new List<RawImage>();

    public static SettingsPanel Create(Transform canvas,Font font)
    {
        var panel=new GameObject("设置",typeof(RectTransform),typeof(CanvasGroup)).AddComponent<SettingsPanel>();
        panel.Build((RectTransform)panel.transform,canvas,font);
        panel.gameObject.SetActive(false);
        return panel;
    }

    static void Fill(RectTransform rect,Transform parent)
    {
        rect.SetParent(parent,false);
        rect.anchorMin=Vector2.zero;rect.anchorMax=Vector2.one;
        rect.offsetMin=rect.offsetMax=Vector2.zero;
    }

    void Build(RectTransform root,Transform canvas,Font font)
    {
        Fill(root,canvas);
        group=GetComponent<CanvasGroup>();
        body=font;
        brush=Font.CreateDynamicFontFromOSFont(new[]{"KaiTi","STKaiti","Kaiti SC","FangSong","Songti SC","Microsoft YaHei","PingFang SC"},48);
        // Coarse pigment grain, multiplied into every flat colour field.
        const int size=128;
        grain=new Texture2D(size,size,TextureFormat.RGBA32,false){name="岩彩颗粒",wrapMode=TextureWrapMode.Repeat,filterMode=FilterMode.Bilinear};
        var pixels=new Color32[size*size];
        var random=new System.Random(7);
        for(int i=0;i<pixels.Length;i++){
            double roll=random.NextDouble();
            byte v=(byte)(roll<.035?150+random.Next(40):roll>.975?255:214+random.Next(34));
            pixels[i]=new Color32(v,v,v,255);
        }
        grain.SetPixels32(pixels);grain.Apply(false,true);

        var painting=new GameObject("山景",typeof(RectTransform),typeof(RawImage)).GetComponent<RawImage>();
        painting.rectTransform.SetParent(root,false);
        painting.texture=Resources.Load<Texture2D>("Mountain");
        var fit=painting.gameObject.AddComponent<AspectRatioFitter>();
        fit.aspectMode=AspectRatioFitter.AspectMode.EnvelopeParent;
        fit.aspectRatio=(float)painting.texture.width/painting.texture.height;
        var sheet=new GameObject("桑皮纸",typeof(RectTransform),typeof(RawImage)).GetComponent<RawImage>();
        Fill(sheet.rectTransform,root);
        sheet.texture=Resources.Load<Texture2D>("Mineral/Mulberry");
        sheet.uvRect=new Rect(0,0,2.4f,1.35f);
        sheet.color=new Color(Paper.r,Paper.g,Paper.b,.87f);

        page=new GameObject("版心",typeof(RectTransform)).GetComponent<RectTransform>();
        page.SetParent(root,false);
        page.anchorMin=page.anchorMax=page.pivot=Vector2.one*.5f;
        page.sizeDelta=new Vector2(1280,720);
    }

    static RectTransform Place(Transform parent,string name,float x,float y,float w,float h)
    {
        var rect=new GameObject(name,typeof(RectTransform)).GetComponent<RectTransform>();
        rect.SetParent(parent,false);
        rect.anchorMin=rect.anchorMax=rect.pivot=Vector2.up;
        rect.anchoredPosition=new Vector2(x,-y);
        rect.sizeDelta=new Vector2(w,h);
        return rect;
    }

    RawImage Box(Transform parent,string name,float x,float y,float w,float h,Color color,bool pigment=false)
    {
        var image=Place(parent,name,x,y,w,h).gameObject.AddComponent<RawImage>();
        image.color=color;image.raycastTarget=false;
        if(pigment){image.texture=grain;image.uvRect=new Rect(x/150f,y/150f,w/150f,h/150f);}
        return image;
    }

    Text Write(Transform parent,string value,Font font,int size,Color color,float x,float y,float w,float h,
        TextAnchor anchor=TextAnchor.MiddleLeft)
    {
        var text=Place(parent,value,x,y,w,h).gameObject.AddComponent<Text>();
        text.font=font;text.fontSize=size;text.color=color;text.alignment=anchor;
        text.horizontalOverflow=HorizontalWrapMode.Overflow;text.verticalOverflow=VerticalWrapMode.Overflow;
        text.supportRichText=false;text.raycastTarget=false;text.text=value;
        return text;
    }

    static void Press(Graphic target,Action act)
    {
        target.raycastTarget=true;
        var button=target.gameObject.AddComponent<Button>();
        button.targetGraphic=target;
        button.navigation=new Navigation{mode=Navigation.Mode.None};
        button.onClick.AddListener(()=>act());
    }

    void Stamp(string caption,float x,float y,float w,Color color,Action act)
    {
        var plate=Box(page,caption,x,y,w,48,color,true);
        Write(plate.transform,caption,brush,24,Paper,0,0,w,48,TextAnchor.MiddleCenter);
        Press(plate,act);
    }

    // onResume closes the page from the caller's side (a level unpauses); without it the page closes itself.
    public void Open(bool level,Action onResume=null,Action onLeave=null)
    {
        inLevel=level;resume=onResume;
        for(int i=page.childCount-1;i>=0;i--)Destroy(page.GetChild(i).gameObject);
        rows=view=null;thumb=null;
        categories.Clear();tabLabels.Clear();tabMarks.Clear();
        foreach(var item in GameSettings.Items)
            if(!(level && item.lobbyOnly) && !categories.Contains(item.category))categories.Add(item.category);
        if(!categories.Contains(category))category=categories[0];

        // The spine runs off the left and both vertical edges whatever the aspect ratio.
        var spine=Box(page,"石青书脊",0,0,0,0,Teal,true);
        spine.rectTransform.anchorMin=spine.rectTransform.anchorMax=new Vector2(0,.5f);
        spine.rectTransform.pivot=new Vector2(1,.5f);
        spine.rectTransform.anchoredPosition=new Vector2(292,0);
        spine.rectTransform.sizeDelta=new Vector2(3300,6000);
        spine.uvRect=new Rect(0,0,22,40);
        spine.raycastTarget=true;
        var edge=Box(page,"泥金线",0,0,0,0,Gold);
        edge.rectTransform.anchorMin=edge.rectTransform.anchorMax=new Vector2(0,.5f);
        edge.rectTransform.pivot=new Vector2(0,.5f);
        edge.rectTransform.anchoredPosition=new Vector2(298,0);
        edge.rectTransform.sizeDelta=new Vector2(2,6000);

        Write(page,"设\n置",brush,76,Paper,64,52,100,190,TextAnchor.UpperLeft).lineSpacing=.95f;
        var seal=Box(page,"印",172,66,34,34,Cinnabar,true);
        Write(seal.transform,"岚",brush,22,Paper,0,0,34,34,TextAnchor.MiddleCenter);
        for(int i=0;i<categories.Count;i++){
            string name=categories[i];
            float y=300+i*60;
            var hit=Box(page,name+"页签",40,y,230,52,Color.clear);
            tabMarks.Add(Box(page,name+"朱线",48,y+13,4,26,Cinnabar));
            tabLabels.Add(Write(page,name,brush,30,Paper,72,y,180,52));
            Press(hit,()=>{category=name;ShowRows();});
        }
        var cloud=Place(page,"如意",70,556,84,126).gameObject.AddComponent<RawImage>();
        cloud.texture=Resources.Load<Texture2D>("Mineral/Ruyi");
        cloud.raycastTarget=false;
        cloud.rectTransform.localRotation=Quaternion.Euler(0,0,4);

        heading=Write(page,category,brush,46,Ink,360,54,300,60);
        Box(page,"朱砂短线",362,120,56,4,Cinnabar,true);
        note=Write(page,"",body,15,new Color(Ink.r,Ink.g,Ink.b,.55f),640,72,580,30,TextAnchor.MiddleRight);
        if(level){
            Stamp("返回选关",880,640,160,Teal,()=>{if(onLeave!=null)onLeave();else Close();});
            Stamp("继 续",1060,640,160,Cinnabar,Resume);
        }else Stamp("返 回",1060,640,160,Cinnabar,Resume);
        ShowRows();
        transform.SetAsLastSibling();
        group.alpha=0;
        gameObject.SetActive(true);
    }

    void ShowRows()
    {
        // Choosing an option rebuilds the rows; stay where the list was scrolled to.
        float scrolled=rows && heading.text==category?rows.anchoredPosition.y:0;
        if(view){view.gameObject.SetActive(false);Destroy(view.gameObject);}
        view=Place(page,"设置项",340,150,880,ViewHeight);
        view.gameObject.AddComponent<RawImage>().color=Color.clear;
        view.gameObject.AddComponent<RectMask2D>();
        rows=Place(view,"行",0,0,880,0);
        heading.text=category;
        for(int i=0;i<categories.Count;i++){
            bool chosen=categories[i]==category;
            tabMarks[i].enabled=chosen;
            tabLabels[i].color=new Color(Paper.r,Paper.g,Paper.b,chosen?1:.5f);
        }
        bool hidden=false;
        float y=0;
        void Rows(string section)
        {
            foreach(var item in GameSettings.Items){
                if(item.category!=category || item.section!=section)continue;
                if(inLevel && item.lobbyOnly){hidden=true;continue;}
                Write(rows,item.label,brush,26,Ink,20,y,260,RowHeight);
                Box(rows,"界线",0,y+RowHeight-1,880,1,Faint);
                if(item.kind==SettingItem.Kind.Slider)Slide(item,y);
                else if(item.kind==SettingItem.Kind.Options)Chips(item,y);
                else Stepper(item,y);
                y+=RowHeight;
            }
        }
        if(category==Controls)foreach(string section in Sections){
            if(y>0)y+=18;
            Box(rows,section+"朱点",20,y+17,8,8,Cinnabar).rectTransform.localRotation=Quaternion.Euler(0,0,45);
            Write(rows,section,brush,22,Cinnabar,40,y,200,42);
            Box(rows,"界线",0,y+41,880,1,new Color(Cinnabar.r,Cinnabar.g,Cinnabar.b,.5f));
            y+=42;
            Rows(section);
            foreach(var key in Keys)if(key[0]==section){KeyRow(key,y);y+=RowHeight;}
        }
        else Rows(null);
        note.text=hidden?"分辨率、显示模式和阴影精度在大厅的设置中调整":"";

        rows.sizeDelta=new Vector2(880,y);
        var scroll=view.gameObject.AddComponent<ScrollRect>();
        scroll.viewport=view;scroll.content=rows;
        scroll.horizontal=false;scroll.movementType=ScrollRect.MovementType.Clamped;
        scroll.scrollSensitivity=36;
        rows.anchoredPosition=new Vector2(0,Mathf.Clamp(scrolled,0,Mathf.Max(0,y-ViewHeight)));
        if(!thumb){
            Box(page,"滚动槽",1234,150,2,ViewHeight,Faint);
            thumb=Box(page,"滚动标",1232,150,6,60,Teal,true);
        }
        MoveThumb();
    }

    void MoveThumb()
    {
        if(!thumb || !rows)return;
        float extra=rows.sizeDelta.y-ViewHeight;
        thumb.enabled=extra>1;
        if(!thumb.enabled)return;
        float length=ViewHeight*ViewHeight/rows.sizeDelta.y;
        thumb.rectTransform.sizeDelta=new Vector2(6,length);
        thumb.rectTransform.anchoredPosition=new Vector2(1232,-150-(ViewHeight-length)*Mathf.Clamp01(rows.anchoredPosition.y/extra));
    }

    // One fixed binding: the action and its key caps.
    void KeyRow(string[] key,float y)
    {
        Write(rows,key[1],brush,26,Ink,20,y,260,RowHeight);
        Box(rows,"界线",0,y+RowHeight-1,880,1,Faint);
        float x=Control;
        for(int k=2;k<key.Length;k++){
            string cap=key[k];
            float width=24;
            foreach(char c in cap)width+=c<128?12:20;
            var plate=Box(rows,cap,x,y+12,width,34,new Color(Ink.r,Ink.g,Ink.b,.1f),true);
            Box(plate.transform,"键底",0,32,width,2,new Color(Ink.r,Ink.g,Ink.b,.45f));
            Write(plate.transform,cap,body,18,Ink,0,0,width,32,TextAnchor.MiddleCenter);
            x+=width+6;
        }
    }

    void Slide(SettingItem item,float y)
    {
        const float length=360;
        var root=Place(rows,item.label+"滑条",Control,y,length,RowHeight);
        root.gameObject.AddComponent<RawImage>().color=Color.clear;
        Box(root,"轨",0,RowHeight*.5f-1,length,2,new Color(Ink.r,Ink.g,Ink.b,.3f));
        var value=Write(rows,item.Value,body,20,Ink,Control+length+24,y,120,RowHeight);

        RectTransform Inset(string name)
        {
            var area=new GameObject(name,typeof(RectTransform)).GetComponent<RectTransform>();
            Fill(area,root);
            area.offsetMin=new Vector2(9,0);area.offsetMax=new Vector2(-9,0);
            return area;
        }
        var fill=new GameObject("石青",typeof(RectTransform),typeof(RawImage)).GetComponent<RawImage>();
        fill.rectTransform.SetParent(Inset("填充区"),false);
        fill.rectTransform.anchorMin=new Vector2(0,.5f);fill.rectTransform.anchorMax=new Vector2(0,.5f);
        fill.rectTransform.sizeDelta=new Vector2(0,4);
        fill.color=Teal;fill.raycastTarget=false;
        var handle=new GameObject("游标",typeof(RectTransform)).GetComponent<RectTransform>();
        handle.SetParent(Inset("游标区"),false);
        handle.anchorMin=handle.anchorMax=new Vector2(0,.5f);
        handle.sizeDelta=new Vector2(22,22);
        var gem=new GameObject("朱砂",typeof(RectTransform),typeof(RawImage)).GetComponent<RawImage>();
        gem.rectTransform.SetParent(handle,false);
        gem.rectTransform.sizeDelta=new Vector2(14,14);
        gem.rectTransform.localRotation=Quaternion.Euler(0,0,45);
        gem.color=Cinnabar;gem.raycastTarget=false;

        var slider=root.gameObject.AddComponent<Slider>();
        slider.navigation=new Navigation{mode=Navigation.Mode.None};
        slider.fillRect=fill.rectTransform;slider.handleRect=handle;slider.targetGraphic=gem;
        slider.direction=Slider.Direction.LeftToRight;
        slider.wholeNumbers=true;slider.minValue=0;slider.maxValue=item.steps;
        slider.SetValueWithoutNotify(item.Index);
        slider.onValueChanged.AddListener(v=>{item.Choose(Mathf.RoundToInt(v));value.text=item.Value;});
    }

    void Chips(SettingItem item,float y)
    {
        float x=Control;
        int chosen=item.Index;
        for(int i=0;i<item.names.Length;i++){
            string name=item.names[i];
            float width=30;
            foreach(char c in name)width+=c<128?12:21;
            var chip=Box(rows,item.label+name,x,y+11,width,36,i==chosen?Teal:Color.clear,i==chosen);
            Write(chip.transform,name,body,19,i==chosen?Paper:new Color(Ink.r,Ink.g,Ink.b,.62f),0,0,width,36,TextAnchor.MiddleCenter);
            int pick=i;
            Press(chip,()=>{item.Choose(pick);ShowRows();});
            x+=width+6;
        }
    }

    void Stepper(SettingItem item,float y)
    {
        var value=Write(rows,item.Value,body,20,Ink,Control+44,y,200,RowHeight,TextAnchor.MiddleCenter);
        for(int side=-1;side<=1;side+=2){
            int direction=side;
            var arrow=Box(rows,item.label+(side<0?"上一项":"下一项"),side<0?Control:Control+252,y+11,36,36,Teal,true);
            Write(arrow.transform,side<0?"‹":"›",body,24,Paper,0,-2,36,36,TextAnchor.MiddleCenter);
            Press(arrow,()=>{item.Choose(direction);value.text=item.Value;});
        }
    }

    void Resume()
    {
        if(resume!=null)resume();
        else Close();
    }

    public void Close()
    {
        if(EventSystem.current)EventSystem.current.SetSelectedGameObject(null);
        gameObject.SetActive(false);
    }

    void Update()
    {
        group.alpha=Mathf.MoveTowards(group.alpha,1,Time.unscaledDeltaTime*7);
        MoveThumb();
        // Inside a level the level itself owns Esc.
        if(!inLevel && Keyboard.current!=null && Keyboard.current.escapeKey.wasPressedThisFrame)Close();
    }

    void OnDestroy(){if(grain)Destroy(grain);}
}
