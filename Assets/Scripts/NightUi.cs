using System;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

// Paper, ink and typography shared by the saved campaign pages and the settings folio.
public static class NightUi
{
    public static readonly Color White=Color.white;
    public static readonly Color Ink=new Color(.21f,.17f,.24f);
    public static readonly Color Blue=new Color(.37f,.27f,.41f);
    public static readonly Color Paper=new Color(.97f,.94f,.90f);
    public static readonly Color Mist=new Color(.88f,.83f,.88f);
    public static readonly Color Line=new Color(.58f,.48f,.59f,.41f);
    static Font body,hand,display;
    public static Font Body=>body?body:body=Resources.Load<Font>("NightUi/Body");
    public static Font Hand=>hand?hand:hand=Resources.Load<Font>("NightUi/Hand");
    public static Font Display=>display?display:display=Resources.Load<Font>("NightUi/Display");

    public static void Fill(RectTransform r,Transform parent){
        r.SetParent(parent,false);r.anchorMin=Vector2.zero;r.anchorMax=Vector2.one;r.offsetMin=r.offsetMax=Vector2.zero;
    }
    public static RectTransform Page(Transform parent,string name){
        var r=new GameObject(name,typeof(RectTransform)).GetComponent<RectTransform>();
        r.SetParent(parent,false);r.sizeDelta=new Vector2(1280,720);return r;
    }
    public static RectTransform Place(Transform parent,string name,float x,float y,float w,float h){
        var r=new GameObject(name,typeof(RectTransform)).GetComponent<RectTransform>();
        r.SetParent(parent,false);r.anchorMin=r.anchorMax=r.pivot=Vector2.up;r.anchoredPosition=new Vector2(x,-y);r.sizeDelta=new Vector2(w,h);return r;
    }
    public static Image Panel(Transform parent,string name,float x,float y,float w,float h,Color color){
        var image=Place(parent,name,x,y,w,h).gameObject.AddComponent<Image>();
        image.color=color;image.raycastTarget=false;return image;
    }
    public static Text Label(Transform parent,string content,float x,float y,float w,float h,int size,Color ink,Font font=null,TextAnchor align=TextAnchor.MiddleLeft){
        var t=Place(parent,content,x,y,w,h).gameObject.AddComponent<Text>();
        t.font=font?font:Body;t.fontSize=size;t.color=ink;t.text=content;t.alignment=align;
        t.horizontalOverflow=HorizontalWrapMode.Wrap;t.verticalOverflow=VerticalWrapMode.Truncate;t.supportRichText=false;t.raycastTarget=false;return t;
    }
    public static SettingShape Shape(Transform parent,string name,SettingShape.Kind kind,float x,float y,float w,float h,Color first,Color second){
        var s=Place(parent,name,x,y,w,h).gameObject.AddComponent<SettingShape>();s.kind=kind;s.color=first;s.second=second;s.raycastTarget=false;return s;
    }
    public static Button Button(Transform p,string name,float x,float y,float w,float h,Action click=null,bool primary=false){
        var tab=Shape(p,name,SettingShape.Kind.Octagon,x,y,w,h,primary?new Color(.97f,.89f,.69f):new Color(.95f,.91f,.94f),primary?new Color(.82f,.68f,.47f):new Color(.83f,.76f,.85f));
        tab.rimWidth=1.1f;tab.rim=primary?SilkUi.Gold:new Color(.64f,.53f,.66f);
        var b=SelectableButton(tab,click);
        Label(tab.transform,name,8,0,w-16,h,primary?31:21,Ink,Hand,TextAnchor.MiddleCenter);
        return b;
    }
    public static Button SelectableButton(Graphic target,Action click){
        target.raycastTarget=true;var b=target.gameObject.AddComponent<Button>();b.targetGraphic=target;
        var colors=b.colors;colors.highlightedColor=colors.selectedColor=new Color(1,.92f,.78f);colors.pressedColor=new Color(.82f,.70f,.66f);b.colors=colors;
        if(click!=null)b.onClick.AddListener(()=>click());return b;
    }
    public static void Clear(Transform parent){
        for(int i=parent.childCount-1;i>=0;i--){var obj=parent.GetChild(i).gameObject;obj.SetActive(false);UnityEngine.Object.Destroy(obj);}
    }
    public static void Focus(Selectable item){
        if(item && item.isActiveAndEnabled && EventSystem.current)EventSystem.current.SetSelectedGameObject(item.gameObject);
    }
}
