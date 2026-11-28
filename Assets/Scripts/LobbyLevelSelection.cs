using System;
using System.Collections;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

public sealed class LobbyLevelSelection : MonoBehaviour
{
    [SerializeField] Button back;
    [SerializeField] RectTransform panel;
    [SerializeField] CanvasGroup fade;
    Vector2 restPosition;
    float progress;
    Button initialFocus;

    public void Bind(Action close, Action<int> play)
    {
        restPosition = panel.anchoredPosition;
        back.onClick.AddListener(() => close());
        var above = back;
        foreach (var card in GetComponentsInChildren<LobbyLevelCard>(true))
        {
            var button = card.Bind(play);
            if (!button.interactable) continue;
            if (!initialFocus) initialFocus = button;
            var nav = above.navigation;
            nav.mode = Navigation.Mode.Explicit;
            nav.selectOnDown = button;
            above.navigation = nav;
            button.navigation = new Navigation { mode = Navigation.Mode.Explicit, selectOnUp = above };
            above = button;
        }
    }

    public void Show(bool open, RectTransform origin, Action closed = null)
    {
        gameObject.SetActive(true);
        StopAllCoroutines();
        fade.interactable = fade.blocksRaycasts = open;
        if (open) NightUi.Focus(initialFocus ? initialFocus : back);
        StartCoroutine(Reveal(open, origin, closed));
    }

    void LateUpdate()
    {
        // Clicking the artwork clears uGUI selection. Keep the open menu navigable.
        if (fade.interactable && EventSystem.current && !EventSystem.current.currentSelectedGameObject)
            NightUi.Focus(initialFocus ? initialFocus : back);
    }

    IEnumerator Reveal(bool open, RectTransform origin, Action closed)
    {
        float target = open ? 1f : 0f;
        do
        {
            progress = Mathf.MoveTowards(progress, target, Time.unscaledDeltaTime / .22f);
            float t = Mathf.SmoothStep(0f, 1f, progress);
            panel.anchoredPosition = restPosition;
            Vector2 source = panel.parent.InverseTransformPoint(origin.TransformPoint(origin.rect.center));
            panel.anchoredPosition += (source - (Vector2)panel.localPosition) * (1f - t);
            panel.localScale = Vector3.one * Mathf.Lerp(.36f, 1f, t);
            fade.alpha = t;
            if (progress != target) yield return null;
        } while (progress != target);
        if (!open)
        {
            gameObject.SetActive(false);
            closed?.Invoke();
        }
    }
}
