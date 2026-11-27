using UnityEngine;

public sealed class StageZone : MonoBehaviour
{
    [SerializeField] Color gizmoColor = Color.cyan;

    public bool Contains(Vector3 world)
    {
        if (!gameObject.activeSelf) return false;
        var p = transform.InverseTransformPoint(world);
        return Mathf.Abs(p.x) <= .5f && Mathf.Abs(p.y) <= .5f && Mathf.Abs(p.z) <= .5f;
    }

    void OnDrawGizmos()
    {
        Gizmos.color = gizmoColor;
        Gizmos.matrix = transform.localToWorldMatrix;
        Gizmos.DrawWireCube(Vector3.zero, Vector3.one);
    }
}
