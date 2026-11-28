using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

// One highlight responds to both hover and keyboard focus.
public sealed class LobbyPlaqueFocus : MonoBehaviour, IPointerEnterHandler, IPointerMoveHandler, IPointerExitHandler, ISelectHandler, IDeselectHandler
{
    [SerializeField] CanvasGroup highlight;
    [SerializeField] bool selectOnHover;
    bool pointerOver, selected;

    public void Bind(CanvasGroup overlay){highlight=overlay;}
    void OnEnable(){selected=EventSystem.current && EventSystem.current.currentSelectedGameObject==gameObject;Refresh();}
    void OnDisable(){pointerOver=selected=false;Refresh();}
    public void OnPointerEnter(PointerEventData data){
        if(selectOnHover){OnPointerMove(data);return;}
        pointerOver=true;Refresh();
    }
    public void OnPointerMove(PointerEventData data){
        // The opening animation can move a card under a stationary cursor.
        if(!selectOnHover || selected || data.delta.sqrMagnitude==0)return;
        var button=GetComponent<Selectable>();
        if(button && button.IsInteractable())NightUi.Focus(button);
    }
    public void OnPointerExit(PointerEventData data){pointerOver=false;Refresh();}
    public void OnSelect(BaseEventData data){selected=true;Refresh();}
    public void OnDeselect(BaseEventData data){selected=false;Refresh();}
    void Refresh(){
        if(!highlight)return;
        float focusAlpha=selectOnHover?1f:.58f;
        highlight.alpha=pointerOver?1f:selected?focusAlpha:0f;
    }
}
