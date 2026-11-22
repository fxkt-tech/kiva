using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.InputSystem.UI;
using UnityEngine.UI;

// Creates the campaign scene: a camera, an event system, a canvas and the in-level HUD.
// Every page outside a level is built by Campaign at run time.
public static class PrototypeSetup
{
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
        text.color = Color.white;
        text.alignment = TextAnchor.MiddleLeft;
        text.raycastTarget = false;
        return text;
    }

    public static void Create()
    {
        var scene = EditorSceneManager.NewScene(NewSceneSetup.EmptyScene);
        var camera = new GameObject("Camera").AddComponent<Camera>();
        camera.clearFlags = CameraClearFlags.SolidColor;
        camera.backgroundColor = Color.black;
        new GameObject("EventSystem",typeof(EventSystem),typeof(InputSystemUIInputModule));
        var canvas = new GameObject("Campaign Canvas",typeof(Canvas),typeof(CanvasScaler),typeof(GraphicRaycaster));
        canvas.GetComponent<Canvas>().renderMode = RenderMode.ScreenSpaceOverlay;
        var scaler = canvas.GetComponent<CanvasScaler>();
        scaler.uiScaleMode = CanvasScaler.ScaleMode.ScaleWithScreenSize;
        scaler.referenceResolution = new Vector2(1280,720);
        scaler.screenMatchMode = CanvasScaler.ScreenMatchMode.Expand;
        var root = Rect(canvas.transform,"绘卷",0,0,1280,720);
        root.anchorMin = root.anchorMax = root.pivot = Vector2.one*.5f;
        var c = root.gameObject.AddComponent<Campaign>();
        var battle = Rect(root,"对局",0,0,1280,720);
        c.battle = battle.gameObject;
        c.stepLabel = Label(battle,"穿过纸门",22,42,595,1050,54);
        var back = Rect(battle,"菜单",32,24,52,52).gameObject.AddComponent<Image>();
        c.battleBack = back.gameObject.AddComponent<Button>();
        c.battleBack.targetGraphic = back;
        Label(back.transform,"菜单",12,0,0,52,52);
        battle.gameObject.SetActive(false);
        const string path = "Assets/Scenes/Campaign.unity";
        EditorSceneManager.SaveScene(scene,path);
        EditorBuildSettings.scenes = new[]{ new EditorBuildSettingsScene(path,true) };
    }
}
