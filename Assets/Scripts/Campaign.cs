using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.InputSystem.UI;
using UnityEngine.UI;

public sealed class Campaign : MonoBehaviour
{
    public GameObject lobby, chapters, stages, battle, complete;
    public Button mainStory, chapterOne, enterBattle, chapterBack, stageBack, battleBack, returnHome;
    public Text stepLabel, result;
    [SerializeField, InspectorName("第一关")] WorkshopLevel firstStage;
    FerryPuzzle level;
    GameObject mountain;
    Transform[] lobbyMarks;
    SettingsPanel settings;

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
        var font = Font.CreateDynamicFontFromOSFont(new[] { "Microsoft YaHei", "PingFang SC", "Noto Sans CJK SC", "Arial" }, 32);
        foreach (var label in GetComponentsInChildren<Text>(true))
        {
            label.font = font;
            if(label.text=="渡口初行") label.text=stage.title;
            if(label.text=="抵达山道尽头的渡口。") label.text=stage.description;
        }
        var theatre = gameObject.AddComponent<PaperTheatre>();
        theatre.Prepare(this,stage.title);
        settings=SettingsPanel.Create(transform.parent,font);
        var openSettings=Instantiate(chapterBack,lobby.transform,false);
        openSettings.name="设置";
        openSettings.GetComponentInChildren<Text>().text="设 置";
        openSettings.onClick.AddListener(()=>settings.Open(false));
        mountain=transform.parent.GetComponentInChildren<RawImage>(true).gameObject;
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
            bar.color=new Color(.96f,.89f,.75f);bar.raycastTarget=false;
        }
        level=new GameObject(stage.title).AddComponent<FerryPuzzle>();
        level.gameObject.SetActive(false);
        level.Build(stepLabel,stage,()=>{
            result.text=stage.title+"   ·   已抵达出口";
            Show(complete);
        },paused=>{
            if(paused)settings.Open(true,level.Resume,()=>Show(stages));
            else settings.Close();
        });
        mainStory.onClick.AddListener(() => theatre.OpenChart(() => Show(chapters)));
        chapterOne.onClick.AddListener(() => { theatre.ResetStage(); Show(stages); });
        chapterBack.onClick.AddListener(() => Show(lobby));
        stageBack.onClick.AddListener(() => Show(chapters));
        battleBack.onClick.AddListener(level.TogglePause);
        returnHome.GetComponentInChildren<Text>().text="返回选关";
        returnHome.onClick.AddListener(() => Show(stages));
        enterBattle.onClick.AddListener(() => { Show(battle); level.Begin(); });
        lobbyMarks=new[]{transform.Find("印"),transform.Find("岚"),transform.Find("山海行纪")};
        Show(lobby);
    }

    void Show(GameObject page)
    {
        settings.Close();
        lobby.SetActive(page == lobby);
        chapters.SetActive(page == chapters);
        stages.SetActive(page == stages);
        battle.SetActive(page == battle);
        complete.SetActive(page == complete);
        mountain.SetActive(page!=battle);
        foreach(var mark in lobbyMarks)if(mark)mark.gameObject.SetActive(page!=battle);
        level.gameObject.SetActive(page==battle);
    }
}
