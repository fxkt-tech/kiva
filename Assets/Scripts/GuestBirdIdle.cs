using UnityEngine;

// Keep the feet near the sill while the breast and crest rise together.
public sealed class GuestBirdIdle : MonoBehaviour
{
    RectTransform paper, viewport;

    void Awake()
    {
        paper=(RectTransform)transform;
        viewport=(RectTransform)paper.parent.parent;
    }

    void Update()
    {
        float breath=Mathf.Sin(Time.unscaledTime*1.55f);
        var area=viewport.rect;
        float aspect=area.width/Mathf.Max(1,area.height);
        float scale=Mathf.Lerp(.82f,1,Mathf.InverseLerp(4f/3f,16f/9f,aspect));
        paper.localScale=new Vector3(scale*(1-.005f*breath),scale*(1+.016f*breath),1);
        paper.localRotation=Quaternion.Euler(0,0,.55f*Mathf.Sin(Time.unscaledTime*.77f));
    }
}
