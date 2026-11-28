using UnityEngine;
using UnityEngine.SceneManagement;

public sealed class Gameplay : MonoBehaviour
{
    [SerializeField] FerryPuzzle level;
    [SerializeField] WorkshopLayout layout;

    void Awake() => level.gameObject.SetActive(false);

    void Start()
    {
        GameSettings.Current.Apply();
        UIManager.Instance.Show<BattleUI>(this, Begin);
    }

    void Begin(BattleUI ui)
    {
        ui.Bind(TogglePause, ReturnToLobby, layout.Definition.title);
        level.gameObject.SetActive(true);
        level.Build(ui.Hud, layout, ReturnToLobby, ui.SetPaused);
        level.Begin();
    }

    public void TogglePause()
    {
        if (enabled && level.isActiveAndEnabled) level.TogglePause();
    }

    void ReturnToLobby()
    {
        if (!enabled) return;
        enabled = false;
        UIManager.Instance.Release(gameObject.scene);
        level.gameObject.SetActive(false);
        SceneManager.LoadSceneAsync(nameof(Lobby));
    }
}
