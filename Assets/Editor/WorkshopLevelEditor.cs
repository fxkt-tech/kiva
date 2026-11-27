using UnityEditor;
using UnityEngine;

[CustomEditor(typeof(WorkshopLevel))]
public sealed class WorkshopLevelEditor : Editor
{
    public override void OnInspectorGUI(){
        var path=AssetDatabase.GetAssetPath(target);
        if(path.StartsWith("Assets/GeneratedLevels/",System.StringComparison.Ordinal) && path.EndsWith("/Level.asset",System.StringComparison.Ordinal)){
            var report=AssetDatabase.LoadAssetAtPath<WorkshopGenerationReport>(path.Substring(0,path.Length-"Level.asset".Length)+"Editor/Report.asset");
            if(report && GUILayout.Button("查看验证报告和回放"))Selection.activeObject=report;
        }
        DrawDefaultInspector();
    }
    [MenuItem("CONTEXT/WorkshopLevel/生成可编辑关卡预设")]
    static void Bake(MenuCommand command)
    {
        var source=(WorkshopLevel)command.context;
        var path=EditorUtility.SaveFilePanelInProject("保存关卡预设",source.name,"prefab","选择关卡预设位置");
        if(string.IsNullOrEmpty(path))return;
        Bake(source,path);
    }

    public static void Bake(WorkshopLevel source, string path)
    {
        const string template="Assets/Prefabs/LayoutTemplate.prefab";
        if(path==template)throw new System.ArgumentException("请选择独立的关卡预设路径。");
        var root=(GameObject)PrefabUtility.InstantiatePrefab(AssetDatabase.LoadAssetAtPath<GameObject>(template));
        try
        {
            root.name=source.name;
            root.GetComponent<WorkshopLayout>().Populate(source,(prefab,parent)=>
                (GameObject)PrefabUtility.InstantiatePrefab(prefab,parent));
            foreach(var node in root.GetComponentsInChildren<Transform>(true))
                PrefabUtility.RecordPrefabInstancePropertyModifications(node.gameObject);
            foreach(var component in root.GetComponentsInChildren<Component>(true))
                if(PrefabUtility.IsPartOfPrefabInstance(component))
                    PrefabUtility.RecordPrefabInstancePropertyModifications(component);
            PrefabUtility.SaveAsPrefabAsset(root,path);
        }
        finally { DestroyImmediate(root); }
    }
}
