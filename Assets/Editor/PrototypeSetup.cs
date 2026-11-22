using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

public static class PrototypeSetup
{
    static readonly Color Ink = new Color(.09f,.19f,.20f,.96f);
    static readonly Color Paper = new Color(.96f,.89f,.75f);
    static readonly Color Seal = new Color(.59f,.24f,.17f);

    static RectTransform Rect(Transform parent,string name,float x,float y,float w,float h)
    {
        var rect = new GameObject(name,typeof(RectTransform)).GetComponent<RectTransform>();
        rect.SetParent(parent,false);
        rect.anchorMin = rect.anchorMax = rect.pivot = Vector2.up;
        rect.anchoredPosition = new Vector2(x,-y);
        rect.sizeDelta = new Vector2(w,h);
        return rect;
    }

    static Text Label(Transform parent,string value,int size,float x,float y,float w,float h)
    {
        var text = Rect(parent,value,x,y,w,h).gameObject.AddComponent<Text>();
        text.font = Resources.GetBuiltinResource<Font>("LegacyRuntime.ttf");
        text.text = value;
        text.fontSize = size;
        text.color = Paper;
        text.alignment = TextAnchor.MiddleLeft;
        text.raycastTarget = false;
        return text;
    }

    static Image Box(Transform parent,string name,Color color,float x,float y,float w,float h)
    {
        var image = Rect(parent,name,x,y,w,h).gameObject.AddComponent<Image>();
        image.color = color;
        return image;
    }

    static Button Action(Transform parent,string caption,float x,float y,float w,float h=55)
    {
        var image = Box(parent,caption,Seal,x,y,w,h);
        var button = image.gameObject.AddComponent<Button>();
        button.targetGraphic = image;
        Label(image.transform,caption,23,0,0,w,h).alignment = TextAnchor.MiddleCenter;
        return button;
    }

    static GameObject Card(Transform root,string name,string chapter,string title,string detail,string action,out Button button,out Text description)
    {
        var page = Rect(root,name,0,0,1280,720);
        Box(page,"墨底",Ink,56,296,495,330);
        Label(page,chapter,20,92,327,425,36);
        Label(page,title,64,89,374,435,84);
        description = Label(page,detail,23,92,463,425,45);
        button = Action(page,action,92,537,310);
        return page.gameObject;
    }

    public static void Create()
    {
        var scene = EditorSceneManager.NewScene(NewSceneSetup.EmptyScene);
        var camera = new GameObject("Camera").AddComponent<Camera>();
        camera.clearFlags = CameraClearFlags.SolidColor;
        camera.backgroundColor = Color.black;
        new GameObject("EventSystem",typeof(EventSystem),typeof(StandaloneInputModule));
        var canvas = new GameObject("Campaign Canvas",typeof(Canvas),typeof(CanvasScaler),typeof(GraphicRaycaster));
        canvas.GetComponent<Canvas>().renderMode = RenderMode.ScreenSpaceOverlay;
        var scaler = canvas.GetComponent<CanvasScaler>();
        scaler.uiScaleMode = CanvasScaler.ScaleMode.ScaleWithScreenSize;
        scaler.referenceResolution = new Vector2(1280,720);
        scaler.screenMatchMode = CanvasScaler.ScreenMatchMode.Expand;
        var background = Rect(canvas.transform,"山景",0,0,1280,720);
        background.anchorMin = background.anchorMax = background.pivot = Vector2.one*.5f;
        var image = background.gameObject.AddComponent<RawImage>();
        image.texture = Resources.Load<Texture2D>("Mountain");
        image.raycastTarget = false;
        var fit = background.gameObject.AddComponent<AspectRatioFitter>();
        fit.aspectRatio = (float)image.texture.width/image.texture.height;
        fit.aspectMode = AspectRatioFitter.AspectMode.EnvelopeParent;
        var root = Rect(canvas.transform,"绘卷",0,0,1280,720);
        root.anchorMin = root.anchorMax = root.pivot = Vector2.one*.5f;
        var c = root.gameObject.AddComponent<Campaign>();
        Box(root,"印",Seal,56,46,38,38);
        Label(root,"岚",26,61,46,34,38);
        Label(root,"山海行纪",24,109,46,300,38).color = Ink;
        c.lobby = Card(root,"大厅","山 海 绘 卷","云岫行记","群峰入画，循古道而行。","主线    →",out c.mainStory,out _);
        c.chapters = Card(root,"主线","主 线","章节绘卷","选择章节","第一章   ·   青岚渡",out c.chapterOne,out _);
        c.stages = Card(root,"选关","第 一 章    /    青 岚 渡","渡口初行","抵达山道尽头的渡口。","1-1    进入对局   →",out c.enterBattle,out _);
        c.complete = Card(root,"结算","青 岚 渡    /    1-1","已抵渡口","关卡完成","返回大厅",out c.returnHome,out c.result);
        var battle = Rect(root,"对局",0,0,1280,720);
        c.battle = battle.gameObject;
        c.stepLabel = Label(battle,"穿过纸门",22,42,595,1050,54);
        c.chapterBack = Action(c.chapters.transform,"←   返回",1030,44,194);
        c.stageBack = Action(c.stages.transform,"←   返回",1030,44,194);
        c.battleBack = Action(battle,"←   返回",1030,44,194);
        foreach (var page in new[]{c.chapters,c.stages,c.battle,c.complete}) page.SetActive(false);
        const string path = "Assets/Scenes/Campaign.unity";
        EditorSceneManager.SaveScene(scene,path);
        EditorBuildSettings.scenes = new[]{ new EditorBuildSettingsScene(path,true) };
    }
}
