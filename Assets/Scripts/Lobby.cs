using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

public sealed class Lobby : MonoBehaviour
{
    [SerializeField] Button storyDoor;

    void Start()
    {
        GameSettings.Current.Apply();
        GameSettings.Current.ApplyDisplay();
        NightUi.Focus(storyDoor);
    }

    public void Play()
    {
        if (!storyDoor.interactable) return;
        storyDoor.interactable = false;
        SceneManager.LoadSceneAsync(nameof(Gameplay));
    }
}
