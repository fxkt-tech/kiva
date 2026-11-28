using System;
using UnityEngine;
using UnityEngine.UI;

public sealed class LobbyUI : SceneUI
{
    [SerializeField] Button mode;
    [SerializeField] GameObject modeRail;
    [SerializeField] CanvasGroup home;
    [SerializeField] LobbyLevelSelection selectionPrefab;
    LobbyLevelSelection selection;
    Action<WorkshopLayout> play;

    public void Bind(Action<WorkshopLayout> onPlay)
    {
        play = onPlay;
        mode.onClick.AddListener(OpenLevels);
        NightUi.Focus(mode);
    }

    void OpenLevels()
    {
        if (!home.interactable) return;
        if (!selection)
        {
            selection = Instantiate(selectionPrefab, transform, false);
            selection.Bind(Back, PlayLevel);
        }
        home.interactable = home.blocksRaycasts = false;
        modeRail.SetActive(false);
        selection.gameObject.SetActive(true);
        selection.Focus();
    }

    public override void Back()
    {
        if (!selection || !selection.isActiveAndEnabled) return;
        selection.gameObject.SetActive(false);
        modeRail.SetActive(true);
        home.interactable = home.blocksRaycasts = true;
        NightUi.Focus(mode);
    }

    void PlayLevel(WorkshopLayout stage)
    {
        if (isActiveAndEnabled && selection.isActiveAndEnabled) play(stage);
    }
}
