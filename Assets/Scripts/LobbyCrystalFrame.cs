using UnityEngine;
using UnityEngine.UI;

// A cut-glass UI face. The rim and the face share the same silhouette at every size.
[RequireComponent(typeof(CanvasRenderer))]
public sealed class LobbyCrystalFrame : MaskableGraphic
{
    [SerializeField, Min(0)] float corner = 12f;
    [SerializeField, Min(0)] float rimWidth = 1.5f;
    [SerializeField] Color lower = Color.white;
    [SerializeField] Color litEdge = Color.white;
    [SerializeField] Color shadedEdge = Color.clear;

    protected override void OnPopulateMesh(VertexHelper vh)
    {
        vh.Clear();
        var rect = GetPixelAdjustedRect();
        float halfSize = Mathf.Min(rect.width, rect.height) * .5f;
        if (halfSize <= 0) return;
        float cut = Mathf.Min(corner, halfSize);
        float width = Mathf.Min(rimWidth, halfSize);
        var inner = Rect.MinMaxRect(rect.xMin + width, rect.yMin + width, rect.xMax - width, rect.yMax - width);
        float innerCut = Mathf.Clamp(cut - width * (2f - Mathf.Sqrt(2f)), 0, halfSize - width);

        for (int i = 0; i < 8; i++)
        {
            Vector2 outer = Corner(rect, cut, i);
            Vector2 inset = Corner(inner, innerCut, i);
            var uv = Rect.PointToNormalized(rect, outer);
            float reflection = .55f * uv.y + .45f * (1f - uv.x);
            Color edge = Color.Lerp(shadedEdge, litEdge, reflection * reflection);
            vh.AddVert(outer, edge, Vector2.zero);
            vh.AddVert(inset, edge, Vector2.zero);
            vh.AddVert(inset, Color.Lerp(lower, color, uv.y), Vector2.zero);
        }
        vh.AddVert(rect.center, Color.Lerp(lower, color, .5f), Vector2.zero);
        for (int i = 0; i < 8; i++)
        {
            int a = i * 3, b = ((i + 1) % 8) * 3;
            vh.AddTriangle(a, b, b + 1);
            vh.AddTriangle(a, b + 1, a + 1);
            vh.AddTriangle(24, a + 2, b + 2);
        }
    }

    static Vector2 Corner(Rect r, float cut, int i) => i switch
    {
        0 => new Vector2(r.xMin + cut, r.yMax), 1 => new Vector2(r.xMax - cut, r.yMax),
        2 => new Vector2(r.xMax, r.yMax - cut), 3 => new Vector2(r.xMax, r.yMin + cut),
        4 => new Vector2(r.xMax - cut, r.yMin), 5 => new Vector2(r.xMin + cut, r.yMin),
        6 => new Vector2(r.xMin, r.yMin + cut), _ => new Vector2(r.xMin, r.yMax - cut)
    };
}
