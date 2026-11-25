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
    void OnEnable(){SceneView.duringSceneGui+=Draw;}
    void OnDisable(){SceneView.duringSceneGui-=Draw;}
    void Draw(SceneView view){
        var stage=(WorkshopLevel)target;
        foreach(var perch in stage.perches){
            Handles.color=new Color(.2f,.65f,.77f);
            var chest=perch.feet+Vector3.up*.25f;
            Handles.DrawWireDisc(perch.feet,Vector3.up,.35f);
            Handles.DrawWireDisc(chest,view.camera.transform.forward,.45f);
            Handles.Label(chest+Vector3.up*.48f,perch.species?perch.species.birdName:perch.id);
        }
    }
}
