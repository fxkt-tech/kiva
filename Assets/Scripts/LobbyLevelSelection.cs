using System;
using UnityEngine;
using UnityEngine.UI;

public sealed class LobbyLevelSelection : MonoBehaviour
{
    [SerializeField] Button back;
    Button initialFocus;

    public void Bind(Action close, Action<int> play)
    {
        back.onClick.AddListener(() => close());
        foreach (var card in GetComponentsInChildren<LobbyLevelCard>(true))
        {
            var button = card.Bind(play);
            if (!initialFocus && button.interactable) initialFocus = button;
        }
    }

    public void Focus() => NightUi.Focus(initialFocus ? initialFocus : back);
}
