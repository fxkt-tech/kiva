using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

// Keep the authored highlight and text colour when the pointer leaves or the page reopens.
public sealed class LobbyEntry : MonoBehaviour, IPointerEnterHandler, IPointerExitHandler
{
    [SerializeField] GameObject highlight;
    [SerializeField] Text label;
    bool initiallyLit;
    Color initialColor;

    void Awake()
    {
        initiallyLit = highlight.activeSelf;
        initialColor = label.color;
    }

    void OnEnable() => SetHovered(false);
    public void OnPointerEnter(PointerEventData eventData) => SetHovered(true);
    public void OnPointerExit(PointerEventData eventData) => SetHovered(false);

    void SetHovered(bool hovered)
    {
        highlight.SetActive(hovered || initiallyLit);
        label.color = hovered ? NightUi.White : initialColor;
    }
}
