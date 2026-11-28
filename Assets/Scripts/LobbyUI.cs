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
    bool choosing;
    Action<int> play;

    public void Bind(Action<int> onPlay)
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
        choosing = true;
        selection.Show(true, (RectTransform)mode.transform);
    }

    public override void Back()
    {
        if (!choosing) return;
        choosing = false;
        selection.Show(false, (RectTransform)mode.transform, () =>
        {
            modeRail.SetActive(true);
            home.interactable = home.blocksRaycasts = true;
            NightUi.Focus(mode);
        });
    }

    void PlayLevel(int sceneIndex)
    {
        if (isActiveAndEnabled && choosing) play(sceneIndex);
    }
}
