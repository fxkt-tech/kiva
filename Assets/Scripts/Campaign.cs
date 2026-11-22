using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.InputSystem.UI;
using UnityEngine.UI;
using static NightUi;

// Everything outside a level: lobby, chapter choice, stage choice and the result page, all built
// here in the night-mountain style. The scene only supplies the canvas and the in-level HUD.
public sealed class Campaign : MonoBehaviour
{
    const string Chapter="第一章 · 青岚渡",StageNumber="1-1";
    public GameObject battle;
    public Button battleBack;
    public Text stepLabel;
    [SerializeField, InspectorName("第一关")] WorkshopLevel firstStage;
    FerryPuzzle level;
    SettingsPanel settings;
    GameObject outside;
    CanvasGroup fade;
    RectTransform lobby,chapters,stages,complete;

    void Awake()
    {
        // The saved scene still carries the Input Manager UI module.
        var legacy=FindAnyObjectByType<StandaloneInputModule>();
        if(legacy){
            var events=legacy.gameObject;
            DestroyImmediate(legacy);
            events.AddComponent<InputSystemUIInputModule>().AssignDefaultActions();
        }
        GameSettings.Current.Apply();
        GameSettings.Current.ApplyDisplay();
        var stage=firstStage?firstStage:Resources.Load<WorkshopLevel>("Levels/WindWell");
        // Older scenes still hold the card pages and the mountain photograph; only the HUD is used.
        var canvas=transform.parent;
        foreach(Transform child in canvas)if(child!=transform)child.gameObject.SetActive(false);
        foreach(Transform child in transform)if(child!=battle.transform)child.gameObject.SetActive(false);
        stepLabel.font=Body;

        outside=new GameObject("局外",typeof(RectTransform),typeof(CanvasGroup));
        Fill((RectTransform)outside.transform,canvas);
        fade=outside.GetComponent<CanvasGroup>();
        Backdrop(outside.transform);
        settings=SettingsPanel.Create(canvas);
        BuildLobby();
        BuildChapters();
        BuildStages(stage);
        var result=BuildComplete();

        foreach(Transform child in battle.transform)
            child.gameObject.SetActive(child==battleBack.transform || child==stepLabel.transform);
        var battleRect=(RectTransform)battle.transform;
        battleRect.anchorMin=Vector2.zero;battleRect.anchorMax=Vector2.one;
        battleRect.offsetMin=battleRect.offsetMax=Vector2.zero;
        battleBack.image.color=new Color(.12f,.14f,.13f,.36f);
        // In battle the back button is a three-bar menu icon in the top-left corner.
        var backRect=(RectTransform)battleBack.transform;
        backRect.anchorMin=backRect.anchorMax=backRect.pivot=Vector2.up;
        backRect.anchoredPosition=new Vector2(32,-24);
        backRect.sizeDelta=new Vector2(52,52);
        battleBack.GetComponentInChildren<Text>().gameObject.SetActive(false);
        for(int i=-1;i<=1;i++){
            var bar=new GameObject("横",typeof(RectTransform),typeof(Image)).GetComponent<Image>();
            bar.rectTransform.SetParent(backRect,false);
            bar.rectTransform.sizeDelta=new Vector2(26,3);
            bar.rectTransform.anchoredPosition=new Vector2(0,i*9);
            bar.color=White;bar.raycastTarget=false;
        }
        level=new GameObject(stage.title).AddComponent<FerryPuzzle>();
        level.gameObject.SetActive(false);
        level.Build(stepLabel,stage,()=>{
            result.text=StageNumber+"　"+stage.title+"　·　已抵达出口";
            Show(complete);
        },paused=>{
            if(paused)settings.Open(true,level.Resume,()=>Show(stages));
            else settings.Close();
        });
        battleBack.onClick.AddListener(level.TogglePause);
        Show(lobby);
    }

    // The game's name down the right edge; the entries as a short list on the left.
    void BuildLobby()
    {
        lobby=Page(outside.transform,"大厅");
        Gem(lobby,58,50,40,true);
        Write(lobby,"山 海 行 纪",Hand,28,White,92,28,300,44,TextAnchor.MiddleLeft,true);
        Entry(lobby,"主线",Chapter,236,true,()=>Show(chapters));
        Entry(lobby,"设置",null,330,false,()=>settings.Open(false));
        Hairline(lobby,44,612,560,false);
        Write(lobby,"群峰入画，循古道而行。",Hand,24,new Color(White.r,White.g,White.b,.88f),64,628,520,40);
        Banner(lobby,"云岫行记");
    }

    // One menu line: a gem, a name and, on the main entry, what it leads to. It lights up under the pointer.
    void Entry(Transform page,string label,string caption,float y,bool main,System.Action act)
    {
        var hit=Box(page,label,44,y,360,72,Color.clear);
        var band=Shape(page,"带",SettingShape.Kind.Bar,72,y+10,330,52,new Color(.16f,.59f,.8f,.75f),new Color(.16f,.59f,.8f,0));
        Shape(band.transform,"线",SettingShape.Kind.Bar,0,0,330,1,new Color(White.r,White.g,White.b,.5f),new Color(1,1,1,0));
        Shape(band.transform,"线",SettingShape.Kind.Bar,0,51,330,1,new Color(White.r,White.g,White.b,.5f),new Color(1,1,1,0));
        band.gameObject.SetActive(main);
        Gem(page,71,y+36,main?66:48,main);
        var name=Write(page,Spaced(label),Hand,main?38:30,new Color(White.r,White.g,White.b,main?1:.7f),122,y,160,72,TextAnchor.MiddleLeft,main);
        if(caption!=null)Write(page,caption,Body,15,new Color(White.r,White.g,White.b,.75f),236,y+6,200,72);
        Press(hit,act);
        var hover=hit.gameObject.AddComponent<HoverWatch>();
        hover.entered=()=>{band.gameObject.SetActive(true);name.color=White;};
        hover.left=()=>{band.gameObject.SetActive(main);name.color=new Color(White.r,White.g,White.b,main?1:.7f);};
    }

    // Chapters are nodes standing in the landscape.
    void BuildChapters()
    {
        chapters=Page(outside.transform,"章节");
        Back(chapters,"主线",()=>Show(lobby));
        Diamond(chapters,"一",420,330,132,56,true,()=>Show(stages));
        Write(chapters,"第 一 章",Hand,30,White,270,412,300,44,TextAnchor.MiddleCenter,true);
        Write(chapters,"青岚渡",Hand,24,Blush,270,452,300,36,TextAnchor.MiddleCenter);
        Hairline(chapters,44,612,560,false);
        Write(chapters,"选择章节",Hand,24,new Color(White.r,White.g,White.b,.88f),64,628,520,40);
        Banner(chapters,"主线");
    }

    // The stage list on the left, the chosen stage between two hairlines, and the departure button.
    void BuildStages(WorkshopLevel stage)
    {
        stages=Page(outside.transform,"选关");
        Back(stages,Chapter,()=>Show(chapters));
        var band=Shape(stages,"带",SettingShape.Kind.Bar,72,134,232,44,new Color(.16f,.59f,.8f,.75f),new Color(.16f,.59f,.8f,0));
        Shape(band.transform,"线",SettingShape.Kind.Bar,0,0,232,1,new Color(White.r,White.g,White.b,.5f),new Color(1,1,1,0));
        Shape(band.transform,"线",SettingShape.Kind.Bar,0,43,232,1,new Color(White.r,White.g,White.b,.5f),new Color(1,1,1,0));
        Gem(stages,71,156,62,true);
        Write(stages,StageNumber,Hand,32,White,116,128,170,56,TextAnchor.MiddleLeft,true);

        Shape(stages,"底",SettingShape.Kind.Hair,304,112,742,450,new Color(.024f,.12f,.2f,.6f),Color.clear).fade=.06f;
        Hairline(stages,284,112,782,true);
        Hairline(stages,284,561,782,false);
        Shape(stages,"标",SettingShape.Kind.Diamond,342,163,13,13,Blush,Blush);
        Write(stages,Chapter+"　"+StageNumber,Hand,22,Blush,366,150,500,40);
        Write(stages,stage.title,Hand,64,White,340,200,640,90,TextAnchor.MiddleLeft,true);
        Shape(stages,"线",SettingShape.Kind.Bar,340,304,560,1,new Color(Blush.r,Blush.g,Blush.b,.7f),new Color(Blush.r,Blush.g,Blush.b,0));
        var about=Write(stages,stage.description,Hand,26,new Color(White.r,White.g,White.b,.92f),340,326,640,120,TextAnchor.UpperLeft);
        about.horizontalOverflow=HorizontalWrapMode.Wrap;about.lineSpacing=1.3f;
        Banner(stages,stage.title);
        Diamond(stages,"出发",1170,620,124,31,true,()=>{Show(null);level.Begin();});
    }

    Text BuildComplete()
    {
        complete=Page(outside.transform,"结算");
        Hairline(complete,250,236,780,true);
        Write(complete,"关 卡 完 成",NightUi.Display,84,Color.white,240,256,800,120,TextAnchor.MiddleCenter,true);
        var result=Write(complete,"",Hand,28,Blush,240,384,800,48,TextAnchor.MiddleCenter);
        Hairline(complete,250,448,780,false);
        Diamond(complete,"选关",640,560,124,31,true,()=>Show(stages));
        return result;
    }

    // A null page is the level itself.
    void Show(RectTransform page)
    {
        settings.Close();
        foreach(var each in new[]{lobby,chapters,stages,complete})each.gameObject.SetActive(each==page);
        outside.SetActive(page);
        fade.alpha=0;
        battle.SetActive(!page);
        level.gameObject.SetActive(!page);
    }

    void Update()
    {
        if(fade.alpha<1)fade.alpha=Mathf.MoveTowards(fade.alpha,1,Time.unscaledDeltaTime*6);
    }
}
