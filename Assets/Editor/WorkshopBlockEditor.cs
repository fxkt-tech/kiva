using UnityEditor;
using UnityEngine;

[CustomEditor(typeof(WorkshopBlock)), CanEditMultipleObjects]
public sealed class WorkshopBlockEditor : Editor
{
    public override void OnInspectorGUI()
    {
        if (!DrawDefaultInspector()) return;
        foreach (WorkshopBlock block in targets)
        {
            var surfaces = block.GetComponentsInChildren<MeshRenderer>(true);
            Undo.RecordObjects(surfaces, "更新纸砖表面");
            block.SetFaces(block.Faces);
            foreach (var surface in surfaces)
                if (PrefabUtility.IsPartOfPrefabInstance(surface))
                    PrefabUtility.RecordPrefabInstancePropertyModifications(surface);
        }
    }
}
