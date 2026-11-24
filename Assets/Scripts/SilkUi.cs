using UnityEngine;
using UnityEngine.UI;
using static NightUi;

public static class SilkUi
{
    public static readonly Color Cream=new Color(.97f,.94f,.83f);
    public static readonly Color Gold=new Color(.85f,.72f,.48f);
    public static readonly Color Plum=new Color(.29f,.20f,.34f);
    public static readonly Color Violet=new Color(.46f,.33f,.50f);
    public static readonly Color Night=new Color(.105f,.14f,.20f);
    public static RawImage Art(Transform parent,string name,Texture texture,float x,float y,float w,float h,bool crop=true){
        var raw=Place(parent,name,x,y,w,h).gameObject.AddComponent<RawImage>();
        raw.texture=texture;raw.raycastTarget=false;
        if(texture && crop){
            float src=(float)texture.width/texture.height,box=w/h;
            float u=Mathf.Min(1,box/src),v=Mathf.Min(1,src/box);
            raw.uvRect=new Rect((1-u)*.5f,(1-v)*.5f,u,v);
        }
        return raw;
    }
    public static void Frame(Transform parent,float w,float h,Color color,float inset=1){
        Panel(parent,"上镶线",inset,inset,w-inset*2,1,color);
        Panel(parent,"下镶线",inset,h-inset-1,w-inset*2,1,color);
        Panel(parent,"左镶线",inset,inset,1,h-inset*2,color);
        Panel(parent,"右镶线",w-inset-1,inset,1,h-inset*2,color);
    }
    public static void ShadowText(Text title,Color color,float depth=2){
        var shade=title.gameObject.AddComponent<Shadow>();
        shade.effectColor=color;shade.effectDistance=new Vector2(1,-depth);
    }
    public static Button Link(Transform parent,string name,float x,float y,float w,float h,System.Action action){
        var paper=Panel(parent,name,x,y,w,h,new Color(1,1,1,.015f));
        return SelectableButton(paper,action);
    }
    public static void Foil(Transform parent,string name,float x,float y,float w){
        Panel(parent,name,x,y,w,1,Gold);
        Shape(parent,"菱花",SettingShape.Kind.Diamond,x+w*.5f-4,y-3,8,7,Gold,Gold);
    }
}
