using UnityEngine;

// Six editable surfaces, ordered like WhiteFaces (left, right, bottom, top, back, front).
public sealed class WorkshopBlock : MonoBehaviour
{
    [SerializeField] MeshRenderer[] surfaces;
    [SerializeField] Material paper, bamboo;
    [SerializeField] WhiteFaces white;
    public WhiteFaces Faces => white;

    public void SetFaces(WhiteFaces value)
    {
        white = value;
        for (int i = 0; i < surfaces.Length; i++)
            surfaces[i].sharedMaterial = ((int)white & (1 << i)) != 0 ? paper : bamboo;
    }

    // Edited blocks outside the shared grid still use their six visible faces for physics.
    public void BuildIndividualFaces(Transform parent)
    {
        for (int i = 0; i < surfaces.Length; i++)
        {
            var side = surfaces[i].transform;
            var right = side.TransformVector(Vector3.right);
            var up = side.TransformVector(Vector3.up);
            var face = new GameObject(side.name).AddComponent<WorkshopFace>();
            face.transform.SetParent(parent, false);
            face.transform.SetPositionAndRotation(side.position, Quaternion.LookRotation(Vector3.Cross(up, right), up));
            var size = new Vector2(right.magnitude, up.magnitude);
            face.Build(new Rect(-size * .5f, size), ((int)white & (1 << i)) != 0);
        }
    }

}
