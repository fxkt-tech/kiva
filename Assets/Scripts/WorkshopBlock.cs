using System.Collections.Generic;
using UnityEngine;

// Six editable surfaces, in grid-neighbour order (left, right, bottom, top, back, front).
public sealed class WorkshopBlock : MonoBehaviour
{
    [SerializeField] MeshRenderer[] surfaces;
    [SerializeField] Material plaster, rock, metal, crystal;
    [SerializeField, UnityEngine.Serialization.FormerlySerializedAs("white")] BlockFaces faces;
    public BlockFaces Faces => faces;

    public void SetFaces(BlockFaces value)
    {
        faces = value;
        for (int i = 0; i < surfaces.Length; i++)
            surfaces[i].sharedMaterial = faces.At(i) switch
            {
                SurfaceKind.Plaster => plaster,
                SurfaceKind.Metal => metal,
                SurfaceKind.Crystal => crystal,
                _ => rock
            };
    }

    public void HideInternalFaces(Vector3Int cell, Dictionary<Vector3Int, BlockFaces> occupied)
    {
        for (int side = 0; side < surfaces.Length; side++)
            surfaces[side].forceRenderingOff = occupied.ContainsKey(cell + PaperTerrain.Neighbour(side));
    }

    public void AnchorMovingTexture()
    {
        var properties = new MaterialPropertyBlock();
        properties.SetFloat("_LocalMapping", 1f);
        foreach (var surface in surfaces) surface.SetPropertyBlock(properties);
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
            face.Build(new Rect(-size * .5f, size), faces.At(i));
        }
    }

}
