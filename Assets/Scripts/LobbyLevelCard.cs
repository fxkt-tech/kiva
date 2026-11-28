using System;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

[RequireComponent(typeof(Button), typeof(CanvasGroup), typeof(LobbyPlaqueFocus))]
public sealed class LobbyLevelCard : MonoBehaviour
{
    [Tooltip("构建列表中的场景路径，例如 Assets/Scenes/Gameplay_01_SurfaceCourt.unity")]
    [SerializeField] string scenePath;
    [SerializeField] Texture cover;
    [SerializeField] RawImage preview;
    [SerializeField] Text title;

    public Button Bind(Action<int> play)
    {
        int sceneIndex = string.IsNullOrEmpty(scenePath) ? -1 : SceneUtility.GetBuildIndexByScenePath(scenePath);
        bool available = sceneIndex >= 0;
        var button = GetComponent<Button>();
        button.interactable = available;
        GetComponent<LobbyPlaqueFocus>().enabled = available;
        GetComponent<CanvasGroup>().alpha = available ? 1f : .64f;
        preview.texture = cover;
        preview.enabled = cover;
        if (available) button.onClick.AddListener(() => play(sceneIndex));
        else title.text = "待开放";
        return button;
    }
}
