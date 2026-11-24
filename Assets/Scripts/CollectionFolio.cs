using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;
using static NightUi;
using static SilkUi;

public sealed class CollectionFolio : MonoBehaviour
{
    enum Leaf { Paintings, Birds, BirdDetail }

    static readonly Color Teal=new Color(.092f,.183f,.189f);
    static readonly Color Pearl=new Color(.985f,.943f,.838f);
    static readonly Color Sepia=new Color(.244f,.198f,.154f);
    static readonly Color Notes=new Color(.316f,.335f,.285f);
    static readonly Color LineGold=new Color(.71f,.57f,.31f,.64f);
    static readonly Rect RibbonUv=new Rect(705/1536f,1-670/1024f,781/1536f,187/1024f);
    static readonly Rect SignUv=new Rect(2/1536f,1-1007/1024f,760/1536f,212/1024f);
    const float Left=345,Right=1228,ContentWidth=Right-Left;

    struct BirdEntry
    {
        public string Name;
        public BirdSpecies Species;
        public StoryVolume Volume;
        public StoryVolume.BirdSpot Spot;
        public bool Known;
    }

    readonly Dictionary<string,BirdSpecies> byId=new Dictionary<string,BirdSpecies>();
    readonly Dictionary<string,BirdSpecies> byName=new Dictionary<string,BirdSpecies>();
    readonly List<BirdEntry> entries=new List<BirdEntry>();
    readonly Vector4[] restored=new Vector4[10];
    BirdCollection book;
    IReadOnlyList<StoryVolume> volumes;
    Action leave;
    RectTransform layout,content;
    RawImage paintingsRibbon,birdsRibbon;
    Button paintingsButton,birdsButton;
    Texture2D paperSigns,birdSetting;
    Material paintingInk,mutedInk,thumbnailInk;
    Leaf leaf;
    int selectedVolume;
    BirdSpecies lastBird;

    public void Open(BirdCollection ledger,IReadOnlyList<StoryVolume> catalog,Action goBack,BirdSpecies featured=null)
    {
        book=ledger;volumes=catalog;leave=goBack;
        if(!layout)Build();
        if(featured && book.HasSpecies(featured.id))ShowBird(featured);
        else ShowPaintings();
    }

    public bool CloseBird()
    {
        if(leaf!=Leaf.BirdDetail)return false;
        ShowBirds(lastBird);
        return true;
    }

    void Build()
    {
        foreach(var species in Resources.LoadAll<BirdSpecies>("Birds"))
        {
            byId[species.id]=species;
            byName[species.birdName]=species;
        }
        var shader=Resources.Load<Shader>("GuestRoom/PaintedMemory");
        paintingInk=new Material(shader);
        mutedInk=new Material(shader);
        thumbnailInk=new Material(shader);
        thumbnailInk.SetFloat("_Thumbnail",1);
        paperSigns=Resources.Load<Texture2D>("GuestRoom/GuestPaperButtons");
        birdSetting=Resources.Load<Texture2D>("Collection/BirdViewingPaper");

        var ground=new GameObject("收藏全屏底色",typeof(RectTransform),typeof(Image)).GetComponent<Image>();
        Fill(ground.rectTransform,transform);
        ground.color=Paper;
        var silk=Resources.Load<Texture2D>("Collection/CollectionPaper");
        const float binding=305-640,artSplit=.244f;
        for(int i=0;i<2;i++)
        {
            bool left=i==0;
            var backdrop=Art(transform,left?"石青绢脊":"月白藏页",silk,0,0,1,1,false);
            var bounds=backdrop.rectTransform;
            bounds.pivot=new Vector2(.5f,.5f);
            bounds.anchorMin=left?Vector2.zero:new Vector2(.5f,0);
            bounds.anchorMax=left?new Vector2(.5f,1):Vector2.one;
            bounds.offsetMin=left?Vector2.zero:new Vector2(binding,0);
            bounds.offsetMax=left?new Vector2(binding,0):Vector2.zero;
            backdrop.uvRect=left?new Rect(0,0,artSplit,1):new Rect(artSplit,0,1-artSplit,1);
            if(!silk)backdrop.color=left?Teal:Paper;
        }

        layout=Page(transform,"收藏版心");
        var back=PaperButton(layout,"返回",47,39,133,48,()=>{if(!CloseBird())leave?.Invoke();});
        back.GetComponentInChildren<Text>().rectTransform.anchoredPosition+=Vector2.right*6;
        Shape(back.transform,"左向箭头",SettingShape.Kind.Chevron,16,17,13,13,Sepia,Sepia);
        var title=Label(layout,"收集",50,117,219,94,68,Pearl,NightUi.Display);
        ShadowText(title,Teal,2);
        Hair(layout,52,220,187);
        paintingsButton=Tab("画卷",252,ShowPaintings,out paintingsRibbon);
        birdsButton=Tab("百鸟",332,()=>ShowBirds(),out birdsRibbon);
        content=Place(layout,"收藏内容",0,0,1280,720);
    }

    Button PaperButton(Transform parent,string name,float x,float y,float width,float height,Action action)
    {
        var click=Link(parent,name,x,y,width,height,action);
        click.transition=Selectable.Transition.None;
        var paper=Art(click.transform,"描金纸签",paperSigns,0,0,width,height,false);
        paper.uvRect=SignUv;
        Label(click.transform,name,8,0,width-16,height,21,Sepia,Hand,TextAnchor.MiddleCenter);
        AddFocus(click,width,height);
        return click;
    }

    Button Tab(string caption,float y,Action action,out RawImage ribbon)
    {
        var click=Link(layout,caption,43,y,212,64,action);
        click.transition=Selectable.Transition.None;
        ribbon=Art(click.transform,"绢签",paperSigns,0,0,212,64,false);
        ribbon.uvRect=RibbonUv;
        Shape(click.transform,"金菱",SettingShape.Kind.Diamond,37,26,12,12,Gold,new Color(.6f,.36f,.16f));
        var label=Label(click.transform,caption,69,0,114,64,31,Pearl,Hand);
        ShadowText(label,Teal);
        AddFocus(click,212,64);
        return click;
    }

    static void AddFocus(Button b,float width,float height)
    {
        var glint=Place(b.transform,"焦点提示",0,0,width,height).gameObject.AddComponent<CanvasGroup>();
        glint.alpha=0;glint.blocksRaycasts=false;
        Shape(glint.transform,"金芒",SettingShape.Kind.Spark,width-22,8,8,13,Pearl,Gold);
        b.gameObject.AddComponent<LobbyPlaqueFocus>().Bind(glint);
    }

    static void Hair(Transform parent,float x,float y,float width)
        =>Shape(parent,"金线",SettingShape.Kind.Hair,x,y,width,1.5f,LineGold,Color.clear).fade=.06f;

    void ChangeLeaf(Leaf next)
    {
        leaf=next;
        paintingsRibbon.color=next==Leaf.Paintings?new Color(.92f,.92f,1):new Color(.67f,.69f,.80f,.50f);
        birdsRibbon.color=next==Leaf.Paintings?new Color(.67f,.69f,.80f,.50f):new Color(.92f,.92f,1);
        Clear(content);
    }

    void ShowPaintings()
    {
        ChangeLeaf(Leaf.Paintings);
        selectedVolume=Mathf.Clamp(selectedVolume,0,volumes.Count-1);
        StoryVolume volume=volumes[selectedVolume];
        int found=volume.Cleared(book);
        bool collected=volume.Collected(book);
        Label(content,volume.number,Left,29,284,36,22,Notes,Hand);
        Label(content,volume.title,Left-3,64,494,67,52,Sepia,NightUi.Display);
        Label(content,collected?"完整画卷已收录":found>0?"栖地已复色":"画卷待修复",844,42,383,38,23,Sepia,Hand,TextAnchor.MiddleRight);
        Label(content,"已寻回  "+found+" / "+volume.birdSpots.Length,844,82,383,34,18,Notes,Body,TextAnchor.MiddleRight);
        Hair(content,Left,133,ContentWidth);
        DrawScroll(volume,collected);
        string note=volume.preface;
        if(!string.IsNullOrEmpty(note)) Label(content,note,Left+8,466,ContentWidth-16,52,19,Notes,Hand);

        float gap=14,tileWidth=(ContentWidth-(volumes.Count-1)*gap)/volumes.Count;
        Button active=null;
        for(int i=0;i<volumes.Count;i++)
        {
            int target=i;
            StoryVolume entry=volumes[i];
            bool selected=i==selectedVolume;
            var backing=Panel(content,entry.title,Left+i*(tileWidth+gap),537,tileWidth,126,
                selected?new Color(.942f,.880f,.682f):new Color(.91f,.902f,.820f,.9f));
            Texture2D source=entry.storyPanorama?entry.storyPanorama:entry.painting;
            var thumbnail=Art(backing.transform,"卷签画片",source,6,6,tileWidth-12,73,false);
            thumbnail.uvRect=entry.storyPanorama?entry.storyCrop:new Rect(0,0,1,1);
            if(!entry.Collected(book))thumbnail.material=thumbnailInk;
            Frame(backing.transform,tileWidth,126,selected?Gold:LineGold,2);
            Label(backing.transform,entry.title,4,81,tileWidth-8,40,21,Sepia,Hand,TextAnchor.MiddleCenter);
            var button=SelectableButton(backing,()=>{if(selectedVolume!=target){selectedVolume=target;ShowPaintings();}});
            button.transition=Selectable.Transition.None;
            AddFocus(button,tileWidth,126);
            if(selected)active=button;
        }
        Focus(active?active:paintingsButton);
    }

    void DrawScroll(StoryVolume volume,bool collected)
    {
        const float width=883,height=307;
        var frame=Art(content,"画卷装裱",Resources.Load<Texture2D>("StoryScroll/StoryScrollConcept"),Left,146,width,height,false);
        frame.uvRect=new Rect(9/1672f,1-686/941f,1654/1672f,595/941f);
        var art=Art(frame.transform,"收藏画心",volume.storyPanorama?volume.storyPanorama:volume.painting,
            width*42/1654,height*22/595,width*1570/1654,height*543/595,false);
        var crop=volume.storyPanorama?volume.storyCrop:new Rect(0,0,1,1);
        paintingInk.SetVector("_Crop",new Vector4(crop.x,crop.y,crop.width,crop.height));
        paintingInk.SetFloat("_State",collected?1:0);
        int count=0;
        foreach(var spot in volume.birdSpots)
        {
            if(count==restored.Length)break;
            if(!spot.chapter || !volume.StageCleared(spot.chapter,book))continue;
            restored[count++]=new Vector4(spot.position.x,1-spot.position.y,.18f,.34f);
        }
        paintingInk.SetInt("_RestoredCount",count);
        paintingInk.SetVectorArray("_RestoredSpots",restored);
        art.material=paintingInk;
    }

    void BuildEntries()
    {
        entries.Clear();
        var names=new HashSet<string>();
        var first=volumes[0];
        foreach(var spot in first.birdSpots)
        {
            if(!names.Add(spot.name))continue;
            var species=spot.chapter && spot.chapter.perches.Length>0?spot.chapter.perches[0].species:null;
            if(!species)byName.TryGetValue(spot.name,out species);
            entries.Add(new BirdEntry {
                Name=spot.name,Species=species,Volume=first,Spot=spot,Known=species && book.HasSpecies(species.id)
            });
        }
        foreach(var saved in book.Entries)
            if(byId.TryGetValue(saved.species,out var bird) && names.Add(bird.birdName))
                entries.Add(new BirdEntry {Name=bird.birdName,Species=bird,Known=true});
    }

    void ShowBirds(BirdSpecies focus=null)
    {
        ChangeLeaf(Leaf.Birds);
        BuildEntries();
        int found=0;
        foreach(var entry in entries)if(entry.Known)found++;
        Label(content,"百鸟",Left-3,48,525,73,56,Sepia,NightUi.Display);
        Label(content,"已收录  "+found+" 种",844,55,383,51,21,Notes,Hand,TextAnchor.MiddleRight);
        Hair(content,Left,133,ContentWidth);
        const float height=515,cardHeight=204,gap=15;
        var clip=Panel(content,"百鸟藏页",Left,148,ContentWidth,height,new Color(1,1,1,.006f));
        clip.raycastTarget=true;
        clip.gameObject.AddComponent<RectMask2D>();
        var scroll=clip.gameObject.AddComponent<ScrollRect>();
        scroll.viewport=clip.rectTransform;scroll.horizontal=false;
        scroll.scrollSensitivity=44;scroll.movementType=ScrollRect.MovementType.Clamped;
        var rack=Place(clip.transform,"百鸟名录",0,0,ContentWidth,Mathf.Max(height,Mathf.CeilToInt(entries.Count/5f)*(cardHeight+gap)));
        scroll.content=rack;
        float cardWidth=(ContentWidth-4*gap)/5;
        Button target=null;
        for(int i=0;i<entries.Count;i++)
        {
            var entry=entries[i];
            var button=BuildCard(rack,entry,i%5*(cardWidth+gap),i/5*(cardHeight+gap)+6,cardWidth,cardHeight);
            if(!button)continue;
            button.gameObject.AddComponent<FolioFocus>().scroll=scroll;
            if(entry.Species==focus)target=button;
        }
        Focus(target?target:birdsButton);
    }

    Button BuildCard(Transform rack,BirdEntry entry,float x,float y,float width,float height)
    {
        var card=Panel(rack,entry.Name,x,y,width,height,
            entry.Known?new Color(.946f,.953f,.842f):new Color(.938f,.901f,.793f));
        Frame(card.transform,width,height,entry.Known?Gold:LineGold,2);
        if(entry.Known)
        {
            var backdrop=Art(card.transform,"栖纸",birdSetting,7,7,width-14,117);
            backdrop.color=new Color(.91f,.91f,.96f);
            var bird=Place(card.transform,"折纸鸟",9,14,width-18,109).gameObject.AddComponent<BirdPortrait>();
            bird.Display(entry.Species);
            bird.raycastTarget=false;
        }
        else if(entry.Volume)
        {
            var picture=Art(card.transform,"栖地墨印",entry.Volume.storyPanorama,7,7,width-14,117,false);
            picture.uvRect=BirdCrop(entry.Volume,entry.Spot,(width-14)/117);
            picture.material=mutedInk;
            picture.color=new Color(1,1,1,.75f);
        }
        Label(card.transform,entry.Name,5,135,width-10,37,entry.Name.Length>4?18:23,Sepia,Hand,TextAnchor.MiddleCenter);
        Label(card.transform,entry.Known?"已收录":"待寻",6,169,width-12,26,15,Notes,Body,TextAnchor.MiddleCenter);
        if(!entry.Known)return null;
        var b=SelectableButton(card,()=>ShowBird(entry.Species));
        b.transition=Selectable.Transition.None;
        AddFocus(b,width,height);
        return b;
    }

    static Rect BirdCrop(StoryVolume volume,StoryVolume.BirdSpot spot,float targetAspect)
    {
        var crop=volume.storyCrop;
        var source=volume.storyPanorama;
        float aspect=source.width*crop.width/(source.height*crop.height);
        float width=Mathf.Clamp(Mathf.Max(spot.hitSize.x*1.19f,spot.hitSize.y*1.15f*targetAspect/aspect),.05f,.32f);
        float height=Mathf.Min(.78f,width*aspect/targetAspect);
        float x=Mathf.Clamp(spot.position.x-width*.5f,0,1-width);
        float y=Mathf.Clamp(1-spot.position.y-height*.5f,0,1-height);
        return new Rect(crop.x+x*crop.width,crop.y+y*crop.height,width*crop.width,height*crop.height);
    }

    void ShowBird(BirdSpecies bird)
    {
        ChangeLeaf(Leaf.BirdDetail);
        lastBird=bird;
        Label(content,"百鸟",Left,39,240,41,25,Notes,Hand);
        Hair(content,Left,102,ContentWidth);
        var plate=Panel(content,"月白栖纸",Left,127,426,523,new Color(.87f,.92f,.845f));
        Art(plate.transform,"纸坊窗影",birdSetting,7,7,412,509);
        Frame(plate.transform,426,523,LineGold,4);
        plate.gameObject.AddComponent<RectMask2D>();
        Shape(plate.transform,"落羽投影",SettingShape.Kind.SoftShadow,142,363,149,28,new Color(.35f,.30f,.28f,.21f),Color.clear);
        var portrait=Place(plate.transform,"折纸鸟观察",19,69,388,402).gameObject.AddComponent<BirdPortrait>();
        portrait.Display(bird);
        portrait.inspect=true;
        portrait.raycastTarget=false;

        const float textX=811,textW=416;
        Label(content,bird.birdName,textX-6,122,textW,96,64,Sepia,NightUi.Display);
        Label(content,Origin(bird),textX,214,textW,43,18,Notes,Hand);
        Hair(content,textX,267,textW);
        Label(content,"鸟类小识",textX,284,textW,43,23,Notes,Hand);
        var facts=string.IsNullOrWhiteSpace(bird.naturalHistory)?bird.description:bird.naturalHistory;
        Label(content,facts,textX,334,textW,92,22,Sepia,Body);
        if(!string.IsNullOrWhiteSpace(bird.albumStory))
        {
            Label(content,"卷中故事",textX,454,textW,39,23,Notes,Hand);
            Label(content,bird.albumStory,textX,494,textW,125,22,Sepia,Hand);
        }
        var back=PaperButton(content,"返回百鸟",1050,610,172,42,()=>CloseBird());
        Focus(back);
    }

    string Origin(BirdSpecies bird)
    {
        foreach(var volume in volumes)
            foreach(var chapter in volume.chapters)
                foreach(var perch in chapter.perches)
                    if(perch.species==bird && book.Has(chapter.stageId,perch.id))
                        return volume.number+" · "+volume.title+"　/　"+chapter.title;
        return "旧卷 · 风井纸坊";
    }

    void OnDestroy()
    {
        if(paintingInk)Destroy(paintingInk);
        if(mutedInk)Destroy(mutedInk);
        if(thumbnailInk)Destroy(thumbnailInk);
    }
}
