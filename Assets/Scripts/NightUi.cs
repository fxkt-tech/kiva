using System;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

// Shared typography and controls for the pause menu.
public static class NightUi
{
    static Font body,hand,display;
    public static Font Body=>body?body:body=Resources.Load<Font>("NightUi/Body");
    public static Font Hand=>hand?hand:hand=Resources.Load<Font>("NightUi/Hand");
    public static Font Display=>display?display:display=Resources.Load<Font>("NightUi/Display");

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
