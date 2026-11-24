using System;
using System.Collections.Generic;
using System.IO;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.InputSystem;
using UnityEngine.UI;
using static NightUi;

public sealed class Campaign : MonoBehaviour
{
    [SerializeField] CanvasGroup outside;
    [SerializeField] RectTransform lobby,stages,archive,complete;
    [SerializeField] Button storyDoor;
    [SerializeField] Text resultStory;
    [SerializeField] StoryScrollPage story;
    [SerializeField] BirdPortrait resultBird;
    [SerializeField] GameObject battle;
    [SerializeField] GameObject eveningVeil;
    [SerializeField] Text hint;
    [SerializeField] Camera menuCamera;
    [SerializeField] CanvasGroup comingSoon;
    [SerializeField] Text comingSoonFeature;
    [SerializeField] Button comingSoonClose;

    BirdCollection book;
    CollectionFolio folio;
    IReadOnlyList<StoryVolume> collectionCatalog;
    SettingsPanel settings;
    FerryPuzzle level;
    WorkshopLevel activeChapter,returnedChapter;
    RectTransform current;
    GameObject priorFocus;
    float transition;
    BirdCollection Book=>book??=new BirdCollection(Path.Combine(Application.persistentDataPath,"bird-collection.json"));

    void Awake(){
        GameSettings.Current.Apply();GameSettings.Current.ApplyDisplay();
        OpenLobby();outside.alpha=1;
    }
    void OnApplicationQuit()=>book?.Flush();

    public void OpenLobby(){Show(lobby);Focus(storyDoor);}
    public void OpenStages(){
        Show(stages);
        story.Open(Book,OpenLobby,StartStage,returnedChapter);
        returnedChapter=null;
    }
    public void OpenArchive()=>OpenFolio(null);

    void OpenFolio(BirdSpecies featured)
    {
        if(collectionCatalog==null)collectionCatalog=ResolveCollectionCatalog();
        if(collectionCatalog.Count==0)
        {
            Debug.LogWarning("收藏画卷资源尚未载入，请在 Unity 中刷新资源。",this);
            collectionCatalog=null;
            return;
        }

        if(!folio && !archive.TryGetComponent(out folio))
        {
            // An already-open scene may still hold the previous archive hierarchy.
            foreach(Transform previous in archive)previous.gameObject.SetActive(false);
            Fill(archive,archive.parent);
            folio=archive.gameObject.AddComponent<CollectionFolio>();
        }
        Show(archive);
        folio.Open(Book,collectionCatalog,OpenLobby,featured);
    }

    IReadOnlyList<StoryVolume> ResolveCollectionCatalog()
    {
        var scroll=story?story:stages.GetComponent<StoryScrollPage>();
        if(scroll && scroll.Volumes!=null)
        {
            var configured=new List<StoryVolume>();
            foreach(var item in scroll.Volumes)if(item)configured.Add(item);
            if(configured.Count>0 && configured.Count==scroll.Volumes.Count)return configured;
        }

        var catalog=new List<StoryVolume>();
        var spring=Resources.Load<StoryVolume>("GuestRoom/SpringVolume");
        if(spring)catalog.Add(spring);
        var additional=Resources.LoadAll<StoryVolume>("StoryScroll");
        Array.Sort(additional,(a,b)=>string.CompareOrdinal(a.name,b.name));
        catalog.AddRange(additional);
        return catalog;
    }
    void StartStage(WorkshopLevel chapter){
        if(level && activeChapter!=chapter){
            level.gameObject.SetActive(false);
            Destroy(level.gameObject);
            level=null;
        }
        activeChapter=chapter;
        if(!level){
            var go=new GameObject(chapter.locationName);go.SetActive(false);level=go.AddComponent<FerryPuzzle>();
            level.Build(hint,chapter,Book,Finish,PauseChanged);
        }
        Show(null);level.Begin();
    }
    void Finish(){
        returnedChapter=activeChapter;
        resultBird.Display(activeChapter.perches[0].species);
        resultStory.text=activeChapter.arrivalStory;
        Show(complete);Focus(complete.GetComponentInChildren<Button>());
    }
    public void ResultBird()=>OpenFolio(resultBird?resultBird.species:null);
    public void OpenSettings()=>Sheet(false);
    public void OpenComingSoon(string feature)
    {
        if(current!=lobby || comingSoon.gameObject.activeSelf || (settings && settings.gameObject.activeSelf))return;
        priorFocus=EventSystem.current?EventSystem.current.currentSelectedGameObject:null;
        comingSoonFeature.text=feature;
        outside.interactable=false;
        comingSoon.transform.SetAsLastSibling();
        comingSoon.gameObject.SetActive(true);
        Focus(comingSoonClose);
    }
    public void CloseComingSoon()
    {
        if(!comingSoon.gameObject.activeSelf)return;
        comingSoon.gameObject.SetActive(false);
        RestoreFocus();
    }
    public void TogglePause(){if(level)level.TogglePause();}
    void PauseChanged(bool paused){if(paused)Sheet(true);else CloseSheet();}
    void Sheet(bool inLevel){
        if(!settings)settings=SettingsPanel.Create(transform);
        priorFocus=EventSystem.current?EventSystem.current.currentSelectedGameObject:null;
        outside.interactable=false;
        battle.GetComponent<CanvasGroup>().interactable=false;
        settings.Open(inLevel,inLevel?(System.Action)level.Resume:CloseSheet,LeaveStage,inLevel?activeChapter.title+"　·　"+level.CollectionStatus:null);
    }
    void CloseSheet(){
        if(settings)settings.Close();outside.interactable=true;battle.GetComponent<CanvasGroup>().interactable=true;
        RestoreFocus();
    }
    void RestoreFocus(){
        outside.interactable=true;
        if(priorFocus && priorFocus.activeInHierarchy && EventSystem.current)EventSystem.current.SetSelectedGameObject(priorFocus);
        priorFocus=null;
    }
    void LeaveStage(){
        if(!level.SaveCollection()){settings.Notice("收藏待保存，请检查磁盘后再次返回剧情");return;}
        OpenStages();
    }
    void Show(RectTransform page){
        CloseComingSoon();
        CloseSheet();current=page;
        eveningVeil.SetActive(page && page!=lobby);
        lobby.gameObject.SetActive(page==lobby);stages.gameObject.SetActive(page==stages);
        archive.gameObject.SetActive(page==archive);
        complete.gameObject.SetActive(page==complete);
        bool playing=!page;outside.gameObject.SetActive(!playing);menuCamera.enabled=!playing;battle.SetActive(playing);
        if(level)level.gameObject.SetActive(playing);
        transition=0;outside.alpha=page?.83f:0;
    }
    public void Back(){
        if(comingSoon.gameObject.activeSelf){CloseComingSoon();return;}
        if(settings && settings.gameObject.activeSelf){settings.Back();return;}
        if(!current){TogglePause();return;}
        if(current==stages && story.BackOutOfCaption())return;
        if(current==archive && folio && folio.CloseBird())return;
        if(current==lobby)OpenSettings();else OpenLobby();
    }
    void Update(){
        if(Keyboard.current!=null && Keyboard.current.escapeKey.wasPressedThisFrame){Back();return;}
        if(!current || outside.alpha>=1)return;
        transition+=Time.unscaledDeltaTime;outside.alpha=Mathf.Lerp(.83f,1,transition/.23f);
    }
}
