using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UI;

// Keep navigation by keyboard inside the visible part of the settings sheet.
public sealed class ScrollFocus : MonoBehaviour, ISelectHandler
{
    public ScrollRect scroll;
    public void OnSelect(BaseEventData data)
    {
        if(!scroll)return;
        Canvas.ForceUpdateCanvases();
        var viewport=scroll.viewport;
        var bounds=RectTransformUtility.CalculateRelativeRectTransformBounds(viewport,transform);
        var pos=scroll.content.anchoredPosition;
        if(bounds.min.y<viewport.rect.yMin)pos.y+=viewport.rect.yMin-bounds.min.y;
        else if(bounds.max.y>viewport.rect.yMax)pos.y-=bounds.max.y-viewport.rect.yMax;
        pos.y=Mathf.Clamp(pos.y,0,Mathf.Max(0,scroll.content.rect.height-viewport.rect.height));
        scroll.content.anchoredPosition=pos;
    }
}
