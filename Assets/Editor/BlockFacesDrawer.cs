using UnityEditor;
using UnityEngine;

[CustomPropertyDrawer(typeof(BlockFaces))]
public sealed class BlockFacesDrawer : PropertyDrawer
{
    internal static readonly string[] Sides = { "左面", "右面", "底面", "顶面", "背面", "正面" };

    public override float GetPropertyHeight(SerializedProperty property, GUIContent label)
    {
        float line = EditorGUIUtility.singleLineHeight;
        return property.isExpanded ? line + Sides.Length * (line + EditorGUIUtility.standardVerticalSpacing) : line;
    }

    public override void OnGUI(Rect position, SerializedProperty property, GUIContent label)
    {
        EditorGUI.BeginProperty(position, label, property);
        position.height = EditorGUIUtility.singleLineHeight;
        property.isExpanded = EditorGUI.Foldout(position, property.isExpanded, "六面材质", true);
        if (property.isExpanded)
        {
            using (new EditorGUI.IndentLevelScope())
            {
                var faces = (BlockFaces)property.intValue;
                for (int side = 0; side < Sides.Length; side++)
                {
                    position.y += position.height + EditorGUIUtility.standardVerticalSpacing;
                    EditorGUI.BeginChangeCheck();
                    var kind = (SurfaceKind)EditorGUI.EnumPopup(position, Sides[side], faces.At(side));
                    if (EditorGUI.EndChangeCheck())
                    {
                        faces = faces.With(side, kind);
                        property.intValue = (int)faces;
                    }
                }
            }
        }
        EditorGUI.EndProperty();
    }
}
