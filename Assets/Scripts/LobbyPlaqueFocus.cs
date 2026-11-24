using UnityEngine;
using UnityEngine.EventSystems;

// One highlight responds to both hover and keyboard focus.
public sealed class LobbyPlaqueFocus : MonoBehaviour, IPointerEnterHandler, IPointerExitHandler, ISelectHandler, IDeselectHandler
{
    [SerializeField] CanvasGroup highlight;
    bool pointerOver, selected;

    public void Bind(CanvasGroup overlay){highlight=overlay;}
    void OnEnable(){selected=EventSystem.current && EventSystem.current.currentSelectedGameObject==gameObject;Refresh();}
    void OnDisable(){pointerOver=selected=false;Refresh();}
    public void OnPointerEnter(PointerEventData data){pointerOver=true;Refresh();}
    public void OnPointerExit(PointerEventData data){pointerOver=false;Refresh();}
    public void OnSelect(BaseEventData data){selected=true;Refresh();}
    public void OnDeselect(BaseEventData data){selected=false;Refresh();}
    void Refresh(){if(highlight)highlight.alpha=pointerOver?1:selected?.58f:0;}
}
