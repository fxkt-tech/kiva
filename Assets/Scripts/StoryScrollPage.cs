using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;
using static NightUi;
using static SilkUi;

public sealed class StoryScrollPage : MonoBehaviour
{
    [SerializeField] Texture2D concept;
    [SerializeField] StoryVolume[] volumes;

    public IReadOnlyList<StoryVolume> Volumes=>volumes;

    // The reference is the texture atlas for the rods, paper edges, back arrow and small rolls.
    const float ReferenceWidth=1672,ReferenceHeight=941;
    const float ScaleX=1280/ReferenceWidth,ScaleY=720/ReferenceHeight;
    static readonly Rect PaperArea=new Rect(51,113,1570,543);
    static readonly Rect[] SmallRolls={
        new Rect(54,701,391,175),new Rect(462,701,370,175),
        new Rect(851,701,370,175),new Rect(1244,701,370,175)
    };
    static readonly Color Dark=new Color(.12f,.095f,.12f);
    static readonly Color Rice=new Color(.987f,.949f,.85f);
    static readonly Color InkTone=new Color(.24f,.17f,.14f);

    RectTransform layout,spots,detail;
    RawImage panorama;
    Material movingPaper, firstMiniature;
    Button back,enter;
    Text volumeTitle,tooltip,detailTitle,detailHabitat,detailState,enterText;
    CanvasGroup flare;
    Text[] coverNames;
    SettingShape[] coverLines;
    readonly List<Button> markers=new List<Button>();

    BirdCollection book;
    Action returnToRoom;
    Action<WorkshopLevel> play;
    int index,hover=-1,selected=-1,restoring=-1;
    float openedAt=-100,restoredAt=-100;
    Vector2 tooltipSize;
    StoryVolume Current=>volumes[index];
    bool HasDetail=>detail && detail.gameObject.activeSelf;

    public void Open(BirdCollection ledger,Action goBack,Action<WorkshopLevel> enterChapter,WorkshopLevel recovered)
    {
        book=ledger;returnToRoom=goBack;play=enterChapter;
        if(!layout)Build();
        index=Mathf.Clamp(index,0,volumes.Length-1);
        if(recovered)
        {
            for(int i=0;i<volumes.Length;i++)
            {
                int match=Array.FindIndex(volumes[i].birdSpots,spot=>spot.chapter==recovered);
                if(match<0)continue;
                index=i;restoring=match;restoredAt=Time.unscaledTime;
                break;
            }
        }
        ShowVolume(false);
        if(restoring>=0)
        {
            ShowCaption(restoring,false);
            ShowFlare(Current.birdSpots[restoring]);
        }
        Focus(back);
    }

    public bool BackOutOfCaption()
    {
        if(!HasDetail)return false;
        int last=selected;
        CloseCaption();
        if(last>=0 && last<markers.Count)Focus(markers[last]);
        return true;
    }

    static RectTransform PlacePixels(Transform parent,string name,Rect at)
        =>Place(parent,name,at.x*ScaleX,at.y*ScaleY,at.width*ScaleX,at.height*ScaleY);

    static Text TextPixels(Transform parent,string value,Rect at,int size,Color tone,Font font,TextAnchor alignment)
        =>Label(parent,value,at.x*ScaleX,at.y*ScaleY,at.width*ScaleX,at.height*ScaleY,size,tone,font,alignment);

    static Image HitArea(Transform parent,string name,Rect at)
    {
        var graphic=PlacePixels(parent,name,at).gameObject.AddComponent<Image>();
        graphic.color=new Color(0,0,0,.006f);
        return graphic;
    }

    RawImage Atlas(Transform parent,string name,Rect part)
    {
        var picture=PlacePixels(parent,name,part).gameObject.AddComponent<RawImage>();
        picture.texture=concept;
        picture.raycastTarget=false;
        picture.uvRect=new Rect(part.x/ReferenceWidth,1-part.yMax/ReferenceHeight,part.width/ReferenceWidth,part.height/ReferenceHeight);
        return picture;
    }

    static void Trigger(Button button,EventTriggerType type,Action handler)
    {
        var events=button.GetComponent<EventTrigger>();
        if(!events)events=button.gameObject.AddComponent<EventTrigger>();
        var entry=new EventTrigger.Entry{eventID=type};
        entry.callback.AddListener(_=>handler());
        events.triggers.Add(entry);
    }

    void Build()
    {
        var ground=new GameObject("墨色外沿",typeof(RectTransform),typeof(Image)).GetComponent<Image>();
        Fill(ground.rectTransform,transform);
        ground.color=Dark;ground.raycastTarget=false;
        layout=Page(transform,"长卷版心");
        Atlas(layout,"漆案、卷轴与小卷",new Rect(0,0,ReferenceWidth,ReferenceHeight));

        panorama=PlacePixels(layout,"可展开的画心",PaperArea).gameObject.AddComponent<RawImage>();
        panorama.raycastTarget=false;
        movingPaper=new Material(Resources.Load<Shader>("StoryScroll/ScrollPainting"));
        panorama.material=movingPaper;

        spots=PlacePixels(layout,"画中栖鸟",PaperArea);
        flare=PlacePixels(layout,"归巢金屑",new Rect(0,0,1,1)).gameObject.AddComponent<CanvasGroup>();
        flare.interactable=flare.blocksRaycasts=false;
        for(int i=0;i<7;i++)
        {
            float angle=i*2*Mathf.PI/7;
            Shape(flare.transform,"金屑",SettingShape.Kind.Spark,31*Mathf.Cos(angle),18*Mathf.Sin(angle),5,8,Cream,Gold);
        }
        flare.alpha=0;

        tooltip=TextPixels(layout,"",new Rect(0,0,186,40),21,Rice,Hand,TextAnchor.MiddleCenter);
        tooltip.fontStyle=FontStyle.Bold;
        ShadowText(tooltip,Dark,1.5f);
        tooltipSize=tooltip.rectTransform.sizeDelta;
        tooltip.enabled=false;

        back=SelectableButton(HitArea(layout,"返回客房",new Rect(40,18,141,68)),()=>returnToRoom?.Invoke());
        back.transition=Selectable.Transition.None;
        volumeTitle=TextPixels(layout,"",new Rect(1267,17,327,58),20,Rice,Hand,TextAnchor.MiddleRight);
        ShadowText(volumeTitle,Dark);

        BuildCovers();
        BuildCaption();
    }

    void BuildCovers()
    {
        coverNames=new Text[volumes.Length];
        coverLines=new SettingShape[volumes.Length];
        for(int i=0;i<volumes.Length;i++)
        {
            Rect rect=SmallRolls[i];
            var volume=volumes[i];
            int target=i;
            if(i==0)CreateFirstMiniature(rect);
            var button=SelectableButton(HitArea(layout,volume.title,new Rect(rect.x,rect.y,rect.width,203)),()=>ChooseVolume(target));
            button.transition=Selectable.Transition.None;
            coverNames[i]=TextPixels(layout,volume.title,new Rect(rect.x,874,rect.width,34),19,Rice,Hand,TextAnchor.MiddleCenter);
            ShadowText(coverNames[i],Dark);
            coverLines[i]=Shape(layout,"当卷金线",SettingShape.Kind.Hair,(rect.center.x-52)*ScaleX,906*ScaleY,104*ScaleX,2,Gold,Color.clear);
        }
    }

    void CreateFirstMiniature(Rect rect)
    {
        var miniature=Atlas(layout,"春溪初醒 · 卷面",rect);
        var uv=miniature.uvRect;
        firstMiniature=new Material(movingPaper.shader);
        firstMiniature.SetVector("_Crop",new Vector4(uv.x,uv.y,uv.width,uv.height));
        miniature.uvRect=new Rect(0,0,1,1);
        miniature.material=firstMiniature;
    }

    void BuildCaption()
    {
        detail=PlacePixels(layout,"入画题签",new Rect(193,15,1046,75));
        var paper=Shape(detail,"月白题签",SettingShape.Kind.FoldTab,0,0,803,57,new Color(.97f,.91f,.76f),new Color(.88f,.77f,.58f));
        paper.rimWidth=1;paper.rim=Gold;
        detailTitle=Label(detail,"",36,1,524,31,26,InkTone,Hand);
        detailHabitat=Label(detail,"",38,32,432,21,14,InkTone,Body);
        detailState=Label(detail,"敬请期待",534,0,186,56,22,InkTone,Hand,TextAnchor.MiddleCenter);
        var close=Button(detail,"收起",716,11,63,35,()=>BackOutOfCaption());
        close.name="合上题签";
        close.GetComponentInChildren<Text>().fontSize=17;
        enter=Button(detail,"进入画境",530,6,173,45,EnterChapter,true);
        enterText=enter.GetComponentInChildren<Text>();
        enterText.fontSize=23;
        detail.gameObject.SetActive(false);
    }

    void ChooseVolume(int value)
    {
        if(index==value)return;
        index=value;
        restoring=-1;
        ShowVolume(true);
    }

    void ShowVolume(bool animate)
    {
        CloseCaption();
        hover=-1;
        openedAt=animate?Time.unscaledTime:-100;
        var volume=Current;
        var crop=volume.storyCrop;
        panorama.texture=volume.storyPanorama;
        movingPaper.SetVector("_Crop",new Vector4(crop.x,crop.y,crop.width,crop.height));
        movingPaper.SetFloat("_PaperEdge",index==0?0:1);
        movingPaper.SetFloat("_Water",index==0 || index==2?1:0);
        movingPaper.SetFloat("_Opening",animate?0:.54f);
        movingPaper.SetFloat("_Recovery",0);
        volumeTitle.text=volume.number+" · "+volume.title;
        firstMiniature.SetFloat("_Dimming",index==0?0:1);
        for(int i=0;i<coverNames.Length;i++)
        {
            coverNames[i].color=i==index?Rice:new Color(.75f,.68f,.63f);
            coverLines[i].enabled=i==index;
        }
        RebuildMarkers();
    }

    void RebuildMarkers()
    {
        Clear(spots);
        markers.Clear();
        for(int i=0;i<Current.birdSpots.Length;i++)
        {
            int target=i;
            var spot=Current.birdSpots[i];
            var size=Vector2.Scale(spot.hitSize,PaperArea.size);
            var center=Vector2.Scale(spot.position,PaperArea.size);
            var rect=new Rect(center-size*.5f,size);
            var button=SelectableButton(HitArea(spots,spot.name,rect),()=>ShowCaption(target,true));
            button.transition=Selectable.Transition.None;
            Trigger(button,EventTriggerType.PointerEnter,()=>AimAt(target));
            Trigger(button,EventTriggerType.PointerExit,()=>HideName(target));
            Trigger(button,EventTriggerType.Select,()=>AimAt(target));
            Trigger(button,EventTriggerType.Deselect,()=>HideName(target));
            markers.Add(button);
        }
    }

    void AimAt(int ordinal)
    {
        hover=ordinal;
        var spot=Current.birdSpots[ordinal];
        float x=(PaperArea.x+spot.position.x*PaperArea.width)*ScaleX;
        float y=(PaperArea.y+(spot.position.y+spot.hitSize.y*.54f)*PaperArea.height)*ScaleY;
        tooltip.rectTransform.anchoredPosition=new Vector2(Mathf.Clamp(x-tooltipSize.x*.5f,34,1246-tooltipSize.x),-Mathf.Min(y,513));
        tooltip.text=spot.name;
        tooltip.enabled=selected!=ordinal;
    }

    void HideName(int ordinal)
    {
        if(hover!=ordinal)return;
        hover=-1;tooltip.enabled=false;
    }

    void ShowCaption(int ordinal,bool focusEntry)
    {
        selected=ordinal;
        var spot=Current.birdSpots[ordinal];
        var chapter=spot.chapter;
        if(chapter){
            int ordinalInVolume=Array.IndexOf(Current.chapters,chapter)+1;
            string chapterNumber=ordinalInVolume>0 && ordinalInVolume<=10?"一二三四五六七八九十"[ordinalInVolume-1].ToString():ordinalInVolume.ToString();
            detailTitle.text=spot.name+"　·　第"+chapterNumber+"关 "+chapter.title;
        }else detailTitle.text=spot.name;
        detailHabitat.text=spot.habitat;
        detailState.enabled=!chapter;
        enter.gameObject.SetActive(chapter);
        if(chapter)enterText.text=Current.StageCleared(chapter,book)?"重访画境":"进入画境";
        tooltip.enabled=false;
        detail.gameObject.SetActive(true);
        if(focusEntry && chapter)Focus(enter);
    }

    void CloseCaption()
    {
        selected=-1;
        detail.gameObject.SetActive(false);
        tooltip.enabled=false;
    }

    void EnterChapter()
    {
        if(selected>=0 && Current.birdSpots[selected].chapter)play?.Invoke(Current.birdSpots[selected].chapter);
    }

    void ShowFlare(StoryVolume.BirdSpot spot)
    {
        var pos=PaperArea.position+Vector2.Scale(spot.position,PaperArea.size);
        ((RectTransform)flare.transform).anchoredPosition=new Vector2(pos.x*ScaleX,-pos.y*ScaleY);
    }

    void Update()
    {
        if(!movingPaper)return;
        float now=Time.unscaledTime;
        float open=Mathf.Clamp01((now-openedAt)/.39f);
        movingPaper.SetFloat("_Opening",Mathf.SmoothStep(0,.54f,open));
        movingPaper.SetFloat("_Clock",now);
        int target=restoring>=0?restoring:hover>=0?hover:selected;
        if(target<0) movingPaper.SetVector("_Bird",Vector4.zero);
        else
        {
            var spot=Current.birdSpots[target];
            movingPaper.SetVector("_Bird",new Vector4(spot.position.x,1-spot.position.y,spot.hitSize.x*.66f,spot.hitSize.y*.66f));
        }
        float recovered=Mathf.Clamp01((now-restoredAt)/1.2f);
        movingPaper.SetFloat("_Recovery",restoring>=0?1-Mathf.SmoothStep(0,1,recovered):0);
        flare.alpha=restoring>=0?Mathf.Sin(recovered*Mathf.PI):0;
        flare.transform.localScale=Vector3.one*Mathf.Lerp(.65f,1.9f,recovered);
        if(recovered>=1)restoring=-1;
    }

    void OnDestroy()
    {
        if(movingPaper)Destroy(movingPaper);
        if(firstMiniature)Destroy(firstMiniature);
    }
}
