using System;
using UnityEngine;
using UnityEngine.UI;

[RequireComponent(typeof(Button), typeof(CanvasGroup), typeof(LobbyPlaqueFocus))]
public sealed class LobbyLevelCard : MonoBehaviour
{
    [SerializeField] WorkshopLayout stage;
    [SerializeField] Texture cover;
    [SerializeField] RawImage preview;
    [SerializeField] Text title;

    public Button Bind(Action<WorkshopLayout> play)
    {
        var button = GetComponent<Button>();
        button.interactable = stage;
        GetComponent<LobbyPlaqueFocus>().enabled = stage;
        GetComponent<CanvasGroup>().alpha = stage ? 1f : .64f;
        preview.texture = cover;
        preview.enabled = cover;
        title.text = stage ? stage.Definition.title : "待开放";
        if (stage) button.onClick.AddListener(() => play(stage));
        return button;
    }
}
