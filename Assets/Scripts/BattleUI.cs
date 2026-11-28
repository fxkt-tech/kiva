using System;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

public sealed class BattleUI : SceneUI
{
    [SerializeField] WorkshopHud hud;
    [SerializeField] CanvasGroup hudGroup;
    [SerializeField] Button pauseButton;
    [SerializeField] SettingsPanel settings;
    Action togglePause, leave;
    string stageTitle;
    GameObject priorFocus;
    public WorkshopHud Hud => hud;

    public void Bind(Action toggle, Action returnToLobby, string title)
    {
        togglePause = toggle;
        leave = returnToLobby;
        stageTitle = title;
        pauseButton.onClick.AddListener(() => togglePause());
    }

    public override void Back()
    {
        if (settings.gameObject.activeSelf) settings.Back();
        else togglePause();
    }

    public void SetPaused(bool paused)
    {
        hudGroup.interactable = hudGroup.blocksRaycasts = !paused;
        if (paused)
        {
            priorFocus = EventSystem.current.currentSelectedGameObject;
            settings.Open(togglePause, leave, stageTitle);
        }
        else
        {
            settings.Close();
            EventSystem.current.SetSelectedGameObject(priorFocus);
            priorFocus = null;
        }
    }
}
