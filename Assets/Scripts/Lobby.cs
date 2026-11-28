using UnityEngine;
using UnityEngine.SceneManagement;

public sealed class Lobby : MonoBehaviour
{
    void Start()
    {
        GameSettings.Current.Apply();
        GameSettings.Current.ApplyDisplay();
        UIManager.Instance.Show<LobbyUI>(this, ui => ui.Bind(Play));
    }

    void Play(int sceneIndex)
    {
        if (!enabled) return;
        enabled = false;
        UIManager.Instance.Release(gameObject.scene);
        SceneManager.LoadSceneAsync(sceneIndex);
    }
}
