using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.InputSystem;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

public sealed class Gameplay : MonoBehaviour
{
    [SerializeField] FerryPuzzle level;
    [SerializeField] WorkshopLayout layout;
    [SerializeField] WorkshopHud hud;
    [SerializeField] SettingsPanel settings;
    GameObject priorFocus;

    void Start()
    {
        GameSettings.Current.Apply();
        hud.GetComponentInChildren<Button>(true).onClick.AddListener(TogglePause);
        level.Build(hud, layout, ReturnToLobby, PauseChanged);
        level.Begin();
    }

    public void TogglePause() => level.TogglePause();

    void Update()
    {
        if (Keyboard.current == null || !Keyboard.current.escapeKey.wasPressedThisFrame) return;
        if (settings.gameObject.activeSelf) settings.Back();
        else TogglePause();
    }

    void PauseChanged(bool paused)
    {
        hud.GetComponent<CanvasGroup>().interactable = !paused;
        if (paused)
        {
            priorFocus = EventSystem.current.currentSelectedGameObject;
            settings.Open(level.Resume, ReturnToLobby, layout.Definition.title);
        }
        else
        {
            settings.Close();
            EventSystem.current.SetSelectedGameObject(priorFocus);
        }
    }

    void ReturnToLobby()
    {
        if (!enabled) return;
        enabled = false;
        settings.Close();
        hud.gameObject.SetActive(false);
        level.gameObject.SetActive(false);
        SceneManager.LoadSceneAsync(nameof(Lobby));
    }
}
