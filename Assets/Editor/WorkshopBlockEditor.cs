using UnityEditor;
using UnityEngine;

[CustomEditor(typeof(WorkshopBlock)), CanEditMultipleObjects]
public sealed class WorkshopBlockEditor : Editor
{
    bool showReferences;

    public override void OnInspectorGUI()
    {
        var first = (WorkshopBlock)target;
        for (int side = 0; side < BlockFacesDrawer.Sides.Length; side++)
        {
            var kind = first.Faces.At(side);
            bool mixed = false;
            foreach (WorkshopBlock block in targets)
                mixed |= block.Faces.At(side) != kind;

            using (new EditorGUI.MixedValueScope(mixed))
            {
                EditorGUI.BeginChangeCheck();
                var choice = (SurfaceKind)EditorGUILayout.EnumPopup(BlockFacesDrawer.Sides[side], kind);
                if (!EditorGUI.EndChangeCheck()) continue;

                foreach (WorkshopBlock block in targets)
                {
                    ApplyFaces(block, block.Faces.With(side, choice));
                }
            }
        }
        showReferences = EditorGUILayout.Foldout(showReferences, "预设引用", true);
        if (!showReferences) return;
        serializedObject.Update();
        DrawPropertiesExcluding(serializedObject, "m_Script", "faces");
        if (serializedObject.ApplyModifiedProperties())
            foreach (WorkshopBlock block in targets) ApplyFaces(block, block.Faces);
    }

    static void ApplyFaces(WorkshopBlock block, BlockFaces faces)
    {
        var renderers = block.GetComponentsInChildren<MeshRenderer>(true);
        Undo.RecordObject(block, "修改方块表面");
        Undo.RecordObjects(renderers, "修改方块表面");
        block.SetFaces(faces);
        if (!PrefabUtility.IsPartOfPrefabInstance(block)) return;
        PrefabUtility.RecordPrefabInstancePropertyModifications(block);
        foreach (var renderer in renderers)
            PrefabUtility.RecordPrefabInstancePropertyModifications(renderer);
    }
}
