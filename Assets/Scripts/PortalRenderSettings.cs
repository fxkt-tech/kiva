using UnityEngine;

public sealed class PortalRenderSettings : ScriptableObject
{
    [SerializeField, Range(0, 4), InspectorName("最大渲染深度")]
    [Tooltip("视线最多连续穿过几次门。0 只显示纸面，2 可以看到门中门。")]
    int maxDepth = 2;

    public int MaxDepth => Mathf.Clamp(maxDepth, 0, 4);
}
