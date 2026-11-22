using System;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

// The shared look of every screen outside a level. Nothing is boxed: text and controls float over a
// dusk mountain painting. Azure, the colour of the first gate, carries the structure; vermilion, the
// colour of the second, marks the current choice and the main action.
// Everything is positioned from the top left of its parent, on a 1280 × 720 page.
public static class NightUi
{
    public static readonly Color White=new Color(.96f,.95f,1f);
    public static readonly Color AzureLight=new Color(.56f,.86f,.95f),Azure=new Color(.16f,.58f,.75f),AzureDeep=new Color(.06f,.28f,.41f);
    public static readonly Color VermilionLight=new Color(1f,.62f,.49f),Vermilion=new Color(.79f,.24f,.13f),Blush=new Color(1f,.71f,.60f);
    public static readonly Color Night=new Color(.024f,.094f,.157f);

    static Font body,hand,display;
    static Texture2D scene,glow;
    public static Font Body=>body?body:body=Font.CreateDynamicFontFromOSFont(new[]{"Microsoft YaHei","PingFang SC","Noto Sans CJK SC","Arial"},32);
    public static Font Hand=>hand?hand:hand=Font.CreateDynamicFontFromOSFont(new[]{"KaiTi","STKaiti","Kaiti SC","Microsoft YaHei","PingFang SC"},40);
    public static Font Display=>display?display:display=Font.CreateDynamicFontFromOSFont(new[]{"STXingkai","Xingkai SC","KaiTi","STKaiti","Kaiti SC","Microsoft YaHei"},104);
    public static Texture2D Glow=>glow?glow:glow=Halo();
    static Texture2D Scene=>scene?scene:scene=NightScene();

    // ---------- generated art ----------

    static Color Rgb(int hex)=>new Color((hex>>16&255)/255f,(hex>>8&255)/255f,(hex&255)/255f);
    static float Noise(int x,int y)
    {
        float v=Mathf.Sin(x*12.9898f+y*78.233f)*43758.5453f;
        return v-Mathf.Floor(v);
    }

    // Mountains at dusk: azure ranges with pale contour lines, a vermilion sun and flat cloud bands.
    static Texture2D NightScene()
    {
        const int w=960,h=540;
        float[] bases={400,520,690},amps={200,220,250},phases={.4f,1.9f,3.3f};
        Color[] tops={Rgb(0x4f9bb0),Rgb(0x1f6f8f),Rgb(0x0b3350)},feet={Rgb(0x7fb7bd),Rgb(0x3f8ea6),Rgb(0x14506e)};
        float[] skyAt={0,.45f,.78f,1};
        Color[] sky={Rgb(0x082033),Rgb(0x14587a),Rgb(0x5aa0ad),Rgb(0xf0b592)};
        // Each band: centre x, top y, length, rows, opacity.
        float[][] far={new[]{360f,470,620,3,.55f},new[]{1080f,520,460,2,.5f}};
        float[][] near={new[]{700f,668,760,2,.5f},new[]{120f,640,380,2,.45f}};
        Color cream=Rgb(0xf3dcc4),line=Rgb(0xf6ecd6),sun=Rgb(0xe0553a);
        var ridge=new float[3][];
        for(int l=0;l<3;l++){
            ridge[l]=new float[w];
            for(int i=0;i<w;i++){
                float u=(i+.5f)/w,p=phases[l];
                ridge[l][i]=bases[l]-amps[l]*(.55f*Mathf.Abs(Mathf.Sin(u*3.1f+p))+.3f*Mathf.Abs(Mathf.Sin(u*7.3f+p*2.1f))
                    +.12f*Mathf.Sin(u*23+p*5)+.05f*Mathf.Sin(u*61+p));
            }
        }
        Color Clouds(Color c,float x,float y,float[][] bands)
        {
            foreach(var b in bands)for(int r=0;r<(int)b[3];r++){
                float length=b[2]*(1-.2f*Mathf.Abs(r-(b[3]-1)*.5f)),left=b[0]-length*.5f+(r%2==1?40:-30);
                float dy=y-(b[1]+r*20+9),dx=x-Mathf.Clamp(x,left+9,left+length-9);
                if(dx*dx+dy*dy<=81)c=Color.Lerp(c,cream,b[4]);
            }
            return c;
        }
        var pixels=new Color[w*h];
        for(int py=0;py<h;py++)for(int px=0;px<w;px++){
            float x=(px+.5f)*1280/w,y=(py+.5f)*720/h,t=y/720;
            int s=t<skyAt[1]?0:t<skyAt[2]?1:2;
            var c=Color.Lerp(sky[s],sky[s+1],Mathf.InverseLerp(skyAt[s],skyAt[s+1],t));
            if(y<430 && Noise(px,py)>.9985f)c=Color.Lerp(c,Color.white,.8f);
            float d=Vector2.Distance(new Vector2(x,y),new Vector2(1010,205));
            if(d<150)c=sun;
            else if(d<240){float g=1-(d-150)/90;c=Color.Lerp(c,sun,g*g*.5f);}
            for(int l=0;l<3;l++){
                if(l==2)c=Clouds(c,x,y,far);
                float over=y-ridge[l][px];
                if(over<0)continue;
                c=Color.Lerp(tops[l],feet[l],Mathf.InverseLerp(200,720,y));
                if(over<2.2f)c=Color.Lerp(c,line,.55f);
            }
            c=Clouds(c,x,y,near)*(.94f+.1f*Noise(px+17,py+31));
            c.a=1;
            pixels[(h-1-py)*w+px]=c;
        }
        var texture=new Texture2D(w,h,TextureFormat.RGBA32,false){name="夜山",wrapMode=TextureWrapMode.Clamp,filterMode=FilterMode.Bilinear};
        texture.SetPixels(pixels);texture.Apply(false,true);
        return texture;
    }

    static Texture2D Halo()
    {
        const int size=64;
        var pixels=new Color[size*size];
        for(int y=0;y<size;y++)for(int x=0;x<size;x++){
            float d=Vector2.Distance(new Vector2(x+.5f,y+.5f),Vector2.one*size*.5f)/(size*.5f);
            float a=Mathf.Clamp01(1-d);
            pixels[y*size+x]=new Color(1,1,1,a*a);
        }
        var texture=new Texture2D(size,size,TextureFormat.RGBA32,false){name="光晕",wrapMode=TextureWrapMode.Clamp};
        texture.SetPixels(pixels);texture.Apply(false,true);
        return texture;
    }

    // ---------- layout ----------

    public static void Fill(RectTransform rect,Transform parent)
    {
        rect.SetParent(parent,false);
        rect.anchorMin=Vector2.zero;rect.anchorMax=Vector2.one;
        rect.offsetMin=rect.offsetMax=Vector2.zero;
    }

    // The painting fills the screen; the veil is darker on the left, where the menus and lists sit.
    public static void Backdrop(Transform parent)
    {
        var painting=new GameObject("夜山",typeof(RectTransform),typeof(RawImage)).GetComponent<RawImage>();
        painting.rectTransform.SetParent(parent,false);
        painting.texture=Scene;
        var fit=painting.gameObject.AddComponent<AspectRatioFitter>();
        fit.aspectMode=AspectRatioFitter.AspectMode.EnvelopeParent;
        fit.aspectRatio=(float)Scene.width/Scene.height;
        var veil=new GameObject("夜幕",typeof(RectTransform),typeof(CanvasRenderer)).AddComponent<SettingShape>();
        Fill(veil.rectTransform,parent);
        veil.kind=SettingShape.Kind.Bar;
        veil.color=new Color(Night.r,Night.g,Night.b,.78f);
        veil.second=new Color(Night.r,Night.g,Night.b,.16f);
    }

    // A centred 1280 × 720 page.
    public static RectTransform Page(Transform parent,string name)
    {
        var page=new GameObject(name,typeof(RectTransform)).GetComponent<RectTransform>();
        page.SetParent(parent,false);
        page.anchorMin=page.anchorMax=page.pivot=Vector2.one*.5f;
        page.sizeDelta=new Vector2(1280,720);
        return page;
    }

    public static RectTransform Place(Transform parent,string name,float x,float y,float w,float h)
    {
        var rect=new GameObject(name,typeof(RectTransform)).GetComponent<RectTransform>();
        rect.SetParent(parent,false);
        rect.anchorMin=rect.anchorMax=rect.pivot=Vector2.up;
        rect.anchoredPosition=new Vector2(x,-y);
        rect.sizeDelta=new Vector2(w,h);
        return rect;
    }

    public static RawImage Box(Transform parent,string name,float x,float y,float w,float h,Color color)
    {
        var image=Place(parent,name,x,y,w,h).gameObject.AddComponent<RawImage>();
        image.color=color;image.raycastTarget=false;
        return image;
    }

    public static SettingShape Shape(Transform parent,string name,SettingShape.Kind kind,float x,float y,float w,float h,Color top,Color bottom)
    {
        var rect=Place(parent,name,x,y,w,h);
        rect.gameObject.AddComponent<CanvasRenderer>();
        var shape=rect.gameObject.AddComponent<SettingShape>();
        shape.kind=kind;shape.color=top;shape.second=bottom;shape.raycastTarget=false;
        return shape;
    }

    public static Text Write(Transform parent,string value,Font font,int size,Color color,float x,float y,float w,float h,
        TextAnchor anchor=TextAnchor.MiddleLeft,bool glow=false)
    {
        var text=Place(parent,value,x,y,w,h).gameObject.AddComponent<Text>();
        text.font=font;text.fontSize=size;text.color=color;text.alignment=anchor;
        text.horizontalOverflow=HorizontalWrapMode.Overflow;text.verticalOverflow=VerticalWrapMode.Overflow;
        text.supportRichText=false;text.raycastTarget=false;text.text=value;
        var shadow=text.gameObject.AddComponent<Shadow>();
        shadow.effectColor=new Color(.016f,.08f,.13f,.8f);shadow.effectDistance=new Vector2(0,-2);
        if(glow){
            var light=text.gameObject.AddComponent<Outline>();
            light.effectColor=new Color(.35f,.75f,.94f,.4f);light.effectDistance=new Vector2(2,-2);
        }
        return text;
    }

    public static string Vertical(string value)=>string.Join("\n",value.ToCharArray());
    public static string Spaced(string value)=>string.Join(" ",value.ToCharArray());

    public static float TextWidth(string value,float latin,float wide)
    {
        float width=0;
        foreach(char c in value)width+=c<128?latin:wide;
        return width;
    }

    public static void Press(Graphic target,Action act)
    {
        target.raycastTarget=true;
        var button=target.gameObject.AddComponent<Button>();
        button.targetGraphic=target;
        button.transition=Selectable.Transition.None;
        button.navigation=new Navigation{mode=Navigation.Mode.None};
        button.onClick.AddListener(()=>act());
    }

    // ---------- recurring pieces ----------

    // A gem centred on (cx, cy): soft light behind, a rimmed diamond, and a pale core.
    public static void Gem(Transform parent,float cx,float cy,float size,bool lit)
    {
        var light=Box(parent,"光",cx-size,cy-size,size*2,size*2,lit?new Color(1,.47f,.31f,.75f):new Color(.24f,.63f,.82f,.4f));
        light.texture=Glow;
        var gem=Shape(parent,"菱",SettingShape.Kind.Diamond,cx-size*.5f,cy-size*.5f,size,size,
            lit?VermilionLight:new Color(.36f,.75f,.88f),lit?Vermilion:AzureDeep);
        gem.rim=lit?Color.white:new Color(1,.89f,.84f,.8f);gem.rimWidth=lit?2:1.5f;
        float core=size*.24f;
        Shape(parent,"芯",SettingShape.Kind.Diamond,cx-core*.5f,cy-core*.5f,core,core,White,White);
    }

    // A diamond button centred on (cx, cy). The main action is vermilion, the others azure.
    public static void Diamond(Transform parent,string caption,float cx,float cy,float size,int fontSize,bool main,Action act)
    {
        var light=Box(parent,caption+"光",cx-size*.8f,cy-size*.8f,size*1.6f,size*1.6f,main?new Color(1,.47f,.31f,.7f):new Color(.27f,.7f,.92f,.6f));
        light.texture=Glow;
        var gem=Shape(parent,caption,SettingShape.Kind.Diamond,cx-size*.5f,cy-size*.5f,size,size,
            main?new Color(1,.67f,.55f):new Color(.42f,.78f,.9f),main?new Color(.66f,.19f,.1f):new Color(.07f,.33f,.47f));
        gem.rim=main?Color.white:new Color(White.r,White.g,White.b,.85f);gem.rimWidth=2;
        Write(gem.transform,caption,Hand,fontSize,Color.white,0,0,size,size,TextAnchor.MiddleCenter);
        Press(gem,act);
    }

    // The page name in the top-left corner; pressing it goes back.
    public static void Back(Transform page,string title,Action act)
    {
        var hit=Box(page,"返回",24,14,72+TextWidth(title,20,40),60,Color.clear);
        Shape(page,"箭头",SettingShape.Kind.Chevron,36,26,20,38,White,White);
        Write(page,title,Hand,36,White,76,20,400,50,TextAnchor.MiddleLeft,true);
        Press(hit,act);
    }

    // A large vertical title down the right edge, beside a slanted ribbon.
    public static Text Banner(Transform page,string title,string ribbonText="山海行纪")
    {
        var ribbon=Place(page,"飘带",1060,104,34,250);
        ribbon.pivot=new Vector2(.5f,1);
        ribbon.anchoredPosition=new Vector2(1077,-104);
        ribbon.localRotation=Quaternion.Euler(0,0,-10);
        Shape(ribbon,"带",SettingShape.Kind.Fall,0,0,34,250,new Color(.24f,.67f,.88f,.6f),new Color(.24f,.67f,.88f,.1f));
        Box(ribbon,"边",0,0,1,250,new Color(White.r,White.g,White.b,.5f));
        Box(ribbon,"边",33,0,1,250,new Color(White.r,White.g,White.b,.5f));
        Write(ribbon,Vertical(ribbonText),Hand,19,White,0,0,34,250,TextAnchor.MiddleCenter).lineSpacing=1.5f;
        var text=Write(page,Vertical(title),Display,104,Color.white,1084,84,150,330,TextAnchor.UpperCenter,true);
        text.lineSpacing=.88f;
        return text;
    }

    // A hairline that fades at both ends, with a spark at one end.
    public static void Hairline(Transform parent,float x,float y,float width,bool sparkLeft)
    {
        Shape(parent,"线",SettingShape.Kind.Hair,x,y,width,1,new Color(White.r,White.g,White.b,.85f),Color.clear);
        Shape(parent,"星",SettingShape.Kind.Spark,sparkLeft?x+6:x+width-24,y-8,16,16,White,White);
    }
}

// Reports the pointer entering and leaving.
public sealed class HoverWatch : MonoBehaviour, IPointerEnterHandler, IPointerExitHandler
{
    public Action entered,left;
    public void OnPointerEnter(PointerEventData data)=>entered?.Invoke();
    public void OnPointerExit(PointerEventData data)=>left?.Invoke();
}
