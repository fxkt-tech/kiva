using UnityEngine;
using UnityEngine.SceneManagement;

public sealed class Gameplay : MonoBehaviour
{
    [SerializeField] FerryPuzzle level;
    [SerializeField] WorkshopLayout layout;
    static WorkshopLayout requestedStage;

    public static void LoadStage(WorkshopLayout prefab)
    {
        requestedStage = prefab;
        SceneManager.LoadSceneAsync(nameof(Gameplay));
    }

    [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
    static void ClearSelection() => requestedStage = null;

    void Awake() => level.gameObject.SetActive(false);

    void Start()
    {
        GameSettings.Current.Apply();
        if (requestedStage && requestedStage.Definition != layout.Definition)
        {
            var previous = layout;
            previous.gameObject.SetActive(false);
            layout = Instantiate(requestedStage, previous.transform.parent);
            Destroy(previous.gameObject);
        }
        requestedStage = null;
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
