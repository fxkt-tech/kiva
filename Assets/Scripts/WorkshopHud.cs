using UnityEngine;
using UnityEngine.UI;

public sealed class WorkshopHud
{
    readonly Transform root;
    readonly Text hint,exit,toolName;
    readonly WorkshopGlyph sight,wing;
    readonly WorkshopGlyph[] slots=new WorkshopGlyph[3];
    readonly Image[] marks=new Image[3];
    float hintUntil,labelUntil;

    public WorkshopHud(Text status)
    {
        hint=status;root=status.transform.parent;
        status.rectTransform.anchorMin=status.rectTransform.anchorMax=new Vector2(0,0);
        status.rectTransform.pivot=Vector2.zero;
        status.rectTransform.anchorMax=new Vector2(.76f,0);
        status.rectTransform.sizeDelta=new Vector2(-65,70);
        status.rectTransform.anchoredPosition=new Vector2(36,30);
        status.fontSize=20;status.color=Color.white;status.raycastTarget=false;
        status.alignment=TextAnchor.LowerLeft;
        if(!status.GetComponent<Shadow>())status.gameObject.AddComponent<Shadow>();
        exit=Label("出口指引",Vector2.zero,Vector2.zero,17,new Vector2(96,30));
        toolName=Label("道具名",Vector2.right,new Vector2(-128,87),18,new Vector2(170,28));
        sight=Glyph("准星",Vector2.one*.5f,Vector2.zero);
        for(int i=0;i<3;i++){
            var pos=new Vector2(-215+i*48,54);
            slots[i]=Glyph(WorkshopToolSpec.For((WorkshopTool)i).name,Vector2.right,pos);
            Label((i+1).ToString(),Vector2.right,pos+Vector2.down*21,13,new Vector2(22,16));
            marks[i]=new GameObject("选中",typeof(RectTransform),typeof(Image)).GetComponent<Image>();
            Rect(marks[i].rectTransform,Vector2.right,pos+Vector2.down*33,new Vector2(17,2));
            marks[i].raycastTarget=false;
        }
        wing=Glyph("滑翔翼",Vector2.right,new Vector2(-53,54));
    }

    void Rect(RectTransform rect,Vector2 anchor,Vector2 position,Vector2 size)
    {
        rect.SetParent(root,false);
        rect.anchorMin=rect.anchorMax=anchor;rect.pivot=Vector2.one*.5f;
        rect.sizeDelta=size;rect.anchoredPosition=position;
    }

    Text Label(string text,Vector2 anchor,Vector2 position,int size,Vector2 dimensions)
    {
        var label=new GameObject(text,typeof(RectTransform),typeof(Text)).GetComponent<Text>();
        Rect(label.rectTransform,anchor,position,dimensions);
        label.font=hint.font;label.fontSize=size;label.color=Color.white;label.alignment=TextAnchor.MiddleCenter;
        label.raycastTarget=false;label.text=text;label.gameObject.AddComponent<Shadow>();
        return label;
    }

    WorkshopGlyph Glyph(string name,Vector2 anchor,Vector2 position)
    {
        var icon=new GameObject(name,typeof(RectTransform),typeof(CanvasRenderer),typeof(WorkshopGlyph)).GetComponent<WorkshopGlyph>();
        Rect(icon.rectTransform,anchor,position,new Vector2(32,32));
        icon.raycastTarget=false;
        icon.gameObject.AddComponent<Shadow>().effectDistance=new Vector2(1,-1);
        return icon;
    }

    public void Hint(string value,float seconds=3){hint.text=value;hintUntil=Time.unscaledTime+seconds;}
    public void Selected(WorkshopTool tool){toolName.text=WorkshopToolSpec.For(tool).name;labelUntil=Time.unscaledTime+1.2f;}

    public void Paint(WorkshopTool tool,WorkshopGear gear,bool valid,bool first,bool second,bool gliding)
    {
        int index=(int)tool;
        var options=GameSettings.Current;
        sight.enabled=options.crosshair && WorkshopToolSpec.Owned(gear,index);
        sight.Show(index,true,!valid,first,second);
        for(int i=0;i<slots.Length;i++){
            bool owned=WorkshopToolSpec.Owned(gear,i);
            slots[i].Show(owned?i:-1);
            slots[i].color=new Color(1,1,1,i==index?1:owned?.7f:.32f);
            marks[i].enabled=owned && i==index;
        }
        wing.enabled=(gear & WorkshopGear.Glider)!=0;wing.Show(3,flying:gliding);
        toolName.enabled=Time.unscaledTime<labelUntil;
        if(Time.unscaledTime>hintUntil || !options.hints)hint.text="";
    }

    public void Exit(Camera eye,Vector3 world)
    {
        exit.enabled=GameSettings.Current.exitGuide;
        if(!exit.enabled)return;
        var p=eye.WorldToViewportPoint(world);bool behind=p.z<0;
        if(behind)p.x=p.x>.5f?0:1;
        float x=Mathf.Clamp(p.x,.07f,.93f),y=Mathf.Clamp(p.y,.16f,.88f);
        if(!behind && x==p.x && y==p.y && eye.transform.position.z<world.z &&
            !Physics.Linecast(eye.transform.position,world,Physics.DefaultRaycastLayers,QueryTriggerInteraction.Ignore)){
            exit.enabled=false;return;
        }
        var rect=exit.rectTransform;rect.anchorMin=rect.anchorMax=new Vector2(x,y);
        rect.anchoredPosition=Vector2.zero;
        exit.text=behind || p.x!=x?(x<.5f?"← 出口":"出口 →"):"出口";
    }
}
