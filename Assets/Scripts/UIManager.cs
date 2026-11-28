using System;
using System.Collections;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.InputSystem;
using UnityEngine.InputSystem.UI;
using UnityEngine.SceneManagement;

[RequireComponent(typeof(EventSystem), typeof(InputSystemUIInputModule))]
public sealed class UIManager : MonoBehaviour
{
    public static UIManager Instance { get; private set; }
    EventSystem events;
    InputAction cancel;
    Scene owner;
    SceneUI screen;
    Coroutine loading;

    [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
    static void ResetInstance() => Instance = null;

    [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.BeforeSceneLoad)]
    static void Bootstrap()
    {
        if (!Instance) Instantiate(Resources.Load<GameObject>("UI/UIManager"));
    }

    void Awake()
    {
        if (Instance && Instance != this)
        {
            gameObject.SetActive(false);
            Destroy(gameObject);
            return;
        }
        Instance = this;
        DontDestroyOnLoad(gameObject);
        events = GetComponent<EventSystem>();
        cancel = GetComponent<InputSystemUIInputModule>().cancel.action;
        cancel.performed += Back;
        SceneManager.sceneUnloaded += Release;
    }

    public void Show<T>(MonoBehaviour entry, Action<T> bind) where T : SceneUI
    {
        ClearScreen();
        owner = entry.gameObject.scene;
        loading = StartCoroutine(Load(entry, owner, bind));
    }

    IEnumerator Load<T>(MonoBehaviour entry, Scene scene, Action<T> bind) where T : SceneUI
    {
        string path = "UI/" + typeof(T).Name;
        var request = Resources.LoadAsync<GameObject>(path);
        yield return request;
        loading = null;
        if (!entry || !entry.isActiveAndEnabled || owner != scene || !scene.isLoaded) yield break;
        var prefab = request.asset as GameObject;
        if (!prefab || !prefab.TryGetComponent<T>(out var root))
            throw new InvalidOperationException($"Missing {typeof(T).Name} on Resources/{path}.");

        var view = Instantiate(root, new InstantiateParameters { scene = scene });
        screen = view;
        bind(view);
    }

    void Back(InputAction.CallbackContext context)
    {
        if (screen && screen.isActiveAndEnabled) screen.Back();
    }

    public void Release(Scene scene)
    {
        if (owner == scene) ClearScreen();
    }

    void ClearScreen()
    {
        if (loading != null) StopCoroutine(loading);
        loading = null;
        if (events.currentSelectedGameObject) events.SetSelectedGameObject(null);
        if (screen)
        {
            screen.gameObject.SetActive(false);
            Destroy(screen.gameObject);
        }
        screen = null;
        owner = default;
    }

    void OnDestroy()
    {
        if (Instance != this) return;
        cancel.performed -= Back;
        SceneManager.sceneUnloaded -= Release;
        Instance = null;
    }
}
